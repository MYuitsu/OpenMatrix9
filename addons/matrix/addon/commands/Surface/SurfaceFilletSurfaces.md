# Fillet Surfaces

Bo cung giữa hai mặt cắt nhau (mặt gốc không bị cắt).

- **Mã lệnh:** `SurfaceFilletSurfaces`
- **Vị trí:** menu Surface, ô số 21
- **Icon:** `Resources/icons/surface-minimal/SurfaceFilletSurfaces.svg`
- **Gọi bằng dòng lệnh:** `SurfaceFilletSurfaces`, `FilletSrf`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn hai mặt.
2. Chỉnh bán kính, phía.
3. Enter.

## Tuỳ chọn (lệnh phụ)

- FlipFirst / FlipSecond: chọn phía.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Radius=` | Bán kính / khoảng | 1 | 0.01 … 50 |
| `FlipFirst=` | Đổi phía mặt 1 | No |  |
| `FlipSecond=` | Đổi phía mặt 2 | No |  |
