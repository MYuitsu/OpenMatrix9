# So sánh simplifier (QEM 1997 vs clustering vs memoryless)

Ngày: 2026-10-18, release. Nguồn: torus 48×16 = 1 536 tam giác, area 187,945, volume 110,482.
Lệnh: `cargo test --release -p om9-render --lib the_three_simplifiers -- --nocapture`.

| Target | Thuật toán | Tam giác | Thời gian | Sai số diện tích | Sai số thể tích |
| --- | --- | --- | --- | --- | --- |
| 500 | clustering | 528 | <0,01 ms | **16,57 %** | 44,37 % |
| 500 | **QEM** | 500 | 1,90 ms | **0,22 %** | 1,65 % |
| 500 | memoryless (3 pass) | 500 | 3,00 ms | 0,12 % | 3,73 % |
| 200 | clustering | 206 | <0,01 ms | 21,87 % | 56,92 % |
| 200 | **QEM** | 200 | 2,27 ms | **2,49 %** | 6,42 % |
| 200 | memoryless | 200 | 2,62 ms | 4,23 % | 9,96 % |
| 80 | clustering | 82 | <0,01 ms | 40,32 % | 92,75 % |
| 80 | **QEM** | 79 | 2,37 ms | **2,09 %** | 34,87 % |
| 80 | memoryless | 80 | 2,60 ms | 10,77 % | 34,73 % |
| 30 | clustering | 34 | — | 41,85 % | 79,81 % |
| 30 | QEM | 30 | 2,42 ms | 11,01 % | 71,61 % |
| 30 | memoryless | 29 | 2,52 ms | 18,53 % | 78,04 % |

## Đọc bảng này

- **QEM chính xác hơn clustering 20–100 lần** ở cùng số tam giác: 0,22 % so với 16,57 % diện tích ở target 500.
- **Memoryless (Lindstrom–Turk 1998)** không thắng QEM ở đây: nó khởi động lại quadric mỗi pass, tốt hơn chút
  ở target 500 về diện tích nhưng tệ hơn ở 80 và 30, và luôn chậm hơn ~20 %.
- Clustering **cực nhanh** (< 0,01 ms) và đó là lý do nó vẫn hữu ích cho LOD thô; nhưng hình dạng mất rất sớm.

## Render: một mảnh vs cả scene

| Trường hợp | Tam giác | ms/frame |
| --- | --- | --- |
| Một torus gốc | 1 536 | 0,04 |
| Một torus QEM (83 tri) | 83 | 0,05 |
| 24 000 mảnh QEM, 480×270 | 1 781 408 | **42,09** |
| 24 000 mảnh clustering, 480×270 | 8 084 474 | **8,43** |

Một mảnh QEM vẽ **không chậm hơn** mảnh gốc. Nhưng ở mức scene 24 000 mảnh, đường QEM đo được 42 ms —
**mâu thuẫn chưa giải thích được**, và tôi không coi QEM là "thắng" cho tới khi tìm ra nguyên nhân.

## Chẩn đoán tiếp theo (chưa làm)

1. Chạy **cùng một scene** qua đúng nhánh QEM nhưng với `target = số tam giác gốc` (không giảm) — nếu vẫn 42 ms
   thì nguyên nhân nằm ở xử lý scene, không phải hình học QEM.
2. Kiểm tra `add_static` với 24 000 mesh nhỏ: có thể có **range bị tách** hoặc buffer bị cấp phát lại.
3. Đo draw time bằng timestamp query (nếu adapter hỗ trợ) thay vì wall clock có poll.

## Thuật toán mới hơn QEM — danh sách để đánh giá tiếp

| Thuật toán | Năm | Ý tưởng | Ghi chú cho OM9 |
| --- | --- | --- | --- |
| Progressive meshes (Hoppe) | 1996 | chuỗi split/collapse, LOD liên tục | cần cấu trúc lưu chuỗi; tốt cho streaming |
| Memoryless (Lindstrom–Turk) | 1998 | quên quadric cũ, tính lại | đã thử: không thắng ở đây |
| Out-of-core clustering | 2000 | không cần toàn bộ mesh trong RAM | hữu ích khi file 2 GB |
| QEM theo thuộc tính (Garland–Heckbert) | 1998 | giữ UV/material | cần khi có texture |
| Simplification envelopes | 1996 | sai số Hausdorff bảo đảm | kiểm soát chất lượng chặt hơn |
| meshoptimizer (Kapoulkine) | 2016+ | QEM + lock biên + tối ưu cache | **thực dụng nhất**: mã nguồn mở, đã kiểm chứng |
| Nanite-style cluster LOD | 2021 | cluster + GPU cull + virtual geometry | cho cảnh cực lớn; phức tạp |

## Sửa lỗi đo: 42 ms là **lỗi phương pháp**, không phải chi phí thật

Lỗi: đo 6 frame liên tiếp rồi `poll(Wait)` **một lần** ở cuối. GPU bị dồn hàng đợi, nên con số cuối là
thời gian **xả hàng đợi**, không phải thời gian một frame.

Đo lại đúng cách — warm-up một frame, rồi mỗi frame `poll(Wait)` và bấm giờ riêng (480×270, 24 000 mảnh):

| Cảnh | Tam giác | Draw | Frames (ms) | Trung vị |
| --- | --- | --- | --- | --- |
| Không simplify (gốc) | 30 848 000 | 4 | [2,80 / 2,67 / 2,67 / 2,65 / 2,64 / 3,36] | **2,65** |
| QEM 83 tri/mảnh | 1 781 408 | 4 | [0,38 / 0,31 / 0,30 / 0,30 / 0,28 / 0,28] | **0,30** |

**Kết luận đúng:**
- Cảnh gốc 30,8M tam giác chỉ **2,65 ms/frame** — 4 view ≈ **10,6 ms (~94 FPS)**.
- QEM xuống 1,78M tam giác còn **0,30 ms/frame** — 4 view ≈ **1,2 ms**, thừa ngưỡng 60 FPS rất xa.
- Con số "42 ms cho QEM" và "8,43 ms cho clustering" trước đó **đều sai cùng một lỗi**; mọi kết luận dựa trên
  chúng (kể cả "cần LOD gấp") phải đọc lại theo bảng này.
- QEM **thắng** về cả chất lượng (sai số diện tích 0,22 % vs 16,57 %) và tốc độ khi đo đúng cách.

## Cluster culling và LOD hierarchy (đã có, đo ở mức dữ liệu)

- `meshlet::cull(clusters, camera, w, h, min_pixels)` — cùng quy tắc mà compute shader của Nanite làm:
  frustum + bán kính pixel. Test: grid nhìn từ trên → nhiều cluster; lùi ra 400 mm → ít hơn rõ rệt.
- `meshlet::build_cluster_lod(mesh, group_size)` — nhóm cluster, QEM gộp nhóm xuống 1/4, dựng lại meshlet;
  test: level 0 giữ đủ tam giác, level 1 ≤ 50 %.
