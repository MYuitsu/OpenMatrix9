# Sphere snapshot admission — 2026-11-06

The production Sphere constructor previously copied the OCCT snapshot into Rust before entering the transaction's shared memory admission. It now uses the existing snapshot admission callback, grows a shared coordinator reservation before copying, rechecks physical memory, and transfers the materialized B-rep reservation to the existing reserved-insert transaction. A callback refusal is returned as its original error; it does not retry through an unreserved path. Failure to obtain the required physical sample is refused.

Windows worktree, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-kernel --lib snapshot_copy_admission_tests -j 2`: 6 passed, including rejection at the pre-copy callback and aggregate length/overflow checks.
- `cargo test -p om9-app --lib sphere_snapshot_admission_refusal_preserves_document_and_redo -j 2`: 1 passed. Under simulated managed pressure the Sphere request was refused with the document, published scene and existing Redo intact; after releasing pressure, a radius-5 sphere committed and survived Undo/Redo.
- `cargo test -p om9-app --test solid solid_sphere -j 2`: 52 passed. The added `solid_sphere_two_point_menu_and_cmd_commit_the_same_geometry` separately passed for menu and CMD invocation, analytic volume, placement and Undo/Redo.
- `cargo test -p om9-app --test solid_fit_source -j 2`: 3 passed. `cargo test -p om9-app --test solid_difference_ui viewport_sphere -j 2`: 6 simulated viewport tests passed.
- `cargo check -p om9-app --no-default-features -j 2`, `python tools/public_source_audit.py`, and `git diff --check` passed. Existing unused/unreachable-code warnings were emitted.

The callback accounts the Rust snapshot and validation scratch, not allocations inside OCCT before or during the native call. This is a simulated pressure fixture, not an observed whole-process or native peak bound. Desktop interaction and probe-failure injection were not tested. Document and Undo still own the display mesh; source-only ownership, eviction/remesh and the Sphere gate remain open. No existing OM9 session or drawing was changed.

## Direct numeric command follow-up

The first implementation above covered the interactive Sphere constructor, but a later caller trace found that `Sphere 5` still used `solid_primitive` and bypassed its reservation. It now calls the same admitted Sphere constructor with world origin and the existing document-unit conversion. The pressure fixture covers both the numeric command and interactive constructor: both preserve document, scene and Redo on refusal, then succeed with Undo/Redo. The direct primitive fixture measures a closed radius-5 sphere, and a centimeter fixture compares numeric and interactive input, volume, origin and one Undo entry. The existing benchmark-session fixture and the no-OCCT fallback test, including `Sphere 5`, also passed. Native OCCT peak, desktop and the ownership gate remain open; other command families that construct spherical geometry were not changed.
