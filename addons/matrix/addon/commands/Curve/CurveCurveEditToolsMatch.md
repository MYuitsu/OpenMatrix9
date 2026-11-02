# Match Curve

Khớp đầu một đường cong vào đầu đường cong khác (Rhino Match).

- **Mã lệnh:** `CurveCurveEditToolsMatch`
- **Vị trí:** menu Curve, ô số 52
- **Icon:** `Resources/icons/curve-minimal/CurveCurveEditToolsMatch.svg`
- **Gọi bằng dòng lệnh:** `CurveCurveEditToolsMatch`, `Match`, `MatchCrv`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Select open curve to change - select near end.
2. Select open curve to match - select near end.
3. Chỉnh tuỳ chọn, Enter.

## Tuỳ chọn (lệnh phụ)

- Continuity=Position/Tangency/Curvature (mặc định Curvature).
- PreserveOtherEnd=None/Position/Tangency/Curvature.
- AverageCurves: cả hai đường cùng đổi, gặp nhau ở giữa.
- Join: nối hai đường sau khi khớp.
- Merge: gộp thành một đường (khi Continuity=Curvature).
