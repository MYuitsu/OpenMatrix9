# OM9-L3-02-04-004 — Curve Edit Points And Grips API and frozen fixture

- **Owner:** `OM9-L2-02-04` (04-febe-freeform-curve-modeling), milestone M03
- **Goal on record:** show and edit freeform control points
- **Crate:** `om9-core` (safe Rust; no native host dependency)
- **Units:** millimetres and radians
- **Evidence date:** 2026-10-22

## Scope delivered

Portable grip description (poles, Greville edit points, knots) and exact pole move/insertion for lines and NURBS records.

## Frozen Rust API

- `curve_ops::curve_grips`
- `curve_ops::move_grip`
- `curve_ops::insert_control_point`
- `curve_ops::line_to_nurbs`
- `curve_ops::CurveGrips`

## Input rules

- Every point, weight, knot and parameter is checked for finiteness before use.
- Curve records are validated against `Limits` (degree, pole/knot count, domain, positive weights).
- A refusal is a typed `Error`; no partial curve or document mutation is produced.

## Behaviour and measured oracle

Positive fixture: 4-pole cubic exposes 4 edit points; moving pole 1 is observable; inserting a control point raises the pole count to 5.

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `nurbs_grip_count` | 4.000000000000 | exact integer |
| `inserted_grip_count` | 5.000000000000 | exact integer |

The focused run is
`cargo test -p om9-core --test l2_02_04 m03_02_04_004_curve_edit_points_and_grips -- --exact --nocapture`;
the receipt below stores its log and SHA-256.

## Negative fixture

- out-of-range grip index
- grip editing on a circle or polycurve (explicit unsupported)
- degenerate line after a move

A refused operation returns before a curve record exists; an input record is an immutable value and is never mutated.

## Ownership, budgets and determinism

- Safe Rust owns every rule; no OCCT, openNURBS or QymCAD handle is involved in this slice.
- Every allocation is `try_reserve`-checked and the record is admitted by `Limits` before publication.
- Operations are deterministic: the same input yields the same record and the same measured value.

## Source decision

`ADD`. QymCAD's sketch fillet/offset model edits 2D sketch entities by Id, not a portable NURBS pole/grip record; no code is reused. See `source_crosswalk.json` for the reviewed QymCAD symbol inventory.

## Deliberate limits

- Control-point deletion is not delivered.
- Grip editing is admitted for line and NURBS records; circle/polycurve grips are display-only.

## Artefact hashes (sha256, this checkout)

| file | sha256 |
|---|---|
| `crates/om9-core/src/curve_build.rs` | `2b3194a5172953293d99d1f2086cde842b75ced4e000a65e69469367b9887c24` |
| `crates/om9-core/src/curve_ops.rs` | `8800e1ebcfef262954bdc31d36f75f523a3b41ca98568ddc4027eff48508bbe0` |
| `crates/om9-core/src/curve_modify.rs` | `6149f685e1187cb7ca86364e194d60066a7b8379c3f591bf27ed71b59fed1a9a` |
| `crates/om9-core/src/curve_surface_ops.rs` | `fcc811f2e0fdb757edc7106bd8e0717a1446552b48ed3a04fcda51064601df33` |
| `crates/om9-core/src/offset_curve.rs` | `284de3c8d5eec33048fa5e54cefd7e90b4ba9e33ee48d4f7032287758551737d` |
| `crates/om9-core/src/surface_kernel.rs` | `f6ee1938df7df299ba25e639c178c80c6e96c789c5e902af5b3406ace383ccc0` |
| `crates/om9-core/src/rhino_semantics.rs` | `9d98a59283cd252e7e0bfddd06d785e59907b3c32b5f904e797da950e49ea4cb` |
| `crates/om9-core/src/lib.rs` | `6ea2de80c72344a7bfc61ab1f6ddd38f47a3b6e42a57c7b98be0a6340f711702` |
| `crates/om9-core/tests/l2_02_04.rs` | `2f5dc436f6723b490a311348b572dd0e88eca198d32a40de68da90206ed39d20` |

## Receipts

- focused: `cargo test -p om9-core --test l2_02_04 m03_02_04_004_curve_edit_points_and_grips -- --exact --nocapture`
- regression: `cargo test -p om9-core -j 2` (52 suites, 447 passed, 0 failed)
- clippy: `cargo clippy -p om9-core --all-targets -j 2 -- -D warnings`
- format: `cargo fmt --all -- --check`

Focused log SHA-256: `55ddc5e824f4904fd1c791123ff3a158567ae20b90df17c0750c41681e78017e`.
