# OM9-L3-02-06-004 — Orient By Two Three Reference Points API and frozen fixture

- **Owner:** `OM9-L2-02-06` (M03 06-febe-rhino-transform-orient-workflows)
- **Goal on record:** reference-target coordinate frame placement
- **Owner crates:** `crates/om9-core` command sessions, `crates/om9-kernel` exact transform math, `crates/om9-app` preview/commit
- **Units:** canonical millimetres and radians
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `om9_kernel::transform_ops::{orient_two_point, orient_three_point}`
- `om9_core::gumball::Frame`

## Input rules (typed refusal, no guessed value)

- two-point orientation maps A->A' and the A->B direction onto A'->B', minimal roll
- three-point orientation maps three non-collinear references exactly
- coincident or collinear references are refused

## Behaviour

The solver builds orthonormal frames from the reference vectors and returns a rigid transform `target_frame^T * source_frame` plus the A translation. The frame-row/column convention is tested with a non-identity rotation, not only a translation.

## Positive fixture and measured oracle

- two-point maps (0,0,0)->(10,20,30) and (1,0,0)->(10,21,30)
- three-point maps [0,0,0],[1,0,0],[0,1,0] to a 90-degree rotated translated frame exactly

| measured key | observed on this checkout | admitted tolerance |
|---|---|---|
| `two_point_a` | `[10.0, 20.0, 30.0]` | `exact` |
| `two_point_b` | `[10.0, 21.0, 30.0]` | `exact` |
| `three_point_1` | `[2.0, 4.0, 4.0]` | `exact` |
| `three_point_2` | `[1.0, 3.0, 4.0]` | `exact` |

## Negative fixture

- coincident two-point references are refused
- collinear three-point references are refused

## Ownership, budget and cancellation

- The session is a pure value; it never touches the document. The app applies the preview only, and commit is one transaction/Undo entry.
- Points, axes and factors are validated as finite/non-degenerate before a matrix is built.
- Preview/cancel keep the previous document and selection unchanged.

## Source decision

`ADD`: QymCAD has OCCT mirror calls and UI pattern flows, not a portable typed transform session. The local session and orientation math are new Rust code; see `source_crosswalk.json`.

## Artefact hashes (sha256, this checkout)

| file | sha256 |
|---|---|
| `crates/om9-core/src/transform_session.rs` | `2f0a44e314909c8895442516539a0122fb195db0c3bda17e635c08847dcd6cac` |
| `crates/om9-core/src/gumball.rs` | `870661d90df2e55e770352458b2d2e8f2476980cebcabd944b32f2232d9cc81e` |
| `crates/om9-kernel/src/transform_ops.rs` | `64925c4f9e63d2ef19c5568df42d89e31c512b1adcd69a3f5eccd960fba020f3` |
| `crates/om9-app/src/session/gumball.rs` | `38ebb662db09592ae315cfacdfd9df1aeb0c494dc661c59f47236c867a6faf4d` |
| `crates/om9-core/tests/transform_sessions.rs` | `bf408a8a024049e1ab332552541972c1900e6c210c71050c4d0328f14a1038fd` |
| `crates/om9-kernel/tests/l2_02_06.rs` | `5c5dcf5303306b3d43b5ec422c421678ce40e5b7fbcdd42fca4eff2e41595c45` |
| `crates/om9-app/tests/gumball_interaction.rs` | `ed0e549c13371c54ad03dfd959d90635c47caa8111502a580fa81e30c6085532` |
| `crates/om9-app/tests/gumball_numeric.rs` | `0662e027df16a170f420dc7fa00a896affbc1495be047f7a095b4ce0e042d2a1` |

## Receipts

- focused: `cargo test -p om9-core --test transform_sessions <L3 filter> -- --nocapture`; `cargo test -p om9-kernel --features 3dm-brep --test l2_02_06 <L3 filter> -- --nocapture`
- regression: `cargo test -p om9-core --test transform_sessions`; `cargo test -p om9-kernel --features 3dm-brep --test l2_02_06`; `cargo test -p om9-kernel --features 3dm-brep --test l2_01_11`; `cargo test -p om9-app --test gumball_interaction --test gumball_numeric --test gumball_placement --test gumball_cold`
- clippy: `cargo clippy -p om9-core --test transform_sessions -j 2`; `cargo clippy -p om9-kernel --features 3dm-brep --test l2_02_06 -j 2` (owned files warning-free; workspace `-D warnings` blocked by an in-flight untracked top11 module)
- format: `rustfmt --edition 2024 --check crates/om9-core/src/transform_session.rs crates/om9-core/tests/transform_sessions.rs crates/om9-kernel/tests/l2_02_06.rs crates/om9-kernel/src/transform_ops.rs`
- public source: `python tools/public_source_audit.py` (exit 0)

## Deliberate limits

- The M03-UI two/three-point picker preview is not signed.
