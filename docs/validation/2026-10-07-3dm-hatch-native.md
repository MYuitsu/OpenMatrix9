# Native Hatch and HatchPattern reference preservation

## Implemented and verified scope

ON_Hatch inventory now includes the exact custom HatchPattern UUID dependency, native plane16 doubles, 2D base point, rotation, pattern scale, gradient type, world tight bounds and per-loop type/native class/bounded Rhino5 curve SHA256. ON_HatchPattern records description, fill type and complete line angle/base/offset/dash arrays. This inventory is semantic evidence for the named native package; it is not a class-wide compatibility or editable display claim.

Selected native Rhino5 hatches now retain their required pattern table and exact native rational outer/inner loops, source/native user text and object attributes. Unused patterns are removed by selection closure. Native preparation validates nonempty closed2D loops, plane/base point, finite rotation and positive finite scale. Pattern native validity validates angle/base/offset; explicit dash finiteness was added because the pinned ON_HatchLine::IsValid omits it. NaN/Infinity dashes reject before inventory JSON can turn them into null.

Unit normalization scales PatternScale as well as boundary coordinates; the pinned ON_Hatch::Transform alone does not scale pattern spacing. The shared helper invokes ScalePattern before native geometry transform for verified conformal, orientation-preserving placements. General shear/nonuniform/reflected pattern transforms and determinant-one scale cases are explicitly guarded pending complete pattern/loop handling. This does not mark those required full-objective cases complete.

Selected table writes can compact a custom pattern index while preserving its UUID. Hatch raw geometry digest uses a clone with only nonnegative custom PatternIndex normalized to0. All other native bytes remain compared; pattern UUID dependency and complete pattern semantic fields are independently verified. Negative built-in indices remain exact. Every hatch is cloned consistently for digest serialization, including index0, so native userdata copy counts are compared on the same basis and temporary Rhino5 basepoint userdata cannot mutate the source object. No generic native-payload comparison was removed.

FreeCAD retains hatch as an explicit source-backed FeaturePython with OM9BlockMemberPlacement. Native export preserves the current placement and physical parent once. Undo/Redo, independent top-level copies, embedded source/pattern persistence after external deletion and FCStd reopen pass in mm/cm. Hatch has no invented CAD Shape or float display/editing authority.

## Evidence and fixes

- RED: fixture selecting a hatch using custom pattern index1 lacked the HatchPattern dependency. Inventory now resolves its component UUID before export closure.
- A native payload comparison exposed pattern-index compaction. The first normalization recursively cloned only positive indices, making userdata copy counts asymmetric against index0. Both sides now serialize a consistent clone and verify the reference UUID independently. Exact pattern lines/dashes and native geometry remain enforced.
- The initial host test used plane origin as an XY translation oracle. Native ON_Hatch::UnrotateHatch can change the plane parameterization while preserving the boundary; the corrected independent geometric oracle uses world tight-bounds center. No geometry workaround was introduced for this test issue.
- Native negative tests prove unsupported shear/reflection/determinant-one scaling reject without field mutation. Standalone NaN/Infinity dash tests failed under native IsValid alone, then passed with explicit finite checks.
- Native14/14 final exit0 on the current source, including both immutable user ring fixtures and the independent legacy5 PointCloud reader. The final LastTest.log is saved in the checkpoint.
- Fresh isolated FreeCAD833/833 across40 suites on the final binary; each final owned process exit0, stable source/runtime/binary fingerprints. Hatch16 checks cover both unit systems, exact pattern identity/fields, native position/spacing, Undo/Redo, parent placement, source removal/FCStd, independent copy and immutable input bytes. Native fixture generation finished before GUI regression execution.
- Rust is unchanged at the preceding75/75/fmt source fingerprints; this is not a fresh Rust test claim.

## Required remaining work

Hatch display, current plane/basepoint/rotation/scale/loop/pattern editing, general affine pattern transformation, reflected/anisotropic physical proxies, native block/canonical/shared member coverage, explicit legacy adapters, built-in/solid/gradient compatibility, corrupt-reference/native-reader repair audit and child-loop userdata/reference handling remain required. Named tested hatches use custom line patterns and two rational circles; do not claim all hatch geometries/styles from them. Broad resource/style/document/history/reference/version coverage, actual Rhino5 application acceptance, final comprehensive review and public integration remain pending. Full openNURBS stays in_progress.

