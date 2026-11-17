# Patch

Mặt vá khớp qua các đường và điểm như Rhino 5 Patch; tự cắt theo đường biên kín.

- **Mã lệnh:** `SurfacePatch`
- **Vị trí:** menu Surface, ô số 12
- **Icon:** `Resources/icons/surface-minimal/SurfacePatch.svg`
- **Gọi bằng dòng lệnh:** `SurfacePatch`, `Patch`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn điểm, đường cong (biên và đường bên trong).
2. Enter.

## Tuỳ chọn (lệnh phụ)

- Khoảng cách điểm mẫu, số nhịp U/V, độ cứng (càng lớn càng phẳng).
- Tự cắt: cắt theo đường kín bên ngoài.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `SampleSpacing=` | Khoảng cách điểm mẫu (Patch, mm) | 1 | 0.001 … 100 |
| `USpans=` | Số nhịp U (Patch) | 10 | 1 … 40 |
| `VSpans=` | Số nhịp V (Patch) | 10 | 1 … 40 |
| `Stiffness=` | Độ cứng (Patch) | 1 | 0 … 100000 |
| `AutoTrim=` | Tự cắt theo biên (Patch) | Yes |  |
