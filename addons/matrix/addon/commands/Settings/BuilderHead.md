# Head Builder

Tạo ổ chấu (basket head): chấu thuôn từ trên gờ xuống đế và các vòng gallery.

- **Mã lệnh:** `BuilderHead`
- **Vị trí:** menu Settings, ô số 1
- **Icon:** `Resources/icons/settings-minimal/BuilderHead.svg`
- **Gọi bằng dòng lệnh:** `BuilderHead`, `gvHeadBuilder`, `Head`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn viên đá chủ.
2. Chọn Head Builder, chỉnh Count, Angle, Prong, Height, Depth, Taper, Rails.
3. Enter để tạo.

## Tuỳ chọn (lệnh phụ)

- Count / Angle: số chấu và góc.
- Prong: đường kính chấu, % cỡ đá.
- Height: chấu cao trên gờ, % cỡ đá.
- Depth: sâu từ gờ xuống đế (0 = tự động).
- Taper: bán kính đế, % gờ.
- Rails / Rail: số vòng gallery và độ dày vòng.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Count=` | Số chấu | 4 | 2 … 12 |
| `Angle=` | Góc chấu đầu | 45 | -360 … 360 |
| `Prong=` | Đường kính chấu (% đá) | 14 | 4 … 50 |
| `Height=` | Cao trên gờ (% đá) | 15 | 0 … 100 |
| `Depth=` | Sâu xuống đế (0 = tự động) | 0 | 0 … 30 |
| `Taper=` | Bán kính đế (% gờ) | 55 | 10 … 150 |
| `Rails=` | Số vòng gallery | 2 | 0 … 4 |
| `Rail=` | Dày vòng (% đá) | 10 | 2 … 40 |
