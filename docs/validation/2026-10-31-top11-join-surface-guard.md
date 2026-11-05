# Top11 Join: guard supporting surfaces during sewing — 2026-10-31

Status: partial automated evidence; not complete Top11 or M03-UI acceptance. The normalized Join contract says sewing must not change underlying surface geometry.

Top11's two-BRep and multi-surface/BRep Join now use a guarded OCCT sewing adapter. Before sewing, it records each input face's surface handle, location and OCCT GeomTools serialized surface; afterward it requires the same face count and a one-to-one match of handle/location and serialized surface. A changed surface or face count is refused before Rust publishes the transaction. This is intentionally conservative: an equivalent copied surface may be refused. OCCT exceptions remain inside the C ABI; Rust owns snapshots, transaction, scene and history. Serialization covers the representation emitted by OCCT, not a formal proof for unsupported/custom surface types; native intermediate allocations and O(n²) matching remain limits.

A kernel fixture checks the 0.0005 mm gap example through the guard. Existing Top11 fixtures cover exact, toleranced, reversed, rational curved and mixed BRep/surface Join. In the toleranced planar case, supporting surface identity/location and serialization pass the guard, while the trimmed face area changes from 12 to 12.0015 mm²; the trim adjustment remains a distinct documented behavior. No fixture forced OCCT to mutate a supporting surface, so the guard's rejection branch has not been observed at runtime.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: 79 OCCT kernel tests with the project's 8.0.1 SDK and 3dm-brep, 48 core curve_ops/top11 tests, 185 app Top11/options/BRep/snapshot tests and 107 no-default tests passed. No-default reports existing unrelated warnings. Physical desktop and signed M03-UI acceptance remain open; a prior desktop launch returned access denied. No openNURBS measurement batch was performed.

Source SHA-256: editing.rs `7807d364f4083d837fcf515c52529a163d6d1cde17e7a57259de753c59a4ecbf`; editing_bridge.cpp `8a3bd4b4240acf2e2aa82f2ca431480def37db13e8d18310de6f4f1c838721d6`; kernel tests/l2_02_05.rs `8adb4c5c4e2f07e973e81b58264d8b9deef35e279a7e4a35e5bcc359716c28ff`; app session/top11.rs `5d733d873fc6361da8b8e5c34fe6f3f1a2da1c4cd6d4bd9b8ca8ca6efd030db3`.

Update: [cylinders and budget evidence](2026-10-31-top11-join-cylinders-budget.md) covers bounded serialization refusal and a closed rational cylinder seam; surface-mutation rejection remains unobserved.
