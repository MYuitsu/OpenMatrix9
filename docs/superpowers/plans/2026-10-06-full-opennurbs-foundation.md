# Full openNURBS — preservation foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Inventory all source3DM content and retain the immutable archive, stable object identity and capability report inside FCStd, including objects that have no editable FreeCAD representation.

**Architecture:** Rust defines preservation/capability policy. Native openNURBS produces a versioned inventory and bounded immutable snapshot; the host adapter persists it with FreeCAD included-file properties and binds supported geometry or explicit retained-record objects. This first package provides reliable inventory/import/persistence; merged preservation export is the next package and must not be advertised as already implemented.

**Tech Stack:** Existing Rust2024 static library, C++23/openNURBS pinned revision, OCC8, Qt6 JSON, FreeCAD App properties and Python host binding. No Python Rhino geometry dependency or new Rust JSON dependency.

**Spec:** `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` — user approved continuation on2026-10-06. This plan implements the inventory/preservation foundation; subsequent package plans below cover the remaining spec.

## Global Constraints

- Pinned openNURBS `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`; Rhino5 output version5/50.
- Retain source CAD without implicit meshing; unsupported editability is disclosed.
- Preserve existing task/edit guards, one Undo transaction and successful-only command history.
- Keep input limit512MiB, expanded-output limit1million, block depth64; foundation manifest limit32MiB and included archive limit512MiB before document mutation.
- Unitless/custom files require an explicit positive finite millimeters-per-file-unit scale.
- Both user fixtures immutable; outputs stay in build/test directories.
- Ring acceptance bounds1e-3mm, area/volume max(1e-3,1e-4relative); use adaptive integration and finite error estimates below supporting tolerance.
- Work in the existing isolated `build/om9-dev` checkout; its feature changes are uncommitted. Preserve primary checkout/user changes. Build/install primary source only after review and intentional integration. No push/publish.
- Prefix every shell command with `rtk`; wait for native link/install completion before launching FreeCAD.

## Review Focus

- Two source archives share UUIDs: namespace identities per import and never conflate their objects (Task2/4).
- Original file/temp staging removed: FCStd reopen must recover the source snapshot and inventory (Task4).
- Unsupported object mixed with supported CAD: retained object stays visible in the tree/report, while geometry-only import still rejects it atomically (Task3/4).
- Malformed or oversized inventory: reject before transaction and never follow an untrusted external path (Task1/3).
- Preserve-mode document exported by legacy writer: fail clearly rather than silently dropping retained content (Task5).

## File and schema decisions

Create `Gui/ThreeDmInventory.h/.cpp` for native inventory, `rust/src/core_3dm_archive.rs` for capability policy, `ThreeDmArchiveState.py` for host persistence and `docs/3dm-coverage.json` for evidence-driven coverage. Modify existing archive/bridge/binding/command/build files only at integration boundaries.

Manifest schema1: `schema_version`, `source_version`, `source_units`, `scale_mm`, `archive_sha256`, `records`, `components`, `settings`, `resources`, `issues`. Records contain `source_uuid`, `class_uuid`, `class_name`, `component_type`, `role` (top-level/definition-member), `dependencies`, `name`, `capability` (editable/display-retained/retained/incompatible). Record identity in the host is `(import_namespace,source_uuid)`. Issues contain code/severity/source_uuid/class_name/message; no serialized executable instructions.

Store an `App::FeaturePython` archive container with `OM9ArchiveSchema`, `OM9ImportNamespace`, `OM9SourceArchive` (`App::PropertyFileIncluded`), `OM9ArchiveManifest`, `OM9ArchiveHash` and `OM9ArchiveMode`. Child geometry/retained records carry `OM9SourceUUID`, `OM9ImportNamespace`, `OM9SourceClass`, `OM9Capability` and `OM9SourceSignature`. Source archive container is metadata, not an exported top-level geometry object.

## Verification commands

