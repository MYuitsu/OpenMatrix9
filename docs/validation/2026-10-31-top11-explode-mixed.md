# Top11 Explode: mixed groups and unreferenced mesh vertices — 2026-10-31

Status: partial automated evidence; not complete Top11 or M03-UI acceptance.

Explode now preserves mesh vertices not referenced by any face, including normals and colors, exactly once on the deterministic first resulting component. A mesh with only one edge-connected component remains refused without mutation. This avoids dropping isolated data when the mesh otherwise has multiple components. The scratch admission includes additional vertex/channel storage; exact aggregate peak allocation remains unproven.

The mesh fixture checks an isolated colored vertex and normal, both CMD and toolbar, .om9 roundtrip and Undo/Redo. A max_outputs=1 fixture refuses an operation producing two components without mutation. A separate mixed fixture selects one polycurve, a two-component mesh and a six-face BRep in one group, leaves a point untouched, and checks all ten replacement objects retain group membership via CMD and toolbar, with .om9 roundtrip and one Undo/Redo. Other topologies, selections and group interactions remain unproven.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: 85 core document graph/change/curve_ops/top11 tests, 171 app Top11/options/BRep/snapshot tests and 99 no-default Top11/options tests passed. The no-default build reports existing unrelated warnings. Physical desktop and signed M03-UI acceptance remain open; the prior desktop launch reported access denied. No native implementation was changed and no openNURBS measurement was performed.

Source SHA-256: core top11.rs `77a5ccbd9ae563107ee96dc04bbee322e12a8ea4af845fab284313af7b9b4f7c`; app tests/top11.rs `3eeb219a380328b3d024e22f12dd1ed18af15c48c4aafa56cb9ed8b3bead4828`; app tests/top11_brep.rs `23aee324ebd20d2240d88dda430b820a443b8a946ea8b4314868d3e01712fadb`.
