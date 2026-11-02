# Tube WallThickness — 2026-10-29

The normalized Tube reference describes WallThickness as the distance between the inner and outer walls. After the first radius is established, OM9 exposes `WallThickness=`. A positive typed thickness sets the second radius inward from the first and must be smaller than it. Diameter mode does not halve a thickness. In the point form, the radial location specifies the second circle, allowing a pick inside or outside the first. Invalid input keeps the session, and Undo can leave the thickness stage without changing the document. This choice of numeric direction is an OM9 interpretation; the available normalized reference does not resolve original Matrix direction semantics.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: Tube controller tests cover numeric and point input, Diameter, invalid thickness, undo and combination with BothSides; a simulated viewport pick and keyboard/pointer/typed option activation passed. Shared Truncated Cone regressions were run.

Original Matrix behavior, other direction conventions, physical desktop and GPU rendering remain unverified. Multi-curve Tangent and further Tube options remain open.
