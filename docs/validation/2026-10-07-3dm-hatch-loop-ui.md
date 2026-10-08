# OM9-FILE-012 — ordinary native Hatch loop controls

The existing numeric editor for five native curve classes has focused FreeCAD
evidence. This does not complete Hatch or full openNURBS. Arbitrary CV/knot/point/
segment insertion, native class/rationality replacement, surface/reference/plugin
adapters, pattern content/full fill rendering and actual Rhino5 acceptance remain
open under the approved plan and original128/16/6 coverage scope.

## User flow and authority

Double-click a native Hatch or choose **Edit native Hatch boundaries…** from its
context menu. The Qt tree exposes numeric native fields, including rational raw
CVs, knots/order/domain, Arc frame/radius/angular domain, Line/Polyline points and
independent PolyCurve child/segment parameters. Curve types/source indices and
metadata remain provenance. Unchanged17-digit display strings do not roundtrip
through a numeric parser; the original JSON number is retained exactly.

Outer/inner role controls and explicit new native circles/loop removal operate
on detached values. Circle coordinates/radius use mm and angles use radians.
Native topology, immutable source/baseline and child-data checks precede a single
document transaction. Cancel or invalid proposals cannot replace the included
loop file. The native bridge returns proposed data; the host wrapper validates
after that call returns. Native errors reopen the editor with proposed values.
Changes to the verified host while the dialog is open require cancel/reopen.
The bounded table refuses >65536 fields/depth64 without truncating data.

## Failure investigation and test-driver correction

The previous in-progress report preserves actual failed processes; those results
were not accepted. Instrumenting the formerly hanging callback produced a
specific Python traceback: `QTreeWidgetItem` had been deleted before `setText`.
Failed diagnostic artifact:
`three_dm_hatch_loop_ui_fenv_probe-1/b623e3106bec4db5ae6593a2275fe9a4`.
The corrected callback's artifact is
`three_dm_hatch_loop_ui_fenv_probe-1/d8eabf51fe6445a19c738ffdd43e352f`.

The smoke driver now operates through the real `QAbstractItemModel` and
`QModelIndex`, avoiding ownership wrappers for native-created tree items. It
still edits the live widgets/model, runs actual native preflight, persists the
host object and exports/rereads3DM; no geometry or archive result is mocked.
Every callback error is checked globally rather than swallowed by the Qt timer.
Historical diagnostic macros are retained under `build/hatch-ui-diagnostics`,
outside maintained smoke entrypoints, with prior source in the WIP checkpoint.

The corrected numeric-rejection probe recorded identical CRT control/status
values before/after (`0x8001f`/`0x11`) and exited0. Thus this case does not support
the earlier floating-environment hypothesis. Moving validation outside the Qt
signal and removing Undo/Redo had not eliminated the faulty-driver failure;
neither is claimed as the proven fix. Full104-check focused UI regression then
exited0 with no callback error. No production geometry conversion was relaxed.

## Focused acceptance

`tests/three_dm_hatch_loop_ui_smoke.FCMacro`:104/104, process0, focused artifact
`396e59d89818457885698fc2be424e1a`, before the final bridge docstring rebuild.

- mm/cm edits of NURBS, Arc, Polyline and nested PolyCurve/Line exact native fields.
- Exact native reread and child user strings, immutable original fixtures.
- Cancel, nonfinite entries, invalid radius/topology, new inner Arc and removal.
- One Undo/Redo operation, unchanged numeric identity and stale host rejection.
- Double-click/context-menu routing; independently edited copies.
- FCStd close/reopen after external source deletion; editor/native export still
  verify the embedded source and exact current loop values.

Module build after the docstring clarification exits0. Full51-suite GUI/native20
regressions are tracked separately and must finish before the final checkpoint.
FreeCAD core and Rust have not changed in this package.

## Final verification

Fresh native20/20,42.04s, process0; isolated GUI1344/1344 across51 suites, all
process0. Source/installed9 scripts, runtime2 binaries and legacy reader stayed
unchanged during GUI runs. Both immutable supplied ring fixtures pass. Original
SDK20130711 source280 files remains byte-identical to the official ZIP.

