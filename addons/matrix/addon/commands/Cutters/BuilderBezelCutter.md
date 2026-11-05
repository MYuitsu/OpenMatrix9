# Bezel Cutter

Dao cắt răng cưa (scallop) quanh vành bezel.

- **Mã lệnh:** `BuilderBezelCutter`
- **Vị trí:** menu Settings, ô số 5
- **Icon:** `Resources/icons/settings-minimal/BuilderBezelCutter.svg`
- **Gọi bằng dòng lệnh:** `BuilderBezelCutter`, `gvBezelCutter`, `ScallopBezel`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn đá (đã có bezel).
2. Chọn Bezel Cutter, chỉnh Count, Size, Height, Length, Angle.
3. Enter để tạo dao.

## Tuỳ chọn (lệnh phụ)

- Count: số dao.
- Size: đường kính dao.
- Height: trục dao cao trên gờ.
- Length: dao dài tới (mm từ tâm đá, 0 = tự động).
- Angle: góc dao đầu.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Count=` | Số dao | 6 | 1 … 36 |
| `Size=` | Đường kính dao | 1 | 0.05 … 10 |
| `Height=` | Trục dao trên gờ | 0.6 | -5 … 5 |
| `Length=` | Dài tới (0 = tự động) | 0 | 0 … 30 |
| `Angle=` | Góc dao đầu | 0 | -360 … 360 |
