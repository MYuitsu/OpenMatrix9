# Slab closed-polyline development session — 2026-10-29

The hidden session now captures closed line polycurves and continuous degree-one NURBS, derives the source plane and signed offset from the nearest segment, previews both mitered rings and their extrusion, and commits through the source-stamped closed publication path. The preview is generated from the same validated ring geometry as publication.

Controller fixtures exercise rectangle sources of both types and both directions, outward and inward/BothSides picks, preview edge counts, measured BRep volumes and Undo. Simulated viewport fixtures select each source after start, choose outward side and height, measure volume within 5 mm³ pixel-pick tolerance and Undo. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 24 tests; `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 12; `cargo check -j 2 -p om9-app --no-default-features --message-format short` passed with 15 warnings.

This does not establish behavior for arbitrary concave offsets, periodic or higher-degree closed curves, multiple loops, actual desktop or Matrix-original semantics. Public Slab remains disabled.
