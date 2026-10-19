# 3DM native reader/writer + Rust host adapter (2026-10-19)

**Status:** `AUTOMATED_E2E_VERIFIED` (expanded layer/curve/surface/mesh/block/BRep scope, 2026-10-19) · **Target:** standalone Rust OM9
**Native source:** `native/native_exchange.cpp` (pinned openNURBS, existing Phase2/standalone bridge)
**Build recipe:** `tools/build-native-exchange.ps1`

This record covers the missing pieces between the existing native openNURBS reader/writer and a
working Rust import/export:
1. a reproducible native executable build;
2. a Rust host adapter that admits the executable, imports into a transaction and exports a document;
3. CLI host commands that exercise the full round trip.

The FreeCAD workbench is not used.

## 1. Native writer/reader

`native/native_exchange.cpp` already implemented `--inspect`, `--project-source`, `--write`,
`--write-retained`, etc. It was not built in this checkout because the pinned nlohmann/json 3.11.3
header was absent and the CMake target rebuilds openNURBS. `tools/build-native-exchange.ps1` builds
only the executable against the already-built pinned static library:

- openNURBS include/source pin: `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`
- `opennurbsStatic.lib` + `zlib.lib` from `build/native-brep/out/opennurbs`
- nlohmann/json 3.11.3 header, SHA-256
  `9bea4c8066ef4a1c206b2be5a36302f8926f7fdc6087af5d20b417d0cf103ea6`
- `/MD`, `/std:c++20`, `/W4 /WX /utf-8 /EHsc`, Windows entry `wmainCRTStartup`

Output: `build/native-exchange/Release/om9-exchange-native.exe` (SHA printed by the script).

Smoke check on the authored fixture:
`om9-exchange-native.exe --inspect build/standalone-deps/g00-04-fixture.3dm` returns a complete
inventory with `objects=5`, `layers=1`, `diagnostics=0`.

## 2. Rust host adapter

New module `crates/om9-app/src/three_dm_host.rs`:

- `ThreeDmHost::admit(path, sha256)` — admits the native executable by absolute path and trusted
  SHA-256 through `om9_native_owner::NativeProgram`; a document or user data never chooses the
  executable.
- `ThreeDmHost::from_environment()` — development seam:
  `OM9_EXCHANGE_NATIVE` + `OM9_EXCHANGE_NATIVE_SHA256` (64 hex).
- `ThreeDmHost::from_directory(dir)` / `from_application_layout()` — packaged layout:
  `om9-exchange-native.exe` plus its sibling `om9-exchange-native.sha256`, looked up next to the
  application executable and in `bin/`.
- `ThreeDmHost::configured()` — layout first, environment fallback. The CLI and UI use this.
- `import(coordinator, document, source, cancel)` — `ImportCapture` → `NativeExchange::prepare_import`
  → `PreparedNativeImport::prepare` → `TransactionCoordinator::commit`. One transaction; the current
  document/Undo is preserved on failure.
- `import_with_options` — explicit `ImportMode`/`ImportOptions`.
- `export(coordinator, document, target, cancel)` — `ExportCapture::all` →
  `NativeExchange::prepare` → `ValidatedExport::publish` (private staged file, native inspection,
  atomic publication).

The app now depends on `om9-exchange`; no FreeCAD/Qt dependency was added.

## 3. Host commands

`crates/om9-app/src/main.rs` adds two CLI host commands:

```
om9 --import-3dm source.3dm target.om9
om9 --export-3dm source.om9 target.3dm
```

Both require the native path/digest in the environment. Import creates a document with the explicit
OM9 template tolerance, imports through the native reader, commits one transaction and saves the OM9
project. Export opens the OM9 project, captures the current document, writes through the native
writer and publishes atomically.

The UI host adapter is also wired:

- `om9-native-owner::file_dialog` gained `DialogFilter::ThreeDm` (title/filter/extension/default
  name) and `choose_3dm`; the existing OM9 dialog path is unchanged.
- `om9-app/src/native_picker.rs::pick_3dm` exposes the Rhino 3DM open/save picker.
- `File > Import 3DM…` and `File > Export 3DM…` are enabled in the File menu; the buttons launch the
  native picker and call `Controller::import_3dm` / `Controller::export_3dm`.
- `Controller` owns both operations: import is one committed transaction and refreshes the scene;
  export only reads the current document. Success/diagnostics and errors go to the transcript/status;
  no document, selection or Undo state changes on export, and import never replaces the existing
  document.

## 4. End-to-end evidence

Manual/e2e flow performed in this checkout:

1. Build native executable with `tools/build-native-exchange.ps1`.
2. Import `build/standalone-deps/g00-04-fixture.3dm` to `build/three-dm-e2e/imported.om9`.
   - CLI exit 0; OM9 report lists the imported layers/objects.
3. Export `imported.om9` to `build/three-dm-e2e/exported.3dm`.
   - CLI exit 0; 9,392-byte 3DM written.
4. Inspect `exported.3dm` with the native reader:
   - `objects=5`, `layers=1`, `diagnostics=0`.
   - Imported object count was 5, so the round trip preserved the fixture's object set.

