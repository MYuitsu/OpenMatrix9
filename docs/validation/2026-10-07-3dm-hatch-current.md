# Native Hatch current fields and derived boundary display

## Verified implementation scope

Persistent schema1 Hatch fields expose origin, orthonormal X/Y axes, 2D base point,
rotation in degrees, positive double pattern scale and named pattern selection.
Normalized native plane16/base2/radians/scale/pattern UUID form the verified
baseline. Untouched fields retain original native doubles, including radians
without a degrees/radians roundoff rewrite. Edited axes produce a frame/equation
validated again by native openNURBS; no implicit normalization is accepted.
Baseline/schema/pattern-choice tampering and ambiguous adapters reject explicitly.

Complete native hatch_fields are applied to a clone after unit normalization and
before each object's own matrix. Source loops, rational weights/knots, attributes
and userdata remain native authority. Current reference resolves against the
actual native pattern table; selecting Grid60 materializes exact system UUID/line
fields. Dependency closure uses the chosen pattern; native payload/attributes and
semantic reread gates run before atomic replacement. Pattern-line content editing
is not included; pattern selection is a reference edit.

The pinned SDK SetPatternScale silently ignores values<=0.001. An exact helper
uses public ScalePattern with a unit XY axis, restores the native plane and checks
the requested double exactly. Native unit normalization and conformal transforms
now use this path. Positive scales that cannot be represented are rejected.

A bounded native boundary API verifies snapshot hash/source identity, normalizes
units, applies current fields/placement and derives OCC edges from native rational
LoopCurve3d curves. The host ViewProvider draws a discretized Coin outline; FCStd
rebuilds it from the embedded archive. This BRep/Coin outline is derived display,
never exported as replacement native geometry. Pattern fills/dashes and exact
Rhino appearance are not yet rendered. Coin float precision/0.05mm discretization
are display limitations; current/export doubles remain authoritative.

Current top-level, independent copy, promoted/copied family and reflected/shared
proxy routes pass in mm/cm. Field edits, physical parent and own placement,
Undo/Redo, source deletion and FCStd persistence pass. Copied families have
independent current fields; shared canonical edits propagate to their siblings
while each proxy's placement remains independent.

Gradient Hatch has no ordinary current-field adapter. A new native RED test showed
ordinary field application previously accepted it; explicit gates now retain its
source inventory with hatch_current_unavailable and reject this unverified Rhino5
export before destination replacement. Full gradient support remains required.

## Evidence

- Native current-field RED initially rejected an unknown overlay; new writer
  paths now pass complete plane/base/rotation/tiny scale/exact reference tests.
- Temporarily restoring the old conformal setter produced the expected tiny-scale
  failure; exactScale restored and tested green. SDK source was not changed.
- New GUI RED failed because the Hatch adapter did not exist. New48 current-field
  checks plus16 shared-current/gradient checks pass in the isolated runtime.
- Native16/16 final process0, including both immutable user ring fixtures and the
  independent 2013 Rhino5 PointCloud reader. Final GUI921/921 across43 suites,
  every final process0. Native fixtures were regenerated before final GUI run.
- All nine runtime scripts match dev source; Python/binary fingerprints remain
  stable throughout the final regression run. Rust unchanged at previous75/75/fmt
  evidence; this is not a fresh Rust test claim.

## Required remaining work

Editable native loop topology/control points, pattern-line editing and complete
pattern rendering remain required. Broader builtin/solid/non-circle/gradient/
legacy behavior, child-loop userdata/references, incomplete-document SDK reader
repair, extreme numeric ranges and large-data performance remain unverified.
Actual Rhino5 renderer/roundtrip acceptance (including the line-base contract
ambiguity documented in the preceding affine report) is still pending. Remaining
geometry/annotations/styles/resources/components/document/history/version
semantics, final comprehensive review and public integration are unfinished.
Full openNURBS remains in_progress; no class-wide completion is claimed.

Source build/om9-dev and runtime build/3dm-preservation-sdk are isolated from the
public checkout. No reusable skill/business rule was edited. Earlier dated pending
notes are superseded only within the tested scope of this report.

## Reproduction

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-hatch-current-tests.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd`

`rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/install-cloud-scripts.py`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd`

`rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-hatch-current-regressions.py`

