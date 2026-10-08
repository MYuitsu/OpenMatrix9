# Independent legacy5 Hatch semantic acceptance

## Verified scope

A separate executable links the unchanged official McNeel SDK20130711. Its
ON::Version is201307115, and a compile assertion proves its ON_Hatch has no modern
gradient API. It decodes native version5/50 data and writes a fresh version5 file.
The modern reader then validates that independent decode/reencode result.

Native20 mm/cm cases compare exact plane16/base2/radians/tiny positive scale,
pattern UUID and every line angle/base/offset/signed dash, loop type/native class,
dimension/order/rational flag/homogeneous CVs/weights/knots, user strings and block
definition/member/reference matrices. Cases include custom multi-line patterns,
selected normalization, Solid/Grid60/HatchDash selection, reflected/sheared
current fields, independent/canonical copied blocks and two genuine definitions
sharing a canonical member. Source snapshots stay immutable. This is semantic
library interoperability evidence, not actual Rhino5 rendering acceptance.

The old SDK uses typed ON_HatchExtra userdata for movable base points. Modern
readers consume that specific extension into the current field. The oracle checks
the exact decoded base point and requires the expected extension exactly once;
only this named representation is normalized for comparison. Unrelated userdata
identities and native user/attribute/loop/pattern strings remain checked.

An initially incorrect probe used plane_equation[3], inherited from a3-vector in
SDK2013; it returned z instead of d. The independent reader now reads explicit
x/y/z/d. The fixture also explicitly declares line fill and verifies serialized
line arrays, avoiding a default-solid false claim. Invalid copy-overlay requests
were corrected to the actual canonical/independent policies before acceptance.

## Observed SDK2013 ownership requirement

A raw20-case investigation found stale attribute userdata owner back-pointers
after ONX_Model's object table grew. SDK2013 WriteObjectUserData skips data whose
Owner differs from its owning attributes. Both raw mm/cm probes reproduce lost
attribute strings while other Hatch fields stay exact. No comparison was removed.

The test reader detects this state, calls the documented public MemoryRelocate()
on the affected attributes and verifies all back-pointers. In both five-object
fixtures it repairs four attribute owners; complete strings then survive reread.
The raw mode and repaired reports are saved separately. SDK source stays unchanged.
This integration repair must not be confused with unassisted SDK2013 model.Write
or actual Rhino application behavior.

## Gradient target-version proof

Modern version80 fixtures preserve enabled linear gradient type, two colors and
repeat2.5. Deliberate version5 serialization loses the gradient type/colors/repeat
before the legacy decoder sees it. The pinned source ON_GradientColorData's
WriteToArchive requires archive>=60. The2013 native Hatch has no gradient API.
The production preservation writer refuses selected gradients atomically and
keeps the source snapshot. Current-field/transform diagnostics now identify the
Rhino5 format limit instead of saying compatibility is merely unverified.

This does not implement gradient display/editing or a deliberate approximation.
Inactive/other gradient cases, broader userdata and complete class semantics
remain open; unsafe attached userdata is still rejected by the general gate.

## Runtime evidence and remaining scope

New24 GUI checks run the independent legacy decoder on real current reflected
shared proxies, siblings, promoted and independent copied families, in mm/cm.
Direct decoded fields/patterns and complete native loop facts/UUID graph agree;
Unicode names and native/source input bytes survive. Final FreeCAD945/945 across44
suites and native17/17 pass with final process0, including both user rings and the
independent PointCloud reader. All nine runtime scripts and binary fingerprints
remain stable throughout the final GUI run. Rust unchanged at prior75/75/fmt.

Actual Rhino5 renderer/roundtrip acceptance, the nonzero line-base/Plus rendering
contract, zero-base/broader builtin/non-circle/legacy cases, native loop/pattern
editing and full fill/dash rendering remain required. Child-loop userdata/reference
and incomplete-document repair audits, extreme ranges/performance and all other
geometry/style/resource/component/document/history/version categories remain
unfinished. Full openNURBS remains in_progress, with no class-wide completion.

Source build/om9-dev and runtime build/3dm-preservation-sdk are isolated; public
integration and final comprehensive review remain pending. No reusable skill
business rule was edited. Earlier dated pending legacy-reader notes are superseded
only in the named scope. Official SDK archive SHA256:
5cb9ff879c94c63145526a188f9ddc8c522f35cd82d8726ad49ed64a63d6e05a.
Source: https://files.mcneel.com/opennurbs/5.0/2013-07-11/opennurbs_20130711.zip

