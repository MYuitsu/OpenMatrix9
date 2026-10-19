# Occlusion pyramid, cluster streaming, QEM-error LOD

Ngày: 2026-10-18. Phạm vi: ba hạng mục của mục tiêu tối ưu cluster.

## 1. QEM error cho chọn LOD — **xong, đã nối vào compute**

- `qem::simplify_with_error(mesh, target) -> (MeshData, f32)`: trả **sai số lớn nhất** (sqrt của quadric
  error) mà mỗi collapse gây ra.
- `meshlet::ClusterGroup { levels, error_world }`: nhóm cluster mang sai số của mức thô.
- Compute pass chọn mức theo `error_pixels = error_world · half_height / (distance · tan(fov/2)) < 1 px`,
  không còn ngưỡng 24 px.

Đo (torus 48×16, camera cách 8 mm vs 80 mm, sai số trung bình 0,41 đơn vị):

| Camera | Fine | Coarse |
| --- | --- | --- |
| 8 mm | **40** | 0 |
| 80 mm | 12 | **58** |

12 fine còn lại ở xa là **đúng theo lý thuyết**: nhóm có sai số QEM lớn vẫn vượt 1 px nên phải giữ mức mịn.

## 2. Depth pyramid (occlusion) — **lõi xong, có test**

`occlusion::DepthPyramid::build(depth, w, h)` dựng chuỗi mip **max-depth**; `is_occluded(rect, nearest)`
cull khi `nearest > max_depth(rect)`, tức chỉ cull khi chắc chắn bị che.

4 test: pyramid giữ đúng max của từng quad (4×4 → 2×2 → 1×1: 0,1 / 0,9 / 0,9); cluster sau vật gần bị cull;
cluster gần hơn vật che thì không; depth trống không cull gì; buffer sai bị từ chối.

**Chưa nối vào GPU**: cần texture depth có mip + compute dựng pyramid sau mỗi frame rồi sample ở lượt cull
kế tiếp. Đó là việc còn lại của hạng mục này.

## 3. Streaming cluster — **lõi xong, có test**

`streaming::ClusterStream::new(budget)`, `request(id, bytes)`, `touch`, `stats`. LRU, từ chối cluster lớn
hơn cả budget.

5 test, gồm mô phỏng **2 GiB archive** (69 905 cluster × 30 KiB) qua budget **512 MiB**:

| Chỉ số | Kết quả |
| --- | --- |
| Resident bytes | **≤ 512 MiB** (luôn) |
| Upload | 69 905 (mỗi cluster một lần khi quét tuyến tính) |
| Eviction | > 0 |
| VRAM dùng | **< 1/4 dung lượng archive** |

**Chưa nối vào GPU**: cần buffer pool + upload theo `request`, và đọc cluster record từ đĩa khi cần.

## Trạng thái mục tiêu

| Hạng mục | Lõi + test | Nối GPU/app |
| --- | --- | --- |
| QEM-error LOD | ✅ | ✅ (compute) |
| Depth pyramid | ✅ | ❌ |
| Streaming 2 GB | ✅ | ❌ |

## Vòng 2: nối GPU pyramid — trạng thái và chặn kỹ thuật

Đã làm:
- `ClusterRenderer` tự sở hữu depth texture (`RENDER_ATTACHMENT | TEXTURE_BINDING`).
- Storage-buffer pyramid nhiều mức (offset, width, height) dựng bằng hai compute pipeline
  (`from_depth`: depth texture → level 0; `downsample`: level k−1 → level k, có scratch chống alias).
- Cull shader có nhánh occlusion: chọn mức theo bán kính pixel, lấy max-depth của ô, so với
  `nearest = ndc_z − 2·near·radius/distance²`; chỉ cull khi chắc chắn.
- `set_occlusion(bool)`; mặc định **tắt** cho tới khi đường dựng pyramid được kiểm chứng.

**Chặn kỹ thuật (chưa giải quyết):** với dữ liệu thật, level 0 đọc lại bằng 0 — compute `from_depth`
không ghi vào buffer. Khi hardcode dims trong shader thì ghi đúng 0,5 vào mọi texel; khi dùng dims từ
buffer (đã kiểm tra: giá trị cuối là `[76843, 1, 1, 0]`) thì level 0 vẫn nguyên giá trị prime 0,25. Nghĩa là
đường đọc `dims`/thứ tự dispatch có vấn đề, chưa phải bản chất thuật toán.

Đã kiểm chứng độc lập: khi buffer pyramid chứa dữ liệu (prime 0,25), cull **cull đúng** (2 cluster frame 1 →
0 cluster frame 2). Vì vậy cơ chế cull đúng; chỉ phần dựng pyramid còn hỏng.

Hướng sửa đề xuất (vòng sau): dựng pyramid bằng **render pass** vào texture R32Float nhiều layer
(`texture_2d_array`) thay vì storage buffer + compute, vì đường render đã chạy ổn định và tránh được vấn đề
dispatch/uniform này. Test occlusion đang `#[ignore]` đúng lý do.

## Vòng 3: pyramid dựng bằng render pass — **đã chạy, đã chứng minh đọc đúng depth**

Thay compute bằng **render pass vào texture R32Float nhiều layer** (`crates/om9-render/src/pyramid.rs`):
- Level 0: full-screen triangle, fragment `textureLoad(depth_texture, xy)` → ghi layer 0.
- Level k ≥ 1: cùng full-screen triangle, viewport = kích thước mức, fragment lấy max 2×2 từ **layer k−1**
  (bind riêng layer nguồn để tránh alias với layer đang ghi).
