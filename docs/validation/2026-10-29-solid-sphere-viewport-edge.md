# Sphere AroundCurve viewport edge — 2026-10-29

The viewport now supports a source-selection stage for standalone curves and BRep edges. For BRep, a second pick must reference the same edge and its world hit is projected onto the edge; ambiguous picks are rejected. Undo before a center removes the source, while Undo after the center restores center selection. Stale source/epoch and instance edges are rejected. Scene/UI uses existing hit revalidation; no selection click publishes geometry.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_sphere_`: 51 passed, including source/center stages, wrong edge, Undo, and commit on a box edge.
- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_sphere_aroundcurve`: 2 passed, simulated egui viewport picking of a line and a box edge with center placement. An initial BRep test point projected outside the viewport; the corrected in-viewport fixture passed.
- `python tools/public_source_audit.py`: 14722 source files, 594 icon bindings, 593 SVGs, no errors.

Only the stated controller and simulated pointer fixtures are proven; desktop/GPU, overlapping-edge disambiguation, instance paths, general BRep curves and Matrix-original behavior remain open. A standalone curve center still uses the existing viewport point path; no claim of native world picking there.
