# Prong Editor

Sửa một chấu: đường kính, cao, vát, bo đầu, nghiêng.

- **Mã lệnh:** `BuilderProngEditor`
- **Vị trí:** menu Settings, ô số 7
- **Icon:** `Resources/icons/settings-minimal/BuilderProngEditor.svg`
- **Gọi bằng dòng lệnh:** `BuilderProngEditor`, `ProngEditor`, `EditSingleProng`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn chấu.
2. Chỉnh trong bảng BUILDERS.
3. Enter.

## Tuỳ chọn (lệnh phụ)


| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Diameter=` | Đường kính (0 = giữ) | 0 | 0 … 5 |
| `Height=` | Cao thêm | 0 | -5 … 5 |
| `Taper=` | Vát (rộng chân) | 0 | -50 … 100 |
| `Fillet=` | Bo đầu | 50 | 0 … 100 |
| `Rotate=` | Nghiêng | 0 | -60 … 60 |