## Reproduction

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-rhino5-reader.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-legacy-hatch-tests.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd`

`rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-hatch-legacy-regressions.py`

|Suite|Checks|Result artifact|
|---|---|---|
|hatch_legacy|24/24|`build/three_dm_hatch_legacy_smoke-1/9cbaad2ca0d642118f83bcbd18d9fb7d/results.json`|
|hatch_current|48/48|`build/three_dm_hatch_current_smoke-1/2ad7e81278a545fb868646cdb76f7944/results.json`|
|hatch_shared_current|16/16|`build/three_dm_hatch_shared_current_smoke-1/1b7ff0140d9f4c0f8d8d023e57753a78/results.json`|
|hatch_affine|24/24|`build/three_dm_hatch_affine_smoke-1/a0cf4a1d04af42c9a1301f9c0336ee26/results.json`|
|hatch|16/16|`build/three_dm_hatch_smoke-1/e9429c4bc90047bc9c3e8a42913c5e3d/results.json`|
|cloud_geometry|50/50|`build/three_dm_cloud_geometry_smoke-1/33a8e7ede9ca4f25a2816eb4213bf0c6/results.json`|
|point_cloud|43/43|`build/three_dm_point_cloud_smoke-1/b1e919e073a845b385caf41df45b3abe/results.json`|
|text_dot|34/34|`build/three_dm_text_dot_smoke-1/65f9aa8913004abe8f63c9de9b1ecfb0/results.json`|
|selected_member|33/33|`build/three_dm_selected_member_smoke-1/f97fdf6d7cbf40beae2252cc1be8b640/results.json`|
|retained_transform|25/25|`build/three_dm_retained_transform_smoke-1/0565b11c3c0b4f5aa1c7a2185a4bff95/results.json`|
|host_upgrade|39/39|`build/three_dm_host_upgrade_smoke-1/defc164356764be3b12759e607eb2e6b/results.json`|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/7ab5c5426db64393b4159e6fdb88064f/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/5e29caa3f3b84edc9aac2702baaa017f/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/5a7669b30e154c16bc0b8afbc001cf97/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/6a8c34ebf9194b48abcc9a7f02dbec11/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/e19cbb8cf322402389d259cf0fde7f35/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/33f3f7824c4e44258bbde289aca2a167/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/e394565bc32e475b80340d9a2e103798/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/146ae0ab6b364f7f999fd2b07b05655d/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/733a895de7884a69a80f9cfee7b86df6/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/49e84461e01141daaf48f19d24d68b74/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/18588c5d40544aafbac7dc7fcc000ad4/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/617d70a052c347f5b16b0507b5d7d92b/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/e51b37edbda241f0acb90e8c962d4ccc/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/b68f027830d84d928e2126a0eabc667b/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/bb6453366cef43d3aea6e8b683ddae42/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/daf3ff710c92441898ebe04a4c27a6d4/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/253ad67c9fe447d98a28336449394820/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/a3b3cb31e3204230b26e3df22302dbe4/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/f042353a79704776b65a8c832e60a8a8/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/d26570927caf48249ba37863f8bd6344/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/4975921f67ba420e8e26b88c7b37e1e1/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/2d14c5c2f64a4b8686ca1afd36b63ecd/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/59ed5ce140ba46c18299763b3d543730/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/45a85f3ad94e4c4786c4d86d96ed6995/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/2603b7f15d4b4b59820b188f067b96c2/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/9df58ebb789b4b90be1c05cf43f6ad38/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/e7bda306d78545fbb64a1f71dce1a870/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/527b7df46e8c43088a7bfe9c8233ef10/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/096494e5de4646ec823ba200b5b5ed24/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/4017fc7b2f5a48b0a3016345b65210f5/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/a1bcbaea37b54d02bb1a3c8c99412b6e/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/8e6a0ef632e74d258798e582dfeedf6f/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/ac0ca613dfb24f7bbb6836960cb22554/results.json`|

