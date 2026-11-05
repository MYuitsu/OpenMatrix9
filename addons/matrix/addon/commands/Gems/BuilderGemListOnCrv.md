# Gem List on Curve

Rải lần lượt các viên đá đã chọn (đúng thứ tự, kích thước, kiểu) dọc đường cong.

- **Mã lệnh:** `BuilderGemListOnCrv`
- **Vị trí:** menu Gems, ô số 5
- **Icon:** `Resources/icons/gems-minimal/BuilderGemListOnCrv.svg`
- **Gọi bằng dòng lệnh:** `BuilderGemListOnCrv`, `GemListOnCrv`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn đường cong và các viên đá của danh sách.
2. Chỉnh vị trí, khoảng cách, hướng, lệch.
3. Enter.

## Tuỳ chọn (lệnh phụ)

- Direction: bắt đầu từ đầu hay cuối.
- Fill: lặp danh sách đến hết đường.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Start=` | Vị trí bắt đầu | 0 | 0 … 100 |
| `End=` | Vị trí kết thúc | 100 | 0 … 100 |
| `Spacing=` | Khoảng cách giữa đá | 0.2 | 0 … 5 |
| `YOffset=` | Lệch ngang (Y) | 0 | -20 … 20 |
| `ZOffset=` | Lệch đứng (Z) | 0 | -20 … 20 |
| `Facing=` | Hướng mặt đá | cplane | cplane, outward, inward |
| `Flip=` | Lật đá (culet lên) | No |  |
| `EndYOffset=` | Lệch Y cuối | 0 | -20 … 20 |
| `EndZOffset=` | Lệch Z cuối | 0 | -20 … 20 |
| `Direction=` | Bắt đầu từ | start | start, end |
| `Fill=` | Lặp danh sách đến hết đường | No |  |
