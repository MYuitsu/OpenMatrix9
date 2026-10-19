# Viewport performance — các phương án và lựa chọn

Mục tiêu: giữ 60 FPS (16,6 ms/khung) với **1000–2000 object**, 4 viewport, trên GPU rời tầm trung
(RTX 3060 đo được ở gate). Tài liệu này liệt kê toàn bộ phương án, chi phí/lợi ích, rồi chốt kiến trúc.
Số đo thật nằm ở `crates/om9-render/tests/scene_bench.rs` và `implementation/evidence/M01/VIEWPORT-BENCH.md`.

## 1. Nút thắt, theo thứ tự thực tế

| # | Nút thắt | Ảnh hưởng khi 2000 object |
| --- | --- | --- |
| 1 | **Số draw call** (mỗi object một lượt) | 2000 lệnh/kview → CPU 3–8 ms chỉ để submit |
| 2 | **Upload mỗi frame** | 2000 buffer tạo mới = cấp phát + copy, 10–30 ms |
| 3 | **egui vẽ scene 2D** (grid + từng line) | tessellation CPU 5–20 ms, tăng theo số cạnh |
| 4 | **Overdraw / không cull** | GPU tô pixel khuất, 2–4× |
| 5 | **Readback / đồng bộ** | chặn pipeline, mất 1–2 frame |
| 6 | **Bind group per object** | 2000 lần bind, ~0,2–0,5 ms |
| 7 | **Tessellation quá mịn** | 10M+ tam giác, vượt ngân sách vertex |

## 2. Toàn bộ phương án

### A. Giảm draw call
| Phương án | Cơ chế | Đánh giá |
| --- | --- | --- |
| **Arena + per-object draw** | một vertex/index buffer chung, `draw_indexed(range, base_vertex)` | đơn giản, 1 pipeline; 2000 draw vẫn ổn trên desktop (~0,2 ms CPU) |
| **Instancing** | cùng mesh, khác transform | chỉ hợp object cùng loại (ring, prong, gem) — nên dùng cho lưới lặp |
| **Multi-draw indirect** | build `DrawIndexedIndirectArgs`, 1 lệnh `multi_draw_indexed_indirect` | nhanh nhất, cần feature `MULTI_DRAW_INDIRECT`; phức tạp hơn |
| **GPU-driven (indirect + cull trên GPU)** | compute pass đọc/ghi indirection | mạnh nhất, không cần cho 2000 object ở màn hình |

Chốt: **arena + per-object draw** làm mặc định; **instancing** cho object lặp; **multi-draw** để dành khi
vượt ~5000 lệnh.

### B. Upload
| Phương án | Đánh giá |
| --- | --- |
| Tạo buffer mỗi frame (hiện tại) | **loại** — cấp phát mỗi frame |
| **Arena cố định + upload một lần khi mesh đổi** | chốt: `add_mesh` cấp phát một lần, `update_transform` chỉ ghi uniform |
| Staging belt / `queue.write_buffer` theo lô | chốt: ghi **một** lần các uniform của object hiển thị |
| `map_async` + ring buffer cho dữ liệu lớn | dùng khi import lưới lớn, không phải mỗi frame |

### C. Culling
| Phương án | Đánh giá |
| --- | --- |
| **Frustum culling theo bounding sphere (CPU)** | chốt: vài chục ns/object, cắt 50–90% khi zoom gần |
| **Cull theo bán kính pixel** | chốt: vật < 0,5 px không vẽ; zoom ra tự bỏ, zoom vào tự hiện (`set_min_pixel_radius`) |
| Occlusion culling (GPU query) | đắt, cần depth prepass; để sau |
| Backface + small-object cull | rẻ, bật theo ngưỡng pixel |

### D. Overdraw và thứ tự
- **Depth test `Less`, ghi depth** (đang có) là đủ cho opaque.
- **Sort front-to-back** khi số object lớn: giảm overdraw, nhưng 2000 object sort ~0,1 ms — bật khi > 256.
- **Depth prepass**: chỉ có lợi khi shader fragment đắt (PBR); shader hiện tại rẻ nên bỏ.

### E. Mesh và LOD
- **Tessellation cache** theo (shape, deflection) — đã có ở `GpuMesh`/`ensure_mesh`.
- **LOD theo kích thước màn hình**: deflection thô hơn khi object nhỏ; cắt tam giác dưới 1 px.
- **Nén vertex**: f16 hoặc oct-encoded normal — tiết kiệm băng thông, chưa cần ở mức 2000.

