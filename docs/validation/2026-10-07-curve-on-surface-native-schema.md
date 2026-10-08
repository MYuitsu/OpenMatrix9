# OM9-FILE-012 — CurveOnSurface source native schema

Read-only source-unit schema now exposes the native owned parameter/optional
approximation/surface tree, raw fields and stable metadata. Source records retain
this schema through FreeCAD save/reopen and external-source deletion. Capability
stays retained; no composed CAD shape/current editor or selected writer completion.

## Verified behavior

-10 exact direct/model field paths: rational homogeneous NURBS CVs/orders/knots,
  Plane frame/extents separate from domains, Rev axis/generatrix/angle parameters,
  Sum native children/base point, Extrusion native profile/path/fractions/up/miter/
  cap/transposed fields. Arc parameter/optional child fields and userdata matrix.
-8 native deferred PolyEdge paths: parameter/approximation, present/missing
  source object, both reversal states. Exact class, UUID, component index, edge/
  trim/proxy/evaluation domains, user strings and source dependencies preserved.
  A matching UUID means dependency presence only; resolution remains deferred.
-7 explicit refusals: missing curve/surface, nonfinite radius, active cycle,
  depth65, actual34MiB metadata and >2million raw CV numeric fields. Limits are
  per tree; metadata uses a conservative4 bytes per source wchar before UTF8
  conversion. Node cap16384 and final aggregate32MiB manifest cap are also coded;
  these are not global process memory bounds or exhaustive parser/fuzz coverage.
-120 FreeCAD checks: source record and retained capability, no false Shape,
  pending export atomic refusal, FCStd reopen, exact embedded archive after
  deleting external file, stable schema/dependencies/original fixture bytes;
  missing dependency fails before host mutation. Every GUI process exits0.
- Native27/27 and GUI1599/1599 across54 suites. Both original supplied rings pass
  native regressions. Native fixture generation finished before GUI execution.
  Source/install scripts, runtime binaries and separate reader executables remain
  stable during all GUI runs. App/Rust were not changed or freshly rerun.

## SDK findings and declared repairs

The modern SDK PolyEdgeSegment::Read called Reverse() before assigning its
serialized domain. Reverse() only toggles the flag on an increasing domain; the
fresh segment has an empty domain, so a true serialized flag was lost. A genuine
RED compared the independently generated expected fields against model inventory.
`cmake/PolyEdgeArchive.cmake` verifies the exact pinned normalized SHA, generates
a copy changing only this Read restoration to the protected native flag setter,
and replaces exactly one compiled source entry. Original SDK source stays intact.
The generated repair applies to the current modern native/module libraries only;
the separately repaired SDK2013 reader still has its declared CurveOnSurface
Read-only repair. It does not independently validate the new PolyEdge reversal
or all surface-schema representations. Pristine diagnostic libraries stay intact.

An attempted deferred CurveOnSurface reencode also exposed the native Write
gate: it calls IsValid(), which requires linked child geometry. No writer bypass
was added. Eight native payload checks verify refusal before child output; the
existing application selected-export gate preserves the destination. Full native
reference resolution/remapping must precede enabling that writer.

The reference fixture transfers exact owned geometry to ONX_Model. Native
DuplicateCurve() is allowed to convert PolyEdge proxies, so a DuplicateCurve fixture
cannot establish preservation of their native serialized reference identities.
Runtime caches/pointers/copy counters are deliberately outside the stable schema.
Opaque plugin userdata is identified with a retained-payload flag, not decoded.

## Remaining full scope

Complete owning-model reference/component validation and remapping, unknown/
proxy/plugin child adapters, source/current field and unit mappings, composed
display/edit/Undo/Redo/copy/block lifecycle, selected/native-block export, all
native parameter/type mappings, independent target readers and actual Rhino5
application acceptance remain required. Dormant extrusion fields and unsupported
native subclasses have not received exhaustive version/roundtrip proof. The
full128 classes/16 component/6 document categories and all Hatch/pattern/render/
resources/history/version requirements remain in_progress. This is scoped
inventory/persistence evidence, not full openNURBS completion.

