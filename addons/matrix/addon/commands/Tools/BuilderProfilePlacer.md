# Profile Placer

Đặt các profile kín lên ring rail (đỉnh, hai bên, đáy) để quét Sweep.

- **Mã lệnh:** `BuilderProfilePlacer`
- **Vị trí:** menu Tools, ô số 2
- **Icon:** `Resources/icons/tools-minimal/BuilderProfilePlacer.svg`
- **Gọi bằng dòng lệnh:** `BuilderProfilePlacer`, `ProfilePlacer`, `gvProfilePlacer`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn ring rail (hoặc dùng cỡ).
2. Chỉnh kiểu, rộng/cao từng vị trí.
3. Enter.

## Tuỳ chọn (lệnh phụ)


| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `SizeSystem=` | Hệ cỡ nhẫn | us | us, iso, diameter |
| `RingSize=` | Cỡ nhẫn | 7 | 0 … 80 |
| `Shape=` | Kiểu profile | comfort | flat, d, comfort, knife, round |
| `Count=` | Số profile (3 hoặc 5) | 3 | 3 … 5 |
| `TopWidth=` | Rộng đỉnh | 4 | 0.5 … 30 |
| `TopHeight=` | Cao đỉnh | 2 | 0.2 … 20 |
| `SideWidth=` | Rộng bên | 3 | 0.5 … 30 |
| `SideHeight=` | Cao bên | 1.8 | 0.2 … 20 |
| `BottomWidth=` | Rộng đáy | 2.5 | 0.5 … 30 |
| `BottomHeight=` | Cao đáy | 1.5 | 0.2 … 20 |