### F. Đường vẽ 2D bằng egui
- Hiện grid + line + point của document vẽ bằng `egui::Painter` → CPU tessellation, **không scale**.
- Chốt: scene 3D (kể cả view trực giao) đi qua **cùng GPU batch**; egui chỉ còn overlay (chữ, selection box).
- Grid: dựng một lần trong batch, không vẽ lại bằng egui.

### G. Đồng bộ và khung hình
- **Không readback** (đã bỏ).
- **Chỉ repaint khi dirty**: camera revision / scene revision (đã có scheduler).
- **Vsync** (eframe mặc định) — giữ; khi đo bench thì tắt.

### H. 4 viewport
- Vẽ tuần tự 4 lượt vào 4 texture, **dùng chung batch** (một arena).
- Với 4 viewport × 2000 object = 8000 draw → vẫn dưới ngưỡng; nếu cần, chỉ render viewport có thay đổi.

## 3. Kiến trúc chốt

```
SceneBatch (một cho cả 4 view)
├─ arena: vertex buffer (grow theo lô) + index buffer
├─ objects: [(base_vertex, index_range, model matrix, tint, bounding sphere, dirty)]
├─ add_mesh(mesh) -> ObjectId        // upload một lần
├─ set_transform(id, matrix)         // chỉ đánh dấu dirty
└─ render_into(view, camera, w, h)
   ├─ frustum cull theo bounding sphere (CPU)
   ├─ sort front-to-back nếu > 256 object
   ├─ ghi uniform một lần: mvp + tint cho từng object hiển thị (stride 256)
   └─ per-object draw_indexed với dynamic offset
```

## 4. Ngân sách

| Hạng mục | Mục tiêu |
| --- | --- |
| CPU/frame (2000 object, 1 view) | < 3 ms |
| GPU/frame (2000 × ~200 tam giác) | < 4 ms |
| Bộ nhớ arena | ~ 2000 × 12 KB ≈ 24 MB |
| Draw call/view | ≤ số object hiển thị |

## 5. Việc chưa làm và lý do

- **Multi-draw indirect**: chưa cần ở 2000 object; bật khi đo thấy CPU submit > 4 ms.
- **GPU occlusion culling**: cần depth prepass + query, chỉ đáng khi cảnh nặng che khuất.
- **Bindless**: wgpu chưa cho texture array không giới hạn; không cần vì không có texture.
- **Mesh shader**: không có trong wgpu/DX12 path hiện tại.

## 6. Kết quả đo

Chi tiết ở `implementation/evidence/M01/VIEWPORT-BENCH.md`. Tóm tắt (release, 960×540):

| Object | Draw call | ms/khung |
| --- | --- | --- |
| 500 | 500 | 0,39 |
| 1000 | 972 | 0,60 |
| 2000 | 1673 (cull 327) | **1,02** |

Đạt mục tiêu: 2000 object ~1 ms, tức còn ~15 ms cho 3 viewport còn lại và mọi thứ khác.

## 7. Không có trần cố định: theo RAM trống và theo pixel

- **Import**: `ImportBudget::detect()` đọc RAM vật lý còn trống (`GlobalMemoryStatusEx`) và dùng **một nửa**
  số đó cho một lần import, sàn 64 MiB. Không còn hằng số 20M/60M; `max_vertices = bytes/108`,
  `max_corners = 6 × max_vertices`.
- **Vẽ**: vật có bán kính chiếu < **0,5 px** bị bỏ (`culled_small`), nên zoom ra ngoài thì cảnh tự nhẹ đi
  và zoom vào thì chi tiết quay lại. Đây là mức LOD thô đầu tiên; LOD theo mesh (proxy) là bước sau.

| Tình huống | Hành vi |
| --- | --- |
| Máy còn 8 GiB trống | import được ~4 GiB dữ liệu (≈ 39M đỉnh) |
| Máy còn 1 GiB trống | import ~512 MiB (≈ 4,9M đỉnh) |
| Zoom ra xa, 2000 object < 0,5 px | **0 draw**, `culled_small = 2000` |
| Zoom vào | object hiện lại, `culled_small = 0` |
