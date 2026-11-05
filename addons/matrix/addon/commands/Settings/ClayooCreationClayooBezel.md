# SubD Bezel

Tạo bezel mềm (thành bo tròn) quanh đá; tự cắt phần chạm finger rail nếu chọn rail.

- **Mã lệnh:** `ClayooCreationClayooBezel`
- **Vị trí:** menu Clayoo, ô số 11
- **Icon:** `Resources/icons/subd-minimal/ClayooCreationClayooBezel.svg`
- **Gọi bằng dòng lệnh:** `ClayooCreationClayooBezel`, `ClayooBezel`, `SubDBezel`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn đá (và finger rail nếu muốn cắt).
2. Chỉnh cao, dày trên/dưới, góc, vị trí.
3. Enter.

## Tuỳ chọn (lệnh phụ)

- Boolean: cắt theo finger rail.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Height=` | Cao bezel (dưới gờ) | 2 | 0.2 … 20 |
| `TopThickness=` | Dày đỉnh | 0.5 | 0.1 … 5 |
| `BaseThickness=` | Dày chân | 0.6 | 0.1 … 5 |
| `Angle=` | Góc thành | 10 | -45 … 45 |
| `Placement=` | Vị trí lên/xuống | 0 | -5 … 5 |
| `Scale=` | Cỡ ổ đá | 100 | 50 … 150 |
| `Lip=` | Gờ trên mặt đá | 0.3 | 0 … 3 |
| `Boolean=` | Cắt theo finger rail | Yes |  |
