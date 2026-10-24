# OM9-L3-01-13-009 — .3dm Import Atomic Scene Transaction API and frozen fixture

- **Owner:** `OM9-L2-01-13` (M03 14-febe-import-rhino-3dm)
- **Goal on record:** avoid partially imported document on failure
- **Owner crates:** `crates/om9-app` (host/UI), `crates/om9-exchange` (transaction/admission), `crates/om9-kernel` (OCCT B-Rep)
- **Native adapter:** `native/native_exchange.cpp` over pinned openNURBS `eb92af3b…`
- **Units:** canonical millimetres; source units/tolerances resolved explicitly
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `Controller::import_3dm(&Path) -> Result<Vec<String>>`
- `om9_exchange::ImportCapture` / `PreparedTransaction` / `TransactionCoordinator::commit`
- `Cancellation`

## Input rules (typed refusal, no guessed value)

- a failed or cancelled import publishes nothing
- a successful import is exactly one Undo step
- the previous document, selection and Undo journal are unchanged on refusal

## Behaviour

The whole conversion is prepared against a private candidate world, validated (including the display scene), then committed through one transaction. Drop of the prepared import discards the candidate; cancellation is checked before and after the native run.

## Positive fixture and measured oracle

- a successful import raises undo_len by exactly one
- undo restores the one pre-existing entity and count

| measured key | observed on this checkout | admitted tolerance |
|---|---|---|
| `truncated_document_delta` | `0` | `exact` |
| `truncated_undo_delta` | `0` | `exact` |
| `cancelled_entities` | `0` | `exact` |
| `successful_undo_delta` | `1` | `exact` |

## Negative fixture

- a truncated archive leaves entity count and undo_len unchanged
- a cancelled import leaves the target document empty

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
| `crates/om9-core/src/transaction.rs` | `7a3e3e66d9a84f897502f3ae619b618d28d6a47fdc62344ce7a5c8d3acc20fae` |
| `crates/om9-exchange/src/import_capture.rs` | `92f38622da55023877c5b648374c2773e949d9103d876fcf49a9864d7b0b0c59` |

## Receipts

- focused: `cargo test -p om9-app --test l2_01_13 m03_01_13_009 -- --ignored --exact --nocapture`
- regression: `cargo test -p om9-app --test l2_01_13 -- --ignored --test-threads=1 --nocapture`
- crate suites: `cargo test -p om9-exchange -p om9-archive -p om9-3dm -j 2`; `cargo test -p om9-kernel --features 3dm-brep -j 2`; `cargo test -p om9-app --lib -j 2`
- clippy: `cargo clippy -p om9-app --lib --test l2_01_13 -j 2 -- -D warnings` (exit 0)
- format: `rustfmt --edition 2024 --check crates/om9-app/src/session.rs crates/om9-app/src/session/files.rs crates/om9-app/tests/l2_01_13.rs`
- public source: `python tools/public_source_audit.py` (exit 0)

## Deliberate limits

- None beyond the L2 supported scope.
