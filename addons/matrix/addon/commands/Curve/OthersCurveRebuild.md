# Rebuild Curve

Dựng lại đường cong với số điểm điều khiển và bậc mới (Rhino Rebuild): nút đều, giữ hai đầu, báo độ lệch lớn nhất.

- **Mã lệnh:** `OthersCurveRebuild`
- **Vị trí:** menu Curve, ô số 9
- **Icon:** `Resources/icons/curve-minimal/OthersCurveRebuild.svg`
- **Gọi bằng dòng lệnh:** `OthersCurveRebuild`, `Rebuild`, `RebuildCrv`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Select curves to rebuild, Enter.
2. Chỉnh PointCount, Degree, DeleteInput (hoặc SelectMasterCurve để lấy theo đường mẫu); xem trước; Enter.

## Tuỳ chọn (lệnh phụ)

- PointCount=10: số điểm điều khiển.
- Degree=3: bậc (1–11).
- DeleteInput=Yes: thay đường gốc; No: tạo đường mới.
- SelectMasterCurve: chọn đường mẫu, lấy số điểm và bậc của nó (như -Rebuild của Rhino 5).
