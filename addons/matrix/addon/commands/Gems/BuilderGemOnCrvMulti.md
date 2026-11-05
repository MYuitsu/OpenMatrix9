# Gem on Curve Multi

Rải nhiều hàng đá cùng lúc: đường thứ nhất quyết định vị trí, các đường còn lại thêm hàng.

- **Mã lệnh:** `BuilderGemOnCrvMulti`
- **Vị trí:** menu Gems, ô số 6
- **Icon:** `Resources/icons/gems-minimal/BuilderGemOnCrvMulti.svg`
- **Gọi bằng dòng lệnh:** `BuilderGemOnCrvMulti`, `GemOnCrvMulti`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn đường giữa trước, rồi các đường khác.
2. Chỉnh kích thước, khoảng cách.
3. Enter.

## Tuỳ chọn (lệnh phụ)


| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Cut=` | Kiểu cắt | round | round, oval, princess, cushion, emerald, asscher, baguette, marquise, pear, heart, trillion, radiant, cabochon_round, cabochon_oval |
| `Size=` | Kích thước (dài / đường kính) | 1.3 | 0.3 … 30 |
| `Width=` | Chiều rộng (0 = tỉ lệ chuẩn) | 0 | 0 … 30 |
| `Material=` | Chất liệu | diamond | diamond, moissanite, cz, ruby, sapphire, emerald, amethyst, topaz, garnet, aquamarine, peridot, tanzanite |
| `Start=` | Vị trí bắt đầu | 0 | 0 … 100 |
| `End=` | Vị trí kết thúc | 100 | 0 … 100 |
| `Spacing=` | Khoảng cách giữa đá | 0.2 | 0 … 5 |
| `YOffset=` | Lệch ngang (Y) | 0 | -20 … 20 |
| `ZOffset=` | Lệch đứng (Z) | 0 | -20 … 20 |
| `Facing=` | Hướng mặt đá | cplane | cplane, outward, inward |
| `Flip=` | Lật đá (culet lên) | No |  |
| `TaperDiameter=` | Thu nhỏ dần kích thước | No |  |
| `EndSize=` | Kích thước cuối | 1 | 0.3 … 30 |
| `TaperSpacing=` | Khoảng cách theo kích thước | No |  |
