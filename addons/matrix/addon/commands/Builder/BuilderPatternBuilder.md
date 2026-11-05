# Pattern Builder

Lặp vật thể (vẽ trong tam giác gốc) thành ô vuông đối xứng rồi lưới X × Y.

- **Mã lệnh:** `BuilderPatternBuilder`
- **Vị trí:** menu Builder, ô số 12
- **Icon:** `Resources/icons/builder-minimal/BuilderPatternBuilder.svg`
- **Gọi bằng dòng lệnh:** `BuilderPatternBuilder`, `PatternBuilder`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn vật thể (quanh gốc CPlane).
2. Chỉnh lặp X, Y, cỡ ô.
3. Enter.

## Tuỳ chọn (lệnh phụ)


| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `XRepeat=` | Lặp X | 3 | 1 … 50 |
| `YRepeat=` | Lặp Y | 3 | 1 … 50 |
| `TileSize=` | Nửa cỡ ô | 5 | 0.2 … 100 |
| `Mirror=` | Đối xứng thành ô vuông | Yes |  |
