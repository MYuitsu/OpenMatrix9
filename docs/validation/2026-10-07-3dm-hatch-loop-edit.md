# Native Hatch loop editing and embedded current data

This implements a bounded native editor and persistent FreeCAD script adapter for
five independent curve classes. It does not complete Hatch or full openNURBS.

## Implemented representation

`hatch_loop_current` carries schema1 and complete typed loops in millimetres.
Original child identities are referenced by immutable source indices. New loops
and children explicitly use null provenance. The staged editor constructs exact
ON_NurbsCurve, ON_ArcCurve, ON_LineCurve, ON_PolylineCurve and recursive ON_PolyCurve
nodes, preserving safe original child strings/userdata identities. The source
curve class cannot change implicitly. Rational CVs retain raw homogeneous weights;
knots/order/domain, Arc plane/radius/angular and independent curve domains,
Polyline points/parameters and PolyCurve child/segment domains remain distinct.

The loop overlay precedes the existing scalar plane/base/rotation/scale/pattern
overlay and own placement. Native export/reread refreshes the typed current
inventory. Derived OCC geometry validates boundaries and drives preview only;
the staged native tree remains export authority.

The native topology checks require closed finite2D loops, positive nondegenerate
areas, no self-crossings/retraced branches or crossing/touching loop boundaries,
and alternating outer/inner containment. Separate outer regions and outer islands
inside holes are supported in the named tests. Bounds: payload32MiB, depth64,
nodes16384, combined numeric budget2million, topology1024 boundaries and1024 knot
spans per loop, absolute geometric tolerance1e-7mm. These bounds are not a broad
performance or hostile-input audit; extremes and other representations remain open.

`ThreeDmHatch.loops(obj)` returns a detached current payload;
`update_loops(obj, payload)` runs native clone/topology preflight before any host
property change. Call it inside a document transaction to group Undo/Redo.
Historical storage dependency below is superseded by
[plugin-owned storage on stock core](2026-10-08-plugin-payload-storage.md).

OM9HatchLoopFile was FileIncluded, with host schema and SHA256 checks for the
current file and native baseline. A fresh temporary file is assigned on every
accepted update. Old documents without a typed baseline retain their scalar
adapter. A user-facing typed loop editor is still pending; this API is not a
completed ordinary editing workflow.

## RED tests and repairs

- Native loop payload was initially rejected as an extra scalar field. The new
  optional typed overlay now passes eight mm/cm subtype edits and exact reread.
- Closed bow-tie loops were initially accepted. Native staged topology now rejects
  them without mutating the input. Added holes/removal, disjoint outer regions,
  nested islands, outside holes, crossing boundaries, wrong roles, empty loops,
  zero rational weights and source class substitution are checked separately.
- A second affine edit of a valid mixed PolyCurve was rejected by the OCC
  self-intersection oracle. Its overlap endpoints were identical parameter pairs
  (113.222222/113.222222 to125.444444/125.444444): the trivial same-parameter
  diagonal. That branch is excluded, and all overlapping knot-span pairs are
  checked independently. A new retraced-line test proved diagonal exclusion alone
  was insufficient; that test and bow-tie rejection now pass with successive edits.
- The first host test failed because no embedded loop file existed. Current
  fields, native export, metadata, Undo/Redo, invalid edits and file/schema/baseline
  rejection now run through the actual FreeCAD process.
- Editing an ordinary document copy initially changed the original loop file.
  FreeCAD PropertyFileIncluded::RestoreDocFile borrowed an existing read-only path,
  although each property deletes/replaces its file. The core now gives that restore
  a distinct owned path and reads the archive entry into it. A standalone eight-
  check host test covers same/cross-document copy, editing, Undo/Redo, closing the
  other document, deleting a copy and independent FCStd values. This core patch is
  required together with the adapter; an older stock FreeCAD build has not been
  established as safe for these new mutable included-file properties.

- The proposed-overlay boundary path initially stopped comparing the host record
  with the owner manifest. A new mm/cm test demonstrated that an altered host
  record could reach preview and loop update. The path now repeats provenance
  validation before native preflight and mutation; both boundary and edit reject
  without changing the included file. The first complete GUI attempt was not
  accepted as final evidence because this last source change invalidated its
  runtime hash. The final full run uses the frozen corrected scripts and binaries.

## Remaining acceptance

CurveOnSurface and PolyEdge/reference adapters, supported plugin data/reference
remapping, typed ordinary editing controls, native pattern content/shared identity,
clipped solid/line/dash fill rendering, independent SDK2013 non-NURBS edited-loop
acceptance and actual Rhino5 rendering/roundtrip remain pending. Class-wide
completion is false; all128 classes,16 components and6 document categories remain
in scope. Existing SDK2013 Hatch cases are NURBS-only evidence and do not prove
this entire new editor. Rust is unchanged; earlier75/75/fmt evidence is historical.

## Reproduction

```powershell
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-included-file-core.cmd
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd
rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-hatch-loop-edit-regressions.py
```

The runtime FreeCADApp.dll was a pre-existing hardlink to the primary build and
a private-preparation runtime backup. Rebuilding the core updated those linked
binary paths. They cannot be treated as independent immutable binary snapshots;
source checkpoints and recorded hashes are the reproducibility record. The public
OpenMatrix9 checkout and SDK source have not been integrated or changed here.
The original core source is saved as `build/PropertyFile-before-loop-copy.cpp`.

