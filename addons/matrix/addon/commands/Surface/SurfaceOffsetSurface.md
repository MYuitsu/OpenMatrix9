# Offset Surface

Mặt song song cách đều như Rhino 5 OffsetSrf: mặt phẳng, cầu, trụ chính xác; mặt tự do trong dung sai.

- **Mã lệnh:** `SurfaceOffsetSurface`
- **Vị trí:** menu Surface, ô số 25
- **Icon:** `Resources/icons/surface-minimal/SurfaceOffsetSurface.svg`
- **Gọi bằng dòng lệnh:** `SurfaceOffsetSurface`, `OffsetSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn mặt cần offset, Enter.
2. Gõ khoảng offset (Enter: giữ mặc định).

## Tuỳ chọn (lệnh phụ)

- FlipAll: đổi phía.
- Loose: giữ nguyên cấu trúc điểm của mặt gốc.
- Tolerance: dung sai (0 = mặc định).
- BothSides: offset cả hai phía.
- DeleteInput: xoá mặt gốc.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Loose=` | Offset lỏng (giữ cấu trúc điểm) | No |  |
| `Tolerance=` | Dung sai offset (0 = mặc định, mm) | 0 | 0 … 1 |
| `BothSides=` | Offset cả hai phía | No |  |
| `Distance=` | Khoảng offset | 1 | -50 … 50 |
| `Flip=` | Phía ngược | No |  |
