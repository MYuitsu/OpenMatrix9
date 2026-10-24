# OM9-L3-02-04-009 — Project Pullback And Curve Intersections API and frozen fixture

- **Owner:** `OM9-L2-02-04` (04-febe-freeform-curve-modeling), milestone M03
- **Goal on record:** project to surface and derive intersection curves
- **Crate:** `om9-core` (safe Rust; no native host dependency)
- **Units:** millimetres and radians
- **Evidence date:** 2026-10-22

## Scope delivered

Closest point on surface/curve, curve pullback with a measured deviation, analytic line-line and coplanar circle-circle intersections, numeric curve/curve and curve/surface intersections.

## Frozen Rust API

- `curve_surface_ops::closest_point_on_surface`
- `curve_surface_ops::closest_point_on_curve`
- `curve_surface_ops::pullback_curve`
- `curve_surface_ops::curve_curve_intersections`
- `curve_surface_ops::curve_surface_intersections`
- `curve_surface_ops::SurfaceProjection`
- `curve_surface_ops::Pullback`
- `curve_surface_ops::CurveIntersection`
- `curve_surface_ops::SurfaceIntersection`

## Input rules

- Every point, weight, knot and parameter is checked for finiteness before use.
- Curve records are validated against `Limits` (degree, pole/knot count, domain, positive weights).
- A refusal is a typed `Error`; no partial curve or document mutation is produced.

## Behaviour and measured oracle

Positive fixture: (2,3,5) projects to (2,3,0) at distance 5; a line 1 mm above the plane pulls back with 1 mm deviation; crossing lines meet at (5,5); a chord cuts a circle twice; a piercing line meets the plane once.

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `projection_distance_mm` | 5.000000000000 | 1e-9 |
| `pullback_deviation_mm` | 1.000000000000 | 1e-6 |

The focused run is
`cargo test -p om9-core --test l2_02_04 m03_02_04_009_project_pullback_and_intersections -- --exact --nocapture`;
the receipt below stores its log and SHA-256.

## Negative fixture

- pullback deviation above tolerance
- intersection tolerance not positive
- empty sketch/intersection input

A refused operation returns before a curve record exists; an input record is an immutable value and is never mutated.

## Ownership, budgets and determinism

- Safe Rust owns every rule; no OCCT, openNURBS or QymCAD handle is involved in this slice.
- Every allocation is `try_reserve`-checked and the record is admitted by `Limits` before publication.
- Operations are deterministic: the same input yields the same record and the same measured value.

## Source decision

`ADD`. QymCAD projects sketch entities onto planes/faces through its native kernel; no portable curve/surface projection or intersection service is exposed. See `source_crosswalk.json` for the reviewed QymCAD symbol inventory.

## Deliberate limits

- Only line-line and coplanar circle-circle intersections are analytic; other curve pairs refine a sampled candidate set.
- Curve/surface intersection is a bounded numeric solver over 192 samples; a reported point is within the requested tolerance.

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

- focused: `cargo test -p om9-core --test l2_02_04 m03_02_04_009_project_pullback_and_intersections -- --exact --nocapture`
- regression: `cargo test -p om9-core -j 2` (52 suites, 447 passed, 0 failed)
- clippy: `cargo clippy -p om9-core --all-targets -j 2 -- -D warnings`
- format: `cargo fmt --all -- --check`

Focused log SHA-256: `68ebe58ae50f37ca0bea9fe20ee2b8fa3c01edf9c1c579acde09b683589f524f`.
