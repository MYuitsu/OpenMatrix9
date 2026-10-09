# Main integration validation — 2026-10-09

Integrated updated `main` (`c55af89`) with the saved CAD/P0 branch
`codex/current-work-20261009` (`5011bab`). Both parent histories are retained.
This report records merge validation independently of the historical P0 checkpoint.
P0 remains partially implemented; these checks do not establish complete Matrix/Rhino parity.

## Integration resolutions

- Retained the newer main 3DM archive, worker, modeling exchange and preservation paths alongside CAD, History, Cage and P0 services. Combined native registrations, CMake sources, Rust modules, command permissions and completion entries. Removed three duplicate curve FFI definitions produced by automatic merging.
- Preserved the strict P0 export validator while accepting the main staged PointCloud schema, SHA256 and finite 4x4 transform. Legacy optional color/tolerance defaults remain valid. Added two regression tests; mixed representations and invalid explicit fields still reject.
- Retained-instance Explode now verifies the current source/owner/preview and a Rust-validated preservation request before decoding the original archive. Modified placement, definition, membership or preview cannot silently explode stale geometry. General edited-instance Explode remains unsupported and rejects without mutation.
- Geometry-only export projects the verified native read result to the explicit write schema. Source/display metadata no longer triggers the strict P0 validator. Current geometry and PointCloud transforms remain covered by runtime checks.
- Updated the obsolete P0 fixture that expected all edited preservation exports to fail. It now verifies current point geometry and UUID after export, then verifies an invalid shape leaves the existing output bytes unchanged.
- Combined all 23 historical progress records by stable key and retained their original evidence scope. The separate merge record below does not promote feature completeness.

Rust owns portable validation and snapshot policy. C++ changes are required native
FreeCAD/openNURBS/Qt adapters. The Python change projects an existing host API
manifest; it adds no geometry solver or portable business logic.

## Verification

- Matching FreeCAD SDK build of `OpenMatrix9Gui`: exit 0. SDK: `build/relWithDebInfo`; dependencies: `.pixi/envs/default/Library`; private runtime: `build/reusable-history-runtime/bin`.
- Full Rust tests: **334 passed**, 50 test-result suites; Clippy all targets: exit 0 with **66 warning diagnostic lines** (not a warning-free result).
- Python tooling: **49 passed, 1 skipped**, 50 tests run; exit 0.
- Eight standalone native kernel suites rebuilt and passed: geometry, archive, modeling exchange, PointCloud geometry, merge, blocks, inventory and preservation.
- Native GUI: **847 passing assertions across 15 exit-0 runs**, including a cold reopen of the newly saved mixed CAD/mesh FCStd file.
- Spec manifest: all **664** entries verified after main's LF normalization. Syntax and source conflict-marker checks passed.

| Runtime macro | Passing assertions |
| --- | ---: |
| modeling_exchange_smoke.FCMacro | 32 |
| rhino_core_units_smoke.FCMacro | 20 |
| core_coordinates_smoke.FCMacro | 162 |
| core_snap_lifecycle_smoke.FCMacro | 10 |
| solid_commands_smoke.FCMacro | 70 |
| surface_commands_smoke.FCMacro | 97 |
| history_smoke.FCMacro | 83 |
| rhino_core_history_io_smoke.FCMacro | 7 |
| edit_3dm_special_smoke.FCMacro | 44 |
| three_dm_cloud_geometry_smoke.FCMacro | 50 |
| three_dm_shared_export_routes_smoke.FCMacro | 30 |
| rhino_core_geometry_smoke.FCMacro | 58 |
| edit_commands_smoke.FCMacro | 115 |
| curve_spline_smoke.FCMacro | 41 |
| rhino_core_geometry_smoke.FCMacro (cold restore) | 28 |

Every GUI run records the module path/hash and macro hash through
`tests/merge_replay.FCMacro`. Earlier unaffected CAD runs used the preceding
merged build; the affected retained-instance, geometry export, shared export,
Edit, spline and cold reopen routes passed on the final build.
Final native module SHA256: `53aa3d7bb14be52a473310442466a9184121ecb5ac5bb1abea4f385b02a32938`.

The [machine evidence](2026-10-09-main-merge-evidence.json) records exact parent
revisions, artifact paths, hashes, process exits and excluded failed attempts.
Build/test artifacts are local and ignored; test macros and this evidence are versioned.
Historical counts in the [P0 report](2026-10-09-rhino-core-p0.md) remain historical.

## Reference handling and limits

Eight commercial reference images encountered during integration were preserved
outside the public repository, with hash verification, under the existing
[private-reference policy](../private-reference-policy.md). The ignored local
reference registry resolves them; neither it nor the images is included in the
merge tree. Existing Git history was not rewritten. The pinned Rhino 5 PDF
exception and its notices remain unchanged.

The full P0 object/selection identity, shared tolerance and History contracts
remain open. Main's implemented current-state 3DM preservation slices are retained;
they do not establish universal edited-object preservation or Rhino application
compatibility. Native testing used an isolated runtime and does not update any
already-running FreeCAD process.

Final source audit: 2014 files, 511 icon bindings, 510 SVGs; no errors. A local audit does not constitute legal clearance.
