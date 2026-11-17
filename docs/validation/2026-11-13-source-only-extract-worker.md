# Source-only Extract Surface worker checkpoint

2026-11-13. Scoped C1 consumer transition; document/Undo still own display and the complete source/display plan remains open.

The background Extract Surface worker now captures tracked B-rep sources and independently pinned scene displays, sharing a lease among selected faces of the same root, instead of capturing the document. The unused `CapturedRootBrepDisplay` API, which retained a document to hold the display, was removed; its test now checks the independent pin across scene replacement and rejects a foreign stamp. Source-only OCCT adapters validate the canonical record and use the existing native witness verification, face extraction and single/multiple-face remainder operations. Rust source-validation and output scratch are admitted alongside already retained output; native OCCT temporary allocations are outside the managed callback. Once construction finishes, source and display pins are released, while outputs and the compact receipt remain reserved until commit or drop. Existing source stamps, scene publication, selection and transaction checks still gate publication.

The grouped lifetime fixture selects two box faces and one sphere face across two roots, with Copy both on and off. After it drops the controller/document, the worker finishes with three positive-area single-face B-reps and the expected keep/delete/remainder group counts; the old document is not retained, display reservations are released, and the output reservation lasts until receipt drop. This fixture's area/face-count checks are not a full independent spatial oracle. Existing tests separately exercise source placement, selected face, Undo/Redo, one-transaction behavior, stale/cancel, changed options, reservation growth and simulated refusal.

## Validation

OCCT SDK runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app -j 2 --lib grouped_extract_worker_does_not_retain_document -- --nocapture`: 1 passed (Copy and Move); `--lib captured_face_pin_retains_scene_override`: 1 passed.
- `--lib extract_worker_uses_remeshed_witness_and_source_output_profile`: 1 passed. A sphere remeshed from 0.1 to 1.0 supplies the selected witness to the background worker; the extracted face retains the 0.1 output profile and expected area, and changing selection invalidates the receipt. This checks the worker receipt, not a desktop interaction or its final transaction.
- `--lib extract_worker`: 9 passed, 2 ignored; `--lib large_face_extract`: 3 passed; `--lib large_numeric_extract`: 2 passed; `--lib large_multi_face_extract`: 1 passed; `--lib large_inline_extract`: 1 passed. These groups overlap.
- `--lib extract_and_remainder_snapshots_honor_admission`: 1 passed; `--lib extract_move_worker_obeys_same_object_limit`: 1 passed; `--lib extract_surface`: 10 passed. `--lib shell_worker`: 4 passed; `--lib isocurve_worker_`: 3 passed.
- `cargo test -p om9-app -j 2 --test solid solid_extrude_pipe_extract_and_fillet_dispatch -- --nocapture`: 1 passed; `--test isocurve`: 55 passed, 4 ignored. `cargo test -p om9-scene -j 2 --lib`: 47 passed. `cargo check -p om9-app -j 2 --no-default-features` passed with 21 warnings. Two attempted `extract_surface`/`extract` filters selected zero integration tests and are not counted as evidence.
- Public source audit: 15,995 source files, 594 icon bindings, 593 SVGs, no errors. `git diff --check` passed with CRLF notices; artifact pruning removed zero runs and retained 169 cited older runs.

Ordinary display pin reservations still conservatively overlap document/source accounting. NativeBrep and block/instance support, general topology, desktop/GPU, physical OOM and native peak are not established. Canonical document/Undo ownership, ordinary scene, full cache/eviction, cold restore, v5 and C0–C5 remain incomplete. Next: continue canonical document/representation separation and migrate remaining consumers; build guarded source-only restore and v5 according to the plan.

SHA-256: app `session.rs` `4e6d4c238f64c00a970e24ee9691f996eed2fe08a375be7970032a024ed7eb3f`; `face_verification.rs` `ad8586a15b3b65ca491958e111faa834a634daccd9b9d8d35ff8e8cc0cac86d6`; kernel `editing.rs` `819405722eeb2ee98b698d84001b9e2c17c0e6151085478264c7291db02964fe`; scene `scene.rs` `b76f2a9d75297231dfef53bf01fe61d18aa63e0034629432621dc89520f1e13b`.
