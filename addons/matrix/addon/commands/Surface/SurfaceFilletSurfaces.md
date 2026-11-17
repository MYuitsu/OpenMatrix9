# Fillet Surfaces

Bo cung bán kính không đổi giữa hai mặt như Rhino 5 FilletSrf (bi lăn); mặt bo trong dung sai.

- **Mã lệnh:** `SurfaceFilletSurfaces`
- **Vị trí:** menu Surface, ô số 21
- **Icon:** `Resources/icons/surface-minimal/SurfaceFilletSurfaces.svg`
- **Gọi bằng dòng lệnh:** `SurfaceFilletSurfaces`, `FilletSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn mặt thứ nhất: bấm vào phía muốn giữ.
2. Chọn mặt thứ hai: bấm vào phía muốn giữ.

## Tuỳ chọn (lệnh phụ)

- Radius: bán kính bo.
- Trim: cắt hai mặt gốc theo mép bo, giữ phía đã bấm (mặc định Yes).

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Extend=` | Kéo dài tới mép mặt dài hơn (Fillet/Chamfer) | Yes |  |
| `Split=` | Tách mặt gốc tại mép (Fillet/Chamfer) | No |  |
| `Trim=` | Cắt mặt gốc (Fillet/Chamfer) | Yes |  |
| `Radius=` | Bán kính / khoảng | 1 | 0.01 … 50 |
| `FlipFirst=` | Đổi phía mặt 1 | No |  |
| `FlipSecond=` | Đổi phía mặt 2 | No |  |
