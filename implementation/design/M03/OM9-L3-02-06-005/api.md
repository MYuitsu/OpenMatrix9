# OM9-L3-02-06-005 — Align And Setpoints API and frozen fixture

- **Owner:** `OM9-L2-02-06` (M03 06-febe-rhino-transform-orient-workflows)
- **Goal on record:** align selected points/objects to CPlane/World axis
- **Owner crates:** `crates/om9-core` command sessions, `crates/om9-kernel` exact transform math, `crates/om9-app` preview/commit
- **Units:** canonical millimetres and radians
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `om9_core::transform_session::{set_point_transform, align_point_to_plane, align_point_to_axis}`
- `om9_core::transform_session::{AlignMode, AlignRequest, align_rotation, align_transform}`

## Input rules (typed refusal, no guessed value)

- SetPt translates only the checked world axes
- align-to-plane translates along the plane normal
- align-to-axis translates the point onto the axis line
- AlignRequest can rotate the source axis onto the target axis and scale the source extent to the target extent
- a zero plane normal, non-finite point, missing axis/extent or zero extent is refused

## Behaviour

The translate services project a point onto a plane/axis. AlignRequest optionally scales the source extent, rotates the source axis onto the target axis about the source origin and then translates that origin onto the target; the host applies the returned Transform to the selection and commits one transaction.

## Positive fixture and measured oracle

- SetPt (1,2,3)->(4,6,8) on XY maps to (4,6,3)
- align to plane z=1 maps (1,1,5) to (1,1,1)
- align to X axis maps (2,3,4) to (2,0,0)
- rotate align maps source X onto target Y and the source origin onto the target origin
- scale align maps extent 2 to 4 (factor 2) about the aligned origin
- rotate+scale maps (1,0,0) to target_origin+(0,2,0)

| measured key | observed on this checkout | admitted tolerance |
|---|---|---|
| `setpoint` | `[4.0, 6.0, 3.0]` | `exact` |
| `plane` | `[1.0, 1.0, 1.0]` | `exact` |
| `axis` | `[2.0, 0.0, 0.0]` | `exact` |
| `rotate_maps` | `[4.0, 6.0, 6.0]` | `exact` |
| `scale_maps` | `[12.0, 0.0, 0.0]` | `exact` |
| `rotate_scale_maps` | `[10.0, 2.0, 0.0]` | `exact` |

## Negative fixture

- a zero plane normal is refused
- a NaN set point is refused
- a Rotate request without an axis, or a Scale request without an extent, is refused
- a zero/negative align extent is refused

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

- The M03-UI Align/SetPt target picker and CPlane target selection are not signed.
