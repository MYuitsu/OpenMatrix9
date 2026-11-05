# Nautilus Builder

Rải vật thể theo xoắn ốc nautilus, nhỏ dần.

- **Mã lệnh:** `BuilderNautilusBuilder`
- **Vị trí:** menu Builder, ô số 13
- **Icon:** `Resources/icons/builder-minimal/BuilderNautilusBuilder.svg`
- **Gọi bằng dòng lệnh:** `BuilderNautilusBuilder`, `Nautilus`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn vật thể (ở gốc CPlane).
2. Chỉnh số lượng, số vòng, bán kính, thu nhỏ.
3. Enter.

## Tuỳ chọn (lệnh phụ)


| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Amount=` | Số lượng | 24 | 2 … 400 |
| `Turns=` | Số vòng | 2 | 0.1 … 20 |
| `Rotation=` | Góc bắt đầu | 0 | -180 … 180 |
| `Shrink=` | Tỉ lệ mỗi bước | 92 | 50 … 100 |
| `Radius=` | Bán kính đầu | 10 | 0.5 … 200 |
| `Flip=` | Đảo chiều | No |  |
