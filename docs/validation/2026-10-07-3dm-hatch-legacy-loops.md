# Independent Rhino5 SDK loop semantics

Continues the approved full Hatch/openNURBS plan. The unchanged SDK20130711 reader
now inventories five independent native curve classes recursively. This is
target-version semantic evidence, not an actual Rhino5 renderer acceptance or
completion of every Hatch/curve class.

## RED and implementation

The new native test initially failed with `legacy fixture boundary requires
original native NURBS` on the four-root-type archive. The independent executable
now reads ON_NurbsCurve, ON_ArcCurve, ON_LineCurve, ON_PolylineCurve and nested
ON_PolyCurve by exact class identity, without modern SDK, Qt or production
conversion calls. It reports native class/UUID/dimension/domain, raw homogeneous
CVs/weights/order/knots, Arc frame/radius/angular versus curve interval, endpoints,
Polyline parameters and independent PolyCurve child/segment parameters. Known
child user text and userdata class/userdata/application UUIDs are compared too.
The old NURBS flat fields remain compatible; `native_loops` adds the complete
typed tree. Unknown/reference/surface representations still require their own
adapter and remain explicitly outside this new inventory.

Bounds remain source512MiB, report32MiB, depth64/nodes16384 and combined numeric
inventory2million. These bounds do not establish broad performance/hostile-input
acceptance. SDK source is unchanged. The reader still declares SDK201307115 and
no gradient API. Existing explicit attribute-owner repair and raw-loss evidence
remain separate from unassisted Rhino application behavior.

## Exact native cases

Twelve complete archive paths, six each in mm/cm, compare the independent decoded
tree and the modern reread of the independent v5 rewrite against native source:

1. Original four native root classes, including Line/NURBS/Polyline children.
2. Selected normalized millimetre output.
3. Current CV/weight/knot/domain/radius/point/child edits and own translation.
4. Zero movable base point with all four root classes and unchanged typed loops.
5. Independent copied native members inside a new block definition/instance.
6. Canonical current members copied into a new block definition/instance.

Definition/member UUIDs and instance matrices must match across the legacy graph.
Source bytes stay immutable; output remains version5/50 with the original output
units. Existing20 independent Hatch/pattern/affine/shared/builtin cases now also
compare the recursive typed tree, retaining their raw attribute-loss and gradient
probes. No threshold was relaxed to accept legacy data.

The first cm expectation incorrectly used host-normalized current fields against
the raw cm reader. The diagnostic isolated scale20 versus2 and base12.5/25 versus
1.25/2.5, with exact loops. The oracle now reads scalar fields directly from the
native source model in its archive units; normalized outputs are tested separately.
This fixes the oracle's units rather than normalizing away a native discrepancy.

## Host acceptance

The84 new FreeCAD assertions exercise all four root types after native loop edits,
raw independent decode, native legacy rewrite/reread, metadata, pattern references,
zero base point, a block with an added native inner Arc, and an independently
edited copied family in both source units. Checks require exact raw native fields
and graph identities rather than only object counts or bounds. The previous
ordinary current fields, lifecycle/copies and two immutable user-ring suites remain
in the applicable full regression run.

## Remaining full scope

Typed ordinary editing controls, CurveOnSurface and PolyEdge/reference adapters,
safe plugin/reference remapping, broader subtype/order/Arc-frame/precision and
version cases, native pattern content/shared identity and clipped fill/dash display
remain pending. Actual Rhino5 rendering/roundtrip remains pending, including
nonzero line bases/Plus and target-incompatible gradient behavior. All128 classes,
16 component categories and6 document categories remain in scope and class-wide
completion stays false. The32 named native legacy Hatch archives are evidence for
their exact fields and routes, not universal compatibility.

## Reproduction

```powershell
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-rhino5-reader.cmd
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-legacy-loop-tests.cmd
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd
rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-hatch-legacy-loop-regressions.py
```

## Verified final state

Fresh native20/20,41.43s; isolated GUI1239/1239 across50 suites, final process0.
Nine source/installed scripts, both runtime binaries and the independent legacy
reader remain stable throughout the GUI run. Both immutable user rings pass.
FreeCAD App and Rust are unchanged, retaining previous556 pass/2 skipped/7 disabled
and75/75/fmt evidence respectively; no fresh App or Rust result is claimed.

