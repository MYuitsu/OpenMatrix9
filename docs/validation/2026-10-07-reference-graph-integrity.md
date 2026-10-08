# OM9-FILE-012 — shared native reference graph integrity

The detached source-model resolver now shares validated native owner copies across
root graphs, retains only each root's required transitive owner closure, and
charges aggregate clone work before allocation. Source schema/model stay immutable
and deferred. Linked analysis does not grant host editing or selected export.

## Fresh verified scope

- Three CurveOnSurface roots referencing one line share the exact proxy target
  pointer, each with two owned entries. One surviving graph retains target lifetime
  after sibling/inventory release. A fourth root referencing the first root reuses
  that native CurveOnSurface, retains exactly three closure entries and evaluates
  after all sibling handles are released. Unique clones are4 and5 respectively.
- Six quota cases: one-byte budget refuses with status limit; exact charged bytes
  permits four unique clones/two cache hits; one byte short refuses the third root;
  three-owner budget admits two roots; zero node and depth quotas refuse traversal
  and cloning respectively. Attempts later discarded stay charged across roots.
- Four real box edge cases use edge domain11..23, strict edge subdomain15..21 and
  evaluation domain31..47. Consistent proxy intervals match RealCurveParameter at
  both endpoints; both reversal paths agree with EdgeParameter at five native
  samples1e-12. Inconsistent but in-bounds intervals are invalid, with no graph
  exposed; FreeCAD import refuses before document mutation.
-38 new FreeCAD checks: exact source records, retained capability/no false Shape,
  embedded source after deletion/FCStd reopen, reconstructed graph diagnostics,
  pending export atomicity, fixture immutability and two invalid import paths.
- Fresh complete native29/29,47.82s; GUI1725/1725 across56 suites, every process0.
  Both supplied ring fixtures pass. Native fixture generation finished before GUI.
  Nine installed scripts, two runtime binaries and two separate reader executables
  remain stable; original modern/280 SDK2013 files and pristine library are guarded.
  Existing archive/transform/schema/reference tests remain passing. No SDK repair
  was added in this change. App/Rust results remain historical, not rerun here.

## Regression evidence

Four native failures were captured before their fixes: duplicate shared target,
ancestor/sibling leakage in owner closure, no aggregate byte refusal, inconsistent
edge/proxy parameter mapping. The new host macro failed against the old module
on the first inconsistent edge case after32 passing checks; all38 pass with the
rebuilt module. Evidence checkpoint includes the native logs and host RED report.

## Limits and remaining work

Clone accounting uses max(ON_Geometry::SizeOf(),sizeof(ON_Geometry)), with512MiB,
16384 clone attempts,16384 visited nodes and depth64 ceilings per request. It bounds
SDK-accounted cloning/work and does not establish actual heap/RSS usage; closure
maps, diagnostics, SDK caches and allocator overhead are not included in that byte
measure. Brep targets are borrowed immutably through source-model lifetime rather
than cloned. Caching is local to one resolution request. Failure/cache/reordered
graphs and arbitrary nonlinear curves need broader acceptance cases.

Affine interval consistency is verified for straight-edged boxes. General trim UV
correspondence, reversed underlying Brep edges, nonlinear/seam/singular topology,
all native/plugin curve/surface classes, units/transforms, reference remap/retarget,
selected export closure and edited owners remain required. Source-model analysis
does not establish current host display/edit/export compatibility. Existing export
gate stays active and preserves the destination. Independent target verification
for these new reference paths and actual Rhino5 application acceptance are pending.
Full128 native classes/16 component categories/6 document categories, history,
resources, plugin/document/version semantics and final review remain in_progress.

This report supersedes the previous reference report's no-sharing/no-aggregate-
clone-budget limitation only for the scope above; other limitations remain open.

## GUI evidence

