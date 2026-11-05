# Boolean Builder

Boolean có kiểm tra: chọn vật, rồi dao cắt; báo cạnh hở / không kín trước khi cắt.

- **Mã lệnh:** `BuilderBooleanBuilder`
- **Vị trí:** menu Tools, ô số 31
- **Icon:** `Resources/icons/tools-minimal/BuilderBooleanBuilder.svg`
- **Gọi bằng dòng lệnh:** `BuilderBooleanBuilder`, `BooleanBuilder`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn vật cần cắt, Enter.
2. Chọn dao cắt, Enter.

## Tuỳ chọn (lệnh phụ)

- Mode: Difference / Union / Intersection.
- KeepCutters: giữ dao cắt.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Mode=` | Kiểu boolean | difference | difference, union, intersection |
| `KeepCutters=` | Giữ dao cắt | No |  |