Task1 creates ignored `build/3dm-preservation-tests.cmd`: initialize the existing VS2022 `vcvars64.bat`, prepend the verified Pixi dependency paths, configure `-S build/om9-dev/tests/native -B build/3dm-preserve-native -G Ninja -DCMAKE_C_COMPILER=cl -DCMAKE_CXX_COMPILER=cl -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_PREFIX_PATH=H:/FreeCAD-src/.pixi/envs/default/Library -DFETCHCONTENT_SOURCE_DIR_OM9_OPENNURBS=H:/FreeCAD-src/build/dependencies/opennurbs`, build the target passed as its first argument, run that executable with dependency DLLs on PATH, and return its exit code. Use `rtk proxy cmd /c build/3dm-preservation-tests.cmd ThreeDmInventoryTests` for Task1 and substitute `ThreeDmPreservationTests` for Task3; successful result is exit0 with fixture assertions passed.

Task4 creates ignored `build/3dm-preservation-module.cmd` using the existing standalone SDK configuration from `build/3dm-module-final.cmd`, changing source to `build/om9-dev` and build output to `build/openmatrix9-preservation`. Run `rtk proxy cmd /c build/3dm-preservation-module.cmd`; require compile/link/copy exit0 before host tests. Run host tests with `rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File build/om9-dev/tests/run_menu_smoke.ps1 -FreeCADExe H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe -DependencyPrefix H:/FreeCAD-src/.pixi/envs/default/Library -Macro three_dm_preservation_smoke.FCMacro -TimeoutSeconds 300`; require report ok=true, every check passed and native process exit0. These commands install the isolated source into the SDK for testing; Task6 rebuilds the intentionally integrated primary source.

### Task1: Native inventory and complete coverage discovery

**Files:** Create `Gui/ThreeDmInventory.h/.cpp`, `tests/native/three_dm_inventory.cpp`, `docs/3dm-coverage.json`; modify `cmake/OpenNURBS.cmake`, `tests/native/CMakeLists.txt`.

**Interfaces:** `ArchiveInventory inspectArchive(const std::filesystem::path&, double customScaleMm=0.0)`; `std::string inventoryJson(const ArchiveInventory&)`. `ArchiveInventory` stores manifest schema1, native source class/component IDs, resolved references and issues. Read-only; no FreeCAD dependencies in the standalone native library.

- [ ] Write independent mixed fixtures with BRep, point cloud, text dot, embedded block, material, layer, group, dimension style and userdata. Assert each actual record/table is accounted for with identity/role, and missing references are reported. Assert raw file/resource paths are never opened automatically.
- [ ] Configure native tests in a task-specific build using the pinned local openNURBS override; build/run `ThreeDmInventoryTests`. Confirm missing implementation fails.
- [ ] Implement bounded parse and inventory across real component categories plus settings/properties/views/userdata. Derive concrete geometry-class coverage from pinned class-registration definitions and inheritance evidence; classify abstract/alias/obsolete entries explicitly. Coverage statuses start unverified except existing test-backed paths.
- [ ] Run inventory tests and assert all discovered real categories have coverage rows, without interpreting a row's existence as completed support. Verify512MiB input and32MiB manifest guards using bounded fixtures/test streams.
- [ ] Review and checkpoint only owned files; record command/results in the live ledger. Preserve uncommitted feature baseline when preparing any Git checkpoint.

### Task2: Rust capability, identity and mode policy

**Files:** Create `rust/src/core_3dm_archive.rs`, `rust/tests/core_3dm_archive.rs`; modify `rust/src/lib.rs`, `Gui/RustBridge.h`.

**Interfaces:** `ArchiveMode { GeometryOnly=0, Preserve=1 }`, `ArchiveCapability { Editable=1, DisplayRetained=2, Retained=3, Incompatible=4 }`. Export C ABI `om9_3dm_archive_mode_valid(u32)->bool`, `om9_3dm_archive_capability_valid(u32)->bool`, `om9_3dm_archive_legacy_export_allowed(u32)->bool`. `ArchiveIdentity { import_namespace: String, source_uuid: String }` uses `ArchiveIdentity::new(import_namespace: &str, source_uuid: &str)->Result<Self, IdentityError>` to validate canonical non-nil UUID strings and compares both fields; native/host produces new import namespace UUIDs.