|Suite|Checks|Result artifact|
|---|---|---|
|hatch_current|48/48|`build/three_dm_hatch_current_smoke-1/d3e912a5fe9a462bb7a57345033d19ce/results.json`|
|hatch_shared_current|16/16|`build/three_dm_hatch_shared_current_smoke-1/dfc1eec9231e4b4cb9d622ec350d9280/results.json`|
|hatch_affine|24/24|`build/three_dm_hatch_affine_smoke-1/74b0f839841f4145b9940ac5110f719d/results.json`|
|hatch|16/16|`build/three_dm_hatch_smoke-1/a29a6298032f42efac991b0643b75cce/results.json`|
|cloud_geometry|50/50|`build/three_dm_cloud_geometry_smoke-1/33b73065c9a1446485da4607e30acc4a/results.json`|
|point_cloud|43/43|`build/three_dm_point_cloud_smoke-1/558f7b6a883a4594a4b39e2c9ad0e5e6/results.json`|
|text_dot|34/34|`build/three_dm_text_dot_smoke-1/fa847dedcf29462db5fc453410907af6/results.json`|
|selected_member|33/33|`build/three_dm_selected_member_smoke-1/45d6d20b742043fab2ab8116620c25ab/results.json`|
|retained_transform|25/25|`build/three_dm_retained_transform_smoke-1/c32b829dd5984a8b9c071401330d80ac/results.json`|
|host_upgrade|39/39|`build/three_dm_host_upgrade_smoke-1/089f3dbe831b472b90a320ec6056e666/results.json`|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/528672f4ca854229b1a8c48fa974d8a3/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/f92269654450476bb894ac92e3bcf7d1/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/9cd63c3b31d947f39ad062ea53f01a69/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/0bea9f08fa08489ab3bf30be0d2da8d0/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/c09f083b40d042a1b36a15fd8ac23bb9/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/7eae0dec334742948e7726c650998448/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/e2d91a7136da4fd1806e479ab130526d/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/ba7e38b64eac4f0a9a7aefa69f7b51b6/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/51ca7b8a8b36499f91e8142a21c1e9bd/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/176c9ae767674e6291a9eb25153ef2ac/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/37d29aff612744818c320f9687c38079/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/720b37582d734cb29ce6f7a9d3846a3f/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/38111441d44f4811b2d7b1fa14ac3cf5/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/df8c51cc843442f1900c8e3f3d2d4370/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/2db4ddd864fa4eceba2d673d5a0db45e/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/870e5d98ff62407984fbfb5df964e022/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/712473acb7344a4d9bb6ca43b95e8abc/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/dc2eb2d6297f45a69f4317f3ee9929cf/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/4538c76f3687427c9112a7023597a69b/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/70002d3ccec74f22ba6d349b8d050c11/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/d6d1159f69c6408b9259a65f05511ba0/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/b75d45c7380f42e98306f0c869300606/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/3e0a217535eb482983ba899704e495e8/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/d8f4228f498a4416a8dbaa0dfd885516/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/3f5aa200a22c487a917a3e25849ca9c9/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/3a11b37f91264d14b974c0c849e51161/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/36d10b3f7a5b46958191d1d73dd4d230/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/60d27254b87f46a1b14a043f5ae01514/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/98ac31ff29b848a5ae61f3cf4ea6e8e5/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/f15945ad5eca4efb929e4f8b94613262/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/cbad20ad3a67449eb63e66f835fa466e/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/6a085024332f48c89af18dfba91bef97/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/9567e8c5f1e346298a7f7af6f1d4c6ee/results.json`|

Checkpoint: `H:\FreeCAD-src\build\checkpoints\hatch-current-20261007-060704.zip`; independent SHA256 verified.

## Current fingerprints

- `ThreeDm.py` SHA256 `449e11f46847c0573b947324925967a0d808c02aca10b75dcc754ee67eba6bba`
- `ThreeDmArchiveState.py` SHA256 `a1c4443a3d3537450336c910fbe1b8dbd95455c33ddb766640754fec355c7101`
- `ThreeDmMigration.py` SHA256 `146cae509edb79f790fe5b8f71a3a7aadde02853c440ce205d50fccec54f2622`
- `ThreeDmPointCloud.py` SHA256 `daa37635ab67b2d2ff38fdea15e9e553d27054b776a8f9b71de7b7a38b48cce6`
- `ThreeDmTextDot.py` SHA256 `d345d8c5f4e58a7270e622f88fed28ce94f9856137d0c5b50c2b16575a544537`
- `ThreeDmHatch.py` SHA256 `694668b9a07ce75643b2cfb3ff95a005b0a103185547aead0e8778f4aaa81513`
- `ThreeDmNativeFields.py` SHA256 `75df1f7720b1193b3582c9e5ee2180e9dfb2d0f537a80c29db2d01288b63a539`
- `ThreeDmUpgrade.py` SHA256 `5fcf9e048f914175136dd489709e82e22adafc512d9369a6773aa701da3d25a6`
- `InitGui.py` SHA256 `2d920887a898a94f90eef120bdc14f989a96567eb8094e66f6b6cd022389f157`
- `CMakeLists.txt` SHA256 `f258952f85dd717893942767f2f02ee24722d2363ba0a7e69ac221fe37650d09`
- `Gui/ThreeDmArchive.h` SHA256 `471aa5a1aaf1c388d4552f68d502e3899dc9fd9dce8e149ccdc42fca3cf4b317`
- `Gui/ThreeDmHatch.h` SHA256 `39ee73b23a256a9d67014169be48d87a92b50192f8e78204ebd4a094a34adfb3`
- `Gui/ThreeDmHatch.cpp` SHA256 `db2135fc7189a8ad71d3ceef92392dad308b4ff75dae2f1c45720aac9b764574`
- `Gui/ThreeDmInventory.cpp` SHA256 `c111dfeeb1f14a31261cc3bd2c85543d88bad428915748c68e3775ef29e6f36e`
- `Gui/ThreeDmTransforms.cpp` SHA256 `d89a73ea66f1cd6d0d432b4f74877d94a80c94dbea76f1522787355978702e89`
- `Gui/ThreeDmMerge.cpp` SHA256 `a5fedc418658ff304730e29f3132c96b4ca1a3ac8d42e14e50c012db7bdce510`
- `Gui/ThreeDmPython.cpp` SHA256 `390d812a1cbef431dcd095ea1d883d32d8c2708bc788c7a8eea363902e541209`
- `cmake/OpenNURBS.cmake` SHA256 `e2d8acaa985303818501ed0bd0b367ab9c5e2fd20c7ec4d0bc40796902aa3410`
- `tests/native/CMakeLists.txt` SHA256 `d43c5ef6485eff0d10dd4d9612fb05ad931545034fb70f2a18ab8b9742bd3e89`
- `tests/native/three_dm_hatch_current.cpp` SHA256 `a3c298c99256548f3744ed4b1c2e253f85027c3c292125952053b6d725c36393`
- `tests/three_dm_hatch_current_smoke.FCMacro` SHA256 `c07d245418ac7c00f1ddd35e60b6d86886333654d5f809fd40ec6ce8db7af123`
- `tests/three_dm_hatch_shared_current_smoke.FCMacro` SHA256 `89f323f660b3e8f91c64fa976df53146f4b7697df937904c2fffa015f87510e4`
- `docs/superpowers/plans/2026-10-07-3dm-hatch-legacy5.md` SHA256 `5be93a862b53f19e786569d76a2c9f3a301847a1def9a60d1c7c32a180e17f96`
- `docs/3dm-coverage.json` SHA256 `038d70af8901972d57eddc6811e2a3f25d042e4d33f99b764a03d15f80fad316`
- `README.md` SHA256 `6be222d1dee50bccfeabb1d1b559a600c4bb62f5da59ccad302b213190d4d192`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `3f1f52dcd8a10174f0306db4f2b60663e3e3d2e36cec2d0b513662d3d7f379d4`
- `docs/features/OM9-FILE-012.md` SHA256 `b50aff37ff11c282883cecf7425bc39c7bf5b11cff249a8fa1a92546df775d91`
- `docs/features/3dm-support-matrix.md` SHA256 `7eda7dd049f0e601129264b24bce932c908991b752443d6a8f7dc248fcbb57f6`
- `docs/development/3dm-structural-blocks.md` SHA256 `9688e9bb80bb8d60fbe80850fae3d897bd40b64a463479b4092f2074ff365022`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `96736227f70bb8144f88c476ed926beb0021bc268278e389b288ea806e7d3f71`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `c1b49d970ae7d6804a98f4d651006922e1f5c1bef11bd9ecd5f6c5aac88e514c`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `8a45c97a6ed8dc22075ad36bde0b6827b06e9fdd2523e616d3de692b71b480a4`
- `docs/superpowers/plans/2026-10-07-3dm-hatch-current.md` SHA256 `bfe711f1c709e6b84ee6565fb3538df64aaee8b37a5cf5694a688bee88a63185`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `9abe1fbaed13e033f965196594be568b14d121f655b3833901d4988915ea6ed0`
