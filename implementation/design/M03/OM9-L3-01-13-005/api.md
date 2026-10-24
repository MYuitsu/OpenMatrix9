# OM9-L3-01-13-005 — .3dm Analytic And NURBS Curves API and frozen fixture

- **Owner:** `OM9-L2-01-13` (M03 14-febe-import-rhino-3dm)
- **Goal on record:** import line/arc/NURBS/Polycurve without flattening
- **Owner crates:** `crates/om9-app` (host/UI), `crates/om9-exchange` (transaction/admission), `crates/om9-kernel` (OCCT B-Rep)
- **Native adapter:** `native/native_exchange.cpp` over pinned openNURBS `eb92af3b…`
- **Units:** canonical millimetres; source units/tolerances resolved explicitly
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `om9_core::geometry::Curve::{Line, Circle, Nurbs, Polycurve}`
- `om9_core::curve_build::polyline`
- native `curve_inventory` / `curve_json`

## Input rules (typed refusal, no guessed value)

- a line stays a line, a circle/arc stays a rational arc with its parameter domain
- NURBS degree, poles, weights, knots and domain are preserved
- no curve is replaced by its tessellation

## Behaviour

Each native curve variant is decoded into the matching host `Curve`, carrying the native parameter domain and angular domain where they differ. Polycurves keep their segments; no flattening or re-approximation occurs.

## Positive fixture and measured oracle

- line, circle, arc, polyline and rational NURBS curve survive export and reimport with the same variants
- NURBS poles agree within 1e-12, weights and knots exactly
- polyline segments keep their exact point sequence

| measured key | observed on this checkout | admitted tolerance |
|---|---|---|
| `variants` | `["Line", "Circle", "Circle(arc)", "Polycurve", "Nurbs"]` | `exact` |
| `nurbs_pole_abs_tolerance` | `1e-12` | `exact` |

## Negative fixture

- an unsupported/reversed polycurve metadata case is refused by the native bridge rather than flattened
- a degenerate curve is rejected by the core curve admission

## Ownership, budget and cancellation

- Safe Rust owns the immutable source snapshot, unit resolution, identity, import bundle, scene validation and one atomic transaction; C++ owns only openNURBS parsing/writing and OCCT conversion.
- The native executable is admitted by absolute path and SHA-256; the process is exited and reaped before any conversion.
- Source bytes are capped by `ImportOptions::max_file_bytes` (default 64 MiB); every allocation is charged to `Limits` before publication.
- Cancellation is checked before the native run, after it and before commit; a refused/cancelled import publishes nothing.

## Source decision

`ADD`: QymCAD has no 3DM reader/writer at the three candidate paths. The bridge is new code against the pinned openNURBS library; see `source_crosswalk.json`. Only the public openNURBS API surface and the M00 3DM gap finding are reused.

## Artefact hashes (sha256, this checkout)

| file | sha256 |
|---|---|
| `crates/om9-app/src/three_dm_host.rs` | `b48c22bc9863bebe9608674fd5744328a254452726eec57a52887770b60c8fa3` |
| `crates/om9-app/src/session/files.rs` | `799dcff9476c67488d5649de498742e0bf1af864a171ff85adbeb9ccf20c4fab` |
| `crates/om9-app/src/session.rs` | `6e44252aab9958b3d10202bbabe33420fbbee3f0240ad74676031e8920a7eb05` |
| `crates/om9-exchange/src/imported.rs` | `7511ebb3d8eb09c1fafd675d2d42080d471a5b551f1b221f1ed98ae89b541e2e` |
| `crates/om9-exchange/src/import_dto.rs` | `52cd4784e1885b9d4fb648da73180e97bf87ecce049438a95bec81221dcea5f8` |
| `native/native_exchange.cpp` | `8cf87d2ca10e785300232219f3742973ecbde4d6249dfb96d190498db27ef6c1` |
| `native/source_inventory_fixture.cpp` | `461f3103859ee024b1f7bf1adb979fe6d3e8b1c4947b838d597901aab8d22dbc` |
| `crates/om9-app/tests/l2_01_13.rs` | `778f96281ebc46db0ba73f331fa628a45c37d84dcf2eacba18d2e3e6c47a2cb0` |
| `crates/om9-core/src/geometry.rs` | `0a1a62351650209e7ccccd67fbb04b5b176c21e35dfe1cd911c0c737fb02373b` |
| `crates/om9-core/src/curve_build.rs` | `2b3194a5172953293d99d1f2086cde842b75ced4e000a65e69469367b9887c24` |

## Receipts

- focused: `cargo test -p om9-app --test l2_01_13 m03_01_13_005 -- --ignored --exact --nocapture`
- regression: `cargo test -p om9-app --test l2_01_13 -- --ignored --test-threads=1 --nocapture`
- crate suites: `cargo test -p om9-exchange -p om9-archive -p om9-3dm -j 2`; `cargo test -p om9-kernel --features 3dm-brep -j 2`; `cargo test -p om9-app --lib -j 2`
- clippy: `cargo clippy -p om9-app --lib --test l2_01_13 -j 2 -- -D warnings` (exit 0)
- format: `rustfmt --edition 2024 --check crates/om9-app/src/session.rs crates/om9-app/src/session/files.rs crates/om9-app/tests/l2_01_13.rs`
- public source: `python tools/public_source_audit.py` (exit 0)

## Deliberate limits

- A host `Curve::Polycurve` re-export is not byte/topology identical: native parameter correspondence is reported as non-identity, so it is excluded from the L3-012 export round trip.
