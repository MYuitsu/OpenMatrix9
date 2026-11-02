# Tube FitPoints and AroundCurve — 2026-10-29

The normalized Tube reference describes FitPoints from point objects and control points, and lists an AroundCurve base on a shared two-column page. OM9 reuses the existing bounded circle fit for direct points or selected sources, and constructs the tube after a second radius and signed height. AroundCurve selects a curve and a center, uses its transformed tangent as the tube axis and stamps the source. A deleted or changed source rejects commit and suppresses preview.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: seven Tube controller tests passed, covering direct and translated selected points, fit Undo, collinear rejection, placed line orientation, deleted-source refusal, geometry and atomic Undo. Four simulated viewport workflows and three shared option-route tests passed; 25 Truncated Cone regression tests passed. The initial option-route test exposed a Pipe prompt for Tube AroundCurve; it was corrected and the test passed on rerun.

The two-column boundary means the precise original AroundCurve attribution and solver remain unverified. Changed-source and general-curve fixtures, physical desktop and GPU display remain open. Tube Tangent, WallThickness, BothSides and other documented options require further work; this is not complete Tube acceptance.
