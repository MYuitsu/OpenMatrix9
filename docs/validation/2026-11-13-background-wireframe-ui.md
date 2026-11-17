# Single-root background ExtractWireframe UI checkpoint

2026-11-13. This is scoped progress toward the source/display completion plan, not completion of I1 or C1–C5.

For one selected root OCCT B-rep, ordinary egui menu and CMD invocation now start the source-lease worker. The worker prepares curves off the UI thread; the controller checks freshness and commits the result in one transaction when polled. Controller-only callers retain their existing synchronous default. Replacing the document preserves the UI's background mode. Other source types and multiple roots still use their existing paths.

Asynchronous managed-memory refusal can offer the existing recovery choice only while the initiating draft remains unchanged. An accepted retry waits for completion and for an earlier worker to release its permit. Changes to the selection, document context or draft stop recovery rather than publishing stale work or discarding Undo. Cancellation is cooperative and does not interrupt an OCCT call already executing.

## Validation

With default features, OCCT SDK runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, and `-j 2`:

- `cargo test -p om9-app -j 2 --lib ui::undo_settings::tests -- --nocapture`: 86 passed, including background refusal, retry and changed-draft cases.
- `cargo test -p om9-app -j 2 --lib menu_and_cmd_start_background_brep_wireframe -- --nocapture`: 1 passed; both routes, delayed publication and Undo/Redo for a Box.
- `cargo test -p om9-app -j 2 --lib source_wire_worker_ -- --nocapture`: 5 passed; controller and egui worker checks including stale state, cancellation and managed pressure.
- `cargo test -p om9-app -j 2 --lib document_replacement_preserves_background_wireframe_mode -- --nocapture`: 1 passed.
- `cargo test -p om9-app -j 2 --test solid_difference_ui viewport_extract_wireframe_reopened_box_via_icon_and_typed_cmd -- --nocapture`: 1 passed, exercising reopened Box via the viewport icon and typed command with frame polling.
- `cargo check -p om9-app -j 2 --no-default-features` passed with warnings; it does not exercise OCCT. Public source audit checked 15,988 source files, 594 icon bindings and 593 SVGs with no errors. Tracked code `git diff --check` passed with CRLF notices. Artifact pruning removed zero runs and retained 169 older referenced runs.

These headless fixtures do not prove physical desktop/GPU behavior, physical OOM, native peak memory, arbitrary topology or the complete M01–M09 plan. NativeBrep, multi-root and block worker integration, ordinary scene wires, source-only document/history ownership, eviction, cold restore and v5 remain open. Previous source-wire checkpoint evidence is historical where it describes menu/CMD as synchronous.

Next: continue the canonical source/display ownership and history migration in `implementation/design/M01/viewport-export-separation-20261102/PLAN_SOURCE_DISPLAY_COMPLETION_M01_M09.md`, and extend background work to the remaining source and occurrence paths with equivalent admission and freshness checks.
