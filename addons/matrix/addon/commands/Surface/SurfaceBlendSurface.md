# Blend Surface

Mặt chuyển tiếp giữa hai cạnh mặt như Rhino 5 BlendSrf: Position / Tangency / Curvature mỗi đầu, độ phồng mỗi đầu.

- **Mã lệnh:** `SurfaceBlendSurface`
- **Vị trí:** menu Surface, ô số 7
- **Icon:** `Resources/icons/surface-minimal/SurfaceBlendSurface.svg`
- **Gọi bằng dòng lệnh:** `SurfaceBlendSurface`, `BlendSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Bấm gần cạnh thứ nhất.
2. Bấm gần cạnh thứ hai.
3. Chỉnh liên tục và độ phồng khi xem trước, Enter.

## Tuỳ chọn (lệnh phụ)

- Continuity1 / Continuity2: Position, Tangency, Curvature.
- Bulge1 / Bulge2: độ phồng mỗi đầu.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Continuity1=` | Liên tục đầu 1 (Blend) | curvature | position, tangency, curvature, g3, g4 |
| `Continuity2=` | Liên tục đầu 2 (Blend) | curvature | position, tangency, curvature, g3, g4 |
| `Bulge=` | Độ phình | 1 | 0.05 … 5 |
