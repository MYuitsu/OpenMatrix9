## Cách dùng

1. Nhấn Ring Rail trong Top11 để mở thanh dưới Projects. Thanh mới hiển thị USA cỡ 7. Nhấn tên vùng, chọn vùng rồi nhấn nút xanh cạnh Set Region.
2. Chọn cỡ nhẫn rồi nhấn tam giác xanh trên thanh chính để tạo vòng trong mặt Front. Undo gỡ vòng vừa tạo. Dấu × đóng thanh.
3. USA dùng bảng Stuller từ cỡ 1 đến 15 theo bước 0.25. Britain, Japan, Swiss, European, France và German dùng cỡ theo vùng. VNM nhận đường kính trong hoặc chu vi trong bằng mm. Custom Ring Size hỗ trợ cả mm và inch.
4. Trong Tools và CMD, mặc định vẫn là VNM 18 mm. CenterX, CenterY và CenterZ đặt tâm theo mm. Enter xác nhận và Esc hủy.

## Tuỳ chọn (lệnh phụ)


| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `SizeSystem=` | Vùng hoặc kích thước tùy chỉnh | vnm | vnm (đường kính mm), vnm_circumference (chu vi mm), us, britain, japan, swiss, european, france, german, iso (chu vi mm), diameter (mm), diameter_inches, circumference_inches |
| `RingSize=` | Cỡ/kích thước theo hệ đã chọn | 18 | Theo vùng; Japan có cỡ 0, US 1–15 theo bước 0.25 |
| `CenterX=`, `CenterY=`, `CenterZ=` | Tâm rail theo mm | 0 | −1000000 … 1000000 |

VNM là chế độ nhập kích thước thực bằng mm, chưa phải bảng ni số Việt Nam.

Nguồn US: [Stuller, Ring Blank Lengths Chart](https://www.stuller.com/articles/view/determining-ring-blank-lengths-in-b-and-s-gauge/). Chu vi hình học tính từ đường kính nguồn theo C = πD.
