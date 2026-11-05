# Auto Pavé

Lát pavé tự động trong đường cong kín (lưới lục giác), viên nhỏ dần ở mép, bám mặt nếu chọn.

- **Mã lệnh:** `BuilderPaveDialog`
- **Vị trí:** menu Gems, ô số 21
- **Icon:** `Resources/icons/gems-minimal/BuilderPaveDialog.svg`
- **Gọi bằng dòng lệnh:** `BuilderPaveDialog`, `PaveDialog`, `AutoPave`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn đường cong kín (và mặt cần lát).
2. Chỉnh kích thước, tối thiểu, khoảng cách, lề, góc.
3. Enter.

## Tuỳ chọn (lệnh phụ)


| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Size=` | Kích thước đá | 1.2 | 0.3 … 10 |
| `MinSize=` | Kích thước nhỏ nhất | 0.8 | 0.3 … 10 |
| `Spacing=` | Khe giữa đá | 0.15 | 0 … 3 |
| `Margin=` | Lề tới đường cong | 0.3 | 0 … 5 |
| `Angle=` | Góc lưới | 0 | -90 … 90 |
| `ZOffset=` | Lệch đứng | 0 | -5 … 5 |
| `Material=` | Chất liệu | diamond | diamond, moissanite, cz, ruby, sapphire, emerald, amethyst, topaz, garnet, aquamarine, peridot, tanzanite |
