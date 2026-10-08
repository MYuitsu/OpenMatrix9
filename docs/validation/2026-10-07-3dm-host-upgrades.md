# Explicit legacy 3dm host upgrades

The isolated development runtime adds an explicit `upgrades` policy to `ThreeDm.migrate_archive`. A supported legacy geometry holder with valid current CAD/mesh payload can become a native Part::Feature/Mesh::Feature. A supported flattened instance holder can become the representation verified from its included native source archive: App::Link, Part::Feature or mixed App::Part preview. Internal host Names, native UUIDs, original signatures/source matrices and current geometry remain independent. Ordinary incoming App::Link aliases are excluded from source inventory when they merely forward an object's UUID.

Each instance policy must name its placement interpretation. `delta` composes a physical root's current rigid delta with the verified native Placement exactly once, while affine preview hosts retain their separate delta over immutable native A. `instance` retains a complete current physical Placement; an affine complete Placement is accepted only when its difference from native A is representable as a proper rigid Placement. No scale/shear is silently discarded. An optional physical `target` names a verified original/copied/new definition; it cannot silently alter an already upgraded physical host on a repeated migration. Existing affine target mappings remain separate.

Physical replacement uses one existing migration transaction: remove/recreate the host with its exact internal Name, restore supported dynamic values/property groups/documentation/editor/status flags and current display metadata, and rewire ordered groups, whole links and supported custom Link/LinkList dependencies. An App::Link uses OM9Color for archive metadata and explicit ShapeAppearance/OverrideMaterial for current display color. Default Python view provider integer 1 restored by FreeCAD is distinguished from an actual custom Python implementation. Replacement/remapping helpers run with the preview observer held busy and restore its previous state even after failure.

Instance replacement requires `rebuild_previews=True`; its former CAD display is derived cache, not source geometry. Mixed CAD/mesh preview rebuilding uses the current canonical definition graph. A geometry holder lacking current payload refuses instead of restoring the source snapshot over a possible user edit. CAD-to-mesh tessellation is not implicit. Unsupported scripted/custom payloads, expression dependencies, arrays/subelements and cross-document rewiring require dedicated adapters and refuse. App::Part-to-other-host replacement and arbitrary historic/plugin layouts remain outside this verified slice.

## Acceptance evidence

- RED: the new explicit host-upgrade macro failed because migrate_archive had no `upgrades` argument. A separate native FreeCAD probe established exact-Name replacement with working Undo/Redo before implementation.
- GREEN: 39/39 focused checks cover physical instance and editable CAD/mesh holders; immutable UUID/native matrix/baseline; current canonical edits; analytic world points (17,29,41)/(100,200,300); ordered group/custom Link/LinkList/incoming alias rewiring; custom property documentation/editor policy; retained display color; idempotence; Undo/Redo; FCStd persistence; and injected late failure rollback.
- An explicit new-definition target exports a box of volume24 at world (15,26,37). A mixed native shear graph exports edited CAD volume120 with bounds X16..22.5/Y27..32/Z38..44. The upgraded mesh exports world vertices (21,27,38),(23,27,38),(22.5,30,38), independently checking native A and current edits.
- Invalid host/type/mode/policy/target, missing current CAD payload, unknown Python payload, subelement/expression dependency and unrepresentable affine complete Placement refuse without host replacement. A late failure restores original type, graph, placement, object count, observer, document set and transaction state.
- Fresh native8/8, final exit0,28.52s; includes both immutable user ring fixtures. Fresh isolated FreeCAD632/632 across34 suites, each final process0. The runner verifies source/runtime byte equality and stable Python/binary fingerprints before every suite and after the full run. Rust is unchanged; prior74/18/fmt evidence remains applicable only to its unchanged hashes.

Legacy fixtures are deliberately synthesized from verified native imports, including a saved/reopened older holder. No actual historical user FCStd was supplied. Actual Rhino5 application acceptance, full class/component/resource/reference/version coverage, linked resources, arbitrary retained/plugin graphs and final public integration remain pending. Aggregate counts do not establish any untested coverage-matrix cell. Development source is build/om9-dev and runtime build/3dm-preservation-sdk; concurrent public Mod/OpenMatrix9 is unchanged by this package.

