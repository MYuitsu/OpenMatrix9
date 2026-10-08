# Shared standard/menu/CMD 3dm preservation export

The isolated development runtime now routes `ThreeDm.export_file` to the existing preservation writer when a supported selection contains source/native block data. Registered File Export (`ThreeDm.export`), named export, native export3dm, menu Export3dm and the actual CMD Export3dm input all use that shared function. Ordinary unsourced CAD/mesh retains the geometry exporter. Plain layer groups expand and deduplicate selected objects while structural definition/archive containers remain explicit and cannot silently select a whole source archive.

Default supported source selections retain native UUIDs, payloads, source tables, selected dependency closure and native block structure. Unsupported classes/references, owner containers, wrong scopes and ambiguous legacy bindings keep the writer's existing guards. The former blanket message claiming preservation export was unimplemented is removed. This routing does not promote unsupported records or bypass native compatibility checks.

`geometry_only=True` is an explicit boolean option on export_file, export_selection, export_named and the registered Python export entry point. Menu/CMD save dialogs provide an unchecked Geometry only control with disclosure that blocks, source tables, retained records, history and userdata are omitted. Default remains preservation for supported source/block selections. The standard File Export plugin entry point defaults to preservation; the explicit option is available through OpenMatrix9's dialog or API.

For placed source/new blocks, geometry-only mode first writes a verified current native selection into staging, expands its native geometry and atomically writes flattened Rhino5 geometry beside the destination. It follows current canonical CAD/mesh and native matrices rather than trusting a stale display cache. Mesh stays mesh under shear/reflection. Standalone retained records and graphs lacking convertible geometry refuse without replacing the destination. Explicit omission does not authorize silently dropping unsupported members or bypassing preservation graph gates.

Active-document/edit/task guards, whole-object selection, subelement/overlap rejection and destination atomicity remain. The menu checks the same active project again after the dialog. Cancellation returns before export or successful history changes. Unitless/custom source rules and source immutability remain intact. Geometry-only flattening uses extra staging work and does not claim a global RSS/performance bound.

## Verification

- RED: default export_file of a verified native block failed with the old unimplemented-preservation error. GREEN: standard/named/native/menu/CMD entry points now emit the selected native reference and definition graph, preserving source UUID and analytic world point(41,52,63). Unrelated source roots are excluded.
- Default top-level source geometry retains its UUID/CRC. A rational NURBS block retains native ON_NurbsCurve CRC. Standalone supported native ON_TextDot remains retained instead of disappearing; geometry-only mode refuses it atomically. Newly created definitions/instances export structurally through registered File Export.
- Plain layer group plus selected root deduplicates without flattening. Owner-container selection is not implicit whole-archive export. Nonboolean mode, active foreign project, locked task, subelements and edited affine display refuse before replacing the destination. The edited display guard applies to preservation and geometry-only modes.
- Explicit geometry-only source block becomes one world-positioned geometry record without ON_InstanceRef. Mixed CAD/mesh shear output remains ON_Mesh plus CAD, with the mesh world vertex(111.5,223,330), applying the rigid delta once. Ordinary unsourced CAD keeps volume24.
- Actual menu, CMD input and standard FreeCAD Std_Export dialog all write native block graphs through the shared route. The OM9 dialog omission option defaults unchecked; explicitly checking it flattens the selected block. Cancelling export preserves document, target and current successful-history list. The existing geometry suite separately verifies extension-before-overwrite confirmation and normal command lifecycle.
- Save/reopen exports correctly after deleting the test's external input copy. The immutable fixture and native source UUID/host fields remain unchanged. Included archive provenance supplies the current preservation export.
- UI RED: placing the long checkbox in one QGridLayout column narrowed filename entry to118px in a749px dialog. GREEN: spanning all grid columns restores a wide filename field. The automated size check passes; actual export-dialog screenshot was inspected. This is current Windows/Qt6 runtime evidence, not a DPI/platform matrix.
- New shared route acceptance29/29. Full fresh isolated FreeCAD593/593 across33 suites; every final owned process exit0 and source/install/binary hashes stable before/after every suite. Existing preservation test19 and new-block test29 now verify the approved default structural/source export behavior instead of requiring obsolete blanket guards.
- Fresh native CTest8/8,27.42s, final process exit0, including both immutable user ring fixtures. The CoreThreeDm C++/Python module rebuilt and linked into the isolated runtime, final exit0. Native exchange tests are unchanged by the dialog/routing work. Rust archive source/tests fingerprints remain unchanged; prior74/18/fmt evidence retained.
- The complete33-suite run passed592 checks. A subsequent test-only extension reran the shared route29 checks against unchanged production files/binary. The final manifest combines that29-check artifact with the other32 fresh suite artifacts for593 checks; the other suites did not need another run. Std_Export initially waited in the Win32 backend, which ignores Qt global dialog settings. Only that owned process was closed after verifying its PID/command line. The isolated test profile explicitly selects Qt dialogs; Windows native dialog interaction remains untested.

```text
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd
rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-shared-export-regressions.py
```

The runner waits for final owned GUI process exits and verifies result files. Screenshot/geometry artifacts accompany each results.json; the new route's export-dialog.png and shared-export-runtime.png are recorded below.

