# Torus 3Point — 2026-10-29

The normalized manual extract associates Torus with a 3Point base circle, but the source pages contain adjacent Torus and Cylinder material. OM9 now accepts `3Point` via command option and `Torus 3Point` / `_Torus _3Point`, collecting three ring points and a tube radius. Points are projected into the CPlane captured at the first point; the circumcircle calculation is safe Rust. This projection and tolerance are OM9 decisions, not verified native Matrix solver behavior.

Collinear or repeated points are rejected without losing earlier inputs. Input Undo removes the third point; the final exact Torus BRep is published once, and document Undo removes it. The tube radius uses the previously documented radial/normal section at the third ring point. The `Radius` branch after two points remains unimplemented.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: three focused Torus controller tests passed, including 3Point in both orientations on a tilted CPlane, degenerate retry, input Undo, analytical volume and document Undo. Two egui viewport workflows covered Center and 3Point, and one shared-option fixture exercised 3Point via keyboard, pointer and typed activation. Desktop/GPU, original Matrix behavior, unusual numerical limits, wire preview and other Torus modes remain unverified.
