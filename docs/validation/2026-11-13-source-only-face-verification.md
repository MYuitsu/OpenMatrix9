# Source-only single-face verification checkpoint

Follow-up: Shell and multi-face verification have since been transitioned; see [source-only Shell worker](2026-11-13-source-only-shell-worker.md). Extract Surface construction remains on the earlier capture path.

2026-11-13. This is a scoped C1 consumer transition; the canonical document and Undo still own display data.

For a single selected root OCCT B-rep when no Shell or Extract Surface output is being built, the background face-verification worker now captures a tracked source/topology lease and a separate display pin instead of a tracked document. This includes the large-face ExtractIsocurve verification path. A published remesh pin keeps its existing owner; an ordinary document display receives a separate managed reservation for its display/topology and pin owner. This is deliberately conservative while the document and source lease already account shared data, so it can refuse under pressure earlier than a fully deduplicated representation store. The source integrity check admits its temporary scratch through the existing worker-to-controller growth protocol. Native OCCT internal allocations remain outside that accounting.

When source-only verification finishes, the worker releases its source lease and display pin and shrinks its reservation to receipt metadata. The receipt retains only the source stamp, scene publication and witness needed for the existing freshness/selection checks. Shell and Extract Surface construction, multiple selected faces, NativeBrep and other paths still use their prior capture or implementation.

One fixture starts real verification with ordinary and remeshed display, drops the controller/document while the job is alive, verifies that the document is not retained, and checks that CPU-cache and kernel reservations are released as the witness/receipt lifetimes end. A separate simulated-pressure fixture rejects capture without altering document, Undo or Redo, then succeeds after pressure is removed. In the larger Isocurve pressure fixture, early release may free enough quota for the next command step to succeed; the fixture checks clean cancellation in that case as well as safe budget refusal.

## Validation

Default features, OCCT runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app -j 2 --lib face_verification_ -- --nocapture`: 3 passed, including the ordinary/remeshed lifetime fixture and stale/cancel checks.
- `cargo test -p om9-app -j 2 --lib source_face_capture_refusal -- --nocapture`: 1 passed.
- `cargo test -p om9-app -j 2 --lib isocurve_worker_ -- --nocapture`: 3 passed; `--lib large_face_extract` and `--lib shell_worker`: 3 passed each.
- `cargo test -p om9-app -j 2 --test isocurve`: 55 passed, 4 ignored; `cargo test -p om9-scene -j 2 --lib`: 47 passed. `cargo check -p om9-app -j 2 --no-default-features` passed with 21 warnings.
- Public source audit: 15,992 source files, 594 icon bindings, 593 SVGs, no errors. `git diff --check` passed with CRLF notices; artifact pruning removed zero runs and retained 169 older cited runs.

The first integration run after early release exposed the old test's unconditional pressure-refusal assumption; the revised test accepts the successful no-document-change path as well, and the separate capture-refusal fixture preserves a deterministic budget-refusal check. These tests do not establish physical desktop/GPU, physical OOM, native peak or all topology/instance behavior. Full C0–C5, source-only document/history, general cache eviction and codec v5 remain open. Next: migrate the remaining face construction consumers away from document captures and complete canonical representation ownership and guarded restore.

SHA-256: `session.rs` `a144412b8fb215fee63ee5f7551fe5b3fd715b65e674530d11d8104042799e25`; `face_verification.rs` `9b9f2d11423a0898d0d92ce668f811e2cf525ed23fdad88a50e99204eebad029`; `isocurve.rs` test `8618f43103a29822aa0b4a2678a9fe5730df85622d34f0d1e4b0957fdcaa4aba`; `scene.rs` `b73f4a895e85cc446bb8060a31da1ac96cab16ff11a0750a8efc60464f3de734`; kernel `editing.rs` `aedf692f066ef4197b80773c98eedef7e99b859de6c9fe1e4819e18e3679867a`.
