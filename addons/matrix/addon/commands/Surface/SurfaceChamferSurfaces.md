# Chamfer Surfaces

Vát phẳng giữa hai mặt cắt nhau như Rhino 5 ChamferSrf: hai khoảng vát tính từ giao tuyến.

- **Mã lệnh:** `SurfaceChamferSurfaces`
- **Vị trí:** menu Surface, ô số 23
- **Icon:** `Resources/icons/surface-minimal/SurfaceChamferSurfaces.svg`
- **Gọi bằng dòng lệnh:** `SurfaceChamferSurfaces`, `ChamferSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn mặt thứ nhất: bấm vào phía muốn giữ.
2. Chọn mặt thứ hai: bấm vào phía muốn giữ.

## Tuỳ chọn (lệnh phụ)

- Distances: khoảng vát trên mặt thứ nhất, rồi trên mặt thứ hai.
- Trim: cắt hai mặt gốc theo mép vát, giữ phía đã bấm (mặc định Yes).

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `FirstDistance=` | Khoảng vát trên mặt thứ nhất (mm) | 1 | 0.0001 … 1000 |
| `SecondDistance=` | Khoảng vát trên mặt thứ hai (mm) | 1 | 0.0001 … 1000 |
| `Extend=` | Kéo dài tới mép mặt dài hơn (Fillet/Chamfer) | Yes |  |
| `Split=` | Tách mặt gốc tại mép (Fillet/Chamfer) | No |  |
| `Trim=` | Cắt mặt gốc (Fillet/Chamfer) | Yes |  |
| `Radius=` | Bán kính / khoảng | 1 | 0.01 … 50 |
| `FlipFirst=` | Đổi phía mặt 1 | No |  |
| `FlipSecond=` | Đổi phía mặt 2 | No |  |
