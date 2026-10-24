# OM9-L3-02-04-003 — Interpolated And Controlpoint NURBS Curve API and frozen fixture

- **Owner:** `OM9-L2-02-04` (04-febe-freeform-curve-modeling), milestone M03
- **Goal on record:** InterpCrv/Curve freeform drawing
- **Crate:** `om9-core` (safe Rust; no native host dependency)
- **Units:** millimetres and radians
- **Evidence date:** 2026-10-22

## Scope delivered

Chord-length/uniform interpolation, control-point NURBS admission and exact control-point insertion.

## Frozen Rust API

- `curve_build::parameters`
- `curve_build::basis`
- `curve_build::interpolate`
- `curve_build::interpolate_any`
- `curve_build::insert_knot`
- `curve_ops::insert_control_point`
- `curve_ops::curve_grips`
- `Curve::Nurbs`

## Input rules

- Every point, weight, knot and parameter is checked for finiteness before use.
- Curve records are validated against `Limits` (degree, pole/knot count, domain, positive weights).
- A refusal is a typed `Error`; no partial curve or document mutation is produced.

## Behaviour and measured oracle

Positive fixture: 4-point cubic interpolation passes through every chord-length parameter; one inserted control point leaves geometry unchanged at 17 stations; 25 curve/degree combinations in stress.

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `interpolated_pole_count` | 4.000000000000 | exact integer |
| `stress_curves` | 25.000000000000 | declared by the focused test |
| `stress_max_control_insertion_deviation_mm` | 0.000000000000 | 1e-7 |

The focused run is
`cargo test -p om9-core --test l2_02_04 m03_02_04_003_interpolated_and_controlpoint_nurbs -- --exact --nocapture`;
the receipt below stores its log and SHA-256.

## Negative fixture

- even or oversized degree
- degree not smaller than the point count
- malformed knot/pole/weight record

A refused operation returns before a curve record exists; an input record is an immutable value and is never mutated.

## Ownership, budgets and determinism

- Safe Rust owns every rule; no OCCT, openNURBS or QymCAD handle is involved in this slice.
- Every allocation is `try_reserve`-checked and the record is admitted by `Limits` before publication.
- Operations are deterministic: the same input yields the same record and the same measured value.

## Source decision

`PORT`. Ported from legacy `rust/src/spline.rs` (degree rule, chord-length parameters, interpolation) in W2; control-point insertion reuses the ported Boehm knot insertion. See `source_crosswalk.json` for the reviewed QymCAD symbol inventory.

## Deliberate limits

- Periodic, reversed and seam construction is not exposed by the creation API.
- Editing is exact knot insertion and pole moves; control-point deletion is not delivered.

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

- focused: `cargo test -p om9-core --test l2_02_04 m03_02_04_003_interpolated_and_controlpoint_nurbs -- --exact --nocapture`
- regression: `cargo test -p om9-core -j 2` (52 suites, 447 passed, 0 failed)
- clippy: `cargo clippy -p om9-core --all-targets -j 2 -- -D warnings`
- format: `cargo fmt --all -- --check`

Focused log SHA-256: `491693390c6307b76555c9cd21eac2016e121e9b23753c7a4d1964b4090bdad7`.
Integration-stress log SHA-256: `c99b8aaaa0db26fe87ad20e2919050be302ba118e68732e3836a2ff30aac6d7f`.
