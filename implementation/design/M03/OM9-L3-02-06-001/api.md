# OM9-L3-02-06-001 — Interactive Move Copy And Rotate API and frozen fixture

- **Owner:** `OM9-L2-02-06` (M03 06-febe-rhino-transform-orient-workflows)
- **Goal on record:** reference point command state and preview
- **Owner crates:** `crates/om9-core` command sessions, `crates/om9-kernel` exact transform math, `crates/om9-app` preview/commit
- **Units:** canonical millimetres and radians
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `om9_core::transform_session::{MoveSession, RotateSession}`
- `om9_core::gumball::Session` (copy mutations, one undo step)
- `crates/om9-app/src/session/gumball.rs` preview/commit path

## Input rules (typed refusal, no guessed value)

- the base/reference point pair defines the preview translation
- the rotate angle is measured around the signed axis and projected perpendicular to it
- Copy inserts fresh identities and keeps the source unchanged
- a non-finite point or a zero rotate axis is refused

## Behaviour

Move stores a base point and a live reference point; Rotate stores a centre, an axis and reference/current points. Each preview is a pure Transform the host applies to the selection; Copy is a flag on the same session. Commit is one transaction.

## Positive fixture and measured oracle

- Move (1,2,3) -> (4,6,8) previews translation (3,4,5)
- Rotate 90 deg about Z maps (1,0,0) to (0,1,0)
- the Copy flag survives preview/cancel

| measured key | observed on this checkout | admitted tolerance |
|---|---|---|
| `move_preview` | `[[1.0, 0.0, 0.0, 3.0], [0.0, 1.0, 0.0, 4.0], [0.0, 0.0, 1.0, 5.0], [0.0, 0.0, 0.0, 1.0]]` | `exact` |
| `rotate_angle_rad` | `1.5707963267948966` | `exact` |
| `copy_option` | `true` | `exact` |

## Negative fixture

- NaN/INF references are refused
- a zero rotate axis is refused
- a malformed numeric transform leaves the document unchanged

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

- The M03-UI reference-point pick/preview workflow is not signed.
