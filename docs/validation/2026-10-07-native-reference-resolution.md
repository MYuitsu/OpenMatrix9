# OM9-FILE-012 — owning-model native reference graph

`Gui/ThreeDmNativeReferences.cpp` builds derived native geometry with explicit
source-model lifetime ownership. It resolves owner UUIDs/components/domain and
direction on detached copies, while source/native schema/model stays deferred.
The inventory stores per-root analysis and structured invalid-reference issues.
Known invalid references fail host import before mutation; unavailable class
adapters retain an explicit capability gap. No selected writer gate was removed.

## Verified scope

-4 real curve-owner paths: parameter2D/approximation3D, both directions, strict
  proxy13..21 and evaluation31..47 domains,5 independently mapped samples1e-12.
-4 real native box Brep edge/trim paths, both directions: actual edge/trim/face/
  surface pointers, EdgeParameter agreement and native C3 samples1e-12. The code
  avoids the pinned PolyEdge factories: the trim factory refuses m_ei>=0, and the
  edge factory accesses a null m_trim. It links using public native proxy APIs
  and validated public runtime ownership fields; no new SDK patch was added.
- All8 native linked records pass IsValid and WriteObject/ReadObject as version5;
  exact serialized reference/source fields and metadata survive. A bare Write
  omits root object userdata by design, so record roundtrip uses WriteObject.
  Source roots remain deferred/invalid and their source schema stays unchanged.
-8 invalid cases: missing UUID, point owner, proxy outside owner domain, component
  requested from a curve, self-cycle, wrong approximation dimension, out-of-range
  actual Brep edge index, out-of-range edge subdomain. A failed graph is not exposed.
-2 lifetime checks: curve and Brep trim targets survive release of inventory/model
  handles through graph ownership, evaluate consistently and release the source
  model when the graph is released.
-88 FreeCAD native-record/analysis checks: retained capability, no false editable
  Shape, source bytes in FCStd, external source deletion/reopen/recreated graph
  diagnostics, immutable fixtures, invalid import and pending export atomicity.
- Fresh native28/28 and GUI1687/1687 across55 suites; every process exits0. Native
  fixture generation completed before GUI. Both supplied rings pass. Runtime9
  scripts2 binaries2 separate reader executables stay stable; original modern
  and280 SDK2013 sources/pristine library and declared generated repairs guarded.

## Remaining requirements and limits

This is source-model native geometry analysis, not complete reference compatibility.
Current host fields/edit/display, unit/transform/UV and native class parameter maps,
reference retarget/remap/cloning/selected closure and export with edited owners,
multi-root sharing/performance and aggregate detached-clone budgets remain required.
The resolver uses per-tree field/depth/node limits, but its per-root graph copies
do not yet establish a global memory bound for many roots sharing a large owner.
It must address that before complete graph acceptance. Existing aggregate32MiB
manifest and512MiB source limits remain in force, not global process RSS limits.

Brep evidence is straight-edged box topology. General nonlinear/seam/singular trim
parameter relationships and all curve/surface/proxy/plugin classes need further
verification. The code currently checks Brep edge/trim domain bounds; complete
parametric consistency checks and tolerance-dependent trim/surface maps remain
required. Nested owned Rev/Sum/Extrusion traversal and recursive curve-owner paths
exist but are not broad acceptance evidence from these fixtures. Unknown/opaque
plugin semantics, history and all128 classes/16 component/6 document categories
remain in_progress. Independent SDK2013/new-schema writer and actual Rhino5
application acceptance are not established by the current modern roundtrip.

The export gate preserves the original destination while these requirements are
implemented. App/Rust were unchanged; prior results remain historical evidence.

## GUI evidence

