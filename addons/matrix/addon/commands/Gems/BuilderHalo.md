# Halo Builder

Tạo vòng đá nhỏ (halo) và khung kim loại quanh viên đá chủ.

- **Mã lệnh:** `BuilderHalo`
- **Vị trí:** menu Gems, ô số 10
- **Icon:** `Resources/icons/gems-minimal/BuilderHalo.svg`
- **Gọi bằng dòng lệnh:** `BuilderHalo`, `gvHaloBuilder`, `Halo`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn Halo Builder.
2. Chọn viên đá chủ (đá tạo bởi Gem Loader, Gem On Crv …).
3. Chỉnh Size, Spacing, Offset … xem trước trực tiếp.
4. Enter để tạo.

## Tuỳ chọn (lệnh phụ)

- Size: cỡ đá halo (mm).
- Spacing: khe giữa các viên halo.
- Offset: khoảng cách từ gờ đá chủ tới gờ đá halo.
- Height: nâng/hạ đá halo theo trục đá chủ.
- Rotation: xoay vị trí viên đầu tiên (độ).
- Angle: nghiêng đá halo ra ngoài (độ).
- Metal: tạo khung kim loại; ProfileHeight, Wall: chiều cao và độ dày thành khung.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Size=` | Cỡ đá halo | 1 | 0.3 … 5 |
| `Spacing=` | Khe giữa đá | 0.15 | 0 … 2 |
| `Offset=` | Cách đá chủ | 0.3 | -1 … 5 |
| `Height=` | Nâng/hạ đá halo | 0 | -5 … 5 |
| `Rotation=` | Xoay viên đầu | 0 | -360 … 360 |
| `Angle=` | Nghiêng đá ra ngoài | 0 | -60 … 60 |
| `Metal=` | Tạo khung kim loại | Yes |  |
| `ProfileHeight=` | Chiều cao khung | 1.2 | 0.1 … 10 |
| `Wall=` | Độ dày thành khung | 0.25 | 0 … 3 |