|Suite|Checks|Artifact|
|---|---|---|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/e116d8275b0e4e9b88fa03e9e563ca5c/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/22c95dc7f406427e8b6dd56f40cd6417/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/299a5ab540cf4499b460cf69e4b726d9/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/8f6b979d1e2342bfb72e05cf0f69321a/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/30a65dd648354833a699b8d8a362c957/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/ddb565714d574c519a1a9bd3dca27e65/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/085dd1b2468d457188c0ff7550549295/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/22a2586a544f4e579b2b61a6fd175994/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/d4f009e403a9444f876348ead4b1d04b/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/6f951dc36a7b4a25882eff8768a39d53/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/c1438c6bb0cf49ad8d94fb4eea445cfd/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/cda9cf6a569d49d58c69714e09ab3993/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/2566445334f2468086ef40ccf54c467c/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/5b7cdb5b18504835b8bfb2408ca90ec9/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/712bcc6ced5f419a84e9451dbab580b7/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/b040cb13a80a4bc299d8723540f0254c/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/869d6ad18dbb4c138234245ab6938108/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/af80f2c0cae04f629cd782fc6daa3be8/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/2e81dae0e6d54992a954091c93e835cc/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/97877039f74a40f7a58fb78d21d1c286/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/2f26c9c607f64352832952ea3fddea4e/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/0a6c7e0804b04d0b8a906bac55cd3016/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/b247c77f29ba43e9b670078be849bfaf/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/f95dc127932f4cfbb11365ef79daa0e1/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/ed738812c68a41e58fa672a5ae6b2d04/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/77a85dda77ea46d2b51a428695e7d4c6/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/377dc5aab3f34b7caad67449f29212ef/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/2b09a7c8af434fcfa8a0d5468df5a07c/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/d1ce16b586c94f14907ead517f5d5061/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/1763fc6550a54dc7bd5d58cbbc5fc316/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/e0761ee99cfc4ae98bded26527851863/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/ee89965bb560401685de5d61774dc281/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/cd6493f7d00d49dd9da1cf75e2ed8de7/results.json`|

- Dialog screenshot: `build/three_dm_shared_export_routes_smoke-1/e116d8275b0e4e9b88fa03e9e563ca5c/export-dialog.png`
- Runtime screenshot: `build/three_dm_shared_export_routes_smoke-1/e116d8275b0e4e9b88fa03e9e563ca5c/shared-export-runtime.png`

## Remaining full-goal scope

Full openNURBS remains in progress: historical representation upgrades, arbitrary retained/plugin proxy families, standalone definition-member/native retained transforms, linked/external resources, complete appearance/annotation/settings/history/plugin reference semantics, arrays/subelements, class/reference/version compatibility and large graphs need more implementation and evidence. Source top-level selections are verified here; selecting a native definition-member alone remains guarded by the writer. Different-source document policy, final whole-package review/public integration and actual Rhino5 application acceptance remain pending. Aggregate counts do not establish untested coverage-matrix cells.

Advanced source stays in `H:/FreeCAD-src/build/om9-dev`, runtime in `H:/FreeCAD-src/build/3dm-preservation-sdk`. The public concurrent checkout has not integrated this package. Git registration is unavailable; verified checkpoint/fingerprints identify the local package. This milestone changes runtime routing under the approved plan, records technical documentation and does not itself authorize reusable skill business-rule writes.

## Fingerprints

- `ThreeDm.py` SHA256 `93c87f09eae7ddccf5329cfd99927791e4a1bc3952c608a0c78c973ab5a1469f`
- `Gui/CoreThreeDm.cpp` SHA256 `b58b468e8c8a78c276b1a92a6ead63921d1de59741770ae38db24ff1ccc9fbda`
- `tests/three_dm_preservation_smoke.FCMacro` SHA256 `704824deb1fd48f407c80c3cb501b4c29f676b00bb6bd32960344b7e1b34b1d5`
- `tests/three_dm_new_blocks_smoke.FCMacro` SHA256 `70ee734ad30714865ef9f98a03e4fcba34c2070895324603a8263bdc7596a55e`
- `tests/three_dm_shared_export_routes_smoke.FCMacro` SHA256 `94e92967c6fe97815f60504ff30e708a19ddd42c7052b271198b39bb3b935001`
- `docs/development/3dm-structural-blocks.md` SHA256 `8a749886a45fb93600733714d80efde099bcc8ce3e32e5bc68368b4a4b5455d0`
- `docs/features/OM9-FILE-012.md` SHA256 `8e1f7dce9f2f8190823caf2120c3fcecd5b776d7aeb8dc0ce8c29960250137de`
- `docs/features/3dm-support-matrix.md` SHA256 `4f7649435126c001234f30a8af6142ac03902fe262044ca32305a72cc450723e`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `4dc766a65b6772ae43146cf417cc912e4395548bf575427cc89f3abfc8918872`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `cfccc7175bd8c893a030f63ee30627bb94929e307a621f936f648c048a53f88b`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `6ee79c9c31129bda6b2488628cef59523a9318744d43dac8ea50821432abe9df`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `d0328460c0bab3e2b4d166415ddac508d429dd2b008d79ded823a418451a195f`
- `README.md` SHA256 `9ad97e7e00dd86dbda22774d6dce67c46fa5e111e789a2aa09743d65ae557516`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `6e65c6499c8fc6db6da8422d30c61a00486488f55b548d6258377152ab9ca2de`
