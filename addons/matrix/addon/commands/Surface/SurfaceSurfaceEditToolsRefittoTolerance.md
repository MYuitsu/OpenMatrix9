# Refit to Tolerance

Như FitSrf Rhino 5: làm lại mặt ít điểm nhất trong dung sai (UDegree, VDegree, DeleteInput).

- **Mã lệnh:** `SurfaceSurfaceEditToolsRefittoTolerance`
- **Vị trí:** menu Surface, ô số 40
- **Icon:** `Resources/icons/surface-minimal/SurfaceSurfaceEditToolsRefittoTolerance.svg`
- **Gọi bằng dòng lệnh:** `SurfaceSurfaceEditToolsRefittoTolerance`, `RefitSrf`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn mặt.
2. Chỉnh dung sai.
3. Enter.

## Tuỳ chọn (lệnh phụ)


| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `UDegree=` | Bậc U | 3 | 1 … 7 |
| `VDegree=` | Bậc V | 3 | 1 … 7 |
| `DeleteInput=` | Xoá mặt gốc | Yes |  |
| `Tolerance=` | Dung sai | 0.01 | 0.0001 … 5 |
