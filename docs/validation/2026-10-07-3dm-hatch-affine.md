# Native Hatch affine boundary and isolated pattern transformation

## Implemented and verified scope

Hatch transforms now explicitly transform the plane and rebase native2D loops,
including determinant-one scaling, world reflections, shear and anisotropic
placements. The pinned SDK's world determinant shortcut is no longer used.
Native rational curve representations, loop types, base point and user text
remain preserved. Work is staged on a clone before committing current fields.

The induced2D map determines pattern handling. Conformal maps retain the original
pattern identity and adjust hatch scale/rotation. General maps produce an isolated
native HatchPattern: transformed direction/base/repetition offset and signed dash
lengths for every line. A sibling hatch using the original pattern stays intact.
Generated UUIDs are deterministic for the same ordered source graph, with bounded
collision salting; existing native components are never overwritten or reused.
Native model context is passed through preservation transform/copy routes, and
new pattern records/current dependency edges are registered before selection
closure. The existing exact native payload, object attributes, complete pattern
facts and semantic reread gate remain enforced before atomic output replacement.
Current generated component manifests retain the32MiB limit.

The pattern endpoint oracle follows the published ON_HatchLine line-frame contract
(base/offset rotated by line angle; signed dash lengths along the line):
https://developer.rhino3d.com/api/cpp/class_o_n___hatch_line.html and the pinned
opennurbs_hatch.h. This establishes mathematical/serialized behavior under that
contract, not an actual Rhino application display comparison. In particular the
shipped Plus pattern's nonzero bases warrant an independent Rhino renderer oracle
before claiming built-in-wide appearance compatibility. No blanket visual or
class-wide Rhino5 compatibility claim is made here.

FreeCAD now composes the canonical native placement when promoting a retained
Hatch proxy into a new block. Selected reflected/anisotropic proxies, unchanged
siblings, promoted blocks, independent copied families, current canonical/scale
edits, Undo/Redo, embedded source deletion/FCStd reopen and atomic singular errors
pass in mm/cm. Hatch remains source-retained; editable loop/pattern fields and
viewport display are not yet implemented.

## Evidence and investigation

- RED: the new native analytical affine fixture failed with the previous precise
  unimplemented-transform gate before production changes.
- Native20 map cases cover5 transforms ×2 plane orientations ×mm/cm, sampling33
  points on each of2 rational loops and every multi-line signed dash endpoint for
  repeated offsets. Every transformed/sibling pattern reference resolves after
  Rhino5 serialization. Repeated complete geometry/component records are stable.
- Native8 canonical/baseline member-copy cases retain only required derived
  patterns in selected block closure. Invalid affine export keeps destination
  bytes intact; source archives stay immutable. Named Grid60/HatchDash/Solid
  transforms have standalone native mathematical checks, not independent Rhino5
  application compatibility evidence for all built-in styles.
- An initially empty style fixture caused SDK reader repair to assign a new
  ON_DimStyle UUID on each read. Diagnostics isolated that component; the valid
  analytical fixture now provides an explicit style/current-style ID. Auditing
  and exposing native reader repair of incomplete real documents remains required;
  no assertion was filtered to hide a random generated hatch pattern identity.
- RED host test found promoted retained proxies omitted OM9BlockMemberPlacement:
  a12mm Z displacement was lost under scale4. The baseline retained branch now
  composes that explicit placement exactly once; selected/new/copied routes agree.
- Final native15/15 exit0,36.07s, includes both user ring files and independent
  SDK201307115 PointCloud reader. Fresh isolated FreeCAD857/857 across41 suites,
  every final process exit0, stable source/install/binary fingerprints. New Hatch
  affine24 plus preceding Hatch16 checks pass. Native fixture generation completed
  before final GUI regression execution.
- Rust source unchanged at previous75/75/fmt evidence; not a fresh Rust test claim.

## Required remaining work

Hatch display/current plane/base/rotation/scale/loop/pattern editing, full legacy
and gradient behavior, broader built-in/solid/non-circle/child-loop userdata and
reference handling, native reader repair/invalid-reference audit and actual Rhino5
renderer/roundtrip acceptance remain required. Near-conformal classification uses
relative1e-12 tolerance; extreme coordinate/dash ranges and large-data performance
need broader numerical/performance validation. Other geometry, annotations,
style/resource/component/document/history/version categories, final comprehensive
review and public integration remain pending. Full openNURBS is in_progress.