The verified module is in build/3dm-preservation-sdk; development source is build/om9-dev. Public source was not integrated. No reusable skill/business rule was changed. README, Spec01-core extension, support matrix, coverage and ledger reflect this scoped evidence.

## Reproduction

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-hatch-tests.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd`

`rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-hatch-regressions.py`

|Suite|Checks|Result artifact|
|---|---|---|
|hatch|16/16|`build/three_dm_hatch_smoke-1/2ec671282c994e6d9ee0694736d26ca8/results.json`|
|cloud_geometry|50/50|`build/three_dm_cloud_geometry_smoke-1/20758ef811a3415f978af12997bbeb9d/results.json`|
|point_cloud|43/43|`build/three_dm_point_cloud_smoke-1/29adb225f7b24b5f98c3ba35478d3b98/results.json`|
|text_dot|34/34|`build/three_dm_text_dot_smoke-1/af395eb8da9c4633b674815e446e89a2/results.json`|
|selected_member|33/33|`build/three_dm_selected_member_smoke-1/e0947f71f5b943a391ede4870b06d278/results.json`|
|retained_transform|25/25|`build/three_dm_retained_transform_smoke-1/de4a456f74944a77834b2190f37038cf/results.json`|
|host_upgrade|39/39|`build/three_dm_host_upgrade_smoke-1/57da9f25e81846c4b2b78a08cbb41ef5/results.json`|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/42eca196999a4a7088d70cfeef4dd9b7/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/c3c20d7686bf45cc987fe2ae48d5ad22/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/8aaf05ed251d4760b930e0673387cd13/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/fb0636dae81e4b67b1991ebb9158496e/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/91cae6543aef4e3f8edbc416482e4a11/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/04a2993fa39e4f37ab46e4ed0a92c5a4/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/0c0bba03f19e4e34aceae9c127cf3577/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/f0256779e3d047fea91c0511937bf3c8/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/6868371b58d84f7dab1b937fa7f99895/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/2c1333157e8f4fca92bfc457de59e461/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/1c3f75fc5ed642fba48cd190b7904010/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/6ecde1a9f576415ba20891c1b1442278/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/446bd64c9f124ea68903e36d1c109a6e/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/5a3f1fd892d24ad78cb5458643dcded0/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/068c6fa908914dbdb4a053334ab5179f/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/7d6da4f7317248599c144ea998ff1eef/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/5e581907a04645b1bbf8ad349140f714/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/66f52cb21d584d30ba6b82047f774177/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/2354043eed2c45ff8c325b7e8e0be616/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/4d2f059fbbac498798f8b46dd2c08dcc/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/3158b51c5fa145d6baea52d1390cf550/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/371b928fb19d40919a11a18483d592ee/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/abba554779e343898e56d6c605d6a9fc/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/34a52b3a466f478e8dc60cf123225b97/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/c5ac49ca6e874278ab59ed6dcb0045cb/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/bf84470081c14bf58068fdd89d6093cc/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/6803f63f649645b2b71aa8ba9a023a8d/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/95e9177a2dbf4bd7b7add3fab0ec6e5a/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/42f805068d814257b10443044f4f31b5/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/e59cf056e6ca4850a01d48d1592b6c96/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/0c0f22c3695f4c69a318ca4c3d987cb7/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/5691ef22a0ec421e80205b898ff57a0d/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/e76d01bfbfc84c7a8e014ef7b37fadd8/results.json`|

Checkpoint: `H:\FreeCAD-src\build\checkpoints\hatch-native-20261007-052710.zip`; all saved files have independently verified SHA256.

## Current fingerprints

