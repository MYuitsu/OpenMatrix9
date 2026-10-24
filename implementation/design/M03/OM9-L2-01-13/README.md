# OM9-L2-01-13 — 14-febe-import-rhino-3dm

- **Milestone:** M03 (Geometry 3DM)
- **Owner:** `OM9-L2-01-13` — twelve L3 owners, 66 L4 records
- **Delivery date:** 2026-10-22; R5-A/best-effort update 2026-10-23
- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` for all twelve L3s; M03-UI human review remains open
- **Supported source:** Rhino 5 (archive version 50), canonical millimetres
- **Crates:** `om9-app` host/UI, `om9-exchange` admission/transaction, `om9-kernel` OCCT B-Rep; native bridge `native/native_exchange.cpp` over pinned openNURBS

## What was delivered

| L3 | Capability | Rust / native entry point | Focused test |
|---|---|---|---|
| `OM9-L3-01-13-001` | .3dm File Signature And Version Probe | ThreeDmHost::import(&mut TransactionCoordinator, DocumentId, &Path, &Cancellation) -> Result<ImportReceipt> | `m03_01_13_001` |
| `OM9-L3-01-13-002` | Opennurbs Reader Bridge Contract | ThreeDmHost::admit(&Path, [u8; 32]) / from_environment() / from_application_layout() | `m03_01_13_002` |
| `OM9-L3-01-13-003` | .3dm Document Units Tolerance Mapping | om9_exchange::ImportOptions { units: UnitResolution, .. } | `m03_01_13_003` |
| `OM9-L3-01-13-004` | .3dm Layer And Object Attribute Mapping | Document.layers / Document.layer_order / Entity.attributes.layer | `m03_01_13_004` |
| `OM9-L3-01-13-005` | .3dm Analytic And NURBS Curves | om9_core::geometry::Curve::{Line, Circle, Nurbs, Polycurve} | `m03_01_13_005` |
| `OM9-L3-01-13-006` | .3dm NURBS Surface And Trimmed B-Rep | ThreeDmHost::import with BrepMode::EditableBestEffort (explicit strict still aborts) | `m03_01_13_006` |
| `OM9-L3-01-13-007` | .3dm Mesh And Instance Blocks | om9_core::geometry::Mesh and Geometry::Instance(DefinitionId) | `m03_01_13_007` |
| `OM9-L3-01-13-008` | .3dm Unsupported Proprietary Object Policy | ImportReceipt::{diagnostics, retained_records} | `m03_01_13_008` |
| `OM9-L3-01-13-009` | .3dm Import Atomic Scene Transaction | Controller::import_3dm(&Path) -> Result<Vec<String>> | `m03_01_13_009` |
| `OM9-L3-01-13-010` | .3dm File Picker And Options Dialog | MatrixApp File menu entries Import 3DM… / Export 3DM… | `m03_01_13_010` |
| `OM9-L3-01-13-011` | .3dm Import Summary Warning UI | om9_app::session::format_import_summary(&ImportReceipt) -> String | `m03_01_13_011` |
| `OM9-L3-01-13-012` | .3dm Golden Roundtrip Fixtures | authored binary fixtures from native/source_inventory_fixture.cpp | `m03_01_13_012` |

## Measured acceptance (this checkout)

```
OM9_EXCHANGE_NATIVE=<om9-exchange-native.exe>
OM9_EXCHANGE_NATIVE_SHA256=<64 hex>
OM9_SOURCE_FIXTURE=<om9-source-fixture.exe>
cargo test -p om9-app --features occt-brep --test l2_01_13 -- --ignored --test-threads=1 --nocapture
cargo test -p om9-app --no-default-features --test r5a_display -- --ignored --test-threads=1
python tools/public_source_audit.py
```

15 focused native tests pass on this checkout: twelve L3 owners, R5-A display proxy, one integration-stress case and one measured receipt. The separate R5-A no-OCCT test binary adds 2 passes.

| Fixture | Measured |
|---|---|
| authored `mm` | archive 50, Millimeter, 15 root objects, 2 layers, 3 definitions, 4 definition members |
| authored `inch` | exact point (1,2,3) in -> (25.4, 50.8, 76.2) mm |
| authored clean-mm | 12 entities, 1 layer, 3 definitions; exact point/line/circle/surface/mesh |
| constructed box B-Rep | 6000.00000 mm3, closed, one solid, editable through OCCT |
| R5-A retained BRep | cached mesh or coarse untrimmed surface proxy; best-effort keeps an unconvertible BRep opaque |
| stress document | 205 objects, 3+ layers, 5 instances, no loss |

The authored archive writer embeds openNURBS-generated component UUIDs, so regenerated archives are structurally identical but not byte-identical; every receipt records the SHA-256 actually read.

## Verification commands

```
cargo test -p om9-app --features occt-brep --test l2_01_13 -- --ignored --test-threads=1 --nocapture
cargo test -p om9-app --no-default-features --test r5a_display -- --ignored --test-threads=1
python tools/public_source_audit.py
cargo test -p om9-exchange -p om9-archive -p om9-3dm -j 2
cargo test -p om9-kernel --features 3dm-brep -j 2
cargo test -p om9-app --lib -j 2
cargo clippy -p om9-app --lib --test l2_01_13 -j 2 -- -D warnings
rustfmt --edition 2024 --check crates/om9-app/src/session.rs crates/om9-app/src/session/files.rs crates/om9-app/tests/l2_01_13.rs
python tools/public_source_audit.py
```

## Evidence layout

- `implementation/design/M03/OM9-L3-01-13-00X/{api.md,source_crosswalk.json}`
- `implementation/fixtures/M03/OM9-L3-01-13-00X/{positive.json,negative.json}`
- `implementation/evidence/M03/OM9-L3-01-13-00X/` — one receipt per planned L4 role (source map, contract
  fixture, implementation+focused log, regression+suite log, integration stress where planned, acceptance+review).

## Deliberate scope limits

- V6/V7/V8 archives are probed and refused; importing them is deferred by the L2 supported scope.
- Default product import is `EditableBestEffort`: an unconvertible B-Rep is retained opaque with a diagnostic instead of aborting the whole file. An explicitly requested `EditableStrict` still aborts.
- R5-A without OCCT now displays a coarse untrimmed surface proxy when cached render meshes are absent; OCCT remains the exact trimmed display path.
- A host `Curve::Polycurve` re-export reports non-identity native parameter correspondence and is not claimed exact.
- Preserve-mode import refuses an archive with an empty group table entry; geometry mode retains the group table.
- L3-010 and L3-011 keep the M03-UI manual gate open until a human signs the picker/warning review.
- The planning tree keeps `SPEC_ONLY_NOT_IMPLEMENTED`; delivery is recorded through `runtime_implementation` plus this evidence package.
