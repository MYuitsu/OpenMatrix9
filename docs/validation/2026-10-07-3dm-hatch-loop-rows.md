# OM9-FILE-012 — native Hatch structural row controls

The approved Hatch plan now has explicit duplicate/remove controls for detached
NURBS CV/knots, Polyline points/parameters and PolyCurve segments/parameters.
This is an OpenMatrix9 raw-field editing choice, not a claim of automatic shape
preserving knot insertion, complete native type editing or full openNURBS.

## Behavior and authority

Double-click a retained native Hatch or choose **Edit native Hatch boundaries…**.
Select a row (or a field beneath it), then use **Duplicate selected row** or
**Remove selected row**. Selecting an eligible array duplicates its final row.
The duplicate retains exact raw values and original source identities. Edit
the new values and paired arrays before confirming. Original metadata is copied
by the native source adapter, rather than rewritten by the UI. New archive
inventory indices are local to the reread; host provenance remains original.

Line endpoint arrays and Arc frame/domain cardinalities remain fixed. At least
one raw row must remain. Editor size/depth is checked before replacing the
detached tree, with the same65536-field/depth64 bound. No structural operation
commits a document edit; the existing native cardinality/domain/closure/topology
preflight must accept the whole proposal before one host transaction is written.
Cancellation or invalid paired arrays leave included files and the host intact.

## RED and focused GREEN

`tests/three_dm_hatch_loop_rows_smoke.FCMacro` first fails against the numeric-only
module at the absent row action, artifact
`three_dm_hatch_loop_rows_smoke-1/f1fdfdfabc71481e80a47d50c9ba131f`.
The real UI, document persistence and native exchange are used; geometry,
preflight and file results are not mocked.

After the native UI implementation and isolated module build, focused103/103
checks exit0 in artifact
`three_dm_hatch_loop_rows_smoke-1/3b2731e294f8464bb3d9acf7b1f619f3`:

- mm/cm Polyline point/parameter insertion; single Undo/Redo and cancel.
- Native rejection of unmatched point/parameter removal without host mutation.
- Degree1 nonrational NURBS square CV/knot insertion/removal with exact fields.
- PolyCurve Line split retaining child domains, original source indices and
  duplicated child user strings; unrelated nested children remain exact.
- Fixed Line endpoint cardinality refusal.
- FCStd close/reopen after deleting each private external source; current fields
  and exact native export/reread remain available.
-12 fresh native archives directly decoded/reencoded by unchanged SDK201307115;
  native child types, raw geometry and metadata match the modern native inventory.
  Exported input bytes and shared original fixtures remain unchanged.

Source module build exits0. Full52-suite regression is recorded below only when
finished. The preceding numeric UI checkpoint (`hatch-loop-ui-20261007-083044.zip`)
remains an immutable historical source/evidence snapshot.

## Remaining full-scope work

These controls expose current arrays, with named fixtures for their lifecycle;
they do not establish all cardinality/precision/performance or rational geometry
operations. New native class/rationality controls, complete nested curve creation,
CurveOnSurface/PolyEdge/reference/plugin adapters, native pattern resources and
full fill rendering, actual Rhino5 application acceptance, all128 native classes,
16 component categories and6 document categories remain in_progress. FreeCAD core
and Rust did not change in this row-control package. Final full-package review
and public integration remain open under the original exchange design.

## Final verification

Fresh native20/20,41.31s, process0; isolated GUI1447/1447 across52 suites, all
process0. Both immutable user ring fixtures pass. Native fixtures finished
before GUI runs. Source/installed9 scripts, runtime2 binaries and legacy reader
remain stable. Original SDK20130711 source280 files remains byte-identical to
its official ZIP.12 additional structural archive paths directly reread/reencode
exact child trees with the independent SDK2013 reader.

App/Rust unchanged: previous556 pass/2 skipped/7 disabled and75/75 remain
historical evidence. No actual Rhino5 application rendering acceptance claim.