## Verified final state

Native19/19,41.65s; isolated GUI1155/1155 across49 suites, final process0.
FreeCAD App556 pass,2 skipped,7 disabled,0 failures. Both user ring suites pass.
Nine source/installed scripts and both runtime binaries remained stable throughout
the complete GUI run. Rust unchanged; no fresh Rust result is claimed.

|Suite|Checks|Result artifact|
|---|---|---|
|hatch_loop_baseline|4/4|`build/three_dm_hatch_loop_baseline_smoke-1/9aef41ad7aba467fb6524bf67d3acf04/results.json`|
|included_file_copy|8/8|`build/three_dm_included_file_copy_smoke-1/1ac601df325942e295fc4538be72df07/results.json`|
|hatch_loop_edit|132/132|`build/three_dm_hatch_loop_edit_smoke-1/0a6c0a88786f4c7180e9dcbfdbf4f347/results.json`|
|hatch_shared_loop_edit|12/12|`build/three_dm_hatch_shared_loop_edit_smoke-1/76881728fc724940a78351101170baf2/results.json`|
|hatch_loops|54/54|`build/three_dm_hatch_loops_smoke-1/e573741ca06049d892174c56e68bdd80/results.json`|
|hatch_legacy|24/24|`build/three_dm_hatch_legacy_smoke-1/3bd7f6843b85437dbd5a7b2996644ccf/results.json`|
|hatch_current|48/48|`build/three_dm_hatch_current_smoke-1/a6abf44cab0d490ab84fb390e122626a/results.json`|
|hatch_shared_current|16/16|`build/three_dm_hatch_shared_current_smoke-1/ef5757a1365246a2a69ebf9ac02344df/results.json`|
|hatch_affine|24/24|`build/three_dm_hatch_affine_smoke-1/a5abe0ebeb6645fbaac48bd30162549a/results.json`|
|hatch|16/16|`build/three_dm_hatch_smoke-1/4439c9d9f19f46cdb083b4f7ad1c8a44/results.json`|
|cloud_geometry|50/50|`build/three_dm_cloud_geometry_smoke-1/56801e6962b64a49916c465295938df9/results.json`|
|point_cloud|43/43|`build/three_dm_point_cloud_smoke-1/f75f2ddf28424e88aae8509db39b1614/results.json`|
|text_dot|34/34|`build/three_dm_text_dot_smoke-1/d3873a3a43044acd882e624413523894/results.json`|
|selected_member|33/33|`build/three_dm_selected_member_smoke-1/f08c467c2c884a74bd340d479c72a70f/results.json`|
|retained_transform|25/25|`build/three_dm_retained_transform_smoke-1/689416b285e44a1ead998f06247bcffc/results.json`|
|host_upgrade|39/39|`build/three_dm_host_upgrade_smoke-1/73a5a7f7ff344bc08792d7b299ccb6fe/results.json`|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/3ba09dbbef3045ea9549c09a567cbbd8/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/9780c1db32f34ea9a37d3ec7a12cbeda/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/80697ecb133a4f74ba0f789fca952430/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/f0b1fdaf3d04444ea12fec377ae41979/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/9a2be48a084541269fbf468c5a410ab8/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/44811ebd77c6424ab2ce29a9c5735df8/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/1745c184df2b432582be293a3955c469/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/26c1374bc3bb489582bccd6d26df0141/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/965388fedfa54da1bb3a6f8e34827b90/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/22b21083498e480ebdcb9d665d3efb99/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/a585847ed4094edebd77049753404ab2/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/df9b8f2ce037448892209e640e6ad6db/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/674f68991607446b84b16e633daf761d/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/dd7a975663e64e699c69ebc8aec96323/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/a89497d63ec04e209b1a9a8e01f7ca6f/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/1936101e232343fb90c7a74a4354b159/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/7810d9442b394ade8027cf93b2fe402b/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/6c74043532ed4a279919db06ffa94316/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/3814d30fb3f844859a8ea855025c021d/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/529e6801eec3483a8f9291531f3e0efe/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/d1c0f28d5b574d478d8332f707dfc547/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/81f22b46b234409f9b46891da661b9b2/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/390681588d2643fbb5d03815e06ad17d/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/254798e0215e4ad69b45cbc0eebd27e8/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/3daec1b5f577498e8d8e6b5c3c595c8f/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/fd991fa6e4b14a6bad797005a828728d/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/320a243cbcde4b40bbb4d84c8c6a8bf7/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/e50f51e363224d6381ab631d4a5458c3/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/049ec3a7db91486886dc67e930278c73/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/78de3ff9a12a483f889c36d8daa2c301/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/c67cfa4048204dd5bca37effcdce3dc0/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/710c8fbb4fb24f7b86aef74a8cb06792/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/6b7cebb263fd49d38883591fbe1a8709/results.json`|

Checkpoint: `H:\FreeCAD-src\build\checkpoints\hatch-loop-edit-20261007-072651.zip` (SHA256 verified).

OpenMatrix9Gui.pyd SHA256 `b035579b765cd885707f672327488fbcbe697ac3b28d21735c0a2eafc8f82c99`.

FreeCADApp.dll SHA256 `8089f152320305a5dd1ff068cfb57c32b9bf7aab00fdb6a171867eb28f05862a`.