## Reproduction

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd`

`rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-host-upgrade-regressions.py`

Finish native fixture generation before starting the GUI regressions. One owned FreeCAD process is started and closed for each macro. A timeout alone is not process completion.

|Suite|Checks|Result artifact|
|---|---|---|
|host_upgrade|39/39|`build/three_dm_host_upgrade_smoke-1/4b7535e8e8dc4f49bde4c09842f0047d/results.json`|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/432534032d0b4754acf122a0a5bfff4d/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/5e3fa4138b804fe89db65fe7fd1fc7ad/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/204cf07b5610435992ee4f13f0ced7c0/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/227f5a1d73fd47789fa8c8bc34e7fa27/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/5543c78f62764e51841dc5945fdad270/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/50ec9f2b05954306a9cde2451c503a4b/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/f88f335fdda049e5a83e52ebf7ef72e6/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/92f10a701da74fcca7813d825df1ba5a/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/310a16669bcb42a19a86743e25dceeb1/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/27e300c540384ab180219d094f9f2d65/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/88129875345f4fa58a875d11755c0a7e/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/91ea550a3a1845cf838d0061ab646fe4/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/36433d3744124251aac434893ffc9ac6/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/c81f33ea90f94ede9cef96cac1fb6f88/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/cf4c735e538f483b9990a415ba87624c/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/e947e7dc885b4ba5a52daeedf88ad168/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/6e3e81832c304ff69a915e0ed15b1890/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/c97759b8b5ea4dc4ac028d4e4a1fffe0/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/339515631da24e68b948a12d9152af20/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/47808fd7f7c749eb8f0aa6b42031c472/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/e456949fe8f840d1bee91e7f46c2b268/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/b28b7837996749728c9421dd6a61d254/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/82e5787d95fc429a9a920ddba72b2248/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/46b35f959c524a3abc1890596a7242a9/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/20e9a211db674062a7af404d04307b07/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/a4e21de1a73f43ba902a542a9da94fa4/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/731aea363729482788bea9aa3f5f32d9/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/e21da6c5515041b9a53e6908698e33c5/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/cf928fafcc4c44fb94909773c1a9262a/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/da65c3799a354279a7f21ad78f9362f4/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/555ae277c0904cf08cc0e5ea24dae94d/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/8335cfcb34594b93974570ebc19b4348/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/287f895527814cefa9dbbd60682800a1/results.json`|

## Fingerprints

- `ThreeDm.py` SHA256 `65b837a9ff628284b74a370f4fe7c8bf1248289412fa38d0d533ff58973b186d`
- `ThreeDmArchiveState.py` SHA256 `a3f8f5df139a370035b6c22c1901fe7af5392a0f2234347f6b9729c0bfccd6bd`
- `ThreeDmMigration.py` SHA256 `86f573e4090b1d80d76627dff849fb742e7f1cebed5ba59346056853f695a9f5`
- `ThreeDmUpgrade.py` SHA256 `5fcf9e048f914175136dd489709e82e22adafc512d9369a6773aa701da3d25a6`
- `InitGui.py` SHA256 `2d920887a898a94f90eef120bdc14f989a96567eb8094e66f6b6cd022389f157`
- `CMakeLists.txt` SHA256 `fe6b50afc56d8e9a087ca9028e803727209c25b518e72579ac76a477f2036ae6`
- `Gui/CoreThreeDm.cpp` SHA256 `b58b468e8c8a78c276b1a92a6ead63921d1de59741770ae38db24ff1ccc9fbda`
- `tests/three_dm_host_upgrade_smoke.FCMacro` SHA256 `637a046d9d87f555e28d54a4a6527f6575bc28270cfbbcc8fbc6660997f72693`
- `tests/three_dm_upgrade_host_probe.FCMacro` SHA256 `c5e5dffa9b497aa2a174902b78e23b32ac2657d68c7acbe3635f6f30b9645f83`
- `docs/development/3dm-structural-blocks.md` SHA256 `006543737b99370172449b17eb27a26100ca723ef2ec5857051e60472cc8f554`
- `docs/features/OM9-FILE-012.md` SHA256 `db0cf1401a013c6bfe6cdc70bb7421b7bbdf2aa02051798fa355fe7eb7b9d90d`
- `docs/features/3dm-support-matrix.md` SHA256 `d43ba6df4ce3452205d1bead725d26a94396c279d0c0855ebfc1dd94c6feae7f`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `e0573709167e59fa65d650d71528e5b30009fa919dd16585ab9de0b0e25d2b28`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `222b9ccad64685f7f58b460ea06c12782e3928f0b2396474a185d38ef2fa84b9`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `d383765048f7dcabf3092936552aac61bc428537661cd54268d79a668964ca9a`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `e6280fee0262e11012d27f3ecb73633d55f9dd915bf2065f0067c9568bd88ea4`
- `README.md` SHA256 `e8d0f30017d104fc608996ceac0bdf0e19753818faa98c19990bfcde2e893731`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `6e65c6499c8fc6db6da8422d30c61a00486488f55b548d6258377152ab9ca2de`