## GUI evidence

|Suite|Checks|Artifact|
|---|---|---|
|curve_on_surface_schema|120/120|build\three_dm_curve_on_surface_schema_smoke-1\43fda19e23194636ac079d4969fbddb5|
|curve_on_surface|32/32|build\three_dm_curve_on_surface_smoke-1\98d266ccc7db450cbca3ea3c29feb044|
|hatch_loop_rows|103/103|build\three_dm_hatch_loop_rows_smoke-1\f885f383308a411d8f660a85fc35800f|
|hatch_loop_ui|104/104|build\three_dm_hatch_loop_ui_smoke-1\948e15e0325146eb9d711466b234515b|
|hatch_legacy_loops|84/84|build\three_dm_hatch_legacy_loops_smoke-1\b31ebb6455c848c69d6874fd580fbca7|
|hatch_loop_baseline|4/4|build\three_dm_hatch_loop_baseline_smoke-1\520e6e4c009f43aaba5a949e367ded82|
|included_file_copy|8/8|build\three_dm_included_file_copy_smoke-1\068c13b2be734165a3bcd3a243e88657|
|hatch_loop_edit|132/132|build\three_dm_hatch_loop_edit_smoke-1\cf0a9b5800854b109c22df8f163772b9|
|hatch_shared_loop_edit|12/12|build\three_dm_hatch_shared_loop_edit_smoke-1\2cad91c9dbf84adf8ed8386db5057036|
|hatch_loops|54/54|build\three_dm_hatch_loops_smoke-1\4393d6850105461f874039d1503d943a|
|hatch_legacy|24/24|build\three_dm_hatch_legacy_smoke-1\29fa2de835fe4510b02f03cf7280a43b|
|hatch_current|48/48|build\three_dm_hatch_current_smoke-1\72a96321970044db9bd9419f7ab52a90|
|hatch_shared_current|16/16|build\three_dm_hatch_shared_current_smoke-1\daa968de161a446db96ad42f833f06ac|
|hatch_affine|24/24|build\three_dm_hatch_affine_smoke-1\8deb2977060245e081abdbb70e933c61|
|hatch|16/16|build\three_dm_hatch_smoke-1\d9cd534ac73b4bf2aaaf5944482027d5|
|cloud_geometry|50/50|build\three_dm_cloud_geometry_smoke-1\c0ef3b7cbb52490f846ba3d5d2c7197c|
|point_cloud|43/43|build\three_dm_point_cloud_smoke-1\cdf160b63233434689bce56b6d59be41|
|text_dot|34/34|build\three_dm_text_dot_smoke-1\aca31eac5c8348f39a723f94a92e3dfc|
|selected_member|33/33|build\three_dm_selected_member_smoke-1\0c2f769e50d84c099fb346c9d50dd442|
|retained_transform|25/25|build\three_dm_retained_transform_smoke-1\530b0b62c06d46f9880cad22ddcb6eda|
|host_upgrade|39/39|build\three_dm_host_upgrade_smoke-1\a4a0ca391e6d4ac9bf676a38e5cca864|
|shared_export_routes|30/30|build\three_dm_shared_export_routes_smoke-1\39f358d2551c44c1b78c5fc6e927586f|
|recursive_shared_migration|24/24|build\three_dm_recursive_shared_migration_smoke-1\92695d4d54ba45eeb15dfa4b33fe4796|
|recursive_namespace_migration|25/25|build\three_dm_recursive_namespace_migration_smoke-1\51aeb445c86345ab8c891744f75cb170|
|affine_target_migration|32/32|build\three_dm_affine_target_migration_smoke-1\554f4962db1245da8b40c2d9c25e126d|
|copy_origin_migration|22/22|build\three_dm_copy_origin_migration_smoke-1\a16fb36c4685424db36ec305bced3490|
|legacy_migration|35/35|build\three_dm_legacy_migration_smoke-1\41994cd203ee40f18ccfb88c420e3f9e|
|proxy_retarget|6/6|build\three_dm_proxy_retarget_smoke-1\92bfe46e8c4a4ab085a74c48d69ffd56|
|source_proxy_creation|34/34|build\three_dm_source_proxy_creation_smoke-1\c7e2bb37d02d44a684e5d651b834a4c9|
|definition_copy|27/27|build\three_dm_definition_copy_smoke-1\3181c06493d84e4c98e74a0be2c16bbc|
|source_members|23/23|build\three_dm_source_members_smoke-1\a2ca106a8d154a38bfe1634532fc65f3|
|source_member_geometry|8/8|build\three_dm_source_member_geometry_smoke-1\326ba9713092479180e9475a1916e4eb|
|source_new_target|27/27|build\three_dm_source_new_target_smoke-1\f536794bbc1a4b23a1fd63c0849f7294|
|new_blocks|29/29|build\three_dm_new_blocks_smoke-1\2d4f15627fe645d3b1cbe8c750bb2630|
|new_member_instance|20/20|build\three_dm_new_member_instance_smoke-1\c0de81232fe14f0fafbf4b00f0b035c4|
|reference_edit|18/18|build\three_dm_reference_edit_smoke-1\46bfce178185433e974ea14b2d135ca7|
|independent_member|18/18|build\three_dm_independent_member_smoke-1\86161997a0f54b86b162bd254c64ac90|
|shared_proxy|10/10|build\three_dm_shared_proxy_smoke-1\6a5d90e0359643beb07cc8110f34ef3e|
|shared_proxy_geometry|8/8|build\three_dm_shared_proxy_geometry_smoke-1\5087e37699934be1b0e429869034f9de|
|retained_member|6/6|build\three_dm_retained_member_smoke-1\9f12ba9227a548869465b29c0b8271fc|
|definition_placement|12/12|build\three_dm_definition_placement_smoke-1\168c1d7897e247a78631297c2731af9c|
|new_geometry|20/20|build\three_dm_new_geometry_smoke-1\446805a87bc043b08bb432dbcaa01017|
|preserved_export|13/13|build\three_dm_preserved_export_smoke-1\2e47945c707840cb9edc5eebedba0e77|
|member_overlay|12/12|build\three_dm_member_overlay_smoke-1\45306041aa714023ba8aef40d3997472|
|block_structure|23/23|build\three_dm_block_structure_smoke-1\b743235a9f584a2e85c3eb8686724ecd|
|affine_refresh|10/10|build\three_dm_affine_refresh_smoke-1\1d90c3c778004cbb84ad8f4f82d85627|
|affine_refresh_reopen|2/2|build\three_dm_affine_refresh_reopen_smoke-1\f2e6fdd5b0cf42858ce53e36a2e3684d|
|source_signature|14/14|build\three_dm_source_signature_smoke-1\2440a13ce2c2471586b9571ef714623f|
|preservation|19/19|build\three_dm_preservation_smoke-1\9766f389ea4948b9b1082b8e2d6451b9|
|definition_overlay|8/8|build\three_dm_definition_overlay_smoke-1\411dbed9abc9456bba313cd8c3ad983d|
|layer_overlay|7/7|build\three_dm_layer_overlay_smoke-1\d0bbfd31ea464504a96dcd1f63b2e46a|
|mixed_shear|11/11|build\three_dm_mixed_shear_smoke-1\20c9ee4c3e154ac58bced3089c75c5a2|
|merge_bridge|8/8|build\three_dm_merge_bridge_smoke-1\8019587b74764ae19e1a971e8621bad7|
|geometry|33/33|build\three_dm_smoke-1\35f5b6a426a349c19dcc8181eb7f0e01|

SHA-verified source/evidence checkpoint: `H:\FreeCAD-src\build\checkpoints\curve-on-surface-native-schema-20261007-102326.zip`.
