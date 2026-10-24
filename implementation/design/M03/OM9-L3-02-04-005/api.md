# OM9-L3-02-04-005 — Join Explode Close And Split Curves API and frozen fixture

- **Owner:** `OM9-L2-02-04` (04-febe-freeform-curve-modeling), milestone M03
- **Goal on record:** curve composition and split behavior
- **Crate:** `om9-core` (safe Rust; no native host dependency)
- **Units:** millimetres and radians
- **Evidence date:** 2026-10-22

## Scope delivered

Join ordered curves within tolerance, explode a polycurve, close an open curve, split at a parameter and trim a range.

## Frozen Rust API

- `curve_ops::join_curves`
- `curve_ops::explode_curve`
- `curve_ops::close_curve`
- `curve_ops::split_curve`
- `curve_ops::trim_curve`
- `curve_ops::split_nurbs`

## Input rules

- Every point, weight, knot and parameter is checked for finiteness before use.
- Curve records are validated against `Limits` (degree, pole/knot count, domain, positive weights).
- A refusal is a typed `Error`; no partial curve or document mutation is produced.

## Behaviour and measured oracle

Positive fixture: two 10 mm lines join to 20 mm; trim [0.25,0.75] is 10 mm; split 0.25 gives 5+15 mm; close adds the exact gap; 9 polygon round trips with no length drift.

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `joined_length_mm` | 20.000000000000 | 1e-9 |
| `trimmed_length_mm` | 10.000000000000 | 1e-9 |
| `stress_polygons` | 9.000000000000 | declared by the focused test |
| `stress_max_length_drift_mm` | 0.000000000000 | 1e-8 |

The focused run is
`cargo test -p om9-core --test l2_02_04 m03_02_04_005_join_explode_close_and_split -- --exact --nocapture`;
the receipt below stores its log and SHA-256.

## Negative fixture

- join with a gap above tolerance
- empty join input
- trim with from >= to
- split at 0 or 1

A refused operation returns before a curve record exists; an input record is an immutable value and is never mutated.

## Ownership, budgets and determinism

- Safe Rust owns every rule; no OCCT, openNURBS or QymCAD handle is involved in this slice.
- Every allocation is `try_reserve`-checked and the record is admitted by `Limits` before publication.
- Operations are deterministic: the same input yields the same record and the same measured value.

## Source decision

`ADD`. QymCAD has sketch join/offset tool state, not a portable curve join/split service over a 3D NURBS/polycurve record. See `source_crosswalk.json` for the reviewed QymCAD symbol inventory.

## Deliberate limits

- A polycurve's split parameter is segment-uniform (the sampling contract); trim is measured by arc length.
- Nested polycurve members are flattened before joining.

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

- focused: `cargo test -p om9-core --test l2_02_04 m03_02_04_005_join_explode_close_and_split -- --exact --nocapture`
- regression: `cargo test -p om9-core -j 2` (52 suites, 447 passed, 0 failed)
- clippy: `cargo clippy -p om9-core --all-targets -j 2 -- -D warnings`
- format: `cargo fmt --all -- --check`

Focused log SHA-256: `320a33ea1bb83c84c317e4334afab112187eea0fc2751800a1fe46e4ccce8e97`.
Integration-stress log SHA-256: `ba31396aa4c11e35fd92858246c28e5f0f41f3a5fa9dc051891badd18ea287f3`.
