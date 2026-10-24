# OM9-L3-01-13-012 — .3dm Golden Roundtrip Fixtures API and frozen fixture

- **Owner:** `OM9-L2-01-13` (M03 14-febe-import-rhino-3dm)
- **Goal on record:** verify geometry, units, instances and layers against examples
- **Owner crates:** `crates/om9-app` (host/UI), `crates/om9-exchange` (transaction/admission), `crates/om9-kernel` (OCCT B-Rep)
- **Native adapter:** `native/native_exchange.cpp` over pinned openNURBS `eb92af3b…`
- **Units:** canonical millimetres; source units/tolerances resolved explicitly
- **Evidence date:** 2026-10-22

## Frozen Rust API

- authored binary fixtures from `native/source_inventory_fixture.cpp`
- `ThreeDmHost::import` / `export`
- `crates/om9-app/tests/l2_01_13.rs` golden oracles

## Input rules (typed refusal, no guessed value)

- golden values are read from the authored archive, not re-derived from the importer
- a host-supported geometry round trip preserves kind and exact values
- fixture provenance and regenerated SHA-256 are recorded per run

## Behaviour

The clean authored archive is imported once and its exact point, line, circle, rational surface and mesh are asserted against the authoring values. A second document with the same host geometry kinds is exported and reimported to prove kind/count/definition parity.

## Positive fixture and measured oracle

- clean-mm imports 12 entities, 1 layer, 3 definitions
- exact point (25.4,50.8,0), line (0,0,0)-(3,4,0) domain [7,9], circle r=6 domain [2,8]
- rational surface poles/weights/knots and mesh vertices/faces/colours are exact
- the constructed host subset round-trips entity count, definition and geometry kinds

| measured key | observed on this checkout | admitted tolerance |
|---|---|---|
| `entities` | `12` | `exact` |
| `layers` | `1` | `exact` |
| `definitions` | `3` | `exact` |
| `exact_point_mm` | `[25.4, 50.8, 0.0]` | `exact` |
| `circle_radius` | `6.0` | `exact` |
| `circle_domain` | `[2.0, 8.0]` | `exact` |

## Negative fixture

- a missing/unknown object class is refused by L3-008; no golden archive is partial

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

## Receipts

- focused: `cargo test -p om9-app --test l2_01_13 m03_01_13_012 -- --ignored --exact --nocapture`
- regression: `cargo test -p om9-app --test l2_01_13 -- --ignored --test-threads=1 --nocapture`
- crate suites: `cargo test -p om9-exchange -p om9-archive -p om9-3dm -j 2`; `cargo test -p om9-kernel --features 3dm-brep -j 2`; `cargo test -p om9-app --lib -j 2`
- clippy: `cargo clippy -p om9-app --lib --test l2_01_13 -j 2 -- -D warnings` (exit 0)
- format: `rustfmt --edition 2024 --check crates/om9-app/src/session.rs crates/om9-app/src/session/files.rs crates/om9-app/tests/l2_01_13.rs`
- public source: `python tools/public_source_audit.py` (exit 0)

## Deliberate limits

- The clean fixture removes retained Breps; editable B-Rep golden coverage is L3-006. A host `Curve::Polycurve` is not exported back exactly (see L3-005).