|Suite|Checks|Artifact|
|---|---|---|
|reference_graph_integrity|38/38|build\three_dm_reference_graph_integrity_smoke-1\2616dad981594299a0f5d138c03db887|
|native_reference_resolution|88/88|build\three_dm_native_reference_resolution_smoke-1\9f2eceedaddc4e3a8be7409def0fda0d|
|curve_on_surface_schema|120/120|build\three_dm_curve_on_surface_schema_smoke-1\e1070abafdf446f0b7b816b518de115d|
|curve_on_surface|32/32|build\three_dm_curve_on_surface_smoke-1\c18af674dc0445358817748a448d3c09|
|hatch_loop_rows|103/103|build\three_dm_hatch_loop_rows_smoke-1\312b9217c4514cabb26fdc8fd869e22e|
|hatch_loop_ui|104/104|build\three_dm_hatch_loop_ui_smoke-1\a6a52f4f344b4ca7bf2b347fd41c4be7|
|hatch_legacy_loops|84/84|build\three_dm_hatch_legacy_loops_smoke-1\ec9d5bd41f044105a5372206b9b22be7|
|hatch_loop_baseline|4/4|build\three_dm_hatch_loop_baseline_smoke-1\0a5eccbe541c45d4a1c560bef57d9504|
|included_file_copy|8/8|build\three_dm_included_file_copy_smoke-1\75cf7f39e5784199b1ade230741986fe|
|hatch_loop_edit|132/132|build\three_dm_hatch_loop_edit_smoke-1\a1c7f54760b24e6788f309fe36d9f539|
|hatch_shared_loop_edit|12/12|build\three_dm_hatch_shared_loop_edit_smoke-1\13be3c770fd2402a8a3dddf45c694771|
|hatch_loops|54/54|build\three_dm_hatch_loops_smoke-1\a6726535d5744143b10fd0ffd83e5592|
|hatch_legacy|24/24|build\three_dm_hatch_legacy_smoke-1\2bc3ec04bd6e4a9ea06432796b61fe34|
|hatch_current|48/48|build\three_dm_hatch_current_smoke-1\16b843d85f0c4408b18c9cc6cdd72caf|
|hatch_shared_current|16/16|build\three_dm_hatch_shared_current_smoke-1\b4b8b168f39848ce927e73bfbd429c6d|
|hatch_affine|24/24|build\three_dm_hatch_affine_smoke-1\8bc84b563b724b11b0e03300e227541b|
|hatch|16/16|build\three_dm_hatch_smoke-1\bbf87540bfa24911af23b6ee9c514d28|
|cloud_geometry|50/50|build\three_dm_cloud_geometry_smoke-1\614ddb93dd0d4f31a74ed085d644468e|
|point_cloud|43/43|build\three_dm_point_cloud_smoke-1\2346f6993d6443cda518589889df5e19|
|text_dot|34/34|build\three_dm_text_dot_smoke-1\c9fefcaae5ac442a9316f3300e0a37fd|
|selected_member|33/33|build\three_dm_selected_member_smoke-1\638237d7d3df456ca5a315b1296b1c48|
|retained_transform|25/25|build\three_dm_retained_transform_smoke-1\4ccf007d08434a12826743489701f836|
|host_upgrade|39/39|build\three_dm_host_upgrade_smoke-1\b3b64e43b5284510994528f4b6747f06|
|shared_export_routes|30/30|build\three_dm_shared_export_routes_smoke-1\9eefdea091f54dbe84c6d52ca1a881a7|
|recursive_shared_migration|24/24|build\three_dm_recursive_shared_migration_smoke-1\9e8eb6d853db4736befcaceb12a777df|
|recursive_namespace_migration|25/25|build\three_dm_recursive_namespace_migration_smoke-1\81d0e02a422c4b608d08edebf8b465cb|
|affine_target_migration|32/32|build\three_dm_affine_target_migration_smoke-1\04620ae41c2a4303b33c336ff680c55d|
|copy_origin_migration|22/22|build\three_dm_copy_origin_migration_smoke-1\7c37d9ad71374c94af8a4aacbad2feb7|
|legacy_migration|35/35|build\three_dm_legacy_migration_smoke-1\27211de63c5e44bea4af466dda6cac7f|
|proxy_retarget|6/6|build\three_dm_proxy_retarget_smoke-1\58c96aa2fbbe425bac0f5621c22f656e|
|source_proxy_creation|34/34|build\three_dm_source_proxy_creation_smoke-1\7942ac9996ec4b8bac05cf6388ad0c61|
|definition_copy|27/27|build\three_dm_definition_copy_smoke-1\3b1f0be9bf42491594a474ccdc0352e9|
|source_members|23/23|build\three_dm_source_members_smoke-1\37e5d691d3e34891b67f59b36783a81e|
|source_member_geometry|8/8|build\three_dm_source_member_geometry_smoke-1\2e99f5c8171c47b39e5128760cfdc8c5|
|source_new_target|27/27|build\three_dm_source_new_target_smoke-1\ca0e1263f5da4817a629eb17d29f1801|
|new_blocks|29/29|build\three_dm_new_blocks_smoke-1\0effaa8b577d4b7e856b814fc3641444|
|new_member_instance|20/20|build\three_dm_new_member_instance_smoke-1\58a69e6da540460d9a29005a96d50d27|
|reference_edit|18/18|build\three_dm_reference_edit_smoke-1\3bce3061f81b4ab6865e3a92ae0995f9|
|independent_member|18/18|build\three_dm_independent_member_smoke-1\ea78aa5ff5e54141a767532f514f72ea|
|shared_proxy|10/10|build\three_dm_shared_proxy_smoke-1\dda89715199d49f7aad61e4012e1d839|
|shared_proxy_geometry|8/8|build\three_dm_shared_proxy_geometry_smoke-1\497983ec99504967b89addcff07c3c3e|
|retained_member|6/6|build\three_dm_retained_member_smoke-1\736e33cb260542d392ddbaca63c4c19d|
|definition_placement|12/12|build\three_dm_definition_placement_smoke-1\aafe796a032343099bf01f8cb8b44833|
|new_geometry|20/20|build\three_dm_new_geometry_smoke-1\393be5f3d31f445e86a26265741306af|
|preserved_export|13/13|build\three_dm_preserved_export_smoke-1\e281344589dd48e88fd3cf64d84b856e|
|member_overlay|12/12|build\three_dm_member_overlay_smoke-1\71dcc538e3b7479482896e1b75859156|
|block_structure|23/23|build\three_dm_block_structure_smoke-1\b07891fb5f0d49dd8cfc235d2fb4a481|
|affine_refresh|10/10|build\three_dm_affine_refresh_smoke-1\8889650fa20b4b649e3a3a7f17f2d26f|
|affine_refresh_reopen|2/2|build\three_dm_affine_refresh_reopen_smoke-1\227655f33f08428cb34370df11def9d3|
|source_signature|14/14|build\three_dm_source_signature_smoke-1\30b3cd3fa8884c759498a07ee08a5eb1|
|preservation|19/19|build\three_dm_preservation_smoke-1\4db48144ccb34d609f6351f6dc3ca54e|
|definition_overlay|8/8|build\three_dm_definition_overlay_smoke-1\818c4d3440d946c19b260e82662581ef|
|layer_overlay|7/7|build\three_dm_layer_overlay_smoke-1\de36ac6d36004ef19a3ee53e0b83b488|
|mixed_shear|11/11|build\three_dm_mixed_shear_smoke-1\e9a47ac5af0644f78ccb06f7753a685e|
|merge_bridge|8/8|build\three_dm_merge_bridge_smoke-1\5834e3d0ac4b4b3a8243490b13d026da|
|geometry|33/33|build\three_dm_smoke-1\3a7cdebd77234dfb8297113bff19b2c8|

SHA-verified source/evidence checkpoint: `H:\FreeCAD-src\build\checkpoints\reference-graph-integrity-20261007-112413.zip`.