|Suite|Checks|Artifact|
|---|---|---|
|hatch_loop_rows|103/103|build\three_dm_hatch_loop_rows_smoke-1\16056db8647348eca4021ef1c6f87685|
|hatch_loop_ui|104/104|build\three_dm_hatch_loop_ui_smoke-1\be4f4c7b5470465aaf48f1d68091d81f|
|hatch_legacy_loops|84/84|build\three_dm_hatch_legacy_loops_smoke-1\4a485b708a5c4a1387842a4e3e7472c0|
|hatch_loop_baseline|4/4|build\three_dm_hatch_loop_baseline_smoke-1\b5084285e73f4cd197a087b8126e4471|
|included_file_copy|8/8|build\three_dm_included_file_copy_smoke-1\d600ae64da9a4524b3f12bb416db7201|
|hatch_loop_edit|132/132|build\three_dm_hatch_loop_edit_smoke-1\10984c28cc784612bb38a0888ea49492|
|hatch_shared_loop_edit|12/12|build\three_dm_hatch_shared_loop_edit_smoke-1\59ed5b0bdc8a4013976ce960851587d1|
|hatch_loops|54/54|build\three_dm_hatch_loops_smoke-1\787fe70274334cebbf139ae522ea4143|
|hatch_legacy|24/24|build\three_dm_hatch_legacy_smoke-1\f5e9ef6e69b54a38ac61671361ae0578|
|hatch_current|48/48|build\three_dm_hatch_current_smoke-1\5d56764c627a4c8f9fc4a6019299d5f0|
|hatch_shared_current|16/16|build\three_dm_hatch_shared_current_smoke-1\c0a55a41f2064b3fab1a3bbc7ff5f16e|
|hatch_affine|24/24|build\three_dm_hatch_affine_smoke-1\0cddb9c592ee486b8ce7a471d33f83c4|
|hatch|16/16|build\three_dm_hatch_smoke-1\b055954437304fe6a532d5dd164deae4|
|cloud_geometry|50/50|build\three_dm_cloud_geometry_smoke-1\d80446cf2085445da78294def7db5c35|
|point_cloud|43/43|build\three_dm_point_cloud_smoke-1\17d43e19beea4adaa69b5bd94ecfd2d0|
|text_dot|34/34|build\three_dm_text_dot_smoke-1\17f7756a18bb4d9b81b9bb91db5b239a|
|selected_member|33/33|build\three_dm_selected_member_smoke-1\f779be586c2a43df9c7b8ecbe536fa23|
|retained_transform|25/25|build\three_dm_retained_transform_smoke-1\ee964f6a65254cc79a1d8627f6eea4c8|
|host_upgrade|39/39|build\three_dm_host_upgrade_smoke-1\8219b1b0f9f741f5beb15868a57e823b|
|shared_export_routes|30/30|build\three_dm_shared_export_routes_smoke-1\7524ef54ee174a69abab871633a2fd0d|
|recursive_shared_migration|24/24|build\three_dm_recursive_shared_migration_smoke-1\5c3e3b86aa0541a0a4da10ed8126d5ef|
|recursive_namespace_migration|25/25|build\three_dm_recursive_namespace_migration_smoke-1\016b24340e8c4b21a390e44e9da69dd1|
|affine_target_migration|32/32|build\three_dm_affine_target_migration_smoke-1\6c248f5a182b4a768f580d1c7d357b49|
|copy_origin_migration|22/22|build\three_dm_copy_origin_migration_smoke-1\378b6713a8404eab9b9bd9da9f4cd088|
|legacy_migration|35/35|build\three_dm_legacy_migration_smoke-1\332334f3a5e548f9a8bec0aae8c0172f|
|proxy_retarget|6/6|build\three_dm_proxy_retarget_smoke-1\be8292133d2f4d5caf6312392333dd1d|
|source_proxy_creation|34/34|build\three_dm_source_proxy_creation_smoke-1\7671f22e94b34e43b64983c27c0b355f|
|definition_copy|27/27|build\three_dm_definition_copy_smoke-1\a06f0709d1ae42f084d183f405a6b3d9|
|source_members|23/23|build\three_dm_source_members_smoke-1\7fa41c21ad8747b1a58d5066102af206|
|source_member_geometry|8/8|build\three_dm_source_member_geometry_smoke-1\95b9239b2c0f4758a4d36d93c436b70a|
|source_new_target|27/27|build\three_dm_source_new_target_smoke-1\27c050274770426daa474236418a6d1f|
|new_blocks|29/29|build\three_dm_new_blocks_smoke-1\0c48d04d285b4d49bce64a2dfad278f1|
|new_member_instance|20/20|build\three_dm_new_member_instance_smoke-1\193f040cde3440e581f81acdf9d327c5|
|reference_edit|18/18|build\three_dm_reference_edit_smoke-1\c1928dd1b2ef4c4cb230fc48c319d185|
|independent_member|18/18|build\three_dm_independent_member_smoke-1\73456bf331754f71b282c6f262b4ab9c|
|shared_proxy|10/10|build\three_dm_shared_proxy_smoke-1\aad60225546746a588f72cb4be4f3401|
|shared_proxy_geometry|8/8|build\three_dm_shared_proxy_geometry_smoke-1\627869a5610a42128fb98ebff65d52e7|
|retained_member|6/6|build\three_dm_retained_member_smoke-1\dd7ac5936b134ce28cdb04953aeaf858|
|definition_placement|12/12|build\three_dm_definition_placement_smoke-1\e96086d0a0f34de88f84e9dd196a9678|
|new_geometry|20/20|build\three_dm_new_geometry_smoke-1\b819f3684ce442ca885f05e75d6e28f4|
|preserved_export|13/13|build\three_dm_preserved_export_smoke-1\38b20416337c4fbd8ec429bbca494a06|
|member_overlay|12/12|build\three_dm_member_overlay_smoke-1\d2908948863e41b7bc2c32d36b97ab72|
|block_structure|23/23|build\three_dm_block_structure_smoke-1\24fccaed3b994c968953f2bb143d4007|
|affine_refresh|10/10|build\three_dm_affine_refresh_smoke-1\c8363d4516764e63b78b8d8989a3cfab|
|affine_refresh_reopen|2/2|build\three_dm_affine_refresh_reopen_smoke-1\0817a0ae9c704dfdbe695b4eed587c71|
|source_signature|14/14|build\three_dm_source_signature_smoke-1\595fe7203dff408d86654a90bb00db94|
|preservation|19/19|build\three_dm_preservation_smoke-1\cb5c498bcf4946b3aecde1fcf3e177d0|
|definition_overlay|8/8|build\three_dm_definition_overlay_smoke-1\0e15e4ec9d5a4777a48350af5a48c135|
|layer_overlay|7/7|build\three_dm_layer_overlay_smoke-1\8442480cecb9415f8140c37794bbc744|
|mixed_shear|11/11|build\three_dm_mixed_shear_smoke-1\c316697ece5942529c4b4b394d1ee9c9|
|merge_bridge|8/8|build\three_dm_merge_bridge_smoke-1\7dfbf67c5b2640858602de92b0b4dab1|
|geometry|33/33|build\three_dm_smoke-1\b509f83e1430496f8bdab1c064f511f1|

SHA-verified source/evidence checkpoint: `H:\FreeCAD-src\build\checkpoints\hatch-loop-rows-20261007-084315.zip`.
