# Jewelry scene — 3,15 GiB, 102 000 mesh, 4 view

Ngày: 2026-10-18. Máy: RTX 3060 (Vulkan), 64 GiB RAM (50 GiB trống), release, 960×540/view.
Test: `crates/om9-3dm/tests/jewelry_scene.rs` với `OM9_JEWELRY_RINGS=90000 OM9_JEWELRY_STONES=12000
OM9_JEWELRY_U=48 OM9_JEWELRY_V=16 OM9_JEWELRY_FRAMES=4`.

## Kích thước và thời gian

| Bước | Kết quả |
| --- | --- |
| Ghi file 3DM | **172,2 s** (3 152,7 MiB, 102 000 object) |
| Import (đọc + dựng `Mesh`) | **44,7 s** (~70 MB/s, đọc 2 lượt) |
| Upload vào arena (chunk) | **2,9 s** |
| Vẽ 4 view × 4 frame | xem bảng dưới |

## Vẽ (mỗi view 4 frame, camera xoay 3°/frame, zoom xen kẽ)

| View | ms/frame | Ghi chú |
| --- | --- | --- |
| 0 | **85,58** | frame đầu: tạo chunk + pipeline lần đầu |
| 1 | **6,97** | 22 482 object hiển thị, 22 482 draw |
| 2 | **0,96** | góc nhìn chỉ thấy ít object |
| 3 | **8,59** | 22 482 draw |

Culling: 78 790 object ngoài frustum, **728 bị bỏ vì < 0,5 px**.

## Kết luận

- **Giới hạn là draw call**, không phải băng thông: ~22 000 draw ≈ 7–9 ms, tức ~0,35 µs/draw.
- 4 view cùng lúc ≈ **25–30 ms** → ~35 FPS, chưa đạt 60 FPS với 2 GB mesh rời.
- Bộ nhớ: import đỉnh 102k × 833 ≈ 85M đỉnh (≈ 5 GB tạm), GPU ≈ 1 GB đỉnh + 1,9 GB index — vừa 12 GB VRAM.
- Frame đầu 85 ms là chi phí khởi tạo, không phải chi phí mỗi frame.

## Cách sửa (đã xác định, chưa làm)

1. **Gộp mesh tĩnh**: hình học import không có transform riêng, nên gộp index của tất cả object trong một
   chunk thành **một draw** — 22 000 draw → ~10 draw (số chunk).
2. **Multi-draw indirect** nếu vẫn cần nhiều object động.
3. LOD proxy cho object nhỏ thay vì bỏ hẳn.

## LOD theo sai số chiếu lên màn hình (đo, 703 MiB / 24 000 object)

Quy tắc khoa học: vertex clustering có sai số tối đa `cell·√3/2`; chiếu lên màn hình:

```
error_px = cell·√3/2 · height / (distance · tan(fov/2))
```

Chọn cell lớn nhất sao cho `error_px ≤ 1`. Quét cell theo bán kính mảnh (24 000 mảnh, 960×540, camera cách 260 mm):

| cell | Tam giác | Draw | ms/frame | Sai số |
| --- | --- | --- | --- | --- |
| 0 (đầy đủ) | 30 848 000 | 4 | **17,96** | 0 |
| 0,02 r | 30 848 000 | 4 | 18,05 | 0,10 px |
| 0,05 r | 30 331 596 | 4 | 18,02 | 0,25 px |
| 0,10 r | 13 449 680 | 4 | **13,23** | 0,49 px |
| 0,20 r | 2 836 718 | 4 | **10,68** | 0,99 px |

- Ở ngưỡng 1 px, LOD giảm **30,8M → 2,84M tam giác** và **17,96 → 10,68 ms** mỗi view.
- 4 view vẫn ≈ **43 ms (~23 FPS)**: nút thắt còn lại là **tam giác quá nhỏ** (2,8M tri / 518k px ≈ 5,4 tri/px)
  — GPU không dùng hết được một pixel quad cho mỗi tam giác.
- Bước tối ưu khoa học tiếp theo: **dynamic resolution** (giảm độ phân giải khi đang xoay/zoom, nâng lại khi
  đứng yên) và/hoặc nới ngưỡng sai số lên 2 px cho chế độ điều hướng.

## Dynamic resolution khi điều hướng (đo)

| Chế độ | Ngưỡng sai số | Target | cell | Tam giác | Draw | ms/frame |
| --- | --- | --- | --- | --- | --- | --- |
| Đứng yên (idle) | 1 px | 960×540 | 0,054 r | 30 848 000 | 4 | 12,79 |
| Điều hướng | 2 px | 960×540 | 0,108 r | 22 817 280 | 4 | 11,42 |
| **Điều hướng** | **2 px** | **480×270** | **0,216 r** | **8 084 474** | **2** | **8,43** |

4 view × 8,43 ms ≈ **34 ms (~30 FPS)**; một view 8,4 ms ≈ **119 FPS**.

## Vì sao chưa đạt 60 FPS cho 4 view

- Ở 480×270 có 130k pixel, 8,08M tam giác ⇒ **62 tam giác/pixel** — vẫn quá dày.
- Vertex clustering bị giới hạn tỉ lệ: muốn torus còn ~30 tam giác thì cell phải lớn hơn cả mảnh, làm mất hình
  dạng. Đây là hạn chế đã biết của clustering (Rossignac–Borrel), không phải lỗi cài đặt.
- Kỹ thuật đúng cho tỉ lệ giảm lớn là **QEM edge-collapse** (Garland–Heckbert 1997) hoặc **progressive
  meshes** (Hoppe 1996): nó tối ưu vị trí đỉnh mới và giữ hình dạng ở 100:1, điều clustering không làm được.
- Kết hợp chuẩn trong công nghiệp: QEM chain + chọn mức theo **ngân sách tam giác/pixel** (ví dụ ≤ 2 tri/px),
  không chỉ theo sai số hình học.

## Đề xuất bước tiếp (theo thứ tự tác động)

1. **QEM decimation** sinh chain 100 % / 25 % / 6 % / 1,5 % cho mỗi mảnh; chọn mức theo budget
   `triangles ≤ 2 × pixels`.
2. Dynamic resolution (đã có số): 0,5× khi điều hướng, nâng lại khi đứng yên.
3. Nếu vẫn cần: cluster culling trên GPU (Nanite-style) cho cảnh cực lớn.
