# OM9-L3-01-13-003 — .3dm Document Units Tolerance Mapping API and frozen fixture

- **Owner:** `OM9-L2-01-13` (M03 14-febe-import-rhino-3dm)
- **Goal on record:** source units to millimeters and tolerances
- **Owner crates:** `crates/om9-app` (host/UI), `crates/om9-exchange` (transaction/admission), `crates/om9-kernel` (OCCT B-Rep)
- **Native adapter:** `native/native_exchange.cpp` over pinned openNURBS `eb92af3b…`
- **Units:** canonical millimetres; source units/tolerances resolved explicitly
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `om9_exchange::ImportOptions { units: UnitResolution, .. }`
- `om9_exchange::import_dto::Inventory::source_units/definition_units` (crate-internal validation)
- native `resolved_unit` / `geometry_to_mm` in `native/native_exchange.cpp`

## Input rules (typed refusal, no guessed value)

- declared units are converted to canonical millimetres; the source factor is never guessed
- unitless or unset units require an explicit caller resolution
- an explicit factor conflicting with the declared unit is refused

## Behaviour

The native projection reports declared model/page/definition units and absolute/relative/angular tolerances. Rust validates the declared/explicit basis, converts every coordinate, radius, surface pole and placement translation to millimetres, and carries the absolute tolerance into the editable B-Rep deflection.

## Positive fixture and measured oracle

- an authored inch archive maps the exact source point (1,2,3) in to (25.4,50.8,76.2) mm
- model_unit is reported as Inch before conversion

| measured key | observed on this checkout | admitted tolerance |
|---|---|---|
| `inch_model_unit` | `"Inch"` | `exact` |
| `inch_point_mm` | `[25.4, 50.8, 76.2]` | `1e-12 per coordinate (z measured 76.19999999999999)` |
| `unitless_without_resolution` | `"refused"` | `exact` |
| `conflicting_factor` | `"refused"` | `exact` |

## Negative fixture

- a unitless archive is refused without an explicit resolution
- a declared millimetre archive is refused when the caller insists on 25.4 mm/unit

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
| `crates/om9-core/src/units.rs` | `25d663a30f8ab4c6fb66797b4e38d48323921f013e09c18ce43a3528301dd21a` |

## Receipts

- focused: `cargo test -p om9-app --test l2_01_13 m03_01_13_003 -- --ignored --exact --nocapture`
- regression: `cargo test -p om9-app --test l2_01_13 -- --ignored --test-threads=1 --nocapture`
- crate suites: `cargo test -p om9-exchange -p om9-archive -p om9-3dm -j 2`; `cargo test -p om9-kernel --features 3dm-brep -j 2`; `cargo test -p om9-app --lib -j 2`
- clippy: `cargo clippy -p om9-app --lib --test l2_01_13 -j 2 -- -D warnings` (exit 0)
- format: `rustfmt --edition 2024 --check crates/om9-app/src/session.rs crates/om9-app/src/session/files.rs crates/om9-app/tests/l2_01_13.rs`
- public source: `python tools/public_source_audit.py` (exit 0)

## Deliberate limits

- No authored inch-scale B-Rep fixture is exercised; the inch proof covers exact curve/point geometry.
