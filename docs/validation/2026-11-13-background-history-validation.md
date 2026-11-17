# Background history candidate validation

2026-11-13. UI Undo/Redo now validates the private history candidate on the same shared-account worker that prepares its scene. Publication rejects an unvalidated candidate or one validated against another account. The primary coordinator's retained structural counters are carried into the peer's validation limits; registered external documents contribute their managed byte ownership there. Validation reserves temporary memory and rechecks the reservation before and after validation. The document's explicit validation-scratch inventory plus an additional document-size allowance is reserved; this is not a measured bound for allocator/native or thread overhead.

The original scene and document remain published until the validated candidate and prepared scene pass freshness and memory checks. Cancellation is checked before and after validation and scene preparation, not within those calls. Candidate capture, retained-owner inventory and parts of publication still run on the caller; this change does not establish a latency guarantee. Programmatic history and Open remain synchronous.

## Verification

With `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`, and the OCCT 8.0.1 SDK runtime on PATH for app tests:

- Core library: 386 passed, 1 ignored. The new candidate test rejects publication before validation, rejects a foreign account and simulated physical-memory refusal, and validates on a separate thread before publication.
- App `--lib history_worker -- --test-threads=2`: 5 passed, including cancellation, stale selection, simulated worker memory refusal and restored OCCT box scene/pick.
- App `--lib undo -- --test-threads=2`: 132 passed, including CMD/toolbar history; the shortcut/cancel and pending GPU image fixtures also passed individually. Workspace `cargo check` passed with two existing warnings; app no-default-features check passed with 21 warnings.
- Public source audit: 16,009 files, 594 bindings, 593 SVGs, no errors. Artifact pruning removed 0; 169 cited older runs were retained.

This remains a partial history-worker improvement, not completion of the source/display plan. Document and Undo retain display; NativeBrep separation, general eviction, attested source-only cold reconstruction, `.om9` v5, desktop and performance acceptance are outstanding. Real physical OOM, native/GPU peak, worst-case scratch accounting and interruption during validation have not been measured.

SHA-256: core `transaction.rs` `10f0b105806a70f6f01b6efa2be549292a3e58b8c05ad8fa69d46d8038965db0`; app `session.rs` `a2751592b4aa61d5790a39682e9dbc9998e009198f73b7007289bcbcaf86d429`.
