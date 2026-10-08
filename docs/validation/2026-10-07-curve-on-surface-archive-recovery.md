# OM9-FILE-012 — CurveOnSurface archive recovery prerequisite

The current module recovers parameter curve, optional approximation and surface
without replacing the native class or dropping children. Preservation import
classifies the object as source retained, without a false empty CAD shape. FCStd
retains byte-identical source archives after the external file is deleted.
Full editable/export CurveOnSurface support remains in progress.

## Declared SDK repair and ownership

Both pinned modern2425133316 and independent SDK201307115 original Read functions
misassign optional m_c3 to m_c2 and then fail before reading the surface. The
original negative probes and modern pristine static library remain preserved.
`cmake/CurveOnSurfaceArchive.cmake` verifies the full normalized upstream source
SHA, generates a translation-unit copy and replaces only Read. All other methods
and original source remain intact. Current modern/legacy compiled libraries do
contain the declared repair; original binary evidence remains historical.
Legacy return type ON_BOOL32 is preserved; modern uses bool. Configuration refuses
unexpected upstream sources or anything other than exactly one source replacement.

Each child is decoded into staged ownership. Successful archive status, actual
curve/surface types, resolved native validity/dimensions and only flags0/1 are
checked before replacing any existing child. Known exact PolyEdgeCurve/Segment
records with dimension0 retain their deferred reference payload for later model
resolution. This exception is archive recovery, not semantic validity. A wrong
type or truncated record leaves the existing valid child objects intact.

## Fresh evidence

- Native21/21, process0,45.74s; both immutable user ring fixtures pass.
- New native suite:12 valid2D/3D x optional absent/present x archive50/60/70
  paths; raw payload, ClassId factory and full model reencode retain native
  identity/child fields/tags/evaluated points and immutable source.
-117 wrong-type/malformed/truncated cases keep existing valid children intact;
  two deferred PolyEdge child identities survive native archive decoding.
- Four separate repaired SDK2013 oracle paths read exact UUID/class/domain/Arc
  frames and radius/NURBS CVs/knots/orders/dimensions/rationality/user strings,
  compare five composed points at declared1e-12 tolerance, reencode version5,
  then reread exact fields with the modern reader. This oracle declares the same
  Read repair and does not replace the original Rhino5HatchReader executable.
-32 new FreeCAD checks cover retained import, original identity and bytes,
  optional child presence, FCStd after external deletion, and refusal of pending
  export before target/document mutation. Regression1479/1479 across53 GUI suites,
  every process0. Native fixture generation finished before sequential GUI runs.
- Nine installed scripts, two runtime binaries and both original/repaired readers
  remain stable throughout GUI verification. All280 SDK2013 source files still
  match the official ZIP; modern source matches its recorded SHA.

RED before repair: `workspace-build/curve-on-surface-red-LastTest.log` records the
optional-child exact recovery failure. RED GUI artifact
`build/three_dm_curve_on_surface_smoke-1/bc4b01956a6841d994e7c3b8d51becb1`
shows selected export incorrectly succeeded and replaced the destination sentinel
before the pending-adapter gate. The final gate preserves destination and host.
The first FCStd test incorrectly referred to OM9ArchiveSnapshot; the test now uses
the production FileIncluded property OM9SourceArchive and passes all four cases.

## Required remaining work

GetNurbForm is unimplemented in the pinned CurveOnSurface SDK; recovered records
are retained, not advertised as CAD editable. SDK Transform updates the surface
without its optional approximation. Full adapters must preserve native child
classes/domains/raw data, resolve/remap object/component references, audit child
userdata/plugin dependencies, couple surface and approximation transforms, and
provide explicit preview/edit/export/version semantics. Selected CurveOnSurface
export is temporarily refused with a precise message until those paths are proven.
The existing non-mm source-normalization stage visits source records before final
selection closure; a pending CurveOnSurface in such a source also prevents that
stage. Moving normalization to verified selected dependencies remains required,
rather than silently transforming an unresolved approximation or surface record.
Full Hatch surface/reference editing, native pattern/fill rendering, rational/
precision/performance coverage and all128 class/16 component/6 document categories
remain required. Actual Rhino5 application rendering/roundtrip is unverified.