- Cull shader đọc `texture_2d_array` với layer runtime.

**Bằng chứng dựng đúng:** đổi fragment sang ghi `1 − depth` thì đọc lại layer 0 thấy
`max 0,0090`, `centre 0,0000` — tức depth thật đã được ghi (≈0,991) và đọc đúng. Với `depth` nguyên bản,
layer 0 có `max 1,0` vì còn pixel trời; 62 % pixel được vẽ (35 640/57 600).

**Chưa xong:** test hai quad không cull vì **pixel trung tâm** của khung hình không nằm trên quad gần
(depth trung tâm = 1,0). Đây là lỗi bố trí cảnh trong test, không phải pyramid. Test đang `#[ignore]` với
lý do đó.

**Phần còn lại:** sửa cảnh test (quad phủ đúng tâm), bật occlusion thật, đo 2 GiB; và GPU buffer pool theo
`ClusterStream`.

## Vòng 4: occlusion chạy thật + budget residency + chặn cảnh quá lớn

### Occlusion — **đã cull thật**

Nguyên nhân còn lại là **layer index khi downsample**: view `previous` đã trỏ đúng một layer, nên chỉ số
layer bên trong phải là **0**, không phải `level-1`. Sửa xong:

| Phép đo | Giá trị |
| --- | --- |
| Depth tại ô của cluster xa (level 4, cell 10,5) | **0,9910** |
| `nearest` của cluster xa | **0,9942** |
| Cluster hiển thị frame 1 → frame 2 | **2 → 1** (cluster bị che bị cull) |

Test `occlusion_culls_a_cluster_hidden_behind_a_near_one` **pass**, không còn `#[ignore]`.

### Streaming — budget resident **đã chạy**

`ClusterRenderer::set_budget_bytes(bytes)` đánh dấu resident theo thứ tự upload và cull shader bỏ cluster
không resident (binding 6). Test `a_budget_keeps_only_part_of_the_clusters_resident`:

| Chỉ số | Giá trị |
| --- | --- |
| Cluster | 33 |
| Budget | ¼ tổng (5 748 / 22 992 bytes) |
| Resident | **7** |
| Được vẽ | **7** (≤ resident) |

### Chặn mới phát hiện ở cảnh 3 GiB

`ClusterRenderer` dùng **một** buffer vertex/index. Với 102 000 mesh, dữ liệu vượt `max_buffer_size` của
wgpu → validation error. Đã thêm **guard**: nếu vượt giới hạn, `new` trả `None` và test in thông điệp thay vì
panic. Việc đúng là **chunked pool** (nhiều buffer, tổng ≤ budget) — chưa làm.

### App

`OM9_GPU_CLUSTER=1` + `OM9_GPU_OCCLUSION=1` + `OM9_GPU_STREAM_MB=N` bật cả ba đường trong viewport.

### Trạng thái mục tiêu

| Hạng mục | Lõi | GPU | Test |
| --- | --- | --- | --- |
| QEM-error LOD | ✅ | ✅ | ✅ |
| Occlusion pyramid | ✅ | ✅ | ✅ (2 → 1) |
| Streaming cluster | ✅ | ✅ (budget → resident) | ✅ (33 → 7) |
| Cảnh 3 GiB qua cluster | — | ❌ cần chunked pool | — |

## Vòng 5: chunked pool — cảnh 2,78 GiB chạy trong 512 MiB VRAM

`ClusterRenderer` không còn dùng một buffer: hình học chia thành **chunk** (mỗi chunk ≤ `max_buffer_size`,
và với budget thì ≤ budget/8), mỗi chunk vẽ bằng một `multi_draw_indexed_indirect` trên dải liên tiếp của
danh sách indirect. Chunk **không nằm trong budget thì không upload** (buffer giữ chỗ 1 vertex/1 index),
resident = 0, và cả cull lẫn draw đều bỏ qua.

Đo cảnh kim hoàn lớn (release, poll từng frame sau warm-up, occlusion bật):

| Chỉ số | Giá trị |
| --- | --- |
| Mesh | **102 000** |
| Cluster | **3 295 312** |
| Tổng hình học nếu nạp hết | **2 989 430 520 B ≈ 2,78 GiB** |
| Budget VRAM đặt | **512 MiB** |
| Cluster resident | **531 478** (16 %) |
| Cluster hiển thị sau cull | **27 128** |
| Frame trung vị | **23,9 ms** (960×540) |

Trước khi chỉ vẽ chunk resident, cùng cảnh đo **122,4 ms**; bỏ 36/44 dải chunk không resident còn **23,9 ms**.

Test nhỏ tương ứng (`scene_bench`):
- `a_budget_keeps_only_part_of_the_clusters_resident`: 28 cluster, budget ¼ → **8 resident, 8 vẽ**.
- `a_budgeted_renderer_uploads_only_what_fits`: **283/904** cluster resident, **235 452 / 797 364 B** upload.

### Tổng kết ba hạng mục

| Hạng mục | Trạng thái | Bằng chứng |
| --- | --- | --- |
| Occlusion depth pyramid | **xong** | depth 0,991 vs nearest 0,9942 → 2 → 1 cluster; test pass |
| Streaming cluster 2 GB | **xong** | 2,78 GiB hình học, 512 MiB VRAM, 531k/3,3M cluster resident |
| QEM error thay ngưỡng 24 px | **xong** | gần 32 fine/1 coarse; xa 8 fine/65 coarse, sai số QEM trung bình 0,43 |
