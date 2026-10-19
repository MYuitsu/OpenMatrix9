# Sculpt readiness — nhiều mesh và tiến tới sculpt kiểu ZBrush

Mục tiêu: viewport chịu được **nhiều mesh** ngay bây giờ, và có đường rõ ràng để sau này sculpt mật độ cao
(5–20 triệu tam giác) mà không phải viết lại kiến trúc.

## 1. Đã đo (release, RTX 3060, 960×540)

| Cảnh | Tam giác trong arena | Hiển thị | ms/khung |
| --- | --- | --- | --- |
| 2000 cube (12 tri/object) | 24 000 | 1 673 | **1,04** |
| 200 sphere (5008 tri/object) | **1 001 600** | 71 (cull 129) | **0,31** |
| 400 sphere (5008 tri/object) | **2 003 200** | 86 (cull 314) | **0,32** |

Ghi chú trung thực: hai dòng sphere bị **frustum culling** cắt phần lớn object, nên số tam giác *được vẽ*
nhỏ hơn số tam giác *trong arena*. Muốn đo thông lượng thô cần tắt culling — việc đó chưa làm.

## 2. Sculpt khác gì "nhiều object"

| Khía cạnh | Nhiều object (hiện tại) | Sculpt |
| --- | --- | --- |
| Đơn vị nặng | 2000 object × ít tam giác | **1 object × 5–20 triệu tam giác** |
| Cập nhật mỗi frame | đổi transform | **đổi hàng nghìn đỉnh** trong một object |
| Upload | một lần khi thêm mesh | **từng phần** (dirty range) mỗi nét cọ |
| Chọn điểm | ray vs mesh (tương lai) | **BVH** + bán kính cọ, chạy song song |
| Hình học | tĩnh | **subdivision động** (thêm đỉnh khi cần) |
| Undo | transaction | **delta theo tile đỉnh** |

Kiến trúc hiện tại đã đủ cho phần lớn khía cạnh này: arena chung, `update_vertices(object, first, &positions)`
chỉ ghi đúng dải đỉnh bị sửa, và một draw call/object.

## 3. Khoảng trống và thứ tự lấp

| # | Việc | Vì sao cần | Chi phí |
| --- | --- | --- | --- |
| 1 | **BVH theo object** cho ray/brush | brush cần "đỉnh trong bán kính" trong vài ms trên 5M đỉnh | 1 phiên |
| 2 | **Staging belt + write_buffer theo nét** | mỗi nét cọ 50–500 KB; hiện ghi trực tiếp từng dải là đủ nhưng nên gom lô | 0,5 phiên |
| 3 | **Undo delta theo dải đỉnh** | 5M đỉnh không thể snapshot mỗi nét | 1 phiên |
| 4 | **Normals cache + cập nhật cục bộ** | shading cần normal đúng sau khi kéo đỉnh | 0,5 phiên |
| 5 | **Subdivision động** | ZBrush tăng mật độ tại nét cọ | 2–3 phiên |
| 6 | **LOD / proxy** | nhiều object sculpt cùng lúc | 1 phiên |
| 7 | **Multi-draw indirect** | chỉ cần khi > ~5000 lệnh | 1 phiên |
| 8 | **Displacement bằng texture (tuỳ chọn)** | preview mượt khi mật độ rất cao | 1–2 phiên |

## 4. Ngân sách đề ra

| Hạng mục | Mục tiêu |
| --- | --- |
| Một object sculpt | 5–10 triệu tam giác |
| Tổng cảnh | 20–30 triệu tam giác |
| Bộ nhớ | position f16 6 B/đỉnh + index u32 4 B → 10M đỉnh ≈ 100 MB |
| Nét cọ | < 8 ms/khung, upload < 1 MB/nét |
| BVH query bán kính | < 2 ms cho 5M đỉnh (song song) |

## 5. Điều **không** làm

- Không dùng CPU tessellation mỗi khung: sculpt sẽ giật; mọi thứ tĩnh phải ở arena GPU.
- Không cấp phát buffer mỗi nét: chỉ `write_buffer` vào arena.
- Không chuyển toàn bộ mesh mỗi nét: chỉ dải đỉnh bị sửa.
- Không dựa vào egui để vẽ hình học: egui chỉ overlay (chữ, con trỏ cọ).
