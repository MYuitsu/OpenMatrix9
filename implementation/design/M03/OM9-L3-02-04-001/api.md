# OM9-L3-02-04-001 — Line Polyline Point Creation API and frozen fixture

- **Owner:** `OM9-L2-02-04` (04-febe-freeform-curve-modeling), milestone M03
- **Goal on record:** Rhino-like 2D/3D point and polyline commands
- **Crate:** `om9-core` (safe Rust; no native host dependency)
- **Units:** millimetres and radians
- **Evidence date:** 2026-10-22

## Scope delivered

Line, open polyline and point creation through the portable Curve/Geometry records, with measured length and segment order.

## Frozen Rust API

- `curve_ops::arc_length`
- `curve_ops::sample_curve`
- `curve_ops::curve_endpoints`
- `curve_ops::join_curves`
- `curve_build::polyline`
- `curve_build::polygon`
- `Curve::Line`
- `Geometry::Point`

## Input rules

- Every point, weight, knot and parameter is checked for finiteness before use.
- Curve records are validated against `Limits` (degree, pole/knot count, domain, positive weights).
- A refusal is a typed `Error`; no partial curve or document mutation is produced.

## Behaviour and measured oracle

Positive fixture: line (0,0,0)->(3,4,0) length 5 mm; polyline of two 10 mm segments length 20 mm; two exploded segments.

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `line_length_mm` | 5.000000000000 | 1e-10 |
| `polyline_length_mm` | 20.000000000000 | 1e-9 |

The focused run is
`cargo test -p om9-core --test l2_02_04 m03_02_04_001_line_polyline_point_creation -- --exact --nocapture`;
the receipt below stores its log and SHA-256.

## Negative fixture

- empty or single-point polyline
- degenerate line with coincident ends
- polygon with fewer than three distinct points

A refused operation returns before a curve record exists; an input record is an immutable value and is never mutated.

## Ownership, budgets and determinism

- Safe Rust owns every rule; no OCCT, openNURBS or QymCAD handle is involved in this slice.
- Every allocation is `try_reserve`-checked and the record is admitted by `Limits` before publication.
- Operations are deterministic: the same input yields the same record and the same measured value.

## Source decision

`PORT`. W2 already ported the legacy OM9 `rust/src/{polycurve_contract,spline}.rs` construction rules into `curve_build`; this slice adds measured length/sampling and records the QymCAD symbol audit. See `source_crosswalk.json` for the reviewed QymCAD symbol inventory.

## Deliberate limits

- The menu/viewport command tier for Point/Polyline is not rewired in this slice; the records and service are delivered.
- Arc mode and PersistentClose prompts remain a command-tier concern.

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

- focused: `cargo test -p om9-core --test l2_02_04 m03_02_04_001_line_polyline_point_creation -- --exact --nocapture`
- regression: `cargo test -p om9-core -j 2` (52 suites, 447 passed, 0 failed)
- clippy: `cargo clippy -p om9-core --all-targets -j 2 -- -D warnings`
- format: `cargo fmt --all -- --check`

Focused log SHA-256: `db2f3532e811be89345302e984f8f0a75ad75b944298c1438c25ede3b3e45578`.
