# OM9-L3-02-06-003 — Interactive Mirror And Array API and frozen fixture

- **Owner:** `OM9-L2-02-06` (M03 06-febe-rhino-transform-orient-workflows)
- **Goal on record:** choose plane/axis/count and generate preview
- **Owner crates:** `crates/om9-core` command sessions, `crates/om9-kernel` exact transform math, `crates/om9-app` preview/commit
- **Units:** canonical millimetres and radians
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `om9_kernel::transform_ops::{mirror, linear_pattern, polar_pattern, apply_pattern}`
- `om9_core::gumball::{rectangular_array, polar_array}`

## Input rules (typed refusal, no guessed value)

- a mirror plane needs a non-zero normal
- linear spacing must be non-negative and non-zero; count must be positive
- a full-turn polar pattern divides by count so the seam instance is not duplicated

## Behaviour

Mirror returns a reflection matrix with negative handedness; the B-Rep adapter reverses the inside-out body. Linear/polar pattern generation returns placements used by instance or bake modes.

## Positive fixture and measured oracle

- mirror about x=0 maps (2,3,4) to (-2,3,4)
- linear pattern count 3 spacing 1 gives X=0,1,2
- polar pattern 4 full turn gives 90-degree stations without a seam duplicate

| measured key | observed on this checkout | admitted tolerance |
|---|---|---|
| `mirror_point` | `[-2.0, 3.0, 4.0]` | `exact` |
| `linear_placements` | `3` | `exact` |
| `polar_placements` | `4` | `exact` |
| `seam_duplicate` | `false` | `exact` |

## Negative fixture

- zero mirror normal, zero/negative spacing and zero count are refused
- polar count < 2 and zero total angle are refused

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

- The M03-UI mirror-plane/pattern-count preview is not signed.
