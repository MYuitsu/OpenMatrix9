# Extrude Curve Normal to Surface

Dựng dải mặt (fin) từ đường cong trên mặt theo pháp tuyến hoặc tiếp tuyến mặt như Rhino 5 Fin.

- **Mã lệnh:** `SurfaceExtrudeCurveNormalToSurface`
- **Vị trí:** menu Surface, ô số 33
- **Icon:** `Resources/icons/surface-minimal/SurfaceExtrudeCurveNormalToSurface.svg`
- **Gọi bằng dòng lệnh:** `SurfaceExtrudeCurveNormalToSurface`, `ExtrudeCrvNormal`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn đường cong trên mặt.
2. Chọn mặt.
3. Gõ chiều cao hoặc bấm điểm (Enter: mặc định).

## Tuỳ chọn (lệnh phụ)

- Direction: Normal / Tangent.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Height=` | Cao dải | 1 | 0.01 … 100 |
| `Flip=` | Đảo phía | No |  |