|Suite|Checks|Artifact|
|---|---|---|
|native_reference_resolution|88/88|build\three_dm_native_reference_resolution_smoke-1\b86eef6960c34e9389a42b50198f888a|
|curve_on_surface_schema|120/120|build\three_dm_curve_on_surface_schema_smoke-1\9fc62cc637c14781b368faf026402135|
|curve_on_surface|32/32|build\three_dm_curve_on_surface_smoke-1\193799c90527457b9ed58b27b1361f4d|
|hatch_loop_rows|103/103|build\three_dm_hatch_loop_rows_smoke-1\fba2061cbb364ca2adfd468ada7bef28|
|hatch_loop_ui|104/104|build\three_dm_hatch_loop_ui_smoke-1\dddda8522d8742778b786c39f6e0d7f1|
|hatch_legacy_loops|84/84|build\three_dm_hatch_legacy_loops_smoke-1\975cf63cd3f04b418e0cdd1ee1d552ce|
|hatch_loop_baseline|4/4|build\three_dm_hatch_loop_baseline_smoke-1\12f4d8c406024572b798a3d168d4c7a0|
|included_file_copy|8/8|build\three_dm_included_file_copy_smoke-1\d613332355a841dc949d0cfe2e5e7a80|
|hatch_loop_edit|132/132|build\three_dm_hatch_loop_edit_smoke-1\68a8d063f3b843fb840f0fe3e891afbc|
|hatch_shared_loop_edit|12/12|build\three_dm_hatch_shared_loop_edit_smoke-1\7d98f58c46a946ee9fb29357b566d009|
|hatch_loops|54/54|build\three_dm_hatch_loops_smoke-1\70e6d6aa2e2c4809a7119a19c14c9305|
|hatch_legacy|24/24|build\three_dm_hatch_legacy_smoke-1\4c8060ea81c041a9bbb88f1eb7d32ed1|
|hatch_current|48/48|build\three_dm_hatch_current_smoke-1\e20684f88ea041ffaed3219f56822a69|
|hatch_shared_current|16/16|build\three_dm_hatch_shared_current_smoke-1\11af6ff459a944e1b100539db03b0470|
|hatch_affine|24/24|build\three_dm_hatch_affine_smoke-1\47c96e39faac4259b8dbe80a9201e9f5|
|hatch|16/16|build\three_dm_hatch_smoke-1\4013ccdcd36042d8a08895c392a1484b|
|cloud_geometry|50/50|build\three_dm_cloud_geometry_smoke-1\61e068e81e3c44ef95123b7821f14f01|
|point_cloud|43/43|build\three_dm_point_cloud_smoke-1\c22d6a527f06410cb855095292e7709e|
|text_dot|34/34|build\three_dm_text_dot_smoke-1\f215b8e38e43424db6a2b1c02c9f17d8|
|selected_member|33/33|build\three_dm_selected_member_smoke-1\9f4abe8603464d4eb1ead61e1e414ad9|
|retained_transform|25/25|build\three_dm_retained_transform_smoke-1\49dc7bebd658433693f2288fa87decfb|
|host_upgrade|39/39|build\three_dm_host_upgrade_smoke-1\07de7623ce0e45a3b07de3185fba5025|
|shared_export_routes|30/30|build\three_dm_shared_export_routes_smoke-1\a71bda851511421a9237bf6a5e1bdf13|
|recursive_shared_migration|24/24|build\three_dm_recursive_shared_migration_smoke-1\3d05533b873344328a74c2d457d0c7db|
|recursive_namespace_migration|25/25|build\three_dm_recursive_namespace_migration_smoke-1\2ed63abdac734cc9967d1a300b1695eb|
|affine_target_migration|32/32|build\three_dm_affine_target_migration_smoke-1\adc02fa38a194458aca7dc2fc9c3659f|
|copy_origin_migration|22/22|build\three_dm_copy_origin_migration_smoke-1\de10b60c1aa54a508019c2a25a12feb8|
|legacy_migration|35/35|build\three_dm_legacy_migration_smoke-1\19ac0f72e53b46e5bfed7264cee2944f|
|proxy_retarget|6/6|build\three_dm_proxy_retarget_smoke-1\2144aa00162348cf90261030603b3aca|
|source_proxy_creation|34/34|build\three_dm_source_proxy_creation_smoke-1\f34431eb7fd64ddd90591e81b133f64e|
|definition_copy|27/27|build\three_dm_definition_copy_smoke-1\f297f7b096ba4fbaa4f63a12f307b95e|
|source_members|23/23|build\three_dm_source_members_smoke-1\b91f01b0eb644ee0b22495330bc15461|
|source_member_geometry|8/8|build\three_dm_source_member_geometry_smoke-1\4130ce8e7ac347e8ab7ae499d49f283b|
|source_new_target|27/27|build\three_dm_source_new_target_smoke-1\fe4f41f9bf934a0781347bed9a13924c|
|new_blocks|29/29|build\three_dm_new_blocks_smoke-1\ae0efba471434eb8a3f3edfba2492d1a|
|new_member_instance|20/20|build\three_dm_new_member_instance_smoke-1\f6065aacb30e4669bff4585524915d58|
|reference_edit|18/18|build\three_dm_reference_edit_smoke-1\de9637e09c9b4231ae284e6aa69b6a86|
|independent_member|18/18|build\three_dm_independent_member_smoke-1\2fa09cee0a744dfcb58894b4d83fbcc9|
|shared_proxy|10/10|build\three_dm_shared_proxy_smoke-1\7d94883c441746678e7fb238be3d806a|
|shared_proxy_geometry|8/8|build\three_dm_shared_proxy_geometry_smoke-1\f4e81bffb3d84968904cdb499e70f677|
|retained_member|6/6|build\three_dm_retained_member_smoke-1\ae2b1bce31d34fff835167472416a65c|
|definition_placement|12/12|build\three_dm_definition_placement_smoke-1\a41d6a26492d4678bca33175e4b49470|
|new_geometry|20/20|build\three_dm_new_geometry_smoke-1\9c8f813d8c774d14a1f5447bbd81703a|
|preserved_export|13/13|build\three_dm_preserved_export_smoke-1\aad4f8713bb74edea325cbeb651f42a5|
|member_overlay|12/12|build\three_dm_member_overlay_smoke-1\70db0beda5bf48ddad733658e38d55ac|
|block_structure|23/23|build\three_dm_block_structure_smoke-1\f00df48c969647fd968c1a5a0d83df79|
|affine_refresh|10/10|build\three_dm_affine_refresh_smoke-1\cd8db45fca504c5ea247a740b3a3f9b7|
|affine_refresh_reopen|2/2|build\three_dm_affine_refresh_reopen_smoke-1\9f56f354cb2c4b8a87e085a4986ba1e7|
|source_signature|14/14|build\three_dm_source_signature_smoke-1\acbd02b857ec42adab423ce7405ac2ef|
|preservation|19/19|build\three_dm_preservation_smoke-1\4592b9b97d6f4b5e88c51e863acc1da6|
|definition_overlay|8/8|build\three_dm_definition_overlay_smoke-1\7601784e1fca463ba287ba33a9b04597|
|layer_overlay|7/7|build\three_dm_layer_overlay_smoke-1\0e4329a746dd481f8bef6e53457ec242|
|mixed_shear|11/11|build\three_dm_mixed_shear_smoke-1\332be1ac0bc24e3a9401969fc4f5ed2b|
|merge_bridge|8/8|build\three_dm_merge_bridge_smoke-1\1d91e1d6ebbf46d2b84f8d62b9369b1b|
|geometry|33/33|build\three_dm_smoke-1\8d76887373ca4ce8ba8242a2889e2ba0|

SHA-verified source/evidence checkpoint: `H:\FreeCAD-src\build\checkpoints\native-reference-resolution-20261007-110112.zip`.
