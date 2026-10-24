# OM9-L3-02-04-006 — Offset Fillet Chamfer Curves API and frozen fixture

- **Owner:** `OM9-L2-02-04` (04-febe-freeform-curve-modeling), milestone M03
- **Goal on record:** curve modifications for jewelry profile design
- **Crate:** `om9-core` (safe Rust; no native host dependency)
- **Units:** millimetres and radians
- **Evidence date:** 2026-10-22

## Scope delivered

Signed offset of line/circle/NURBS, line-line fillet with tangent arc and trimmed legs, line-line chamfer, ArcBlend arc.

## Frozen Rust API

- `curve_modify::offset_curve`
- `curve_modify::fillet_lines`
- `curve_modify::chamfer_lines`
- `curve_modify::arc_blend_lines`
- `curve_modify::CornerFillet`
- `curve_modify::CornerChamfer`
- `offset_curve::OffsetCurve`
- `offset_curve::NurbsBasis`

## Input rules

- Every point, weight, knot and parameter is checked for finiteness before use.
- Curve records are validated against `Limits` (degree, pole/knot count, domain, positive weights).
- A refusal is a typed `Error`; no partial curve or document mutation is produced.

## Behaviour and measured oracle

Positive fixture: line offset +2 mm moves to y=-2; circle radius 5 grows to 7; perpendicular fillet radius 2 gives a pi arc and 8 mm legs; chamfer 3/4 gives a 5 mm line; 100-corner stress.

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `fillet_arc_length_mm` | 3.141592653590 | 1e-6 |
| `chamfer_length_mm` | 5.000000000000 | 1e-9 |
| `stress_corners` | 100.000000000000 | declared by the focused test |
| `stress_max_tangency_error_mm` | 0.000000000000 | 1e-12 |

The focused run is
`cargo test -p om9-core --test l2_02_04 m03_02_04_006_offset_fillet_chamfer -- --exact --nocapture`;
the receipt below stores its log and SHA-256.

## Negative fixture

- fillet radius longer than a leg
- offset that would make a non-positive circle radius
- parallel or non-intersecting fillet lines

A refused operation returns before a curve record exists; an input record is an immutable value and is never mutated.

## Ownership, budgets and determinism

- Safe Rust owns every rule; no OCCT, openNURBS or QymCAD handle is involved in this slice.
- Every allocation is `try_reserve`-checked and the record is admitted by `Limits` before publication.
- Operations are deterministic: the same input yields the same record and the same measured value.

## Source decision

`ADAPT`. The exact offset derivative contract already lives in OM9 `offset_curve`; this slice adds the record-producing path and line-line corner arithmetic. QymCAD offsets 2D bulge contours through `cavalier_contours` and fillets sketch entities by Id, so no code is copied. See `source_crosswalk.json` for the reviewed QymCAD symbol inventory.

## Deliberate limits

- Fillet and chamfer are delivered for line-line corners only; polyline corner chains and line-arc fillets are open.
- NURBS offset is sampled from the exact OffsetCurve derivative contract and fitted back to NURBS; the test bounds the radial deviation at 0.02 mm.

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

- focused: `cargo test -p om9-core --test l2_02_04 m03_02_04_006_offset_fillet_chamfer -- --exact --nocapture`
- regression: `cargo test -p om9-core -j 2` (52 suites, 447 passed, 0 failed)
- clippy: `cargo clippy -p om9-core --all-targets -j 2 -- -D warnings`
- format: `cargo fmt --all -- --check`

Focused log SHA-256: `940e1be1f12c3aee24696869d83868d52a3a6c1a47ad04f923538e4c69ceb27d`.
Integration-stress log SHA-256: `d5c0075995ca7f44e92c6a40c9df76bc6af12da5c93a028b6d706deaae2b30aa`.