|Suite|Checks|Result artifact|
|---|---|---|
|hatch_legacy_loops|84/84|`build/three_dm_hatch_legacy_loops_smoke-1/3c0f5236b1914ca1a3b0841e07b34930/results.json`|
|hatch_loop_baseline|4/4|`build/three_dm_hatch_loop_baseline_smoke-1/e5aa1968391241a7b115671b982904e6/results.json`|
|included_file_copy|8/8|`build/three_dm_included_file_copy_smoke-1/108c8676ccb4415e9dc7970854cd3d5c/results.json`|
|hatch_loop_edit|132/132|`build/three_dm_hatch_loop_edit_smoke-1/6acb606cd5d4485e8d4ada423372b679/results.json`|
|hatch_shared_loop_edit|12/12|`build/three_dm_hatch_shared_loop_edit_smoke-1/d8a58df55d11453eabc18de62c3f9e4e/results.json`|
|hatch_loops|54/54|`build/three_dm_hatch_loops_smoke-1/2119f28ec3d14b6e8a92f5bd922afed3/results.json`|
|hatch_legacy|24/24|`build/three_dm_hatch_legacy_smoke-1/a3d3113c85704de88b83cba291f025e0/results.json`|
|hatch_current|48/48|`build/three_dm_hatch_current_smoke-1/9c5fc258bbcb4e03a2fc3bc9e8e23a95/results.json`|
|hatch_shared_current|16/16|`build/three_dm_hatch_shared_current_smoke-1/db020fe4fc2849838c895d541d024fde/results.json`|
|hatch_affine|24/24|`build/three_dm_hatch_affine_smoke-1/460b1b9c3ac74ed38b5e1a7a4f48c7e7/results.json`|
|hatch|16/16|`build/three_dm_hatch_smoke-1/4de0fa39da6c442aa19c9bff363885cd/results.json`|
|cloud_geometry|50/50|`build/three_dm_cloud_geometry_smoke-1/6a7b60a57a8d47f0a63fbc6a8979001d/results.json`|
|point_cloud|43/43|`build/three_dm_point_cloud_smoke-1/f1cce6768d0149168c681fc59d4e268f/results.json`|
|text_dot|34/34|`build/three_dm_text_dot_smoke-1/73769c02381048699b25ee61707016b4/results.json`|
|selected_member|33/33|`build/three_dm_selected_member_smoke-1/529eb3d4eb024f3c91c874f8d3922b30/results.json`|
|retained_transform|25/25|`build/three_dm_retained_transform_smoke-1/889334f93a6c460cb88ecd05ec26235f/results.json`|
|host_upgrade|39/39|`build/three_dm_host_upgrade_smoke-1/12835375f1454227bf98b99337077367/results.json`|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/7b3f86bf236245f2ad151dc649ffacd2/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/04a40cfd844e4212adb1ea4cb1ca6b25/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/4e7dac2ccfe4486dac657a8dcb22322a/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/d53140b89b5d4e6aa5055dfa6c7f8d56/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/23fcbc10d4e84065b681d46521c6b88d/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/706601625cce4282b1090d3ecafa61d5/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/dd9d4d2a0a2a44af86d46b2d55b984c9/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/78ab261f0a2049dd8aa416c6eda0cce8/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/20b1601ed0ae425282c19fafe6e0c7c2/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/cb5f09fb6fd24fb580f30a35e1cfe55f/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/929fb6b685a540cea67416430199cbea/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/dfc0eca3d54c4bf89f87d8bd79179961/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/cf694f98616e4b3d9fdcbc13de500cc8/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/3b8ea9d0321b45dfa62a435830de596c/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/87b34d1134be4ad599057c7a6e731029/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/b1f873e483834206b99af8ec82c26923/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/36f973d5a49341f799dafb0e7b100bf3/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/c3a0c3267c474d9bb96d3278c0d871e2/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/7e0ad36f79f54261b2723019487754ca/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/b13c2e47f3e54e6ab05713973e6e0926/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/613851ee2ca749eba9cbe23d4acf5f75/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/1097149c15194e00a09cf02daa49c1a1/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/1cf299bb585a4d45beaf473398f76b82/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/e554f616c51349a9ac8e0494758fe508/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/32f662276bbe42309dba30feb17bb6d3/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/2a51085305e94feb8f317154958fb169/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/0a7ba32856a641559abcedcf86a31737/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/f0ef375be73c4d9d8a7459486198afb0/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/f6ee8a0951864e69bc9e4a2c49997ebe/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/8749873457944222ae3e9a2f82c4ce53/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/2ba3ef8f2dac45e2b2dc9699d14c9f28/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/f872be38ef554f2d91f9523250adc5db/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/30c3e39cb2774b1fbde23d14650ba3a4/results.json`|

Checkpoint: `H:\FreeCAD-src\build\checkpoints\hatch-legacy-loops-20261007-074303.zip` (SHA256 verified).

Legacy reader SHA256 `33313304ed32ec2434129e067beda95bf0ec204bc67e63616e95eec83ff0f873`.
