# Metal from Gems

Tạo khối kim loại ôm hàng đá: thành ngoài, máng trong, nắp, dao cắt Top, góc tròn / nhọn.

- **Mã lệnh:** `BuilderMetalPiece`
- **Vị trí:** menu Settings, ô số 10
- **Icon:** `Resources/icons/settings-minimal/BuilderMetalPiece.svg`
- **Gọi bằng dòng lệnh:** `BuilderMetalPiece`, `MetalPiece`, `gvMetalPiece`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn hàng (hoặc vòng) đá.
2. Mode trong bảng BUILDERS.
3. Enter.

## Tuỳ chọn (lệnh phụ)

- Mode: Outside / Inside / OutsideCenter / Top / RoundCorners / PinchedEnds.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Mode=` | Kiểu kim loại | outside | outside, inside, outside_center, top, round_corners, pinched_ends |
| `Wall=` | Dày thành | 0.5 | 0.1 … 5 |
| `Height=` | Cao trên gờ | 0.3 | 0 … 5 |
| `Depth=` | Sâu dưới gờ | 1.5 | 0.1 … 10 |
| `Floor=` | Dày đáy | 0.5 | 0.1 … 5 |
| `Gap=` | Khe với đá | 0.05 | 0 … 1 |
