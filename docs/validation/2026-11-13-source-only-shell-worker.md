# Source-only Shell worker checkpoint

Follow-up: Extract Surface construction has also moved to source/display leases; see [source-only Extract worker](2026-11-13-source-only-extract-worker.md). Other limitations below remain.

2026-11-13. Scoped C1 consumer transition; document/Undo still own viewport displays and the broader plan remains open.

The background Shell worker now uses tracked canonical source and separately pinned scene display instead of retaining a document. Selections on the same root share the source lease and display pin. The OCCT adapter validates/adopts the captured source and calls the same Shell operation with the source, verified face witnesses and separate witness/output display deflections. Rust source-validation and output scratch are admitted through the worker reservation; native OCCT temporary allocations are not covered. After construction, the worker releases the source and display pins while keeping the resulting B-rep and its reservation until the receipt is consumed or dropped. Existing freshness and transaction checks remain in the controller.

A worker fixture selected one and two faces of a box, dropped the controller and document before awaiting the result, and verified that the document was not retained. In both cases it measured a positive Shell volume below the original 120 cubic units, observed the display reservation released after construction, and saw the output reservation released after receipt drop. This is not a complete spatial oracle. Existing Shell tests cover additional success, cancellation, stale and managed-pressure branches. The ordinary display pin is conservatively charged even while its allocations are also represented in the document/source accounting; this may refuse earlier under pressure.

## Validation

OCCT SDK runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app -j 2 --lib shell -- --nocapture`: 17 passed, including the one/two-face lifetime fixture. `--lib shell_worker`: 4 passed. `--lib face_verification_`: 3 passed; `--lib isocurve_worker_`: 3 passed.
- `cargo test -p om9-app -j 2 --test solid shell -- --nocapture`: 1 passed.
- `cargo test -p om9-kernel -j 2 --features 3dm-brep --test l2_01_10 shell -- --nocapture`: 5 passed. An initial run without the `3dm-brep` feature failed to link 3DM conversion symbols; the feature-enabled run is the result above.
- `cargo check -p om9-app -j 2 --no-default-features`: passed with 21 warnings. Public source audit: 15,993 source files, 594 icon bindings, 593 SVGs, no errors. `git diff --check` passed with CRLF notices; artifact pruning removed zero runs and retained 169 cited older runs.

Extract Surface construction still uses a document capture; NativeBrep, instances, full ordinary-scene and broad topology coverage, physical desktop/GPU, physical OOM and native peak remain unproved. C0–C5, source-only canonical document/history, general eviction, cold restore and v5 are not complete. Next: transition Extract Surface construction and its grouped source validation to independently accounted source/display leases, then continue the canonical ownership and restore migration.

SHA-256: app `session.rs` `fbb991bce9fb950048f5c1a918b115c62734c9be174572911e78854b8a20b885`; `face_verification.rs` `fd4ba33d943811814c347011c4b1e82ebc39d3ee5f5b887b2c08edfde8ba6f2f`; kernel `creation.rs` `ea21b0b40b435021837a6f30c9f642650f4599a541d3358573c1365eff712e11`; `editing.rs` `2eb993a674d6f9aae7d1fffa7f0d80591ee77dbeb45dc635a7eb714ef111154a`.
