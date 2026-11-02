# Pyramid inscribed base — 2026-10-29

The normalized Pyramid reference describes choosing a base center, corner and apex, with a configurable number of sides and Inscribed as the base style. Previously OM9 accepted only numeric sides, radius and height at the world origin. OM9 now provides a staged center/corner/apex workflow with the captured construction plane, an oriented inscribed regular polygon and signed height. `NumSides=` changes the side count before the base is fixed; the OM9 default is four. Wire preview shows the polygon and apex edges without document mutation. Resource limits are checked before preview and native construction.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: three Pyramid controller tests cover four and five sides, rotated CPlane, both height signs, preview, invalid inputs, resource limit, analytic volume and Undo. One simulated viewport workflow and one keyboard/pointer/typed NumSides workflow passed; twelve Tube regressions passed.

The four-side default and original solver behavior have not been verified against Matrix. Star, Edge, Circumscribed, direction options, right-click Star, physical desktop and GPU display remain open. This is not complete Pyramid or Solid acceptance.
