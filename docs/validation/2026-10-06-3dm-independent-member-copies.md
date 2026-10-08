# Independent source-member copies in structural blocks

Verified in the development checkout and installed SDK module/scripts. Full openNURBS remains in progress; the public primary checkout has not been integrated or pushed. This extends the shared-proxy package and supersedes its blanket rejection of independent source-backed member copies.

## Behavior

- Source-backed BRep/point/mesh copies in existing definitions now receive fresh output UUIDs and explicit independent payloads. Their current host geometry is staged locally; it cannot be replaced by current canonical geometry merely because the copies share source UUIDs. Several copies may share provenance while retaining distinct output identities, shape, placement, label, layer and color.
- Copied nested instance members retain native ON_InstanceRef and the deep definition graph, with their own current placement/LinkTransform/ScaleVector matrix. Host definition retargeting is still guarded. Independent copies continue exporting after the canonical host member is deleted; the native snapshot supplies verified source attributes, while geometry comes from the live copy.
- Shared-member proxies targeting an independent host copy use that copy's own payload, then apply the proxy transform and metadata differences. The original canonical host may be absent. Native copy-to-copy descriptor sources still reject; host copies and proxies are staged against the original archive identity, without constructing such an unsafe native chain.
- The native member_copies descriptor accepts an independent_overlay object with action replace and BRep or bounded mesh arrays, or action instance and a finite nonsingular affine4x4 matrix. Unknown/ambiguous fields, unsupported actions/classes, malformed data, identity collisions and unreachable copies fail before replacing the destination. Existing Rust edit/duplicate policy gates remain authoritative.
- A shared native replacement converter serves canonical edits, new geometry and independent copies. Geometry user strings are transferred from the verified source record; the complete source object attributes are cloned with the new UUID/definition-member role and supported host metadata changes. Native fixtures explicitly check GeometryNote and MemberMetadata after Rhino5 reread. This covers understood user strings, not arbitrary plugin userdata/reference semantics.
- The serialized copy budget uses each independent payload rather than canonical geometry size. Existing512MiB pre-clone and post-canonical-overlay checks remain; an additional final check includes transformed geometry and copy-specific metadata before archive output. The manifest remains bounded32MiB. This does not bound total decoded process heap.
- The observer now notices copied source-member edits and resolves copies by host identity when refreshing affine previews. Canonical source identities are not mutated by export. FCStd, later geometry edits, Undo/Redo and source immutability remain tested.

## RED to GREEN

Native independent_overlay requests first failed as unsupported member-copy fields. The writer now validates and stages independent BRep/mesh/instance payloads before allocating physical native output copies. A changed canonical fixture cannot override an independently staged box: reflected/nonuniform output volume2880 is checked, alongside exact double mesh vertices and copied nested-instance world point(390,1842,4884).

The FreeCAD test first rejected a copied member as ambiguous canonical identity. Independent overlays now distinguish live host identities from source UUIDs. Subsequent tests exposed a derived BRep preview Location issue after a copied shape was restored: preview bounds(25,20,30) differed from native(45,50,70). FreeCAD TopoShape::transformGShape uses OCC GTransform, which can retain the input Location. The preview builder now strips the wrapper Location before copying, composes it into the display matrix, and resets the derived child Placement without discarding the canonical transform. Full exported/preview bounds agree after the fix.

Two test setup issues were corrected without changing product behavior: FreeCAD interprets integer color tuples as byte channels, so normalized red must use floating values; a copy of a hidden shared canonical holder needed explicit visibility for comparison with live Part.getShape output. Native output retains selected invisible geometry with its visibility metadata.

## Validation

- SDK module/scripts builds exit0; linker completed before every host suite. Installed ThreeDmArchiveState.py matches the source SHA256.
- Rust74 tests across18 suites, exit0; cargo fmt --check exit0.
- Native CTest8/8 exit0,30.26s, including both immutable real-ring geometry fixtures. The final run includes independent BRep/mesh/nested-instance payloads, source geometry/attribute user text, invalid/ambiguous independent overlays, immutable snapshot and atomic destination checks. The earlier512MiB copy-family rejection fixture remains passing. Real-ring geometry suites do not prove complete preservation of all resources/classes in those files.
- FreeCAD182 passing checks across15 suites; every final process exit0. New independent-member suite18/18 covers three different CAD payloads sharing provenance, separate UUIDs, Unicode layer/color, mesh/world-coordinate preview, FCStd without external source, later copy edit and observer refresh, Undo/Redo, invalid-copy atomic rejection, canonical deletion, a proxy of a copy, unchanged provenance and copied nested-instance native structure. The fresh-process reopen suite ran after the current affine fixture was saved.

|Suite|Checks|Artifact|
|---|---|---|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/36bb365f2c5a4697a6e99fec322985d7/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/90a6a70cc6f049d9a5188f679fe20757/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/e748de918e2b4f4db27652714eb225c0/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/54d795aadafa4b49b3f281d292cdfa94/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/1e292e80fa1d43cea79050e4e12adcd9/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/80131bd8849f4f4590eeef0511be63a4/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/d8062a36d7f04c1fbba7f73825b37feb/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/24058e94f993481ab3f44527ca0bb980/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/99492e3c90d648c6b7a0f9c84686d3d0/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/bea4c4e52f5c461f857d66568bf6d7f7/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/d0b0814bb2174c0d91c2ef524bcad227/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/5e9934b2c3c94493936f3b2b147afca7/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/3d4f8e4e20be4fbe8ab2df1cbb564067/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/a3479e1690a44f18aa2ed1d4d3e74597/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/147f523217ff4c908a60ec679e7a8c69/results.json`|

## Remaining work

New standalone definitions/reference editing and explicit legacy owner/baseline/proxy migration remain pending. Distinct-source document merge policy, shared standard/menu/CMD preservation routing, whole-package review and primary public integration remain pending. Appearance/rendering, annotations, resources/settings/history, arbitrary safe plugin-reference handling, complete class/version coverage and actual Rhino5 application acceptance remain incomplete. The full goal stays active; broad coverage rows are not promoted from these fixture-specific checks.

Independent editable copies are explicitly converted from current host CAD/mesh; original native topology/payload retention for an untouched independently staged copy is not claimed by that path. The unchanged-source preservation path continues retaining native payloads. Source-backed retained classes without a supported copy geometry/instance overlay still reject precisely.

## Source fingerprints

- `Gui/ThreeDmMerge.cpp` SHA256 `ab4813144b1811672e29058c452f833d420b9f6dd8508d2ff74ea88c7af02b50`
- `ThreeDmArchiveState.py` SHA256 `008e2944d8b52a6a231653a0c1cac2b03e18ba3eea072fd740639174854aeb0b`
- `tests/native/three_dm_merge.cpp` SHA256 `29cf5b159993014e3774cc3b68a84107dd5c4053ba8468e6c1d4d8b62dbb0d60`
- `tests/three_dm_independent_member_smoke.FCMacro` SHA256 `745040b10d46f3891e5b266c359cc4b1446fde21f4ff84d6d2e0cccf2f49a246`
- `tests/three_dm_shared_proxy_smoke.FCMacro` SHA256 `c3177edb022c56ec8b2e7f78c0d1caf71e735482d0189f30f262822b0bc79888`
