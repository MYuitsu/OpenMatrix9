# Offset Curve

Offset đường cong như Rhino: đoạn thẳng giữ thẳng, cung giữ đồng tâm, góc polyline theo Corner; nằm trong mặt phẳng của đường cong.

- **Mã lệnh:** `CurveOffsetCurve`
- **Vị trí:** menu Curve, ô số 18
- **Icon:** `Resources/icons/curve-minimal/CurveOffsetCurve.svg`
- **Gọi bằng dòng lệnh:** `CurveOffsetCurve`, `Offset`, `OffsetCrv`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Select curve to offset: bấm đường cong (gõ số để đặt Distance).
2. Side to offset: bấm phía cần offset.

## Tuỳ chọn (lệnh phụ)

- Distance=1.
- Corner=Sharp/Round/Smooth/Chamfer (chỉ góc phía ngoài).
- ThroughPoint: offset qua điểm bấm.
- Tolerance=0.001.
- BothSides: offset hai phía.
- InCPlane: offset trong CPlane thay vì mặt phẳng đường cong.
- Cap=None/Flat/Round: bịt hai đầu khi BothSides.