Checkpoint: `H:\FreeCAD-src\build\checkpoints\hatch-legacy-20261007-062550.zip`; independent SHA256 verified.

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
- `Gui/ThreeDmHatch.cpp` SHA256 `c767bf66ae9ede9d3e0075f7cf29871cea32afdd9089c5a08ab4a0ce57a89198`
- `Gui/ThreeDmInventory.cpp` SHA256 `c111dfeeb1f14a31261cc3bd2c85543d88bad428915748c68e3775ef29e6f36e`
- `Gui/ThreeDmTransforms.cpp` SHA256 `d89a73ea66f1cd6d0d432b4f74877d94a80c94dbea76f1522787355978702e89`
- `Gui/ThreeDmMerge.cpp` SHA256 `a5fedc418658ff304730e29f3132c96b4ca1a3ac8d42e14e50c012db7bdce510`
- `Gui/ThreeDmPython.cpp` SHA256 `390d812a1cbef431dcd095ea1d883d32d8c2708bc788c7a8eea363902e541209`
- `cmake/OpenNURBS.cmake` SHA256 `e2d8acaa985303818501ed0bd0b367ab9c5e2fd20c7ec4d0bc40796902aa3410`
- `tests/native/CMakeLists.txt` SHA256 `e768f36c394063f007096b25c389287b1b2d6334394f708f08a1aaf57353a8dc`
- `tests/native/three_dm_legacy_hatch.cpp` SHA256 `6db7e7e94fe93595f0005106bb15cc48dc2258ee5496d8d71027699b9bf0d958`
- `tests/native/rhino5_reader/CMakeLists.txt` SHA256 `0aa89c57928387bc0a97abee728c9f8f9f4043d3af575ed4066cda23c4c7f257`
- `tests/native/rhino5_reader/hatch_reader.cpp` SHA256 `93851ed390ef96e4ef204ecc236bdb55dc1bb39417b8bc7b527fe91c934d25ad`
- `tests/three_dm_hatch_legacy_smoke.FCMacro` SHA256 `05e87454fcdcf56b850cde574fc372e6657a57cdb9ee1213d17dd4d16926d22a`
- `tests/native/three_dm_hatch_current.cpp` SHA256 `a3c298c99256548f3744ed4b1c2e253f85027c3c292125952053b6d725c36393`
- `tests/three_dm_hatch_current_smoke.FCMacro` SHA256 `c07d245418ac7c00f1ddd35e60b6d86886333654d5f809fd40ec6ce8db7af123`
- `tests/three_dm_hatch_shared_current_smoke.FCMacro` SHA256 `89f323f660b3e8f91c64fa976df53146f4b7697df937904c2fffa015f87510e4`
- `docs/superpowers/plans/2026-10-07-3dm-hatch-legacy5.md` SHA256 `57d1d5fe464b36f154cdc28c0658afcd3b27138c9a2dcc8be212098a7d08a650`
- `docs/superpowers/plans/2026-10-07-3dm-hatch-loops.md` SHA256 `c7651cd1f68260433dfeca3c626fd652b67f11744d423d6139d6e2efd8de4ca2`
- `docs/3dm-coverage.json` SHA256 `a883f2002bbd5a418aa57ab1835b3047b695e9b0f5fc7ca5f870b92031bb5498`
- `README.md` SHA256 `2a37ac68799a6b97192423dd01bd481259acba82182d50d11e4f7ca1e3bc6bee`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `30566301a0e1a02e5e090748185c4f0cc47c3d809fc3c919f2924bccbd53b50f`
- `docs/features/OM9-FILE-012.md` SHA256 `3a07732a700244e857e1578134359e6728baec06a04cc6a7050c8d461ae2a1dc`
- `docs/features/3dm-support-matrix.md` SHA256 `26a460c2cd814926c90e8fd7a2a5c41ed239795f6ebef469ea7a3bd04a75dada`
- `docs/development/3dm-structural-blocks.md` SHA256 `679a8dfdf2b32e03da3ffeb4752c34b2aae37c27743f4c484780b897dfbe28ea`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `da36e7fbae19d402dc65ff206b70bfd19a037f45b01c5b13ccb823b0c349987d`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `0eb29360280c8d81e784a007fa47b310c27b2cf57bd788caad8500e4a07c0c24`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `ccbffa38a49ac8e3ff5880fd5d2293d21b2f426eeb447aeca827387cd7a5b26e`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `c867857243b20ae18a2dcc0e920e52f94136caecd1a02fae52902a8495980f59`
- Independent `Rhino5HatchReader.exe` SHA256 `7386a5f2d6f12fa3c3db260e73d8a15ff2892f88d9fd1ac809ab4c6f6c5fbf3a`
