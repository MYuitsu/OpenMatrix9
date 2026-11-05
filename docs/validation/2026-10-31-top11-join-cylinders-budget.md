# Top11 Join: cylindrical seams and surface-guard budget — 2026-10-31

Status: partial automated evidence; not full Top11 or M03-UI acceptance.

The OCCT supporting-surface guard now writes each serialized surface through a bounded stream, refusing a snapshot that exceeds the remaining byte allowance before emitting a BRep snapshot. Rust still owns document and transaction state, and native exceptions remain confined to the C ABI. This bounds the stream's logical output; OCCT intermediate allocations, string capacity and overall native peak memory are not certified by this fixture.

A direct ABI unit fixture passes a four-byte limit and verifies refusal before the output callback. It exercises the snapshot budget path, not the surface-mutation rejection branch. An app fixture joins two stacked closed rational NURBS cylinder surfaces along a circular edge through CMD and toolbar; it checks two faces, one shell, total area 24π mm² and Undo/Redo. These surfaces have closed clamped representations with periodic=false; this does not establish general periodic-surface support.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: one native guard unit test, 79 OCCT kernel tests with the project's 8.0.1 SDK and 3dm-brep, 48 core curve_ops/top11 tests, 186 app Top11/options/BRep/snapshot tests and 107 no-default tests passed. No-default reports existing unrelated warnings. Surface-mutation rejection, tolerance trim semantics, general periodic surfaces, native peak allocation, 3DM and physical M03-UI acceptance remain open. No openNURBS measurement batch was performed.

Source SHA-256: editing.rs `f37d2d74cbd58787e5c8d8ea26e5210a522f0b9d74b7aff2ae78ace0ceadba8b`; editing_bridge.cpp `36191827750b8389354c847c7183c414b7892e27aee30f4ced0490b5855cc6a6`; app tests/top11_brep.rs `0aa3a00c478f3565ac4eca74f2d3120aed5903ed7afff17ef71ab2dae7abdd7e`.

Update: [periodic surface evidence](2026-10-31-top11-join-periodic-surfaces.md) covers certified repeated-basis periodic surfaces; other periodic forms remain open.
