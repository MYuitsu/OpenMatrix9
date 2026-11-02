# Tube Tangent + Point — 2026-10-29

The normalized Tube reference describes Tangent and its Point option. OM9 now supports selecting one planar curve, selecting a contact (or entering `OnCurve`), choosing Point, and selecting a non-tangent point defining the base circle. The second radius and signed height then complete the tube. The shared construction stamps the source; a deleted or changed source blocks commit and suppresses preview.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: eight Tube controller tests passed, including both tangent sides, invalid contact constraint, analytic volume, deleted-source refusal and Undo. Five simulated Tube viewport tests, four option-route tests and two Truncated Cone tangent regressions passed.

This is a bounded OM9 interpretation. Multiple tangent curves, other Tangent variants, general curves, changed-source fixture, original Matrix solver, GPU and physical desktop are not verified. WallThickness and BothSides remain open.
