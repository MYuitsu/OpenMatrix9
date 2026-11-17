# Explicit source-wire worker checkpoint

2026-11-13. This is a transition toward background source work and does not close I1 or C1–C5.

An explicitly invoked controller API now captures a tracked source/topology lease for one selected root OCCT B-rep and prepares ExtractWireframe curves on a worker. The worker does not capture a document or scene; it receives placement, the source lease and whether the validated scene has interior wires. It sends reservation-growth requests back to the controller, and never holds the accounting lock across native calls. A single live worker is permitted; cancellation is cooperative and cannot interrupt an OCCT operation already executing.

Polling is nonblocking. It checks session authority, epoch, source stamp, root selection revision, current layer, command run and scene publication before growing memory or publishing. On successful completion it commits the owned output in one controller transaction; stale context, cancel, allocation failure and worker errors do not publish. The egui frame now polls an explicitly started worker, deferring completion while a pointer gesture is active.

The regular menu/CMD dispatch still invokes the synchronous implementation; it does not yet start this worker. Memory recovery UI and normal command invocation must be connected and verified before calling ExtractWireframe a background product command. NativeBrep, multiple roots, blocks, ordinary scene isocurve construction and source-only document/Undo remain open.

## Validation

- Default `om9-app` features, OCCT SDK runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: `cargo test -p om9-app -j 2 --lib source_wire_worker_ -- --nocapture`: **5 passed**, 850 filtered. Four controller fixtures cover actual worker thread, explicit poll/one Undo, changed-and-restored selection, same-epoch scene replacement, cancel and managed-pressure refusal/retry after remesh; one egui fixture explicitly starts the worker and verifies frame polling and Undo. Physical desktop/GPU, physical OOM and native peak are not proved.
- `cargo test -p om9-app -j 2 --lib extract_wireframe_`: **4 passed**, 851 filtered. `cargo check -p om9-app -j 2 --no-default-features`: exit 0, 20 warnings; that check does not exercise OCCT.
- Public source audit: 15,987 source files, 594 bindings, 593 SVGs, no errors. `git diff --check` passed for tracked code with CRLF warnings; the new worker file has no trailing whitespace. Artifact pruning removed zero runs.
- SHA-256: `wireframe_worker.rs` `422ac5bf3f30cda241ffcbc7daa26a59f1fe33f5889cc3a6e95ed8e79f0ccadb`; `session.rs` `05efa3f720dd2195fbdcb2ddebe277ed29cbdd4fd7d3f589e79be849846a1d33`; `command_line.rs` `0ecc23c91ea895b4a878c0ee3f6c8926076d33d5abe308917cf20bcc719e2f10`; `wireframe.rs` `90804b534cbafe78cd1a1b7c14ee29c005b8044078f2ff4fd87ea63fe847e386`; `ui.rs` `aa5b4009928a90de89e826874ab4c1075ba91373b96ee69b92b82204793e7d37`.

Next: route real menu and CMD invocation to the worker while preserving command events, input/cancel semantics and memory recovery consent/retry, then cover multi-root/native/instance and continue canonical ownership, cache eviction, cold restore and v5.
