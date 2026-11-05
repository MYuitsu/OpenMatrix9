# Channel Builder

Một dao rãnh (channel) chạy theo hàng đá.

- **Mã lệnh:** `BuilderChannelBuilder`
- **Vị trí:** menu Cutters, ô số 3
- **Icon:** `Resources/icons/cutters-minimal/BuilderChannelBuilder.svg`
- **Gọi bằng dòng lệnh:** `BuilderChannelBuilder`, `gvChannelBuilder`, `ChannelBuilder`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn hàng đá.
2. Chọn Channel Builder, chỉnh Width, Top, Bottom, Extension, ZOffset.
3. Enter để tạo dao.

## Tuỳ chọn (lệnh phụ)

- Width: rộng rãnh, % cỡ đá.
- Top / Bottom: trên / dưới gờ, % cỡ đá.
- Extension: kéo dài qua hai đầu (mm).
- ZOffset: nâng/hạ dao.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Width=` | Rộng rãnh (% đá) | 100 | 20 … 200 |
| `Top=` | Trên gờ (% đá) | 40 | 0 … 200 |
| `Bottom=` | Dưới gờ (% đá) | 70 | 0 … 300 |
| `Extension=` | Kéo dài hai đầu | 0.5 | 0 … 10 |
| `ZOffset=` | Nâng/hạ dao | 0 | -5 … 5 |
