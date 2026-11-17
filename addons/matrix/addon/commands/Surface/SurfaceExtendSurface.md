# Extend Surface

Kéo dài mặt qua một cạnh như Rhino 5 ExtendSrf: Smooth là chính mặt đó kéo dài (chính xác), Line kéo thẳng theo tiếp tuyến.

- **Mã lệnh:** `SurfaceExtendSurface`
- **Vị trí:** menu Surface, ô số 32
- **Icon:** `Resources/icons/surface-minimal/SurfaceExtendSurface.svg`
- **Gọi bằng dòng lệnh:** `SurfaceExtendSurface`, `ExtendSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn cạnh mặt cần kéo dài (bấm gần cạnh).
2. Gõ độ dài (Enter: mặc định).

## Tuỳ chọn (lệnh phụ)

- Type: Smooth / Line.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Type=` | Kiểu kéo dài mặt | smooth | smooth, line |
| `ExtensionFactor=` | Độ dài kéo dài mặt mặc định (mm) | 1 | 0.0001 … 1000 |
