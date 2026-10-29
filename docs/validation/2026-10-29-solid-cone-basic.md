# Cone staged base and apex — 2026-10-29

The normalized Cone reference describes Center, 2Point, 3Point/Radius, FitPoints, Tangent/Point, AroundCurve, Area, Solid and a 2Point right-click action. The recovered wrapper maps the right-click to `_Cone _P` (`docs/analysis/matrix-command-layers.md`). Previously OM9 only accepted two numeric dimensions at the world origin.

OM9 now uses the shared staged base construction, then builds an OCCT cone to the signed apex height. Closed and open lateral surfaces are supported; the latter has one naked boundary because its apex is degenerate. A failed first open-surface test exposed the previous truncated-cone assumption of two naked boundaries; the condition was corrected against a direct topology measurement. Cone preview draws the base and four generators, while a changed or deleted referenced curve prevents commit and hides preview.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: five Cone controller tests cover Center/Area, 2Point, 3Point/Radius, FitPoints, one-curve Tangent + Point, AroundCurve, open topology, source deletion, signed height, preview, volume and Undo. The simulated viewport and right-click workflows passed; eleven Cone/Truncated Cone option-route tests, nine Cylinder and 25 Truncated Cone regressions passed.

These checks do not establish the original Matrix solver, multiple-curve tangency, changed-source fixture, direction options, actual desktop or GPU display. Full Cone and Solid acceptance remain open.
