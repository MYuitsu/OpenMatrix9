# Cylinder 3Point and Radius — 2026-10-29

The normalized Cylinder section describes a base circle through three points and an alternative Radius/side choice after two points. Its source pages interleave adjacent Torus and Tube material. OM9 now supports `Cylinder 3Point` and its command option using the existing captured-CPlane circle and chord/radius solver. It then asks for signed height and uses the same `Solid=Yes/No` cylinder commit as Center. The projection, side convention and tolerance are OM9 interpretations; original Matrix solver behavior remains unverified.

Invalid collinear points and invalid radius/side are rejected without committing. Selection-stage Undo and one-transaction document Undo use the existing shared session. The shared partial wire shows the base circle and height guide; this is not a complete BRep preview.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: three focused Cylinder controller tests passed, including both Radius sides, 3Point geometry, open lateral face and Undo. Two simulated viewport workflows passed, including 3Point Radius side and height; one shared-option fixture exercised 3Point via keyboard, pointer and typed input. Twenty-five Truncated Cone regression workflows passed. Actual desktop/GPU, original Matrix solver, other Cylinder base modes and general numeric limits remain unverified.
