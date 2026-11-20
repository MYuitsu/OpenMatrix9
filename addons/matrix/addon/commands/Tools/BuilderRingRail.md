# Ring Rail

Đặt ring rail (vòng ngón tay) theo cỡ nhẫn ở Front view.

- **Mã lệnh:** `BuilderRingRail`
- **Vị trí:** menu Tools, ô số 1
- **Icon:** `Resources/icons/tools-minimal/BuilderRingRail.svg`
- **Gọi bằng dòng lệnh:** `BuilderRingRail`, `RingRail`, `gvRingRail`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Mặc định VNM (Việt Nam): nhập đường kính trong bằng mm; có lựa chọn VNM chu vi mm. US nhận cỡ 1–15 theo bước 0.25, dùng bảng nửa cỡ Stuller và nội suy các cỡ ¼, làm tròn đường kính đến 0.01 mm.
2. Chọn tâm đặt qua CenterX/CenterY/CenterZ (mm); preview nằm trên mặt Front qua tâm này.
3. Enter để tạo trong một bước Undo; Esc để hủy.

## Tuỳ chọn (lệnh phụ)


| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `SizeSystem=` | VNM, US hoặc kích thước tùy chỉnh | vnm | vnm (đường kính mm), vnm_circumference (chu vi mm), us, iso (chu vi mm), diameter (mm), diameter_inches, circumference_inches |
| `RingSize=` | Cỡ/kích thước theo hệ đã chọn | 18 | Dương; US 1–15 theo bước 0.25 |
| `CenterX=`, `CenterY=`, `CenterZ=` | Tâm rail theo mm | 0 | −1000000 … 1000000 |

VNM là chế độ nhập kích thước thực bằng mm, chưa phải bảng ni số Việt Nam. Region khác, custom shape/library và workflow hỏi thay rail còn chưa hoàn thiện.

Nguồn US: [Stuller, Ring Blank Lengths Chart](https://www.stuller.com/articles/view/determining-ring-blank-lengths-in-b-and-s-gauge/). Chu vi hình học tính từ đường kính nguồn theo C = πD.
