# Variable Offset Surface

Offset với khoảng thay đổi trên mặt như Rhino 5 VariableOffsetSrf: tay nắm ở bốn góc, thêm tay nắm; mặt khớp trong dung sai.

- **Mã lệnh:** `SurfaceVariableOffsetSurface`
- **Vị trí:** menu Surface, ô số 26
- **Icon:** `Resources/icons/surface-minimal/SurfaceVariableOffsetSurface.svg`
- **Gọi bằng dòng lệnh:** `SurfaceVariableOffsetSurface`, `VariableOffsetSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn mặt.
2. Bấm gần tay nắm để gõ khoảng; AddHandle thêm tay nắm; Enter để tạo.

## Tuỳ chọn (lệnh phụ)

- SetAll: một khoảng cho mọi tay nắm.
- AddHandle: thêm tay nắm tại điểm trên mặt.
- Flip: đổi phía.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Distance=` | Khoảng offset | 1 | -50 … 50 |
| `Corner10=` | Khoảng góc (1,0) | 1 | -50 … 50 |
| `Corner01=` | Khoảng góc (0,1) | 1 | -50 … 50 |
| `Corner11=` | Khoảng góc (1,1) | 1 | -50 … 50 |
| `Flip=` | Phía ngược | No |  |
