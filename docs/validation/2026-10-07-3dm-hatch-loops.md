# Native Hatch curve inventory and child-data guards

This is a prerequisite for the approved loop editor. It does not complete native
loop editing, pattern rendering or full openNURBS support.

## Native representation proved

`hatch_loop_native` records loop role, exact class/UUID, dimension, domain and
recursive child strings/userdata identities. NURBS exposes homogeneous CVs,
weights, order and knots; ArcCurve exposes its plane/radius/angle interval;
LineCurve exposes endpoints; PolylineCurve exposes points and parameters;
PolyCurve exposes each original child and its independent segment parameters.
There is no NURBS conversion or mesh replacement. Geometry inventory is bounded
to64 nesting levels and16384 nodes; combined CV/knot/polyline arrays are capped
at2000000 numeric entries. SDK validation still
precedes this traversal; a valid native tree exceeding64 inventory levels is
explicitly rejected in the native test. This is not a hostile-input/performance audit.

Eight native mm/cm roundtrips preserve NURBS, Arc, Polyline and nested PolyCurve
root loops, including LineCurve/NURBS/Polyline children and their independent
domains. Scalar edits preserve the original native curve tree and metadata.
FreeCAD54 checks cover derived boundaries for all four root types, current scalar
fields, placement, independent copies, copied block families, FCStd after source
deletion, retained unknown child data and atomic export rejection.

## All nine concrete curve registrations reviewed

|Class|Observed representation and remaining work|
|---|---|
|ON_NurbsCurve|Typed homogeneous CV/weight/knot/order/domain inventory; editing pending.|
|ON_ArcCurve|Native arc plane/radius/angular interval and curve domain; editing pending.|
|ON_LineCurve|Native endpoints/domain as PolyCurve child; a nondegenerate line cannot be a closed root.|
|ON_PolylineCurve|Original points/parameters/domain; editing pending.|
|ON_PolyCurve|Nested child classes and independent segment parameters; editing pending.|
|ON_CurveProxy|SDK Write/Read return false; referenced curve/domain/reversal needs explicit applicability handling.|
|ON_PolyEdgeSegment|Writes object/component references and edge/trim/proxy domains; test decode has no resolved ProxyCurve and dimension0. Reference resolution/remapping remains pending.|
|ON_PolyEdgeCurve|Derives from PolyCurve but owns reference segments; exact-class gate rejects it rather than treating it as ordinary PolyCurve.|
|ON_CurveOnSurface|A valid dimension2 NURBS surface yields a valid closed dimension2 Hatch loop in the SDK probe. Its parameter/optional spatial curve and surface need their own adapter; it must not be excluded from scope.|

Pinned local primary sources: `opennurbs_curveproxy.cpp:323`,
`opennurbs_polyedgecurve.cpp:780`, `opennurbs_curveonsurface.cpp:235`, and the
corresponding public headers. Nine class UUID registrations are saved in the SDK
probe JSON. Registration/limited probes do not prove class-wide compatibility.

## Fixed missing guards

The new native RED test demonstrated that scalar Hatch editing accepted nested
plugin userdata. Current fields and native transforms now inspect every supported
curve node. Only the exact ON_UserStringList class is accepted; opaque plugin
dependencies and unresolved UUID-looking child strings produce explicit errors.
Selected unchanged export also runs the native current-field gate before writing.
Both mm/cm destination sentinels remain unchanged on failure. Source snapshots and
the typed inventory stay retained; source bytes are immutable.

The named gradient userdata UUID is checked even when GetGradientType is None.
Setting the type to None does not remove its data, and version5 cannot retain it.
The adapter no longer advertises ordinary safe current fields for that case.

## Investigation rulings

The first mixed fixture used an rvalue ON_PolyCurve. Pinned ON_PolyCurve's move
constructor passes its owning ON_CurveArray through std::move, while that class
declares a destructor and no move constructor; its implicit copy copies pointer
values. The destroyed temporary left dangling child pointers. The fixture now
constructs the owning PolyCurve in place behind a unique_ptr. SDK source is
unchanged. Broader move/ownership integration audit remains pending.

ON_UserData increments m_userdata_copycount on native copies. This generation is
SDK lifecycle bookkeeping, not a stable semantic identity. The inventory compares
class/userdata/application UUIDs and actual known strings; plugin payloads remain
blocked. Existing native v5 curve SHA and complete payload digests continue to
validate the archive representation. No semantic curve/metadata assertion was
removed to pass a failing roundtrip.

## Remaining acceptance

Typed loop editing, loop addition/removal/topology, all reference curve and child
surface adapters, safe plugin/reference semantics, editable pattern resources,
clipped fill/dash rendering, and independent SDK2013 tests beyond NURBS fixtures
remain open. Actual Rhino5 rendering/roundtrip, nonzero pattern bases/Plus and the
remaining128-class/16-component/6-document scope remain open. No reusable skill
business rule, public integration or final comprehensive review was changed.

