# Polyline

Vẽ đường gấp khúc gồm đoạn thẳng và cung tròn nối tiếp nhau.

- **Mã lệnh:** `CurvePolylinePolyline`
- **Vị trí:** menu Curve, ô số 1
- **Icon:** `Resources/icons/curve-minimal/CurvePolylinePolyline.svg`
- **Gọi bằng dòng lệnh:** `CurvePolylinePolyline`, `Polyline`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Bấm điểm đầu.
2. Bấm các điểm tiếp theo (gõ x,y hoặc @dx,dy; gõ số = chiều dài đoạn).
3. Mode=Arc: đoạn tiếp theo là cung tiếp tuyến với đoạn trước; Direction chọn hướng đầu cung, Center chọn tâm cung.
4. Enter hoặc chuột phải để kết thúc; bấm lại điểm đầu để đóng.

## Tuỳ chọn (lệnh phụ)

- PersistentClose: luôn khép kín khi vẽ.
- Close: khép kín và kết thúc (từ điểm thứ 3).
- Mode=Line/Arc: vẽ đoạn thẳng hay cung tròn.
- Helpers: hiện đường gợi ý tiếp tuyến và trục X/Y.
- Length: khoá chiều dài đoạn tiếp theo.
- Undo: bỏ điểm vừa bấm.
