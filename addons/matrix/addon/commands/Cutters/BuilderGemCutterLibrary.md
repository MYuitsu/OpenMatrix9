# Gem Cutter Library

Dao cắt lỗ đá theo kiểu có sẵn: Setting, Azure (thủng), Straight, Brite.

- **Mã lệnh:** `BuilderGemCutterLibrary`
- **Vị trí:** menu Cutters, ô số 2
- **Icon:** `Resources/icons/cutters-minimal/BuilderGemCutterLibrary.svg`
- **Gọi bằng dòng lệnh:** `BuilderGemCutterLibrary`, `GemCutterLibrary`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn đá.
2. Chọn Gem Cutter Library, chọn Style, ScaleXY, ScaleZ.
3. Enter: dao cắt nằm ở layer Cutting.

## Tuỳ chọn (lệnh phụ)

- Style: kiểu dao.
- ScaleXY / ScaleZ: phóng ngang / dọc (%).

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Style=` | Kiểu dao | setting | setting, azure, straight, brite |
| `ScaleXY=` | Phóng ngang | 100 | 50 … 200 |
| `ScaleZ=` | Phóng dọc | 100 | 50 … 300 |