## Reproduction

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-hatch-loop-tests.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd`

`rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-hatch-loop-regressions.py`

## Final verification

Fresh native18/18 and isolated FreeCAD999/999 across45 suites, final process0.
Source/install9 scripts and isolated binary stable throughout GUI verification.
Rust unchanged at prior75/75/fmt evidence. Both user ring suites pass.

|Suite|Checks|Result artifact|
|---|---|---|
|hatch_loops|54/54|`build/three_dm_hatch_loops_smoke-1/ac2da2e7838d40eca5a7d98b88485b31/results.json`|
|hatch_legacy|24/24|`build/three_dm_hatch_legacy_smoke-1/0380b90bfc1f4b22a02e89e35e8d05df/results.json`|
|hatch_current|48/48|`build/three_dm_hatch_current_smoke-1/74b4dbc45e21408ca2eb4ef60460b748/results.json`|
|hatch_shared_current|16/16|`build/three_dm_hatch_shared_current_smoke-1/76ae0184dbb1434984ad69b51f461fc4/results.json`|
|hatch_affine|24/24|`build/three_dm_hatch_affine_smoke-1/759fd75fc07a4123b02f952964eac071/results.json`|
|hatch|16/16|`build/three_dm_hatch_smoke-1/68c4ce0915694c11ba1861bc38843420/results.json`|
|cloud_geometry|50/50|`build/three_dm_cloud_geometry_smoke-1/8f4cf107281c4fd8b2f417b364c39265/results.json`|
|point_cloud|43/43|`build/three_dm_point_cloud_smoke-1/e418a048926841fb8e1e7e530c09a1b4/results.json`|
|text_dot|34/34|`build/three_dm_text_dot_smoke-1/24bb7eafe53f4b55bc9d7dab0540e00d/results.json`|
|selected_member|33/33|`build/three_dm_selected_member_smoke-1/f149d696d4d84664a95ea9a7f6817fc1/results.json`|
|retained_transform|25/25|`build/three_dm_retained_transform_smoke-1/7e347a2ce8624ef48b6cc11efa3e48de/results.json`|
|host_upgrade|39/39|`build/three_dm_host_upgrade_smoke-1/979cc33a3ece4814a5482ee3886cf557/results.json`|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/4f93a1f1faad42209fa2be7525f2aeef/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/27da4bb25aa24c6cb43b9f5d40c5b178/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/b2f00deb18684ca5abe3c2701b23ea34/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/c5b490e4afd746adbfb222b512c37364/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/5a4951192d1942148b69e5e25516f89c/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/3d45134e5e484d668bde10ed6d59e9ef/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/fcec2ae7e6fa4b9d993ba31fc1296b75/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/0057c12733084cc1992cfddf63944492/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/9beb8ab030cf4663981e7492700e2869/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/711de842cfb44f3cbc8d16f9f68f37ce/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/2e71a512ab1445c6bfd7cb92aac43d25/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/c8e9c5324bab460388dd7d3c8915787e/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/9d53a1237c354db88810ba9bbdf9637b/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/35bfd6a6081347b49810ee5dd3806da3/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/bf02a16639ac4b96a1ddc3cf6f17b592/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/86e872dac23f45ed92b58bfa9d212021/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/14cb07dc92044c609bf33c2f3bf9110e/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/fd5c487e6f884a5bb54f474fff4b2ebf/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/c10400b0b908469ab76f2b91a05370d1/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/05ccd890d44b41baa413c4f66ae37aef/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/51e4ddf6ebba4c0d925316c45074a529/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/6069998c7a8942e48cea45cbf3f3e255/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/cff03d4b70254ef6b17708410870dbef/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/ccd87c63ef77435e93edc89024755f0c/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/dd5ff5ade47f430086227242e0a0a2f7/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/d119d736af9a4ece8a36886f5b4aabaf/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/26d1deb0ef814d7fa576f002c9994f3c/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/1c2b500a834c4b04b97361338115aa01/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/8d2be04a1b1642ada6a97f9019687f5b/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/4c699f31289b43fdbfc2e8dae2c349bf/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/626fd976b3d94cc0baec9b6d411554bb/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/120cd9df3b014d4384fa5092c4cfebc2/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/eb1aba9ee0164e7fa86db7992210862c/results.json`|

Checkpoint: `H:\FreeCAD-src\build\checkpoints\hatch-loops-20261007-064833.zip` (SHA256 verified).

Current isolated binary SHA256: `06b014b1f37b8cd8fd1c83cffc34614085d3baa17e844525543f46af29d22955`.
