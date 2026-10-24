# OM9-L3-02-05-007 — Surface Match Blend And Join Continuity API and frozen fixture

- **Owner:** `OM9-L2-02-05` (M03 05-febe-freeform-surface-modeling)
- **Goal on record:** G0/G1/G2 matching with continuity feedback
- **Owner crates:** `crates/om9-core` (portable NURBS surface services), `crates/om9-kernel` (exact OCCT B-Rep)
- **Units:** canonical millimetres and radians
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `om9_core::surface_ops::{match_surface, blend_surface, SurfaceContinuity, SurfaceEdge, MatchReport, BlendTangents}`
- `om9_kernel::editing::sew_all` (join)

## Input rules (typed refusal, no guessed value)

- G0/G1/G2 match copies the target boundary rows and reports gap and tangent angle
- G1/G2 blend needs explicit tangents/second derivatives
- joining uses the shared OCCT sewing service and keeps an open shell open

## Behaviour

Match replaces the boundary row (and one/two adjacent rows) with the target's inward rows and reports the measured gap and cross-edge angle. Blend builds a ruled/Hermite/quintic surface from two boundaries. Join sews exact B-Rep sheets.

## Positive fixture and measured oracle

- G0 match gap 5.55e-17 mm; G1 angle 1.20e-4 rad; G2 angle 2.11e-8 rad
- G0/G1/G2 blends with straight tangents all pass through (2,1,0)
- two 2x2 sheets join to 2 faces, 1 shell

| measured key | observed on this checkout | admitted tolerance |
|---|---|---|
| `match_g0_gap_mm` | `5.551115123125783e-17` | `exact` |
| `match_g1_angle_rad` | `0.00011999519952506682` | `exact` |
| `match_g2_angle_rad` | `2.1073424255447017e-08` | `exact` |
| `blend_midpoint` | `[2.0, 1.0, 0.0]` | `exact` |
| `join_faces` | `2` | `exact` |
| `join_shells` | `1` | `exact` |

## Negative fixture

- G1/G2 blend without tangents is refused
- G2 match against a target with too few cross rows is refused
- a boundary gap in the patch is refused

## Ownership, budget and cancellation

- Safe Rust owns the NURBS control structure, UV loops, validation and measured deviation; C++ is limited to the OCCT construction/edit call and exception containment.
- Every constructor returns a validated surface or a typed error; a refused input publishes nothing.
- Grid/sample counts, degree bounds and loop membership are checked before allocation; no partial result is returned.

## Source decision

`ADD`: QymCAD exposes OCCT shape C ABI calls and profile naming only, not a portable NURBS surface workflow. The local module is new Rust code; see `source_crosswalk.json`.

## Artefact hashes (sha256, this checkout)

| file | sha256 |
|---|---|
| `crates/om9-core/src/surface_ops.rs` | `1c9a8cf3f4fde9da026a7c032c6f3c06976b593390c204bd4516250a8750b244` |
| `crates/om9-core/src/surface_kernel.rs` | `f6ee1938df7df299ba25e639c178c80c6e96c789c5e902af5b3406ace383ccc0` |
| `crates/om9-core/src/curve_build.rs` | `2b3194a5172953293d99d1f2086cde842b75ced4e000a65e69469367b9887c24` |
| `crates/om9-core/src/rhino_semantics.rs` | `9d98a59283cd252e7e0bfddd06d785e59907b3c32b5f904e797da950e49ea4cb` |
| `crates/om9-kernel/src/creation.rs` | `7f2e2dc0849d9497d9b202bdd23332989091ee2e8224a04ebfc8be1beb2a5e0f` |
| `crates/om9-kernel/src/editing.rs` | `9b93d9eff62bfdb583d5682d7da17d59ba1315fd8ef968be405e30332a9dad7c` |
| `crates/om9-core/tests/surface_ops.rs` | `d8ba5f44c8f8650670fb69f543f2a233ef2451bab9c1a20891310db2550405b5` |
| `crates/om9-kernel/tests/l2_02_05.rs` | `4a4543c0e07da00a96d2ec12eeffa4cd5ff8e1f512f2c33539ba6833cd581150` |
| `crates/om9-core/src/lib.rs` | `00e29902e540594acb83e75868329a30ae3a2c6f1cc16b3932b4024dd9b7ca15` |

## Receipts

- focused: `cargo test -p om9-core --test surface_ops <L3 filter> -- --nocapture` (logs in the evidence folder)
- regression: `cargo test -p om9-core --test surface_ops`; `cargo test -p om9-kernel --features 3dm-brep --test l2_02_05`; `cargo test -p om9-core --lib`; `cargo test -p om9-kernel --features 3dm-brep`
- clippy: `cargo clippy -p om9-core --test surface_ops -- -D warnings`; `cargo clippy -p om9-kernel --features 3dm-brep --test l2_02_05 -- -D warnings`
- format: `rustfmt --edition 2024 --check crates/om9-core/src/surface_ops.rs crates/om9-core/src/lib.rs crates/om9-core/tests/surface_ops.rs crates/om9-kernel/tests/l2_02_05.rs`
- public source: `python tools/public_source_audit.py` (exit 0)

## Deliberate limits

- G1/G2 matching is a boundary-row replacement, not a full surface optimization; the measured angle is reported.
