# OM9-L3-02-04-008 — Rebuild Refit And Knot Operations API and frozen fixture

- **Owner:** `OM9-L2-02-04` (04-febe-freeform-curve-modeling), milestone M03
- **Goal on record:** fit tolerances, curve degree and control-count edits
- **Crate:** `om9-core` (safe Rust; no native host dependency)
- **Units:** millimetres and radians
- **Evidence date:** 2026-10-22

## Scope delivered

Rebuild by sampling, least-squares refit with an exact control count, exact knot insertion and degree elevation.

## Frozen Rust API

- `curve_modify::rebuild_curve`
- `curve_ops::refit_curve`
- `curve_build::rebuild`
- `curve_build::fit_points`
- `curve_build::insert_knot`
- `curve_build::elevate_degree`

## Input rules

- Every point, weight, knot and parameter is checked for finiteness before use.
- Curve records are validated against `Limits` (degree, pole/knot count, domain, positive weights).
- A refusal is a typed `Error`; no partial curve or document mutation is produced.

## Behaviour and measured oracle

Positive fixture: circle rebuild radial deviation 1.7e-5 mm < 0.02 mm; one knot insertion adds one pole without moving 17 sampled points; degree elevates by one; 6-pole fit keeps 6 poles.

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `rebuild_max_radial_deviation_mm` | 0.000016688046 | 0.02 |
| `knot_inserted_poles` | 5.000000000000 | exact integer |

The focused run is
`cargo test -p om9-core --test l2_02_04 m03_02_04_008_rebuild_refit_and_knot_operations -- --exact --nocapture`;
the receipt below stores its log and SHA-256.

## Negative fixture

- fit control count not greater than degree
- fit control count above the sample count
- knot insertion outside the domain

A refused operation returns before a curve record exists; an input record is an immutable value and is never mutated.

## Ownership, budgets and determinism

- Safe Rust owns every rule; no OCCT, openNURBS or QymCAD handle is involved in this slice.
- Every allocation is `try_reserve`-checked and the record is admitted by `Limits` before publication.
- Operations are deterministic: the same input yields the same record and the same measured value.

## Source decision

`ADAPT`. Reuses the W2 ports of legacy `spline.rs::rebuild` and `fit_point_input.rs`; this slice adds the record-level rebuild/refit wrappers and measures the deviation. See `source_crosswalk.json` for the reviewed QymCAD symbol inventory.

## Deliberate limits

- Exact knot removal and degree reduction are not delivered.
- Rebuild/refit is sampling-based; the sample count and tolerance are caller policy.

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

- focused: `cargo test -p om9-core --test l2_02_04 m03_02_04_008_rebuild_refit_and_knot_operations -- --exact --nocapture`
- regression: `cargo test -p om9-core -j 2` (52 suites, 447 passed, 0 failed)
- clippy: `cargo clippy -p om9-core --all-targets -j 2 -- -D warnings`
- format: `cargo fmt --all -- --check`

Focused log SHA-256: `8598a87e567ee37f44dd86106ea23fdd59f6f7dcdec4e49cce219231cd9e3368`.
