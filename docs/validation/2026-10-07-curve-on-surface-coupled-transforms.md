# OM9-FILE-012 — coupled CurveOnSurface native transforms

The real native transform dispatcher now stages complete owned geometry and
transforms the surface and optional approximation together. Parameter-space UV
child identity and serialized fields stay unchanged in these parameter-preserving
paths. Full host editable/display/export CurveOnSurface remains in progress;
the selected-export child/reference gate is still enabled.

## Native implementation and SDK findings

`Gui/ThreeDmCurveOnSurface.cpp` audits exact native child classes recursively,
rejects unresolved PolyEdge/proxy/unknown children and opaque userdata, and checks
depth64/nodes16384/numeric fields2million/user string userdata32MiB before cloning.
Commit moves only the known ON_Object userdata base and swaps staged surface and
approximation pointers, without replacing the original UV child. Curve caches
are invalidated. It avoids the SDK CurveOnSurface move assignment, which explicitly
calls its destructor before accessing the object again. No new SDK source patch.

NURBS2D promotion rebuilds a dense native3D grid, retaining orders/counts/knots/raw
homogeneous CVs/weights and userdata. The pinned ChangeDimension can retain a CV
stride smaller than the new rational CV size and overlap rows. A tested fresh-grid
adapter fixes the application path without editing the original SDK.

Arc/Circle native Transform can return true under shear while choosing a replacement
circle radius. The adapter analytically requires equal-length orthogonal transformed
circle axes within the declared1e-12 relative plane tolerance; unrepresentable class
maps refuse without mutation. No silent conversion to NURBS. Plane axes/extents
are transformed explicitly even at determinant1; parameter domains stay intact.
General Plane shear/UV/type rebasing remains pending.

SumSurface dispatches each owned curve once and transforms its base vector as
A*base-translation, preserving the surface sum equation. RevSurface requires native
similarity and3D generatrix; nested curve transforms are staged separately and the
native axis/orientation result is retained. Extrusion currently requires proper
similarity and known profiles. Reflected/sheared extrusion UV mapping,2D revolution
generatrix interpretation and all remaining native parameter/type mappings stay
required. This is tested scope, not a narrower definition of the full goal.

## Fresh validation

- Native26/26, process0,48.79s, including both immutable user ring fixtures.
- Five genuine RED cases, one for each owned surface kind, failed because the
  optional approximation stayed behind after a surface translation. RED log:
  `build/curve-on-surface-transform-red-LastTest.log`.
-159 native positive paths:112 NURBS (rational/nonrational, degree1/3,2D/3D),
  12 Plane,12 Rev,14 Sum and9 Extrusion. Approximation absent/native Arc/native
  rational NURBS, translation/rotation/scale/reflection and supported shear.
-32 atomic refusals cover native Arc limitations, opaque child/root userdata,
  owning pointer alias, depth65, actual PolyEdge dependency and34MiB metadata,
  unsupported Plane/Rev shear and reflected Extrusion. Metadata guard had its
  own real RED before implementation, saved separately.
- Raw surface CV fields at1e-12 arithmetic tolerance, weights/knots/orders/counts/
  rationality/domain exact; original UV archive bytes and pointer unchanged;
  composed/approximation samples at1e-10; child/root user strings exact and model
  userdata transform matrices applied once. Version5 native write/read retains
  coupled children. No transformed five-surface independent/application claim.
- Fresh FreeCAD1479/1479 across53 suites, every process0. Native fixtures finished
  before GUI. Source/installed9 scripts, runtime2 binaries, original Hatch reader
  and separate repaired CurveOnSurface SDK2013 reader stayed stable throughout.
- Previous archive-recovery prerequisite also passes12 valid117 malformed2 deferred
  reference cases and4 separate repaired SDK2013 source decode/reencode paths;
  those independent paths establish archive recovery, not all new transforms.
- All280 original SDK2013 source files and modern original source unchanged;
  generated SDK Read-only correction matches the audited source/replacement exactly.

## Remaining full requirements

