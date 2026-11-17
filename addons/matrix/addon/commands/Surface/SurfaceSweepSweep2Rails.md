# Sweep 2 Rails

Quét mặt cắt giữa hai đường ray như Rhino 5; hai cạnh mặt nằm đúng trên hai ray.

- **Mã lệnh:** `SurfaceSweepSweep2Rails`
- **Vị trí:** menu Surface, ô số 2
- **Icon:** `Resources/icons/surface-minimal/SurfaceSweepSweep2Rails.svg`
- **Gọi bằng dòng lệnh:** `SurfaceSweepSweep2Rails`, `Sweep2`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn ray thứ nhất.
2. Chọn ray thứ hai.
3. Chọn các mặt cắt theo thứ tự (Point: bắt đầu/kết thúc tại một điểm), Enter.
4. Chỉnh tuỳ chọn khi xem trước, Enter để tạo.

## Tuỳ chọn (lệnh phụ)

- MaintainHeight: mặt cắt co giãn theo bề ngang nhưng giữ chiều cao.
- Closed: quét kín quay về mặt cắt đầu (hai ray kín, từ hai mặt cắt).
- Simplify: None / Rebuild (số điểm) / Refit (dung sai) cho mặt cắt.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `SweepTolerance=` | Dung sai quét (mm) | 0.001 | 0 … 1 |
| `Closed=` | Quét kín (2 ray) | No |  |
| `Simplify=` | Đơn giản mặt cắt (2 ray) | none | none, rebuild, refit |
| `RebuildPoints=` | Số điểm dựng lại mặt cắt (2 ray) | 10 | 4 … 1000 |
| `RefitTolerance=` | Dung sai khớp lại mặt cắt (2 ray, mm) | 0.01 | 0.0001 … 10 |
| `MaintainHeight=` | Giữ chiều cao profile | No |  |
