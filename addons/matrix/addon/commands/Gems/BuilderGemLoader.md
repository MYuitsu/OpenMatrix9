# Gem Loader

Nạp đá theo kiểu cắt và kích thước, bấm để đặt lên CPlane hoặc thay cho đá đang chọn.

- **Mã lệnh:** `BuilderGemLoader`
- **Vị trí:** menu Gems, ô số 1
- **Icon:** `Resources/icons/gems-minimal/BuilderGemLoader.svg`
- **Gọi bằng dòng lệnh:** `BuilderGemLoader`, `gvLoader`, `GemLoader`
- **Cách chạy:** từng bước trên dòng lệnh, kèm bảng BUILDERS; Enter để tạo, Esc để huỷ

## Cách dùng

1. Chọn Gem Loader trong menu Gems (hoặc gõ GemLoader).
2. Chọn Cut, Length, Width, Material ở bảng BUILDERS hoặc trên dòng lệnh (Cut=oval, Length=7).
3. Bấm vào viewport để đặt từng viên; U hoặc Undo bỏ viên vừa đặt.
4. Enter: tạo các viên đã đặt. Không bấm điểm nào thì đá đặt ở gốc CPlane; nếu đang chọn đá thì thay đá đó bằng kiểu mới.

## Tuỳ chọn (lệnh phụ)

- Cut: kiểu cắt (Round, Oval, Princess, Cushion, Emerald, Asscher, Baguette, Marquise, Pear, Heart, Trillion, Radiant, Cabochon).
- Length / Width: kích thước (mm). Width = 0 dùng tỉ lệ chuẩn của kiểu cắt.
- Material: chất liệu (tính carat).
- Undo: bỏ điểm vừa đặt.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Cut=` | Kiểu cắt | round | round, oval, princess, cushion, emerald, asscher, baguette, marquise, pear, heart, trillion, radiant, cabochon_round, cabochon_oval |
| `Length=` | Chiều dài / đường kính | 1.5 | 0.3 … 40 |
| `Width=` | Chiều rộng (0 = tỉ lệ chuẩn) | 0 | 0 … 40 |
| `Material=` | Chất liệu | diamond | diamond, moissanite, cz, ruby, sapphire, emerald, amethyst, topaz, garnet, aquamarine, peridot, tanzanite |