Full current-field schema, native curve/surface/reference/plugin dependencies,
owning-model reference remapping, composed preview/current editing, host Undo/Redo/
copy/block lifecycle for transformed source curves, all native UV/type mappings,
independent transformed Rhino5 reader and actual Rhino5 application acceptance
remain required. Generic retained export remains gated until those are implemented.
Full Hatch surface/reference/new-child/type/rationality/pattern/fill rendering and
all128 classes/16 component/6 document categories, resources/history/version
semantics, final review and public integration remain in_progress.

App/Rust unchanged; prior App556 pass/2 skipped/7 disabled and Rust75/75 remain
historical evidence. Source/read recovery32 FCStd checks still pass; no host native
field-edit/display/transform UI is claimed from the C++ dispatcher tests.

## GUI artifacts

|Suite|Checks|Artifact|
|---|---|---|
|curve_on_surface|32/32|build\three_dm_curve_on_surface_smoke-1\3d257e29441f425e862d84d6580660d3|
|hatch_loop_rows|103/103|build\three_dm_hatch_loop_rows_smoke-1\d17b44565e1947f5a888005df20a86df|
|hatch_loop_ui|104/104|build\three_dm_hatch_loop_ui_smoke-1\c33d007a7242472795fa46f9739c5b39|
|hatch_legacy_loops|84/84|build\three_dm_hatch_legacy_loops_smoke-1\a679b57fe4ff4c749205cfd16993ce61|
|hatch_loop_baseline|4/4|build\three_dm_hatch_loop_baseline_smoke-1\b66d991971bb42658f5430da66429766|
|included_file_copy|8/8|build\three_dm_included_file_copy_smoke-1\bc6486841f2d4cbdb187e1699d3a8920|
|hatch_loop_edit|132/132|build\three_dm_hatch_loop_edit_smoke-1\437ef6817e8c408cb8ac43ca21d7c20e|
|hatch_shared_loop_edit|12/12|build\three_dm_hatch_shared_loop_edit_smoke-1\f57dbe5446874a7e99be1c6452f07dbf|
|hatch_loops|54/54|build\three_dm_hatch_loops_smoke-1\e488dc0ddf634bc6894e9bde28db441e|
|hatch_legacy|24/24|build\three_dm_hatch_legacy_smoke-1\5d03eb718ac54013a7a727c44eb8545c|
|hatch_current|48/48|build\three_dm_hatch_current_smoke-1\2a87e8440ae349838637ebcf5ca0d355|
|hatch_shared_current|16/16|build\three_dm_hatch_shared_current_smoke-1\ba1385026bd848d08c55fdafeb2a8911|
|hatch_affine|24/24|build\three_dm_hatch_affine_smoke-1\3b5a0f4d0a464382824720c1f8e9d305|
|hatch|16/16|build\three_dm_hatch_smoke-1\f2a4fdc039614cbbbff24874baaf47ff|
|cloud_geometry|50/50|build\three_dm_cloud_geometry_smoke-1\0b481115f27a4844b261d15b2ca71e64|
|point_cloud|43/43|build\three_dm_point_cloud_smoke-1\308c8c587c1247c9939bca2d58b31a3d|
|text_dot|34/34|build\three_dm_text_dot_smoke-1\0715798b086243209de93e7ed627137b|
|selected_member|33/33|build\three_dm_selected_member_smoke-1\4c0bc2abdc5c4cbb9116c167d1a7d456|
|retained_transform|25/25|build\three_dm_retained_transform_smoke-1\dcec157fbe0a41ad8d155d01f9b863c7|
|host_upgrade|39/39|build\three_dm_host_upgrade_smoke-1\9ee09a4adac445e69747ff5a96113bef|
|shared_export_routes|30/30|build\three_dm_shared_export_routes_smoke-1\f4240e08da15457bb6ca7311dad1c529|
|recursive_shared_migration|24/24|build\three_dm_recursive_shared_migration_smoke-1\a79ba75d08c246f3bc8cd85e5bcc26f3|
|recursive_namespace_migration|25/25|build\three_dm_recursive_namespace_migration_smoke-1\4ed819d0fea14092ad8a664947d71062|
|affine_target_migration|32/32|build\three_dm_affine_target_migration_smoke-1\cafb7bf251b84c2a87f43efb5a3ec748|
|copy_origin_migration|22/22|build\three_dm_copy_origin_migration_smoke-1\288271d69dcc4d33aaf7f8e357559e39|
|legacy_migration|35/35|build\three_dm_legacy_migration_smoke-1\925990ddfd8c4381960189363205eb3c|
|proxy_retarget|6/6|build\three_dm_proxy_retarget_smoke-1\62a33aec6f054134b7862b05c90407ba|
|source_proxy_creation|34/34|build\three_dm_source_proxy_creation_smoke-1\ffd5a1c0d55547e18976469bc7351041|
|definition_copy|27/27|build\three_dm_definition_copy_smoke-1\3773810a0a6e4680ba0034f57ba265fc|
|source_members|23/23|build\three_dm_source_members_smoke-1\ab70e941dfd34b5790263392429a131e|
|source_member_geometry|8/8|build\three_dm_source_member_geometry_smoke-1\0abb70c9b98742b398a952a8b8a78d44|
|source_new_target|27/27|build\three_dm_source_new_target_smoke-1\b7218fc3d23d4e07a172c2237c6bbe0d|
|new_blocks|29/29|build\three_dm_new_blocks_smoke-1\fd994561d6dd48d5900da168fc99c3f3|
|new_member_instance|20/20|build\three_dm_new_member_instance_smoke-1\98b49e2eaf454f789520c805ccfe99e2|
|reference_edit|18/18|build\three_dm_reference_edit_smoke-1\185e864bb0a04e70a3d8a8aa157e23d7|
|independent_member|18/18|build\three_dm_independent_member_smoke-1\1a075910c548479ba373892ef7ebdd1f|
|shared_proxy|10/10|build\three_dm_shared_proxy_smoke-1\df60482d56714533b112efd718afb585|
|shared_proxy_geometry|8/8|build\three_dm_shared_proxy_geometry_smoke-1\81ca37373411437b8357b3a1d469ac5e|
|retained_member|6/6|build\three_dm_retained_member_smoke-1\b99682b3ae4a4e1a961a0ba57e94f045|
|definition_placement|12/12|build\three_dm_definition_placement_smoke-1\ea922f00166e43ebae62b538226916a4|
|new_geometry|20/20|build\three_dm_new_geometry_smoke-1\65ea4a72a897437a9ea4dd1e73bd27a3|
|preserved_export|13/13|build\three_dm_preserved_export_smoke-1\6dcff2afe6684dc8942fecff7bc0e07d|
|member_overlay|12/12|build\three_dm_member_overlay_smoke-1\e7310c32b2b944939453500fe054332e|
|block_structure|23/23|build\three_dm_block_structure_smoke-1\bc368ec614c3482bbf290a7255511a4e|
|affine_refresh|10/10|build\three_dm_affine_refresh_smoke-1\b1a4fc602bf64cf390c23cbbc9560b58|
|affine_refresh_reopen|2/2|build\three_dm_affine_refresh_reopen_smoke-1\9a74154ffadc448b856f1a016d2bd720|
|source_signature|14/14|build\three_dm_source_signature_smoke-1\f3d3408b161148ecb8a87bbf7bc01e6b|
|preservation|19/19|build\three_dm_preservation_smoke-1\16848d00e994472a97eabf6f007a1ab5|
|definition_overlay|8/8|build\three_dm_definition_overlay_smoke-1\2beecf0a52464fd09de4455b530a09e4|
|layer_overlay|7/7|build\three_dm_layer_overlay_smoke-1\b3c1673a9b404458bb3189be140bfff8|
|mixed_shear|11/11|build\three_dm_mixed_shear_smoke-1\0c11030962d1441d8d0db2571bd41dce|
|merge_bridge|8/8|build\three_dm_merge_bridge_smoke-1\b6d305cc31a240cabb9bd6743546f54d|
|geometry|33/33|build\three_dm_smoke-1\381f9800137a459299c4506414db9998|

SHA-verified source/evidence checkpoint: `H:\FreeCAD-src\build\checkpoints\curve-on-surface-coupled-transforms-20261007-095714.zip`.
