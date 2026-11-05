# Gem Cutter

Dao cắt lỗ cho từng viên đá (theo % cỡ đá), nằm ở layer Cutting.

- **Mã lệnh:** `BuilderGemCutter`
- **Vị trí:** menu Cutters, ô số 1
- **Icon:** `Resources/icons/cutters-minimal/BuilderGemCutter.svg`
- **Gọi bằng dòng lệnh:** `BuilderGemCutter`, `gvAzureCutter`, `GemCutter`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn đá.
2. Chỉnh GirdleScale, GirdleWidth, TopHeight, SeatSize, LowerDepth, LowerTaper, ZOffset trên bảng BUILDERS hoặc dòng lệnh.
3. Enter để tạo dao.

## Tuỳ chọn (lệnh phụ)


| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `GirdleScale=` | Cỡ gờ (Girdle Scale) | 105 | 80 … 150 |
| `GirdleWidth=` | Dày gờ (Girdle Width) | 5 | 0 … 50 |
| `TopHeight=` | Cao phần trên | 50 | 0 … 300 |
| `SeatSize=` | Cỡ đế (Lower Seat) | 50 | 10 … 100 |
| `LowerDepth=` | Sâu lỗ (Lower Depth) | 150 | 10 … 500 |
| `LowerTaper=` | Loe đáy (Lower Taper) | 0 | -90 … 200 |
| `ZOffset=` | Nâng/hạ (Z Offset) | 0 | -100 … 100 |
