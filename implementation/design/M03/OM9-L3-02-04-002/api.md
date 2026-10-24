# OM9-L3-02-04-002 — Circle Arc Ellipse Polygon Creation API and frozen fixture

- **Owner:** `OM9-L2-02-04` (04-febe-freeform-curve-modeling), milestone M03
- **Goal on record:** analytic primitives with option prompts
- **Crate:** `om9-core` (safe Rust; no native host dependency)
- **Units:** millimetres and radians
- **Evidence date:** 2026-10-22

## Scope delivered

Framed circle, arc by centre/three points, exact rational-quadratic ellipse, rectangle and closed polygon creation.

## Frozen Rust API

- `curve_build::circle_through`
- `curve_build::circle_on_frame`
- `curve_build::arc_through`
- `curve_build::arc_by_center`
- `curve_build::ellipse`
- `curve_build::rectangle`
- `curve_build::polygon`
- `curve_build::polyline`
- `curve_ops::arc_length`

## Input rules

- Every point, weight, knot and parameter is checked for finiteness before use.
- Curve records are validated against `Limits` (degree, pole/knot count, domain, positive weights).
- A refusal is a typed `Error`; no partial curve or document mutation is produced.

## Behaviour and measured oracle

Positive fixture: circle radius 5 -> 10*pi mm; quarter arc -> (pi/2)*5 mm; semicircle -> 5*pi mm; ellipse (5,5) -> 10*pi mm; square polygon -> 40 mm.

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `circle_length_mm` | 31.415926535898 | 1e-6 |
| `polygon_length_mm` | 40.000000000000 | 1e-9 |

The focused run is
`cargo test -p om9-core --test l2_02_04 m03_02_04_002_circle_arc_ellipse_polygon_creation -- --exact --nocapture`;
the receipt below stores its log and SHA-256.

## Negative fixture

- collinear polygon (zero area)
- non-unit or non-perpendicular frame
- non-positive radius
- zero or negative arc sweep

A refused operation returns before a curve record exists; an input record is an immutable value and is never mutated.

## Ownership, budgets and determinism

- Safe Rust owns every rule; no OCCT, openNURBS or QymCAD handle is involved in this slice.
- Every allocation is `try_reserve`-checked and the record is admitted by `Limits` before publication.
- Operations are deterministic: the same input yields the same record and the same measured value.

## Source decision

`PORT`. The circle/arc/ellipse arithmetic was ported in W2 from legacy `rust/src/{circle,ellipse}.rs`; framed circle, arc-by-centre and polygon builders are additive and validated against the same evaluator. See `source_crosswalk.json` for the reviewed QymCAD symbol inventory.

## Deliberate limits

- Ellipse is stored as an exact rational-quadratic NURBS because the document has no analytic ellipse kind.
- Polygon is a flat closed polycurve; the command option prompts are not part of this kernel slice.

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

- focused: `cargo test -p om9-core --test l2_02_04 m03_02_04_002_circle_arc_ellipse_polygon_creation -- --exact --nocapture`
- regression: `cargo test -p om9-core -j 2` (52 suites, 447 passed, 0 failed)
- clippy: `cargo clippy -p om9-core --all-targets -j 2 -- -D warnings`
- format: `cargo fmt --all -- --check`

Focused log SHA-256: `4040cc4191f3adc9196aa7eaac7303ca22a623cc915e5884897c5ad9f4a658af`.