- `ThreeDm.py` SHA256 `449e11f46847c0573b947324925967a0d808c02aca10b75dcc754ee67eba6bba`
- `ThreeDmArchiveState.py` SHA256 `19d15ba70ae081c305273dd699204dbcbe07c966883ac44101c6cda63e9915e6`
- `ThreeDmMigration.py` SHA256 `146cae509edb79f790fe5b8f71a3a7aadde02853c440ce205d50fccec54f2622`
- `ThreeDmPointCloud.py` SHA256 `daa37635ab67b2d2ff38fdea15e9e553d27054b776a8f9b71de7b7a38b48cce6`
- `ThreeDmTextDot.py` SHA256 `d345d8c5f4e58a7270e622f88fed28ce94f9856137d0c5b50c2b16575a544537`
- `ThreeDmNativeFields.py` SHA256 `b7000a7d672e2f99296b8480e1edc5eadea9984b8226dfde6b1204aea1f0b21d`
- `ThreeDmUpgrade.py` SHA256 `5fcf9e048f914175136dd489709e82e22adafc512d9369a6773aa701da3d25a6`
- `InitGui.py` SHA256 `2d920887a898a94f90eef120bdc14f989a96567eb8094e66f6b6cd022389f157`
- `Gui/ThreeDmHatch.h` SHA256 `1f7afd31417c5e3a5b1a0950b8718a553fb3e2e9228ce2a4a4021a327fe4dd46`
- `Gui/ThreeDmHatch.cpp` SHA256 `2066cf0edd67dbab2b96f461f0e24f61c23b85467d5b8eacd1e1da772a7967f9`
- `Gui/ThreeDmInventory.cpp` SHA256 `8a65b54d186763159e095d7d28449c94256265eeda23cd94638cc0a603fc0354`
- `Gui/ThreeDmTransforms.cpp` SHA256 `9b65989d7350176febca3bdb54e15de2bb6a1377b1f2ae6516949d968d97223f`
- `Gui/ThreeDmMerge.cpp` SHA256 `078827c465c987693ef72bb8b9c96a820e3b185315f8a7c3a715c7509654de31`
- `cmake/OpenNURBS.cmake` SHA256 `e2d8acaa985303818501ed0bd0b367ab9c5e2fd20c7ec4d0bc40796902aa3410`
- `tests/native/CMakeLists.txt` SHA256 `cb27afe56e534fbae26dbddc13e159d16c63dd175e9732f1498dfbc5494ed106`
- `tests/native/three_dm_hatch.cpp` SHA256 `7f363e2aa47484c607c3aead8ec96ac7aca944f43534feea16925e86f5929919`
- `tests/three_dm_hatch_smoke.FCMacro` SHA256 `3438b2983235467aacd90c14596d71fbb66ff37abf750a022fc61d53df06a8fc`
- `docs/3dm-coverage.json` SHA256 `7e2a0ce6321e252392e9c6fc275f1fc51e7c3eff58a433e7f55b346cbd63520c`
- `README.md` SHA256 `a23f4a5183897d1ce354e2f58b6f960d7663bc61ec1af250e480bf1d96f9ade4`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `7eff255e0fe059c4c2fa197b4521d35a4e3609628371f9b348af0e37c829baec`
- `docs/features/OM9-FILE-012.md` SHA256 `0b1085281f5ceae8f39d168941a0b3d94f6d8c7fc2dc60ac5fe5d6659fa62d36`
- `docs/features/3dm-support-matrix.md` SHA256 `5438835131b024024cbb4cb6bf333ce071bb6de672e1bcc20ff44224bd54a14c`
- `docs/development/3dm-structural-blocks.md` SHA256 `18a80e9f2f2b2ec912e77b5eb86744f4c56fd924cdb8b283d7d2f98c764c6c1a`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `779c15727241f8452beb064cc512ffbf546dae679f17a9b3e7103103e51fd730`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `c86d9e9bfa18c3489af37dbad9800d1924bbb20da956f1351b5418f1b8cb7f55`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `7d9597c62a00f08a297fa51c9c37b85d49de33a999c17acc50dbaffd85cba172`
- `docs/superpowers/plans/2026-10-07-3dm-hatch-native.md` SHA256 `f72019c24a77d67c948b8c0408c9909bb73a888240d695be67fca782abd90910`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `ac5566ecf02d3e7d116440edbbe311a5dcff366ca7c7747f7ae32af0a222860d`
