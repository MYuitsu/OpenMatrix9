# Sweep 1 Rail

Quét các mặt cắt dọc một đường ray như Rhino 5; mặt đi qua đúng từng mặt cắt.

- **Mã lệnh:** `SurfaceSweepSweep1Rail`
- **Vị trí:** menu Surface, ô số 1
- **Icon:** `Resources/icons/surface-minimal/SurfaceSweepSweep1Rail.svg`
- **Gọi bằng dòng lệnh:** `SurfaceSweepSweep1Rail`, `Sweep1`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn ray.
2. Chọn các mặt cắt theo thứ tự (Point: bắt đầu/kết thúc tại một điểm), Enter.
3. Chỉnh tuỳ chọn khi xem trước, Enter để tạo.

## Tuỳ chọn (lệnh phụ)

- Style: Freeform giữ góc với ray; Roadlike Top/Right/Front giữ theo trục cố định.
- Closed: quét kín quay về mặt cắt đầu (ray kín, từ hai mặt cắt).
- GlobalBlend: chuyển hình tuyến tính từ đầu đến cuối.
- Simplify: None / Rebuild (số điểm) / Refit (dung sai) cho mặt cắt.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `SweepTolerance=` | Dung sai quét (mm) | 0.001 | 0 … 1 |
| `Style=` | Kiểu khung quét | freeform | freeform, roadlike_top, roadlike_right, roadlike_front |
| `Closed=` | Quét kín | No |  |
| `GlobalBlend=` | Chuyển hình toàn cục | No |  |
| `Simplify=` | Đơn giản mặt cắt | none | none, rebuild, refit |
| `RebuildPoints=` | Số điểm dựng lại mặt cắt | 10 | 4 … 1000 |
| `RefitTolerance=` | Dung sai khớp lại mặt cắt (mm) | 0.01 | 0.0001 … 10 |