The verified runtime is build/3dm-preservation-sdk; source is build/om9-dev.
Public source was not integrated. No reusable skill/business rule was changed.
Earlier dated affine guards/pending notes are superseded only by this named native
and host evidence; all untested class/category requirements stay open.

## Reproduction

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-hatch-affine-tests.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd`

`rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-hatch-affine-regressions.py`

|Suite|Checks|Result artifact|
|---|---|---|
|hatch_affine|24/24|`build/three_dm_hatch_affine_smoke-1/732baeaaae5548ac879fabfe442b0e08/results.json`|
|hatch|16/16|`build/three_dm_hatch_smoke-1/8b1460ca883a4269b734174bfb4b66aa/results.json`|
|cloud_geometry|50/50|`build/three_dm_cloud_geometry_smoke-1/ccac034d17764dca885e546782b129a8/results.json`|
|point_cloud|43/43|`build/three_dm_point_cloud_smoke-1/8c23b897ada54f698dd96c2380ee3e62/results.json`|
|text_dot|34/34|`build/three_dm_text_dot_smoke-1/61a9e63a64784b91a0c503ee9eee27c2/results.json`|
|selected_member|33/33|`build/three_dm_selected_member_smoke-1/e0d0667308984fc8ac71641195953367/results.json`|
|retained_transform|25/25|`build/three_dm_retained_transform_smoke-1/2f2e33380cb1433cbe4824ef13d7141f/results.json`|
|host_upgrade|39/39|`build/three_dm_host_upgrade_smoke-1/f89ec2b898184712bb49d70ac2c02056/results.json`|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/4d406d69491044f8b77eaabb395830ec/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/de2bf3cae3904679abd4988676cf9462/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/7cea3ba1c89b4d01828e2af6c592df46/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/35d648c2da1540e88e5823f6497181a2/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/0eca1703e9ef4f6b82cfdfbbdcbbb317/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/2746d6a21ba841cf93c08d754bfaa601/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/27c626c52e9c47c69325a40b176f499b/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/9d7bf0ea0bf2498ebb0ca111261ed561/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/2d86311c91024485b13b6d4f45531e03/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/f633731bd3414455bee71b312e3e7b9a/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/7a085d1ca71f45638d4eb8c6f86ab3e1/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/41ae89246ef041f79b3d7693379fadbc/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/8ad934ad6b2c463b8a648033f4f9290f/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/756f3a8bdb9b4d8a8cf1d3d9ca9ec1a3/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/329aff7788bc47039a97f797d0b3d367/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/6452b89dff1249448a3f4e9742806521/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/045a64c6226a40ef8621908340fd72dc/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/01bc85e332c9445bb9eb537622b3088f/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/19828595ae624a3990bdfaeee7522b71/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/76252e5856f6432ba9387b1f8a3b8599/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/bb1a6ff7f3394c11adf8ab1d17e0c92e/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/40ae5c709b604754ab705b838a305d4e/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/3689e416dfde4b4b9b84733fb549fc19/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/f698054901b94adb8dbc64e7057dc2fb/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/b7a1cd99f6af48beaf3d20dfb81071d8/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/36a33881e4a7488e9fcc745ec54f55e5/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/056c8ad680374bf8927319745f7cf004/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/2408b153566043789d2d7d04f279936f/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/9c3d355cf09e4c39afeed93e93a5b4e4/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/7b3e3d7bb920466e9e136ddc409fdaf4/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/666cafbc08d94e8fa19781a2e7fd157f/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/f730ca782f554779a347eca21f6b9a8a/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/8332ea5c4ab94728b0de03c7b0c17463/results.json`|

Checkpoint: `H:\FreeCAD-src\build\checkpoints\hatch-affine-20261007-054327.zip`; independent SHA256 verified.

## Current fingerprints