The first full GUI attempt ran concurrently with native fixture generation and
failed the original temporary cm fixture immutability check. The native
HatchLoops suite writes the same shared temp fixtures; final verification runs
GUI only after native completes. This was a fixture dependency, not permission
to weaken source immutability.

FreeCAD core/Rust unchanged: previous App556 pass/2 skipped/7 disabled and Rust75/75
remain historical evidence; no fresh App/Rust test claim in this package.

|Suite|Checks|Artifact|
|---|---|---|
|hatch_loop_ui|104/104|build\three_dm_hatch_loop_ui_smoke-1\1d9981a71bb94f3ba03da4ceb1076d4d|
|hatch_legacy_loops|84/84|build\three_dm_hatch_legacy_loops_smoke-1\b5258c356e4e48e7b571855235060894|
|hatch_loop_baseline|4/4|build\three_dm_hatch_loop_baseline_smoke-1\e0dde09564b74901a660c2fa92ef883b|
|included_file_copy|8/8|build\three_dm_included_file_copy_smoke-1\54e46761a9df4e76842c482405bd01ba|
|hatch_loop_edit|132/132|build\three_dm_hatch_loop_edit_smoke-1\c33c1f6c3ea44eea84180310db349e3f|
|hatch_shared_loop_edit|12/12|build\three_dm_hatch_shared_loop_edit_smoke-1\134bc51534324115a4dfd899d31e4907|
|hatch_loops|54/54|build\three_dm_hatch_loops_smoke-1\e4c5082412944d6c9ab1b26a2c04e7dc|
|hatch_legacy|24/24|build\three_dm_hatch_legacy_smoke-1\a8b988689be84917840798dab9b08fa7|
|hatch_current|48/48|build\three_dm_hatch_current_smoke-1\a36c0ae6c3104b7dab4597f49a199acf|
|hatch_shared_current|16/16|build\three_dm_hatch_shared_current_smoke-1\b356d800e20b4f8f88f5fefa3c8a5d07|
|hatch_affine|24/24|build\three_dm_hatch_affine_smoke-1\375dab20ec864aa9948e23dfdad2ed37|
|hatch|16/16|build\three_dm_hatch_smoke-1\7f769636ccf34c129c16732d84105b30|
|cloud_geometry|50/50|build\three_dm_cloud_geometry_smoke-1\f7f068adb8b34040a8e656c215b2c3ae|
|point_cloud|43/43|build\three_dm_point_cloud_smoke-1\48df4ff8a8b249428fa2bcbfb70b3f6d|
|text_dot|34/34|build\three_dm_text_dot_smoke-1\538b5eccb2eb496f88ff576af12ee83c|
|selected_member|33/33|build\three_dm_selected_member_smoke-1\78c6233bf616462c9be5fbbbc59bd33b|
|retained_transform|25/25|build\three_dm_retained_transform_smoke-1\16be2ede466c4769b04d576f578a290c|
|host_upgrade|39/39|build\three_dm_host_upgrade_smoke-1\8abd640f92724a23806124c77e8d2578|
|shared_export_routes|30/30|build\three_dm_shared_export_routes_smoke-1\02320db2b9524c6e966702d40b72a8b0|
|recursive_shared_migration|24/24|build\three_dm_recursive_shared_migration_smoke-1\e36c370124ce4fa2877334bb995610af|
|recursive_namespace_migration|25/25|build\three_dm_recursive_namespace_migration_smoke-1\d389faf4a7244a70800d8c3c474b6c27|
|affine_target_migration|32/32|build\three_dm_affine_target_migration_smoke-1\4ac237b060b94b418513a6ef890e3a15|
|copy_origin_migration|22/22|build\three_dm_copy_origin_migration_smoke-1\26548adc9d7b44c08514799b6910da85|
|legacy_migration|35/35|build\three_dm_legacy_migration_smoke-1\3efbf1e20c04458d98767aebae6694f1|
|proxy_retarget|6/6|build\three_dm_proxy_retarget_smoke-1\5f75a229b8074236874fd5035264738a|
|source_proxy_creation|34/34|build\three_dm_source_proxy_creation_smoke-1\e498e74c52e54502a435ceefd435cdbb|
|definition_copy|27/27|build\three_dm_definition_copy_smoke-1\88d7937e261c40a3b6037ddab7a14dbf|
|source_members|23/23|build\three_dm_source_members_smoke-1\dcd2936b76094ddb8b734083cb327492|
|source_member_geometry|8/8|build\three_dm_source_member_geometry_smoke-1\37b67cbcf28841f38a94ebb5fdec1777|
|source_new_target|27/27|build\three_dm_source_new_target_smoke-1\ca689e3b18774d438b171ee0c0322cad|
|new_blocks|29/29|build\three_dm_new_blocks_smoke-1\5822bc6a76a84d1b9927a2b28bf0baa4|
|new_member_instance|20/20|build\three_dm_new_member_instance_smoke-1\365b03faf43643aa9ab22c00404679e1|
|reference_edit|18/18|build\three_dm_reference_edit_smoke-1\331b6b7a0958467098d8a99a2b3603f3|
|independent_member|18/18|build\three_dm_independent_member_smoke-1\f43e684fc37743ab97658cbcaa61970c|
|shared_proxy|10/10|build\three_dm_shared_proxy_smoke-1\66936af4f36f4a70984459a13820ffa2|
|shared_proxy_geometry|8/8|build\three_dm_shared_proxy_geometry_smoke-1\33114225daa54e239f48781e4bc7bb57|
|retained_member|6/6|build\three_dm_retained_member_smoke-1\a68a169473d043f9a911768eb42e42a8|
|definition_placement|12/12|build\three_dm_definition_placement_smoke-1\3f448690fbfe4517a2c0c8ba152740f5|
|new_geometry|20/20|build\three_dm_new_geometry_smoke-1\9911bec18d2d4748b83c08a00cc293ef|
|preserved_export|13/13|build\three_dm_preserved_export_smoke-1\7919412d73984dd98f6ce5fb949b7870|
|member_overlay|12/12|build\three_dm_member_overlay_smoke-1\218bd1089da542e1b55f5c5e999959ae|
|block_structure|23/23|build\three_dm_block_structure_smoke-1\6c842af106d542e59b6a5792d9bb3a6c|
|affine_refresh|10/10|build\three_dm_affine_refresh_smoke-1\af9171fc7ba74f93bbcf28316cf2f7a3|
|affine_refresh_reopen|2/2|build\three_dm_affine_refresh_reopen_smoke-1\3e092667052e48519ab523fc97a40bad|
|source_signature|14/14|build\three_dm_source_signature_smoke-1\cde3414a172c425cbb7e45176978a392|
|preservation|19/19|build\three_dm_preservation_smoke-1\3d36623c0b2f4a47a4053c9127c62f1b|
|definition_overlay|8/8|build\three_dm_definition_overlay_smoke-1\a1f9aef0e41f41f587dfa7520b1350a2|
|layer_overlay|7/7|build\three_dm_layer_overlay_smoke-1\b52017fcb7e04d1daa5b3191954d5b60|
|mixed_shear|11/11|build\three_dm_mixed_shear_smoke-1\f7e3394538954ba2a0c98ab6163eadf8|
|merge_bridge|8/8|build\three_dm_merge_bridge_smoke-1\94c6b641495c468cac51d44423c7642b|
|geometry|33/33|build\three_dm_smoke-1\896a9281a47b4c808fbe0f53bb74da61|

## Shared export history test timing

The production sidebar refreshes history on a350ms timer; the existing test
waited250ms and could capture two entries before the third successful export
appeared. Controlled RED `three_dm_shared_export_history_probe-1/137b8f90072445d4bc2455f86b22e184` retained92 host objects and unchanged destination bytes, but history advanced2→3 across cancellation.
The test now waits up to2s for all three successful records before capturing
its cancellation baseline. Controlled GREEN `three_dm_shared_export_history_probe-1/e14a40dd011a4aca800450106dd6a4d5` passes30 checks with history3→3. The original object/file/history equality check remains. Production history code was not changed.

SHA-verified source/evidence checkpoint: `H:\FreeCAD-src\build\checkpoints\hatch-loop-ui-20261007-083044.zip`.
