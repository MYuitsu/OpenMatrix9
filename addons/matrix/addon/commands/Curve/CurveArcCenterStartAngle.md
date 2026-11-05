# Arc

Vẽ cung tròn như Rhino Arc: tâm, điểm đầu, góc; có StartPoint, Tangent, Extension, Deformable.

- **Mã lệnh:** `CurveArcCenterStartAngle`
- **Vị trí:** menu Curve, ô số 7
- **Icon:** `Resources/icons/curve-minimal/CurveArcCenterStartAngle.svg`
- **Gọi bằng dòng lệnh:** `CurveArcCenterStartAngle`, `Arc`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Center of arc: bấm tâm (hoặc chọn StartPoint/Tangent/Extension).
2. Start of arc: bấm điểm đầu hoặc gõ bán kính.
3. End point or angle: bấm điểm cuối, gõ góc, hoặc Length=chiều dài cung.

## Tuỳ chọn (lệnh phụ)

- Deformable: cung NURBS sửa được (Degree=3, PointCount=10).
- StartPoint: điểm đầu › điểm cuối › điểm trên cung; Direction, ThroughPoint, Radius đổi thứ tự.
- Tangent: cung tiếp xúc hai đường cong với bán kính.
- Extension: cung nối tiếp tuyến từ đầu một đường cong.
- Length: chiều dài cung; Direction=CounterClockwise/Clockwise.