- `ThreeDm.py` SHA256 `449e11f46847c0573b947324925967a0d808c02aca10b75dcc754ee67eba6bba`
- `ThreeDmArchiveState.py` SHA256 `8bb04fe5a7bda09399e2148463d41bd1d3c14c9c0ccb9bff5b12823dabc2e313`
- `ThreeDmMigration.py` SHA256 `146cae509edb79f790fe5b8f71a3a7aadde02853c440ce205d50fccec54f2622`
- `ThreeDmPointCloud.py` SHA256 `daa37635ab67b2d2ff38fdea15e9e553d27054b776a8f9b71de7b7a38b48cce6`
- `ThreeDmTextDot.py` SHA256 `d345d8c5f4e58a7270e622f88fed28ce94f9856137d0c5b50c2b16575a544537`
- `ThreeDmNativeFields.py` SHA256 `b7000a7d672e2f99296b8480e1edc5eadea9984b8226dfde6b1204aea1f0b21d`
- `ThreeDmUpgrade.py` SHA256 `5fcf9e048f914175136dd489709e82e22adafc512d9369a6773aa701da3d25a6`
- `InitGui.py` SHA256 `2d920887a898a94f90eef120bdc14f989a96567eb8094e66f6b6cd022389f157`
- `Gui/ThreeDmArchive.h` SHA256 `471aa5a1aaf1c388d4552f68d502e3899dc9fd9dce8e149ccdc42fca3cf4b317`
- `Gui/ThreeDmHatch.h` SHA256 `48cde62e4a05a33b96d56b30afd99ffe869bc40a6536e0076802a5fdd01be41f`
- `Gui/ThreeDmHatch.cpp` SHA256 `2333c4678122e0e3cf6e2cbf696348ebf14807b5dc7e0b44430cb622ea3e1a5e`
- `Gui/ThreeDmInventory.cpp` SHA256 `8a65b54d186763159e095d7d28449c94256265eeda23cd94638cc0a603fc0354`
- `Gui/ThreeDmTransforms.cpp` SHA256 `d89a73ea66f1cd6d0d432b4f74877d94a80c94dbea76f1522787355978702e89`
- `Gui/ThreeDmMerge.cpp` SHA256 `0061a9bac9b55abbb31d536ffe815acb6a9e9b597c0a8a63734816a8c57db3d1`
- `cmake/OpenNURBS.cmake` SHA256 `e2d8acaa985303818501ed0bd0b367ab9c5e2fd20c7ec4d0bc40796902aa3410`
- `tests/native/CMakeLists.txt` SHA256 `3100e5a8a274d13f3989e901a4b455760e539af15f61c6ef40c72cc35043e268`
- `tests/native/three_dm_hatch.cpp` SHA256 `d99c9436656d8e3215b74d7208a10027722cf2ca03d92a88b8c7cf2c0023a2af`
- `tests/native/three_dm_hatch_affine.cpp` SHA256 `95a2247f13d79cffa849c84f74db5a76c6c562f503d3688d4f6000d0c1083b13`
- `tests/three_dm_hatch_smoke.FCMacro` SHA256 `3438b2983235467aacd90c14596d71fbb66ff37abf750a022fc61d53df06a8fc`
- `tests/three_dm_hatch_affine_smoke.FCMacro` SHA256 `ad464109fd652682bc7b60faa428a4f02df8c1e02f2584c602b95b5654dd2e8c`
- `docs/3dm-coverage.json` SHA256 `b0ac76309500fce503b837911ff351a2f276cb367e874801c274585c0f29ab35`
- `README.md` SHA256 `f153f95b85869cbae3e9dfced8f176c7aade0374ce3fd0f299290df1f88e46ec`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `a8b360f053963c660e21cfe2623a15f0afd2b5403de32038b3f05818142c049f`
- `docs/features/OM9-FILE-012.md` SHA256 `af383f4c9e41d451bd2ab6e09cddaf5d532601a02029c106f0be81f6753ed96b`
- `docs/features/3dm-support-matrix.md` SHA256 `272fb0d52d725020f23d293dd75d25cd8cd43a3866e0beb9105cf02e849bdf2e`
- `docs/development/3dm-structural-blocks.md` SHA256 `309a3522fc90be82cf93100a06981275fa8c1c689daef4a5fe2b08e341715f4e`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `ebd467300b8f99b87e534b84462216f3af47a155fef46517fd9f2e7f5e719fb1`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `c2fed9ebe22589de03021b2c17e5d21baf8c9ffedee9ee18c139c34f65e0c77b`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `4f405c12492ae598b8245b090a5f555460ae0e7e19e2891142a4aed83ac5d0bc`
- `docs/superpowers/plans/2026-10-07-3dm-hatch-native.md` SHA256 `18c9d7cb5e5e709c91b39b9f3e916714d63efa6b188ed4bad690a9230207f0bb`
- `docs/superpowers/plans/2026-10-07-3dm-hatch-affine.md` SHA256 `48334a228279bdd8e00e7fdc378370def7062a43b149ca9de935d7009e00a1f7`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `33bc696d98ea9852d6af8c032b91224d1029be82353f6eb7d82b02c0bf320767`
