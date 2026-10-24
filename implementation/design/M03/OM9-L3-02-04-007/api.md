# OM9-L3-02-04-007 — Blend Arcblend And Match Curve Continuity API and frozen fixture

- **Owner:** `OM9-L2-02-04` (04-febe-freeform-curve-modeling), milestone M03
- **Goal on record:** G0/G1/G2 connection controls
- **Crate:** `om9-core` (safe Rust; no native host dependency)
- **Units:** millimetres and radians
- **Evidence date:** 2026-10-22

## Scope delivered

G0 line, G1 cubic Hermite and G2 quintic Hermite blends between curve ends; ArcBlend arc for line corners; end matching with promoted degree.

## Frozen Rust API

- `curve_modify::blend_curves`
- `curve_modify::arc_blend_lines`
- `curve_modify::match_curve`
- `curve_modify::Continuity`

## Input rules

- Every point, weight, knot and parameter is checked for finiteness before use.
- Curve records are validated against `Limits` (degree, pole/knot count, domain, positive weights).
- A refusal is a typed `Error`; no partial curve or document mutation is produced.

## Behaviour and measured oracle

Positive fixture: G1 blend reaches (10,0)->(20,10) with +X/-Y end tangents; G2 has zero second derivative at both ends; G0 is the 14.142 mm chord; ArcBlend is a pi arc; match puts the source end on the target with the target tangent.

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `g1_blend_length_mm` | 14.862666324708 | no fixed oracle; endpoint/tangent assertions at 1e-9 |
| `g2_blend_length_mm` | 15.243044351953 | no fixed oracle; D2 assertions at 1e-9 |

The focused run is
`cargo test -p om9-core --test l2_02_04 m03_02_04_007_blend_arcblend_and_match -- --exact --nocapture`;
the receipt below stores its log and SHA-256.

## Negative fixture

- blend with coincident endpoints
- match on a rational or reversed NURBS record
- parallel fillet lines for ArcBlend

A refused operation returns before a curve record exists; an input record is an immutable value and is never mutated.

## Ownership, budgets and determinism

- Safe Rust owns every rule; no OCCT, openNURBS or QymCAD handle is involved in this slice.
- Every allocation is `try_reserve`-checked and the record is admitted by `Limits` before publication.
- Operations are deterministic: the same input yields the same record and the same measured value.

## Source decision

`ADD`. QymCAD has no curve blend/match service; its blend code is an OCCT solid-edge adapter with a different ownership boundary. See `source_crosswalk.json` for the reviewed QymCAD symbol inventory.

## Deliberate limits

- Match is admitted for clamped non-rational NURBS records (a line is promoted to a NURBS); only the end span poles change.
- ArcBlend is the line-line fillet arc; blending arbitrary curve pairs with an arc is not claimed.

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

- focused: `cargo test -p om9-core --test l2_02_04 m03_02_04_007_blend_arcblend_and_match -- --exact --nocapture`
- regression: `cargo test -p om9-core -j 2` (52 suites, 447 passed, 0 failed)
- clippy: `cargo clippy -p om9-core --all-targets -j 2 -- -D warnings`
- format: `cargo fmt --all -- --check`

Focused log SHA-256: `bbe1238796ec40c622248375d875950c0179d758b65009e70d38f542174d08a5`.
