# Ring B-Rep feasibility uses source measurements

Follow-up: the producer subsequently uses a same-call native volume receipt rather than decoding the finished source again; see [sewed ring volume receipt](2026-11-13-sewed-ring-volume-receipt.md). This record describes the earlier verified build.

2026-11-13. The OCCT ring-family producer previously passed viewport-triangle volume to the portable gate and assumed the B-Rep was closed. It now measures the B-Rep source with the selected OCCT adapter and requires one solid, one shell, no naked edges, closedness and positive measured volume before publication. OCCT performs numerical surface/volume integration; this is source-based measurement, not a claim of symbolic exactness for every surface. Authored mesh continues through the Rust edge/volume gate. The core `ring_builder::inspect_geometry` API now explicitly returns `Unsupported` for B-Rep; callers needing B-Rep inspection must use `om9_kernel::associative::inspect_ring_geometry`.

The shared `creation::measure` API validates canonical source rather than the display, but inventories and charges still-live display bytes while admitting source validation scratch. It does not use display points or triangles as the geometry oracle. An invalid display is not thereby accepted for document publication.

On Windows with `CARGO_TARGET_DIR=L:\om9-target`, OCCT 8.0.1 runtime and `-j 2`:

- Kernel `--features 3dm-brep --test ring_family_brep`: 5 passed. Four ring families are produced/rebuilt and compared to the mesh construction; a missing face is refused. A sphere fixture changes every display point, and a second damages display with NaN; source volume remains within 1e-6 mm³ of the analytic 36π mm³. A budget fixture rejects when live source/display and validation scratch cannot fit.
- App `--test ring_family_panel`: 5 passed with OCCT; the same test with `--no-default-features` passed 5 using authored meshes. App `--lib measure`: 6 passed. Kernel `--features 3dm-brep --test extrusion`: 3 passed. Core lib: 386 passed, 1 ignored. Workspace check passed with two existing warnings.
- An initial kernel integration-test invocation without `3dm-brep` failed to link three optional 3DM conversion symbols; rerunning with the SDK feature succeeded. The no-default app test passed with 21 existing warnings.

This closes only the identified ring producer's display-volume dependency, not C1–C5. Builder preview/rebuild remains synchronous in the current caller; the added native measurement may affect responsiveness and its process/native peak and real physical OOM were not measured. General geometry consumers, source-only Document/Undo, cache eviction/cold restore, NativeBrep disposition, v5 and desktop acceptance remain open. No complete M04 or overall milestone acceptance is asserted.

SHA-256: `ring_builder.rs` `799bc40f36741dd0543901626874822175eb73623b24afdf1e4f9f8896463ec4`; `creation.rs` `9dd043c6a2a16208c811a9700e38bd6cda60bfd9000191d40245f4d6359b60b4`; `associative.rs` `569b71ebd49a759f180582bda988d5bcdbc071151dc7d2195c73aa849ffd8fc5`; `ring_family_brep.rs` `19275ffd66649c7b4a6ced7993dc5dab6a2f78da345cee9c65bc67976482f7e4`; `ring_family_panel.rs` `e662a7c5b86bb8b6d5515c5fb308a85ff194ebed55c4f0c2ace37c0e4b64817d`.
