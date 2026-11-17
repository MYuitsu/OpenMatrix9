# Variable Chamfer

Vát với khoảng thay đổi dọc cạnh như Rhino 5 VariableChamferSrf, có tay nắm.

- **Mã lệnh:** `SurfaceVariableFilletBlendChamferVariableChamferSurfaces`
- **Vị trí:** menu Surface, ô số 24
- **Icon:** `Resources/icons/surface-minimal/SurfaceVariableFilletBlendChamferVariableChamferSurfaces.svg`
- **Gọi bằng dòng lệnh:** `SurfaceVariableFilletBlendChamferVariableChamferSurfaces`, `VariableChamferSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn mặt thứ nhất (bấm phía giữ).
2. Chọn mặt thứ hai (bấm phía giữ).
3. Bấm gần tay nắm để gõ khoảng vát (một số, hoặc hai số: thứ nhất,thứ hai); Enter để tạo.

## Tuỳ chọn (lệnh phụ)

- SetAll: một khoảng cho mọi tay nắm.
- AddHandle: thêm tay nắm tại điểm trên cạnh.
- Trim: cắt mặt gốc theo mép vát.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `FirstDistance=` | Khoảng vát trên mặt thứ nhất (mm) | 1 | 0.0001 … 1000 |
| `Trim=` | Cắt mặt gốc (Fillet/Chamfer) | Yes |  |
| `EndDistance=` | Khoảng vát ở đầu cuối (Variable Chamfer, mm) | 1 | 0.0001 … 1000 |
| `Radius=` | Bán kính / khoảng | 1 | 0.01 … 50 |
| `EndRadius=` | Bán kính / khoảng cuối | 2 | 0.01 … 50 |
| `FlipFirst=` | Đổi phía mặt 1 | No |  |
| `FlipSecond=` | Đổi phía mặt 2 | No |  |
