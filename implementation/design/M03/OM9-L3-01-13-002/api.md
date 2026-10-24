# OM9-L3-01-13-002 — Opennurbs Reader Bridge Contract API and frozen fixture

- **Owner:** `OM9-L2-01-13` (M03 14-febe-import-rhino-3dm)
- **Goal on record:** reader abstraction, parser lifetime, compatibility limits
- **Owner crates:** `crates/om9-app` (host/UI), `crates/om9-exchange` (transaction/admission), `crates/om9-kernel` (OCCT B-Rep)
- **Native adapter:** `native/native_exchange.cpp` over pinned openNURBS `eb92af3b…`
- **Units:** canonical millimetres; source units/tolerances resolved explicitly
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `ThreeDmHost::admit(&Path, [u8; 32])` / `from_environment()` / `from_application_layout()`
- `om9_exchange::NativeExchange::prepare_import_with_brep(...)`
- `om9_native_owner::NativeProgram` process ownership and reaping

## Input rules (typed refusal, no guessed value)

- the native executable is admitted by absolute path and trusted SHA-256
- the process must be exited and reaped with zero active processes
- the native reply byte length and SHA-256 must match the immutable source snapshot

## Behaviour

Rust snapshots the source into a private staging file, runs the admitted native reader with a unit-resolution packet, decodes the strict version-3 JSON inventory and only then converts it into an import bundle. No ONX_Model handle, OCCT handle or foreign pointer crosses into safe Rust.

## Positive fixture and measured oracle

- hosted import of the authored `mm` archive succeeds with 15 root objects, 2 layers, 3 definitions and 4 definition members
- receipt.native.exited is true and receipt.native.active_processes is 0
- receipt.source_sha256 equals the SHA-256 of the source file

| measured key | observed on this checkout | admitted tolerance |
|---|---|---|
| `root_objects` | `15` | `exact` |
| `layers` | `2` | `exact` |
| `definitions` | `3` | `exact` |
| `definition_members` | `4` | `exact` |
| `active_processes` | `0` | `exact` |

## Negative fixture

- a caller-supplied model factor of 25.4 against a declared millimetre archive is refused before commit

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
| `crates/om9-native-owner/src/lib.rs` | `a29343777eee9a8183e53d4869a3c0b958a96f8e2efbfd1367b6e6900d4784fc` |
| `crates/om9-exchange/src/staged.rs` | `87f162926d903be5ebd56247dfafd2937115aa9df91b567045fd54d5dbab1105` |

## Receipts

- focused: `cargo test -p om9-app --test l2_01_13 m03_01_13_002 -- --ignored --exact --nocapture`
- regression: `cargo test -p om9-app --test l2_01_13 -- --ignored --test-threads=1 --nocapture`
- crate suites: `cargo test -p om9-exchange -p om9-archive -p om9-3dm -j 2`; `cargo test -p om9-kernel --features 3dm-brep -j 2`; `cargo test -p om9-app --lib -j 2`
- clippy: `cargo clippy -p om9-app --lib --test l2_01_13 -j 2 -- -D warnings` (exit 0)
- format: `rustfmt --edition 2024 --check crates/om9-app/src/session.rs crates/om9-app/src/session/files.rs crates/om9-app/tests/l2_01_13.rs`
- public source: `python tools/public_source_audit.py` (exit 0)

## Deliberate limits

- The reader bridge is Windows x64 only in this evidence; macOS execution is unverified.
