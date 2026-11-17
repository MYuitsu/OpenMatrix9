# ExtractWireframe root source lease

2026-11-13. Behavior-preserving transition toward C1; the full separation is not complete.

For a single selected root OCCT B-rep, ExtractWireframe copies the placement and source stamp, checks the active scene and then captures a tracked `CapturedBrepSource`. It validates that source/topology under admission, extracts native edges and, when already displayed by the remesh scene, default interior isocurves from the borrowed source. It drops the source lease before committing the independently owned output curves. This path no longer creates a tracked whole-document capture for the extraction. The source is also included conservatively in the validation reservation, even though the lease is independently tracked.

The existing tracked-source lifetime test closes its source document while a source lease is alive, verifies the display allocation is no longer alive and the source remains available, and checks shared accounting. This test proves the lease's owner behavior, not that ordinary active scenes or Undo are source-only. The command still runs synchronously; multiple-root, NativeBrep, mesh and nested occurrence paths retain their existing handling. `Brep` and `NativeBrep` still own display and the v4 codec still stores it.

## Validation

- Default `om9-app` features, OCCT runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: `cargo test -p om9-app -j 2 --lib extract_wireframe_ -- --nocapture`: 4 passed, 846 filtered. `cargo test -p om9-app -j 2 --test wireframe`: 89 passed, 8 ignored. These cover existing menu/CMD, placement, remesh, Undo/Redo and simulated pressure fixtures at their recorded scopes.
- `cargo test -p om9-core -j 2 tracked_brep_source_outlives_document_without_retaining_display_or_double_charge`: 1 passed, 1,215 filtered. This is an existing focused ownership fixture, not a cold Undo or process-memory measurement.
- Public-source audit: 15,985 source files, 594 bindings, 593 SVGs, no errors. `git diff --check` passed with a CRLF warning. SHA-256 `crates/om9-app/src/session/wireframe.rs`: `a5674365944d71d1e5def5b148e7fb96cf44eee7f3f0be7fb11689565557bfb3`.

Next: transition the other consumers to tracked source and independently owned representations, and implement source-only document/history, worker publication and v5 migration. I1 and C0–C5 remain open; no GPU, desktop, actual OOM or native peak claim is made here.
