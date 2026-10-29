# Tube 3Point Radius, preview and right-click — 2026-10-29

The normalized Tube reference describes the Radius option after two circumference points. OM9 now accepts a typed or picked radius at least half the chord and a side pick, then continues to the second radius and signed height. The SolidTube right-click binding now dispatches the existing Tube2Point mode, matching the recovered `_Tube _P` wrapper recorded in `docs/analysis/matrix-command-layers.md`.

Wire preview shows the first base circle, then both base circles, and at height both end circles and four generators per wall. It uses the captured frame and the same signed-height calculation as commit; it does not change the document. Invalid or zero sizes do not produce a body preview.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: four Tube controller tests passed, including both side solutions, invalid radius, Undo, signed-height preview and analytic volume; two simulated viewport workflows, one simulated right-click and one keyboard/pointer/typed Radius option workflow passed. Cylinder and Truncated Cone regression tests cover shared machinery.

These are controller and simulated egui checks, not evidence of original Matrix solver behavior, GPU rendering or physical desktop interaction. FitPoints, Tangent, AroundCurve and full Tube coverage remain open.