App/Rust are unchanged in this slice; earlier App556 pass/2 skipped/7 disabled and
Rust75/75 are historical evidence. No public integration or push in this workstream.

## GUI suite artifacts

|Suite|Checks|Artifact|
|---|---|---|
|curve_on_surface|32/32|build\three_dm_curve_on_surface_smoke-1\1e3803551c6e4827a9b5b627ebef90e1|
|hatch_loop_rows|103/103|build\three_dm_hatch_loop_rows_smoke-1\f7437e0a0bc945beb9a2b76e8a58d0e8|
|hatch_loop_ui|104/104|build\three_dm_hatch_loop_ui_smoke-1\a3e4b602787e4d249c306a65a89659a6|
|hatch_legacy_loops|84/84|build\three_dm_hatch_legacy_loops_smoke-1\79db3ea0d50343ca8243cc94dbe02c88|
|hatch_loop_baseline|4/4|build\three_dm_hatch_loop_baseline_smoke-1\f46bfc0e6f684cd994b5f5cc155b4877|
|included_file_copy|8/8|build\three_dm_included_file_copy_smoke-1\a4f360a952194df4905ee158cc36af83|
|hatch_loop_edit|132/132|build\three_dm_hatch_loop_edit_smoke-1\de15e6306e4c4468838a5a43ef3f30f3|
|hatch_shared_loop_edit|12/12|build\three_dm_hatch_shared_loop_edit_smoke-1\6c379c91cbc24967b98d0aabd5584211|
|hatch_loops|54/54|build\three_dm_hatch_loops_smoke-1\708be00a4ceb44db8a8fd1681ddcbb69|
|hatch_legacy|24/24|build\three_dm_hatch_legacy_smoke-1\4e445e4f34374ae19c1bc829de991f18|
|hatch_current|48/48|build\three_dm_hatch_current_smoke-1\4690ad2400544ce1b3bb1e09f91747ae|
|hatch_shared_current|16/16|build\three_dm_hatch_shared_current_smoke-1\9c8eec5f752a40ab84bb97908abadcd0|
|hatch_affine|24/24|build\three_dm_hatch_affine_smoke-1\9a85b9f4f7424ba381643de9cc8c601b|
|hatch|16/16|build\three_dm_hatch_smoke-1\554954a3cca74a13878d747e029a664e|
|cloud_geometry|50/50|build\three_dm_cloud_geometry_smoke-1\94bab415787745dca823d19000e028c7|
|point_cloud|43/43|build\three_dm_point_cloud_smoke-1\5a73a5371a88491ea3b89166e304e5e7|
|text_dot|34/34|build\three_dm_text_dot_smoke-1\e5139267ee42472db0f3c42bdc93e3b1|
|selected_member|33/33|build\three_dm_selected_member_smoke-1\ddf27ff283e14038b442a83f9bc85d3c|
|retained_transform|25/25|build\three_dm_retained_transform_smoke-1\9147eebf500349bd85c7125b9672ab56|
|host_upgrade|39/39|build\three_dm_host_upgrade_smoke-1\c17210e69c534eb6b12bda626f72d0a4|
|shared_export_routes|30/30|build\three_dm_shared_export_routes_smoke-1\70c2cc37a72842028a44129477727ed8|
|recursive_shared_migration|24/24|build\three_dm_recursive_shared_migration_smoke-1\bacd581a7e934fef935ba6b8eed2e5ed|
|recursive_namespace_migration|25/25|build\three_dm_recursive_namespace_migration_smoke-1\0724a1f2300c4e518f55b05afea244f9|
|affine_target_migration|32/32|build\three_dm_affine_target_migration_smoke-1\e9172c24b1da41cfac7c72bff679b293|
|copy_origin_migration|22/22|build\three_dm_copy_origin_migration_smoke-1\98e732e2c7054a2ea5ebe2848f4b75d8|
|legacy_migration|35/35|build\three_dm_legacy_migration_smoke-1\c0eff7c4c3464bf0a0dcc013cde97a99|
|proxy_retarget|6/6|build\three_dm_proxy_retarget_smoke-1\ecb2b7d52ef14cf08a138fe52455ccc1|
|source_proxy_creation|34/34|build\three_dm_source_proxy_creation_smoke-1\603659f37004474ab81fd6f78eb9300f|
|definition_copy|27/27|build\three_dm_definition_copy_smoke-1\7020b38e79f04cdc8053b36afec8d5b2|
|source_members|23/23|build\three_dm_source_members_smoke-1\6459f48305174cb0bbe980362f79199c|
|source_member_geometry|8/8|build\three_dm_source_member_geometry_smoke-1\e146b4685c17421cb3b23b35a219d7e6|
|source_new_target|27/27|build\three_dm_source_new_target_smoke-1\77392db4b7094072a47377a60f8a824b|
|new_blocks|29/29|build\three_dm_new_blocks_smoke-1\1c6089335ede4f349d6a915474b54ae6|
|new_member_instance|20/20|build\three_dm_new_member_instance_smoke-1\0e23c9ffe97a429aa3892c650cbdb21e|
|reference_edit|18/18|build\three_dm_reference_edit_smoke-1\21bd66aeb73849fb9fb4b0be4970b474|
|independent_member|18/18|build\three_dm_independent_member_smoke-1\a12226c0afd94a9b920f8a48c3ac474b|
|shared_proxy|10/10|build\three_dm_shared_proxy_smoke-1\ac9df2274efa41dbbdc514538cc75c9a|
|shared_proxy_geometry|8/8|build\three_dm_shared_proxy_geometry_smoke-1\d70a3cc323d54932a11fab0f553e0fe6|
|retained_member|6/6|build\three_dm_retained_member_smoke-1\3d001e48f4f54c4eb2b6725c6b8f36f8|
|definition_placement|12/12|build\three_dm_definition_placement_smoke-1\bef9c93a84f14599b1d182bb2a3ac648|
|new_geometry|20/20|build\three_dm_new_geometry_smoke-1\8636828e461342a8b51626b6bf69818d|
|preserved_export|13/13|build\three_dm_preserved_export_smoke-1\8d2a99d438054f28b8da11e2d8618b02|
|member_overlay|12/12|build\three_dm_member_overlay_smoke-1\91423d461d2f4c0bbac49068b6fb664b|
|block_structure|23/23|build\three_dm_block_structure_smoke-1\72eeafdc64b8421a8e2a98b16f5d367b|
|affine_refresh|10/10|build\three_dm_affine_refresh_smoke-1\23c99b6ab3bf4340af8ee025a1c2acd3|
|affine_refresh_reopen|2/2|build\three_dm_affine_refresh_reopen_smoke-1\a953872d3b8a4de499746c6ba46148fd|
|source_signature|14/14|build\three_dm_source_signature_smoke-1\530893c0ccae4b71b31287ac74c551db|
|preservation|19/19|build\three_dm_preservation_smoke-1\6571c6aa389941f891fe9b8de0c07d8d|
|definition_overlay|8/8|build\three_dm_definition_overlay_smoke-1\62c25c0087da4105990378ce8611ec89|
|layer_overlay|7/7|build\three_dm_layer_overlay_smoke-1\dd692595570549d9863d3066e84333c4|
|mixed_shear|11/11|build\three_dm_mixed_shear_smoke-1\266b838a688c4f2987a609c1550b0abb|
|merge_bridge|8/8|build\three_dm_merge_bridge_smoke-1\a3f2fb3f54a4484489baf6f0d8ed1306|
|geometry|33/33|build\three_dm_smoke-1\97c9384be6274a28a5936cf4379701e6|

SHA-verified source/evidence checkpoint: `H:\FreeCAD-src\build\checkpoints\curve-on-surface-archive-recovery-20261007-092217.zip`.
