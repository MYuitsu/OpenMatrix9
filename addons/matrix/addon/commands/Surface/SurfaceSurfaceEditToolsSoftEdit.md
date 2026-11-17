# Soft Edit Surface

Kéo một điểm trên mặt như Rhino 5 SoftEditSrf: các điểm điều khiển quanh đó dời mềm, điểm bấm tới đúng chỗ.

- **Mã lệnh:** `SurfaceSurfaceEditToolsSoftEdit`
- **Vị trí:** menu Surface, ô số 36
- **Icon:** `Resources/icons/surface-minimal/SurfaceSurfaceEditToolsSoftEdit.svg`
- **Gọi bằng dòng lệnh:** `SurfaceSurfaceEditToolsSoftEdit`, `SoftEditSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn mặt.
2. Bấm điểm gốc trên mặt.
3. Bấm điểm đích (mặt đi theo chuột).

## Tuỳ chọn (lệnh phụ)

- UDistance / VDistance: vùng ảnh hưởng dọc mặt (mm).
- FixEdges: giữ nguyên các cạnh.
- Copy: tạo bản sao, giữ mặt gốc.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `UDistance=` | Vùng ảnh hưởng U (Soft Edit, mm) | 5 | 0.001 … 10000 |
| `VDistance=` | Vùng ảnh hưởng V (Soft Edit, mm) | 5 | 0.001 … 10000 |
| `FixEdges=` | Giữ cạnh (Soft Edit) | No |  |
