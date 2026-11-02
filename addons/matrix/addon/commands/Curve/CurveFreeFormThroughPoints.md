# Curve Through Points

Dựng đường cong lấy các điểm đã chọn làm điểm điều khiển (Rhino CurveThroughPt).

- **Mã lệnh:** `CurveFreeFormThroughPoints`
- **Vị trí:** menu Curve, ô số 26
- **Icon:** `Resources/icons/curve-minimal/CurveFreeFormThroughPoints.svg`
- **Gọi bằng dòng lệnh:** `CurveFreeFormThroughPoints`, `CurveThroughPt`, `CrvThroughPt`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Select points for control points of curve, Enter.

## Tuỳ chọn (lệnh phụ)

- Degree=3.
- CurveType=ControlPoint: điểm là điểm điều khiển; Interpolated: đường đi qua các điểm.
- Knots=Uniform/Chord/SqrtChrd: cách chia nút khi Interpolated.
