# Gem on Curve Advanced

Rải đá theo đường cong có lệch Y/Z, thu nhỏ dần, bám góc, hoặc rải bi / vật thể.

- **Mã lệnh:** `BuilderGemOnCrvAdvanced`
- **Vị trí:** menu Gems, ô số 3
- **Icon:** `Resources/icons/gems-minimal/BuilderGemOnCrvAdvanced.svg`
- **Gọi bằng dòng lệnh:** `BuilderGemOnCrvAdvanced`, `GemOnCrvAdvanced`, `gvGemOnCrvAdvanced`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn đường cong (và một viên đá mẫu nếu muốn dùng đá đó).
2. Chỉnh trong bảng BUILDERS: kiểu đá, kích thước, khoảng cách, vị trí, hướng.
3. Enter để đặt đá.

## Tuỳ chọn (lệnh phụ)

- Mode: Gem / Sphere (bi) / Object (rải vật thể đặt ở gốc F4).
- SnapCorners: đặt đá ở các góc.
- Facing, Flip, TaperDiameter, TaperSpacing.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Cut=` | Kiểu cắt | round | round, oval, princess, cushion, emerald, asscher, baguette, marquise, pear, heart, trillion, radiant, cabochon_round, cabochon_oval |
| `Size=` | Kích thước (dài / đường kính) | 1.5 | 0.3 … 30 |
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
| `Mode=` | Kiểu rải | gem | gem, sphere, object |
| `SnapCorners=` | Đặt đá ở góc | No |  |
