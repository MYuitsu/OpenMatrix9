# Cylinder AroundCurve — 2026-10-29

The normalized Cylinder section describes a base circle perpendicular to a curve, but its text is interleaved with Torus and Tube. OM9 now accepts `Cylinder AroundCurve` and its command option. Select one curve, pick the center or enter `OnCurve=0..1`, then choose the base radius and signed height. The tangent-aligned frame comes from the existing selected Curve solver. Projection of the active CPlane into that frame is an OM9 interpretation, not proof of the original Matrix solver.

The curve is preserved and stamped. A deleted or changed source suppresses the partial preview and prevents commit; the nearest-point picker requires a similarity placement. One transaction produces the cylinder, and Undo removes it.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: five Cylinder controller tests passed, including translated line and circle, tangent direction, analytic closed volume, source preservation, deleted-source refusal and Undo. One egui viewport test exercised curve and center selection; one shared-option test exercised keyboard, pointer and typed activation. Two existing Truncated Cone AroundCurve regressions passed. Changed-source fixture, general curves, open Cylinder AroundCurve, actual desktop/GPU and original Matrix behavior remain unverified.
