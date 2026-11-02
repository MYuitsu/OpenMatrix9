# Pyramid Edge and Flip — 2026-10-29

The normalized Pyramid reference describes Edge using two endpoints and Flip to reverse the polygon across that edge. OM9 now constructs the regular base from the projected endpoints and configured side count, then accepts a signed apex height. Flip changes the side of the edge before or after the second endpoint. Preview uses the same construction, and Undo after the second endpoint restores the first endpoint and original input plane. Mode and preview do not create document history.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: eight Pyramid controller tests passed, including Flip before/after the edge, Undo, four and five sides, a rotated CPlane, signed height, analytic volume and budget checks. Three simulated viewport workflows, four keyboard/pointer/typed option-route tests, and twelve Tube regressions passed.

The initial side of the edge is an OM9 interpretation; original Matrix Flip behavior, physical desktop and GPU rendering remain unverified. Pyramid Star, direction options and full Solid coverage remain open.
