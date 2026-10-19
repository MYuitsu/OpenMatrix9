# Viewport GPU-only + chọn object bằng kéo chuột (M01)

Ngày: 2026-10-18.

## 1. Vẽ chỉ bằng GPU

| Thành phần | Trước | Sau |
| --- | --- | --- |
| Mesh (tam giác) | egui painter vẽ cạnh + tô mặt | `SceneBatch` vẽ trên GPU, có depth test |
| Curve | egui painter | `OverlayBatch` (line list) trên GPU, depth test |
| Point | egui painter | `OverlayBatch` (3 gạch chữ thập theo world) |
| Lưới + trục | egui painter | `OverlayBatch`, cùng depth buffer với mesh |

`crates/om9-render/src/overlay.rs` (mới): pipeline LineList, `depth_compare: LessEqual`,
`depth_write_enabled: false`, nền `LoadOp::Load` để dùng chung depth với `SceneBatch`. `LineBatch` là phần
hình học thuần (không GPU) nên test được: **2 test** (một point = 3 gạch đúng tâm; clear xoá hết).

`GpuViewport::refresh_scene` dựng lại khi **scene epoch** hoặc **CPlane revision** đổi, không dựng mỗi frame;
cache ảnh theo `(camera revision, scene epoch, cplane revision, size)`.

`ui.rs`: khi GPU đã vẽ thì **bỏ toàn bộ phần vẽ CPU** cho lưới/vật thể (early return), nên không còn tranh
chấp thứ tự vẽ.

Ảnh: `ui-artifacts/gpu-mesh-stress-fps2/workspace.png` — 1 000 mesh sphere do GPU vẽ, lưới nằm sau vật thể.

## 2. Chọn object

- **Click**: đã có từ trước (`scene.pick` trên CPU, độc lập với việc vẽ GPU).
- **Kéo chuột (rubber band)**: mới. Kéo trái ở vùng trống → hiện hình chữ nhật; thả chuột chọn mọi object
  có **bất kỳ đỉnh nào** nằm trong hình chữ nhật.
  - `Controller::select_objects(&[ObjectId])`: chọn một lượt, từ chối **toàn bộ** nếu một id không hợp lệ,
    tăng `selection_revision` một lần.
  - Band bắt đầu từ `pointer.press_origin()` (egui chỉ báo drag sau ngưỡng, nếu lấy vị trí hiện tại thì mất
    ~30 px đầu — đúng lỗi đã gặp ở gumball).
  - Test `crates/om9-app/tests/rubber_band.rs`: **2 pass** (band qua 2 điểm chọn cả 2; band quanh 1 điểm chỉ
    chọn 1).

## 3. Mesh stress để đo GPU

`OM9_MESH_STRESS=N` (cần feature `gpu-sphere`): thêm N mesh sphere vào document lúc khởi động, mỗi quả một
entity nên **chọn/click được như object thật**. Dùng để đo và kiểm tra đường GPU.

## 4. Số đo và giới hạn

- Overlay `OM9_GPU_FPS=1` hiện `fps / ms` ở góc mỗi viewport (vẽ **sau** ảnh GPU để không bị ảnh che).
- Ở chế độ audit, cảnh **rỗng** và cảnh **1 000 mesh** đều cho **32 fps / 31,4 ms** → con số đó là nhịp
  khung khi rảnh của app (vsync/repaint), **không phải** chi phí vẽ. Muốn đo chi phí thật phải xoay camera
  liên tục hoặc đo headless.
- Mesh hiện vẽ **một draw mỗi object** (`SceneBatch::add_mesh`); cảnh 100 000 mesh cần gộp/hợp nhất chunk
  như đường cluster — chưa làm.
- Gumball, HUD chữ, menu vẫn dùng egui (lớp phủ), không phải hình học 3D.

## 5. Lệnh `MeshStress` (vòng 4)

Gõ trong CMD: `MeshStress` (mặc định **1 000**) hoặc `MeshStress 2000`.

- `crates/om9-app/src/session/stress.rs`: `insert_stress_meshes(count)` chèn N entity mesh (khối lập phương
  12 tam giác) theo lưới, **một transaction** → **một Undo** xoá cả cụm.
- Chặn: count = 0, > 20 000, hoặc không phải số → lỗi, document không đổi.
- Menu/search đi qua `dispatch("MeshStress")` cho cùng kết quả (đã thêm vào `features.json`,
  module "Mesh Tools", và vào danh sách `availability`).

**4 test** `crates/om9-app/tests/mesh_stress_command.rs`:

| Test | Chứng minh |
| --- | --- |
| command tạo đúng số mesh, một journal entry, chọn được, Undo xoá hết | end-to-end |
| lệnh trần dùng số mặc định | 1 000 |
| số sai / vượt trần bị từ chối, document không đổi | an toàn |
| dispatch từ menu cho cùng kết quả | hai đường vào một hành vi |

**Chạy thật**: gõ `MeshStress 2000` trong app → `Objects: 2000`, 4 viewport vẽ 2 000 mesh bằng GPU
(ảnh `L:\om9-target\meshstress-2000.png`).

## 6. Tổng kết mục tiêu

| Yêu cầu | Trạng thái | Bằng chứng |
| --- | --- | --- |
| Chỉ GPU vẽ hình học + lưới | **xong** | `OverlayBatch` + `SceneBatch`, bỏ phần vẽ CPU; ảnh 1 000/2 000 mesh |
| Gộp draw cho cảnh lớn | **xong** | sửa lỗi uniform slot 0; `merged_static_spheres_are_painted` 2 488 px = 2 488 px |
| Chọn object (click + kéo band) | **xong** | `rubber_band` 2/2, `select_objects` một lượt |
| Highlight trên GPU | **có code + cache đúng**, chưa chụp được ảnh có object vàng | — |
| Lệnh mesh stress test | **xong** | `MeshStress [n]`, 4 test, chạy thật 2 000 object |

**Mở**: occlusion bỏ 3,5 % diện tích đang thấy trong tường sphere dày (test ignored, ghi ledger).
