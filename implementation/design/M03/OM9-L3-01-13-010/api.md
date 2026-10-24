# OM9-L3-01-13-010 — .3dm File Picker And Options Dialog API and frozen fixture

- **Owner:** `OM9-L2-01-13` (M03 14-febe-import-rhino-3dm)
- **Goal on record:** File Import selection, preview and options
- **Owner crates:** `crates/om9-app` (host/UI), `crates/om9-exchange` (transaction/admission), `crates/om9-kernel` (OCCT B-Rep)
- **Native adapter:** `native/native_exchange.cpp` over pinned openNURBS `eb92af3b…`
- **Units:** canonical millimetres; source units/tolerances resolved explicitly
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `MatrixApp` File menu entries `Import 3DM…` / `Export 3DM…`
- `om9_app::native_picker::pick_3dm` and `om9_native_owner::file_dialog::{DialogFilter::ThreeDm, choose_3dm}`
- `ThreeDmHost::admit` and `ImportOptions` validation

## Input rules (typed refusal, no guessed value)

- the native dialog filters to `.3dm` and returns a path re-validated by the host
- the native executable is admitted by absolute path and SHA-256
- import options are validated before the native process starts

## Behaviour

The File menu dispatches to the native 3DM picker (Open for import, Save for export). The host adapter re-admits the native executable and applies the typed options; cancellation of the dialog leaves the document untouched.

## Positive fixture and measured oracle

- both 3DM menu entries render in the Matrix File menu
- the admitted host exposes the configured native path
- a zero byte limit is refused before the native run

| measured key | observed on this checkout | admitted tolerance |
|---|---|---|
| `menu_import_entry` | `true` | `exact` |
| `menu_export_entry` | `true` | `exact` |
| `wrong_digest` | `"refused"` | `exact` |
| `relative_path` | `"refused"` | `exact` |
| `zero_byte_limit` | `"refused"` | `exact` |

## Negative fixture

- a wrong SHA-256 or a relative native path is refused
- cancelling the picker changes nothing (manual gate below)

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
| `crates/om9-app/src/native_picker.rs` | `08d83a86e2fe10b645a66137676f0f7a441dc057cb8251b9cc30784c044cfd5e` |
| `crates/om9-native-owner/src/file_dialog.rs` | `11544f7e291daf41aa394752e25fbde1e3f96a107718112eef7adbdbeac5c36e` |
| `crates/om9-app/src/ui/chrome.rs` | `f03f1f08dc98c8e1bec730f7ca992ebc783cf124a4063bb5c392643ee39b1b3f` |
| `crates/om9-app/src/ui/files.rs` | `1c7b6e3e973dbffa2196185a705a79fe98aec232757ebf31333f6aaad8be451f` |

## Receipts

- focused: `cargo test -p om9-app --test l2_01_13 m03_01_13_010 -- --ignored --exact --nocapture`
- regression: `cargo test -p om9-app --test l2_01_13 -- --ignored --test-threads=1 --nocapture`
- crate suites: `cargo test -p om9-exchange -p om9-archive -p om9-3dm -j 2`; `cargo test -p om9-kernel --features 3dm-brep -j 2`; `cargo test -p om9-app --lib -j 2`
- clippy: `cargo clippy -p om9-app --lib --test l2_01_13 -j 2 -- -D warnings` (exit 0)
- format: `rustfmt --edition 2024 --check crates/om9-app/src/session.rs crates/om9-app/src/session/files.rs crates/om9-app/tests/l2_01_13.rs`
- public source: `python tools/public_source_audit.py` (exit 0)

## Deliberate limits

- Manual M03-UI review of the native picker, filter, options and cancel path is not signed; automated menu/admission evidence is recorded.
