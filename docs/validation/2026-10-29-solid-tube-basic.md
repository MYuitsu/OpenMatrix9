# Tube Center, 2Point and 3Point — 2026-10-29

The normalized Tube reference describes choosing a base circle, a second concentric circle and a height; the recovered Matrix command wrapper maps `SolidTube` to `_Tube` and its right-click action to `_Tube _P` (`docs/analysis/matrix-command-layers.md`). Previously OM9 only accepted three numeric dimensions at the world origin.

OM9 now supports Center, 2Point and 3Point base input, followed by a second radius and signed height. Point input uses the captured construction plane; either radius may be entered first. Typed Diameter applies to both circular sizes. Invalid equal radii and zero height are rejected without committing; the closed OCCT tube is inserted atomically. `Tube2Point` dispatch is connected to the 2Point session. This does not prove a physical right-click menu binding.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: two Tube controller tests passed, covering the three base modes, rotated CPlane, positive/negative height, swapped radii, Diameter, invalid input, volume and Undo. One simulated viewport Center workflow passed. Nine Cylinder regression tests passed.

Tube FitPoints, Tangent, AroundCurve and 3Point Radius option, preview, physical desktop/menu behavior and the original Matrix solver remain unverified or unimplemented. The interpretation allowing either radius order is OM9 behavior and has not been checked against Matrix.
