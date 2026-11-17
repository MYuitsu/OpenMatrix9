# Variable Blend Surface

Mặt chuyển tiếp liên tục độ cong với bán kính thay đổi dọc cạnh như Rhino 5 VariableBlendSrf, có tay nắm.

- **Mã lệnh:** `SurfaceVariableFilletBlendChamferVariableBlendSurfaces`
- **Vị trí:** menu Surface, ô số 8
- **Icon:** `Resources/icons/surface-minimal/SurfaceVariableFilletBlendChamferVariableBlendSurfaces.svg`
- **Gọi bằng dòng lệnh:** `SurfaceVariableFilletBlendChamferVariableBlendSurfaces`, `VariableBlendSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn mặt thứ nhất (bấm phía giữ).
2. Chọn mặt thứ hai (bấm phía giữ).
3. Bấm gần tay nắm để gõ bán kính; AddHandle thêm tay nắm; Enter để tạo.

## Tuỳ chọn (lệnh phụ)

- SetAll: một bán kính cho mọi tay nắm.
- AddHandle: thêm tay nắm tại điểm trên cạnh.
- Trim: cắt mặt gốc theo mép.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Trim=` | Cắt mặt gốc (Fillet/Chamfer) | Yes |  |
| `Bulge=` | Độ phình | 1 | 0.05 … 5 |
| `EndBulge=` | Độ phình cuối | 1 | 0.05 … 5 |
| `Radius=` | Bán kính / khoảng | 1 | 0.01 … 50 |
| `EndRadius=` | Bán kính / khoảng cuối | 2 | 0.01 … 50 |
