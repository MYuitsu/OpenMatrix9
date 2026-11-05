# Adjustable Curve Blend

Nối chuyển tiếp chỉnh được (Rhino BlendCrv): chọn hai đầu, chỉnh độ liên tục và độ phình rồi Enter.

- **Mã lệnh:** `CurveBlendCurvesBlendCrv`
- **Vị trí:** menu Curve, ô số 12
- **Icon:** `Resources/icons/curve-minimal/CurveBlendCurvesBlendCrv.svg`
- **Gọi bằng dòng lệnh:** `CurveBlendCurvesBlendCrv`, `BlendCrv`, `AdjustableBlend`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Select first curve to blend - select near end.
2. Select second curve to blend - select near end.
3. Chỉnh Continuity1/2, Flip1/2, Bulge1/2, Join; xem trước; Enter.

## Tuỳ chọn (lệnh phụ)

- Continuity1/Continuity2=Position/Tangency/Curvature/G3/G4 (mặc định Curvature).
- Flip1/Flip2: đảo hướng ở đầu đó.
- Bulge1/Bulge2: độ phình (thay cho kéo điểm điều khiển).
- Join: nối đường nối với hai đường cong.
