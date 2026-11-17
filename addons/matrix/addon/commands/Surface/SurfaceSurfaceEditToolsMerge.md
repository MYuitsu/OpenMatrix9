# Merge Surfaces

Gộp hai mặt chung một cạnh thành một mặt như Rhino 5 MergeSrf (Smooth=No: chính xác).

- **Mã lệnh:** `SurfaceSurfaceEditToolsMerge`
- **Vị trí:** menu Surface, ô số 27
- **Icon:** `Resources/icons/surface-minimal/SurfaceSurfaceEditToolsMerge.svg`
- **Gọi bằng dòng lệnh:** `SurfaceSurfaceEditToolsMerge`, `MergeSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn mặt thứ nhất.
2. Chọn mặt thứ hai: gộp ngay.

## Tuỳ chọn (lệnh phụ)

- Smooth: nối mượt (đổi hình gần đường nối).
- Tolerance: hai cạnh phải trùng trong dung sai này.
- RoundnessFactor: 0 sắc … 1 mượt.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Smooth=` | Gộp mượt (Merge) | Yes |  |
| `Tolerance=` | Dung sai gộp mặt (mm) | 0.001 | 0 … 1 |
| `RoundnessFactor=` | Độ tròn khi gộp (0–1) | 1 | 0 … 1 |
