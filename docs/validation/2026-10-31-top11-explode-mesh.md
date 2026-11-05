# Top11 Explode: mesh components — 2026-10-31

Status: partial automated evidence; not full Top11 or M03-UI acceptance. The local Top11 reference specifies splitting meshes along unwelded edges.

Rust groups mesh faces through shared-index edges, then creates one mesh per connected group. Distinct indices at coincident positions do not weld an edge. Each result preserves face winding, vertex positions, per-vertex normals/colors, per-corner UVs, placement and source attributes. A mesh with only one edge-connected component or unreferenced vertices is refused without modifying the document. Mixed curve/mesh selections and mesh/BRep selection admission use the same Explode action; broader mixed cases are not separately verified here.

A two-component fixture has a welded pair of triangles and a third triangle with coincident but distinct edge vertices, channel data and placement. It checks CMD and toolbar, preselection and select-after-invocation, .om9 roundtrip, Undo/Redo, and connected/orphan refusal. These are representative cases, not proof for every topology, non-manifold mesh, resource bound or mixed selection. Resource admission limits output count before constructing pieces and uses a conservative temporary allocation estimate; exact aggregate peak memory remains unproven.

Windows x64, CARGO_TARGET_DIR=L:\om9-target and -j 2: 48 core curve_ops/top11 tests, 167 app Top11/options/BRep/snapshot tests and 96 no-default Top11/options tests passed. The expanded select-after fixture was rerun in both configurations after the broader suites. No-default reports existing unrelated warnings. The final executable built successfully.

A desktop verification attempt using the previously built executable was made before the mesh change; Windows returned `GetCursorPos failed: Access is denied. (0x80070005)` on two launch attempts. No interactive desktop test or human M03-UI signoff was obtained. The current executable was rebuilt after the code change, but no later launch was attempted following that access failure. Native SDK and openNURBS measurement were not changed or measured.

Source SHA-256: core top11.rs `7b266d4000a2d7ab1bb5b1b4c62b30f110291b773d37bba1d0dea5b3199e15bc`; app session/top11.rs `3c9315cb527690052e1fdddf35b92269380d6270e5714b855e650cec717c8ba1`; app tests/top11.rs `5086c99b535df6f5fe4e8a8e49b7650dbda242afed3c5881f469bffef6d8d869`.

Update: the earlier refusal of unreferenced vertices is superseded by [mixed Explode evidence](2026-10-31-top11-explode-mixed.md), which preserves them once.
