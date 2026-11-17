# Variable Fillet

Bo cung bán kính thay đổi dọc cạnh như Rhino 5 VariableFilletSrf (bi lăn), có tay nắm.

- **Mã lệnh:** `SurfaceVariableFilletBlendChamferVariableFilletSurfaces`
- **Vị trí:** menu Surface, ô số 22
- **Icon:** `Resources/icons/surface-minimal/SurfaceVariableFilletBlendChamferVariableFilletSurfaces.svg`
- **Gọi bằng dòng lệnh:** `SurfaceVariableFilletBlendChamferVariableFilletSurfaces`, `VariableFilletSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn mặt thứ nhất (bấm phía giữ).
2. Chọn mặt thứ hai (bấm phía giữ).
3. Bấm gần tay nắm để gõ bán kính; AddHandle thêm tay nắm; Enter để tạo.

## Tuỳ chọn (lệnh phụ)

- SetAll: một bán kính cho mọi tay nắm.
- AddHandle: thêm tay nắm tại điểm trên cạnh.
- Trim: cắt mặt gốc theo mép bo.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `RailType=` | Kiểu ray (Variable Fillet) | RollingBall | RollingBall, DistFromEdge, DistBetweenRails |
| `Trim=` | Cắt mặt gốc (Fillet/Chamfer) | Yes |  |
| `Radius=` | Bán kính / khoảng | 1 | 0.01 … 50 |
| `EndRadius=` | Bán kính / khoảng cuối | 2 | 0.01 … 50 |
| `FlipFirst=` | Đổi phía mặt 1 | No |  |
| `FlipSecond=` | Đổi phía mặt 2 | No |  |
