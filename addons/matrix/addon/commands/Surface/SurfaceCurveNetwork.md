# Curve Network

Mặt từ lưới đường cong hai hướng (Rhino 5 NetworkSrf): mặt đi qua mọi đường trong dung sai.

- **Mã lệnh:** `SurfaceCurveNetwork`
- **Vị trí:** menu Surface, ô số 10
- **Icon:** `Resources/icons/surface-minimal/SurfaceCurveNetwork.svg`
- **Gọi bằng dòng lệnh:** `SurfaceCurveNetwork`, `NetworkSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn các đường cong cả hai hướng (tự sắp xếp).
2. Enter.

## Tuỳ chọn (lệnh phụ)

- Dung sai cạnh / bên trong: mặt bám đường cong trong khoảng này.
- Khoảng hở giao: hai đường cách nhau hơn mức này coi như không cắt nhau.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `EdgeTolerance=` | Dung sai cạnh (Curve Network, mm) | 0.001 | 0 … 1 |
| `InteriorTolerance=` | Dung sai bên trong (Curve Network, mm) | 0.001 | 0 … 1 |
| `NoAutoSort=` | Chọn đường theo thứ tự (NetworkSrf NoAutoSort) | No |  |
| `CrossingGap=` | Khoảng hở tối đa tại giao điểm (mm) | 0.1 | 0.0001 … 10 |