Exact geometry round trip (`crates/om9-app/tests/three_dm_curve_roundtrip.rs`, ignored,
1 passed): a document containing an arbitrary-precision `Line`, full `Circle`, partial `arc`
(`angle_domain`), connected `Polyline`, rational cubic `NurbsCurve` (weights 1/0.7/0.7/1), a NURBS
`Surface`, a `Mesh`, a hidden/locked/colored `Layer`, and a block definition/instance was exported
to 3DM and re-imported. Line/Circle/Arc/Polyline/Surface/Mesh compare exactly; NURBS
degree/weights/knots/domain compare exactly and every pole is within `1e-12` (ON's homogeneous
conversion can move one ULP). Layer name/color/visible/locked/parent and the object's layer
assignment, block definition members and instance identity are preserved. Kinds remain distinct:
mesh is still mesh, curve/surface/BRep blocks are not reclassified. `history.recipes` is empty.
A second 3DM -> OM9 pass preserves entity/definition counts.

Opaque BRep round trip (`crates/om9-app/tests/three_dm_brep_roundtrip.rs`, ignored,
1 passed): a native solid imports as `Geometry::NativeBrep` (display mesh + chunked native payload),
exports through the native writer, is inspected as `ON_Brep`, and re-imports as `NativeBrep`; it never
becomes a mesh. Export correspondence accepts re-serialized native framing while the BRep geometry is
independently re-imported/inspected. BRep is retained/opaque and cannot be topologically edited yet.
Display uses the cached render mesh stored by Rhino; a BRep without one (for example the authored box
fixture) still round-trips exactly but has no shaded display.

Automated ignored tests: `crates/om9-app/tests/three_dm_native_roundtrip.rs`
(`rust_cli_imports_and_exports_a_3dm_roundtrip` and
`controller_host_adapter_imports_and_exports_without_the_cli` and
`failed_import_keeps_document_and_undo`) — ran with `--ignored`, exit 0, **3 passed**. They require `OM9_EXCHANGE_NATIVE`, `OM9_EXCHANGE_NATIVE_SHA256`,
`OM9_THREE_DM_FIXTURE`. The Controller test pre-populates an existing point, imports the fixture,
asserts the point still exists (import does not replace the document), exports and re-inspects.

Packaged-layout test: `om9-exchange-native.exe` and `.sha256` were copied beside `om9.exe`, all
`OM9_EXCHANGE_NATIVE*` variables removed, and `om9 --import-3dm` still imported the 5-object
fixture (exit 0, `objects=5`). Host module tests cover fingerprint validation,
absolute-path/trusted-digest admission and the sibling-digest layout lookup.
Native-owner file-dialog tests: **8 passed** (OM9 dialog behavior unchanged).
App library tests: **101 passed**; `file_widgets` **21 passed**. App clippy: exit 0 with `-D warnings`.
Workspace regression: **1,185 passed / 171 suites** excluding `om9-render` (`workspace-all-final7.log`), with the same two known `mesh_fill`/`ui_events` failures, with the same two
known unrelated `mesh_fill`/`ui_events` failures. The workspace run also exposed a flaky assertion in
`gumball_numeric` (it assumed `ids[0]` was the point at x=0; BTreeMap UUID order varies). The test
now compares against the same object's measured world position; the product code is unchanged.

## 5. What is still missing

- **Packaging signature**: the layout is now read without environment variables
  (`om9-exchange-native.exe` + `.sha256` beside the application or in `bin/`), but the files still
  need to come from a signed installer/layout; a digest file next to an unsigned executable is not a
  trust root. `tools/build-native-exchange.ps1` writes both files.
- **CI**: `.github/workflows/release-windows.yml` has a `native-exchange` job that builds pinned
  openNURBS + the native reader/writer and uploads the executable, `.sha256` and manifest as the
  `om9-native-exchange` artifact. The job is build-only, matching the repository's CI policy.
- **Geometry coverage** is line, circle/partial arc, polyline, rational NURBS curve, NURBS surface,
  mesh, block/instance and opaque BRep. The BRep is retained/display-only; editing topology requires
  a separate ON_Brep ↔ OCCT bridge. Material/linetype/dimstyle and other Rhino tables are outside the
  requested scope; the layer table and object display attributes are retained.
- Advanced native import/export options (units, definitions, retained tables) remain available
  through the existing `ImportOptions` pipeline; the host adapter uses `ImportMode::Geometry` and
  does not import history/material/other tables.

## 6. Verification commands

```
cargo test -p om9-app --lib -j 2
cargo clippy -p om9-app --all-targets -j 2 -- -D warnings
OM9_EXCHANGE_NATIVE=... OM9_EXCHANGE_NATIVE_SHA256=... \
OM9_THREE_DM_FIXTURE=... cargo test -p om9-app --test three_dm_native_roundtrip -- --ignored
cargo test --workspace --no-fail-fast -j 2
```

Receipt: `implementation/evidence/M01/2026-10-19-three-dm-native-host-checks.json`.

## 7. Milestone records

- Primary owner: M03 L2-01-13 (all 12 L3); L3-006 is marked partial retained BRep.
- UI entrypoint: M01 L2-03-15 L3-002.
- Model/persistence: M01 L2-01-03 L3-002/004/005/006/008/013 and M01 L2-01-12 L3-003/010.
- Semantics: M01 L2-02-01 L3-005/007/009/010.
- Layer UI: M01 L2-03-14 L3-003/008.
- M00 L2-04/G00-04 remains a feasibility gate; it is not marked implemented by this work.
