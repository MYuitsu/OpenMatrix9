# Text

Tạo chữ từ phông TrueType/OpenType đã cài: đường viền, mặt phẳng hoặc khối (như TextObject của Rhino 5).

- **Mã lệnh:** `SolidText`
- **Vị trí:** menu Solid, ô số 8
- **Icon:** `Resources/icons/solid-minimal/SolidText.svg`
- **Gọi bằng dòng lệnh:** `SolidText`, `TextObject`
- **Cách chạy:** lệnh có sẵn của OM9 (khối chính xác bằng lõi B-Rep); bản không có lõi dùng lệnh của add-on

## Cách dùng

1. Chọn tuỳ chọn ở bảng Text: chữ, phông (Bold/Italic), tạo Curves/Surfaces/Solids, cao, dày.
2. Bấm điểm đặt chữ (xem trước chạy theo chuột).

## Tuỳ chọn (lệnh phụ)

- Text: nội dung.
- Font, FontName, Bold, Italic: phông chữ đã cài trong Windows.
- Create: Curves / Surfaces / Solids.
- Height: cao chữ hoa.
- SolidThickness: dày khối.
- Spacing, SmallCaps: khoảng cách, chữ thường thành hoa nhỏ.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Text=` | Chữ cần tạo (\n: xuống dòng) | OM9 |  |
| `Font=` | Phông chữ (Font) | Arial | Arial, Arial Black, Bahnschrift, Calibri, Cambria, Candara, Comic Sans MS, Consolas, Constantia, Corbel, Courier New, Franklin Gothic Medium, Gabriola, Georgia, Impact, Lucida Console, Palatino Linotype, Segoe Print, Segoe Script, Segoe UI, Tahoma, Times New Roman, Trebuchet MS, Verdana |
| `FontName=` | Phông khác đã cài (để trống: dùng ô trên) |  |  |
| `Bold=` | Đậm (Bold) | No |  |
| `Italic=` | Nghiêng (Italic) | No |  |
| `Create=` | Tạo (Create) | solids | curves, surfaces, solids |
| `Height=` | Cao chữ (Height) | 3 | 0.1 … 500 |
| `SolidThickness=` | Độ dày khối (Solid thickness) | 0.5 | 0.01 … 100 |
| `Spacing=` | Thêm khoảng cách chữ (Add spacing) | 0 | -10 … 50 |
| `SmallCaps=` | Chữ thường thành hoa nhỏ (Lower case as small caps) | No |  |
| `SmallCapsSize=` | Cỡ hoa nhỏ (%) | 70 | 10 … 100 |