- [ ] Write Rust tests for unknown enum rejection, preserve-mode legacy export denial and identical source UUIDs under different import namespaces remaining distinct.
- [ ] Run `rtk cargo test --manifest-path rust/Cargo.toml --test core_3dm_archive`; confirm failure before implementation.
- [ ] Implement pure policy without Qt, archive parsing or JSON dependencies. Keep geometry-only command semantics unchanged.
- [ ] Run focused tests, fmt and existing Rust suite; check every new C ABI declaration agrees with its Rust export.
- [ ] Review/checkpoint owned files and update evidence.

### Task3: Native preflight and immutable snapshot bridge

**Files:** Modify `Gui/ThreeDmPython.cpp`, `Gui/ThreeDmArchive.h/.cpp`; create `tests/native/three_dm_preservation.cpp`.

**Interfaces:** Python native method `inspect3dm(path: str, scale: float=0)->str` returns schema1 JSON. `prepare3dmArchive(path: str, staging: str, scale: float, mode: int)->str` returns `{manifest, snapshot, prepared_geometry, retained_records}`; geometry records include staged BRep/mesh data. Snapshot is a verified immutable copy in caller staging, not a promise of lossless reserialization.

- [ ] Write tests asserting geometry-only mode still rejects TextDot, while preservation preflight inventories and stages it alongside valid CAD. Corrupt input, invalid mode/scale, excessive manifest and missing required dependencies must fail before a host transaction.
- [ ] Build/run `ThreeDmPreservationTests`; confirm the new interface/behavior is missing.
- [ ] Implement inventory-backed preflight and snapshot hashing; call current converters only for supported geometry. Unsupported records are retained as native data with explicit capability. Mark incompatible/unreadable data as blocking issues; do not convert every error into a retained success. Verify snapshot bytes/hash match input and clean incomplete staging.
- [ ] Run native tests and existing five suites. Verify no external resource/network reads and no conversion of CAD to mesh; embedded raw bytes remain intact.
- [ ] Review/checkpoint and record exact ABI/preflight evidence.

### Task4: FCStd archive persistence and retained-object binding

**Files:** Create `ThreeDmArchiveState.py`, `tests/three_dm_preservation_smoke.FCMacro`; modify `ThreeDm.py`, module `CMakeLists.txt`.

**Interfaces:** `bind_archive(document, prepared: dict)->list` inserts container plus source-backed objects inside the existing native-owned transaction. `load_archive_state(container)->dict` validates schema/hash and returns persisted snapshot/manifest. `archive_identity(obj)->tuple[str,str]|None` reads immutable mapping. `import_file(..., mode='geometry')` adds explicit preserve mode while retaining the current default until Task5.

- [ ] Write host tests importing mixed CAD/TextDot, checking explicit retained-object properties, one Undo/Redo, two archives with the same UUID and no collision, plus source/staging removal before FCStd reopen.
- [ ] Run the new macro with a fresh profile and installed module; confirm unsupported/absent preservation behavior fails.
- [ ] Implement included-file storage and bounded manifest properties, import namespace generation and stable identity binding. Retained objects use labeled App features without fake empty Part solids. Register serialization behavior via packaged module code; included data must outlive staging cleanup. No arbitrary Python payload execution from source data.
- [ ] Verify reopen reproduces hashes/identities and CAD validity without original files. Abort injected binding failure and assert the existing document stays unchanged. Old geometry-only documents remain exportable.
- [ ] Review/checkpoint and record SDK reports/screenshots.

### Task5: User-visible preservation mode, reports and export guard

**Files:** Modify `Gui/CoreThreeDm.cpp/.h`, `Gui/ThreeDmPython.cpp`, `ThreeDm.py`, `rust/src/core_3dm_archive.rs`; extend `tests/three_dm_preservation_smoke.FCMacro`.

**Interfaces:** Native import dialog exposes Geometry only / Preserve source data; pass selected mode to the common handler. Standard `ThreeDm.open/insert` use the documented preservation default only after its tests pass. `export_file` detects source-retained content/containers and uses Rust mode policy to deny legacy export where omission could occur. Later merge writer must replace this guard, not bypass it.

