# Bezel Builder

Tạo ổ bezel (vành ôm) quanh mỗi viên đá.

- **Mã lệnh:** `BuilderBezel`
- **Vị trí:** menu Settings, ô số 2
- **Icon:** `Resources/icons/settings-minimal/BuilderBezel.svg`
- **Gọi bằng dòng lệnh:** `BuilderBezel`, `gvBezelBuilder`, `Bezel`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn đá.
2. Chọn Bezel Builder, chỉnh các thông số.
3. Enter để tạo.

## Tuỳ chọn (lệnh phụ)

- Thickness: dày vành ở gờ đá (mm).
- BaseThickness: dày vành ở đáy.
- Lip: vành cao hơn gờ đá.
- Height: sâu dưới gờ (0 = tự theo đáy đá).
- Seat: cỡ bậc đỡ đá, % gờ.
- Angle: nghiêng thành vành.
- Placement: nâng/hạ cả vành.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Thickness=` | Dày vành ở gờ | 0.4 | 0.05 … 5 |
| `BaseThickness=` | Dày vành ở đáy | 0.4 | 0.05 … 5 |
| `Lip=` | Cao hơn gờ | 0.3 | 0 … 5 |
| `Height=` | Sâu dưới gờ (0 = tự động) | 0 | 0 … 30 |
| `Seat=` | Bậc đỡ (% gờ) | 92 | 30 … 100 |
| `Angle=` | Nghiêng thành | 0 | -45 … 45 |
| `Placement=` | Nâng/hạ vành | 0 | -10 … 10 |
