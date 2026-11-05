# Prong Builder

Thêm chấu cho đá: N chấu mỗi viên (Gem North) hoặc chấu dùng chung giữa các viên trong một hàng (Gem Flow).

- **Mã lệnh:** `BuilderProngAdder`
- **Vị trí:** menu Settings, ô số 6
- **Icon:** `Resources/icons/settings-minimal/BuilderProngAdder.svg`
- **Gọi bằng dòng lệnh:** `BuilderProngAdder`, `gvProngAdder`, `ProngBuilder`, `Prongs`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn các viên đá (một viên hoặc một hàng đá).
2. Chọn Prong Builder; chấu xem trước hiện ngay.
3. Chỉnh Mode, Count, Angle, Size, Height, Drop, Inset trên bảng BUILDERS hoặc dòng lệnh.
4. Enter để tạo chấu.

## Tuỳ chọn (lệnh phụ)

- Mode: North (N chấu/viên), Shared2 (2 chấu chung giữa hai viên, 2 chấu mỗi đầu), Shared1 (1 chấu chung, so le).
- Count, Angle: số chấu và góc bắt đầu (chế độ North).
- Size: đường kính chấu, % cỡ đá.
- Height / Drop: chấu cao trên gờ / sâu dưới gờ, % cỡ đá.
- Inset: phần chấu ăn vào gờ đá, % đường kính chấu.
- Dome: đầu chấu tròn.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Mode=` | Kiểu chấu | north | north, shared2, shared1 |
| `Count=` | Số chấu mỗi viên | 4 | 1 … 12 |
| `Angle=` | Góc chấu đầu | 45 | -360 … 360 |
| `Size=` | Đường kính chấu (% đá) | 25 | 5 … 80 |
| `Height=` | Cao trên gờ (% đá) | 25 | 0 … 200 |
| `Drop=` | Sâu dưới gờ (% đá) | 70 | 0 … 300 |
| `Inset=` | Ăn vào gờ (% chấu) | 30 | 0 … 50 |
| `Dome=` | Đầu chấu tròn | Yes |  |