- [ ] Add menu/CMD mode/report tests and cancellation/task guards. Assert a preserve-mode document cannot successfully use the legacy exporter to discard its retained records; original destination/history unchanged. Explicitly chosen geometry-only objects remain exportable with a disclosed preservation limitation.
- [ ] Run tests and confirm currently missing mode/report/guard fails.
- [ ] Implement concise capability report, mode choice and legacy-export guard with shared invocation path. Report native preservation as storage-only until merged export is delivered; do not advertise structural-block roundtrip or full support yet.
- [ ] Verify standard entrypoints, menu and CMD agree; test pure-CAD documents, old documents, preserve containers mixed with selected geometry and unsupported edits.
- [ ] Review/checkpoint and update current support matrix without upgrading future coverage rows.

### Task6: Integrate and prove the foundation package

**Files:** Create `docs/validation/2026-10-06-3dm-preservation-foundation.md`; update `docs/openmatrix9-progress.json`, `docs/3dm-coverage.json`, `docs/features/3dm-support-matrix.md`; integrate only verified package source into primary checkout.

**Interfaces:** Reuse Tasks1–5 APIs; no new product behavior.

- [ ] Assert persisted source hashes, inventory rows, identities and finite native measurement error estimates in final acceptance reports. Add missing regression assertions before declaring completion.
- [ ] Run Rust policy/ABI checks and all relevant native tests; rebuild/install primary module after intentional source integration. Wait for linker completion.
- [ ] Run preservation macro, host exchange suite and both immutable rings. Confirm geometry-only ring behavior remains135/101 objects with existing tolerance criteria, and archive preservation mode accounts for original definitions/instances without labeling flattened children as original records.
- [ ] Request one fresh final reviewer; resolve important findings, retest affected checks, and record exact command, process exit and artifact paths. Actual Rhino5 checks remain explicitly untested if unavailable.
- [ ] Mark only foundation rows validated. Present before/after business behavior and ask whether to persist these newly verified preservation rules into the skill; prior block-rule consent does not cover this new behavior.

## Subsequent package plans — complete spec coverage

These packages depend on the validated foundation. Create and review a focused implementation plan for each before coding; this roadmap is not a substitute for those plans.

1. **Merge writer and structural blocks:** native archive overlay, stable source UUID mapping, edit/delete/duplicate signatures, collision-safe multi-import merge, selected dependency closure and safe opaque refusal, reusable definitions/instances and nonuniform placements. Tests compare actual block graphs and world geometry; legacy export guard is removed only for supported safe paths.
2. **Remaining geometry and annotation:** inventory-driven native mappings for Points/point clouds, hatches/patterns, TextDot/text/dimensions/leaders/styles and clipping/detail/page records. Every pinned concrete geometry class has preservation/display/edit/write statuses and independent fixtures.
3. **Appearance and resource portability:** materials, images, textures, embedded files, mappings and lights/render content. Add included-file resource persistence and explicit local external-resource resolution; test texture hashes, relative paths and no implicit downloads.
4. **Document/component semantics:** full layer/group/style/settings/view/cplane/property handling, source-unit provenance and consistent millimeter transforms; distinguish selected-object closure from explicit whole-project export.
5. **History/userdata/plugins and compatibility closure:** preserve safe raw/native payloads, invalidate stale history under edits, reject unknown dependencies and incompatible Rhino5 content. Test target-version reread rather than equating a readable class with Rhino5 writability.
6. **Full coverage acceptance:** reconcile every coverage row against implemented native and applicable host tests, mixed archives/units/collisions/resources/selection edits and real Rhino5 interoperability when available. No full-support claim while rows lack evidence.

## Plan self-review and execution handoff

Foundation API names, schema/property names and modes above are shared across tasks. Future merged export, editable structural blocks and external-resource collection are explicitly not delivered by this package; the roadmap assigns them to dependent packages. This keeps the first deliverable independently testable without claiming whole-spec completion.

Recommended execution: native implementation in this chat with one fresh final reviewer, because Tasks1–5 share manifest and persistence interfaces. Plan status: awaiting user review and execution-method selection. Checkboxes are planning only, not completion evidence.
