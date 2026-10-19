# 3DM Rust port — Phase2 Rust → standalone target (2026-10-19)

**Status:** `PORTED_WITH_AUTOMATED_EVIDENCE` · **Target:** standalone Rust OM9
**Source:** the existing Phase2 Rust owner under `rust/` (not the FreeCAD Python workbench)
**Source revision:** working tree `b5a15dae9ece3ba4a01e7c26fe60b051c9837e2a`

The request was to read the Rust code that already exists and move it over, not to re-implement the
3DM import/export/copy work. This record maps every ported item and states exactly what stayed
outside the standalone target.

## 1. Source files read

| Source | What it owns |
| --- | --- |
| `rust/src/core_3dm.rs` | icon→command/caption table, operation codes, unit scale, supported geometry classes |
| `rust/src/core_3dm_archive.rs` | archive mode/capability/identity, copy budget, retained-overlay policy, dependency closure, geometry-only export manifest |
| `rust/src/layer_state.rs` | canonical layer snapshot, identity/path/parent/cycle/object validation |
| `rust/src/layer_codec.rs` | generation-as-string wire rule and bounded metadata intent |
| `rust/src/layer_clipboard.rs` | Rhino 5 clipboard envelope, layer/geometry binding, digest/size policy |
| `rust/tests/core_3dm.rs`, `core_3dm_archive.rs`, `core_3dm_manifest.rs`, `core_3dm_schema2.rs`, `layer_clipboard.rs`, `native_merge_catalog.rs` | the behavior to port |

## 2. Ported destinations

| New file | Ported from | Notes |
| --- | --- | --- |
| `crates/om9-core/src/three_dm.rs` | `core_3dm.rs` | Same icon/command/caption table, operation codes, `scale`, `type_supported`; added `mutates_document` for the legacy permission split (Import/Paste mutate, export/copy do not). |
| `crates/om9-exchange/src/three_dm.rs` | `core_3dm_archive.rs` + layer validation from `layer_state.rs` | `ArchiveMode`, `ArchiveCapability`, `ArchiveIdentity`, `copy_budget`, `overlay_allowed`, iterative `dependency_closure`, strict `validate_export_manifest` (schema 1 and schema 2), canonical `ThreeDmLayerSnapshot` validation. |
| `crates/om9-exchange/src/three_dm_clipboard.rs` | `layer_clipboard.rs` | Rhino 5 clipboard envelope, geometry header/version check, SHA-256 binding, `bind_geometry_objects`, size/backup policy, rollback ownership. |
| `crates/om9-core/tests/three_dm.rs` | `rust/tests/core_3dm.rs`, `native_merge_catalog.rs` | Command/caption/scale/type/permission behavior. |
| `crates/om9-exchange/tests/three_dm_archive.rs` | `rust/tests/core_3dm_archive.rs` | Overlay, closure (including 100,000-node chain), modes, identity, copy budget. |
| `crates/om9-exchange/tests/three_dm_manifest.rs` | `rust/tests/core_3dm_manifest.rs`, `core_3dm_schema2.rs` | Mesh/B-Rep/PointCloud manifest, malformed tuples, duplicate keys, canonical layer bindings/palette-only. |

The clipboard unit tests in `three_dm_clipboard.rs` are the ported `rust/tests/layer_clipboard.rs`
cases.

## 3. Behavior-preserving adaptations

- **FFI removed.** The Phase2 owner exposed `#[unsafe(no_mangle)]` C functions (`om9_3dm_*`). The
  standalone target calls safe Rust: `copy_budget`, `overlay_allowed`, `dependency_closure`,
  `validate_export_manifest`. No pointer/capacity protocol is needed here; the native bridge keeps
  its own ABI.
- **Strict JSON without a second parser.** The Phase2 owner used a bespoke `Json` parser to reject
  duplicate keys. The port calls `om9_archive::admit_json` (duplicate-key/string/depth admission)
  before `serde_json`, preserving the duplicate-key refusal and the 32 MiB manifest bound.
- **Generation as text.** The `u64` generation is still serialized as canonical decimal text so a
  Qt/JSON number round-trip cannot lose a bit.
- **Layer validation scope.** The port validates identity, path components, parent links, cycles,
  depth, ambiguous paths, the active layer and object→layer references. The Phase2 effective-state
  cache and mutation policy already exist in `om9-core::layers`; they were not duplicated.
- **No host dependency.** Nothing in the port depends on FreeCAD, Qt, `OpenMatrix9Gui`, or a native
  module at build time.

## 4. Deliberately not ported

- `#[unsafe(no_mangle)]` FFI wrappers: standalone Rust calls the safe API directly.
- FreeCAD/Qt/Python host code (`ThreeDm.py`, `ThreeDmClipboard.py`, `Gui/`): historical adapter, not
  target runtime per `AGENTS.md`.
- The native openNURBS reader/writer and the geometry→native mapping (S3 in
  `THREEDM-READINESS.md`): the policy/manifest/clipboard admission is now in Rust, but a full `.3dm`
  export still needs the native writer and OCCT/BRep mapping.
- `rust/src/layer_state.rs` mutation/effective-state functions: already covered by
  `om9-core::layers`/`layer_ops`.

## 5. Verification

| Command | Result |
| --- | --- |
| `cargo test -p om9-core -j 2` | **393 passed**, 0 failed |
| `cargo test -p om9-exchange -j 2` | **126 passed**, 0 failed |
| `cargo clippy -p om9-core -p om9-exchange --all-targets -D warnings` | exit 0 |
| `cargo test --workspace --no-fail-fast -j 2` | **1,236 passed / 173 suites**, 2 known pre-existing failures (`mesh_fill`, `ui_events`); see `workspace-all.log` |

Logs: `build/standalone-validation/ui-artifacts/m01-three-dm-port-20261019/`
Receipt: `implementation/evidence/M01/2026-10-19-three-dm-rust-port-checks.json`

## 6. Residual / next

- The Rust *policy* layer for 3DM import/export/copy is now present in the standalone target.
- Full `.3dm` export and full native geometry import remain the native-writer/kernel slice
  (`THREEDM-READINESS.md` S3), not part of this port.
- The FreeCAD workbench remains the legacy path and was not rebuilt or modified.
