# New native structural blocks and placed instances

Verified in the development checkout and installed SDK module/scripts. The additive schema1 new_definitions/new_instances tables retain real ON_InstanceDefinition and ON_InstanceRef records; new geometry is not flattened into every placement. Full openNURBS remains in progress. The public primary checkout has not integrated this package.

## Supported behavior

The explicit host APIs ThreeDm.create_definition(document, members, name) and ThreeDm.create_instance(definition, placement, name) create reusable App::Part definitions and placed App::Links. Definition creation accepts new CAD/mesh and explicitly created nested references; it validates before a single Undo transaction and preserves existing world placements when regrouping. New instances may target verified imported definitions. The included source snapshot supplies their dependencies and current supported edits. Creation and repeated exports do not invent source baselines or mutate provenance.

Definition, member and instance UUIDs persist through repeated export, FCStd and Undo/Redo. A copied new instance receives a separate stable deterministic UUID. Selecting placed objects controls roots; definitions are not implicit whole-project selections. Empty new definitions remain real native blocks. Units are host millimeters; centimeter source dependencies normalize once before applying the new instance matrix. Native source references may retarget newly allocated definitions through the protocol; the corresponding existing-imported-host-to-new-host target route remains guarded.

Native allocation checks fresh canonical identities, host identity, names, ordered member UUIDs, role/class, namespace, reachability and finite nonsingular affine matrices. The existing Rust closure, native bbox/payload/attribute reread and atomic destination replacement apply to the resulting graph. Missing targets, cycles, malformed/unknown fields, unreachable allocations, unsupported member classes, duplicate IDs/names and depth65 reject. Depth64 passes. Same-source multiple namespaces, including a namespace with only a new root, retain separate remapped graphs.

Usage and limitations: [structural block API](../development/3dm-structural-blocks.md).

## RED to GREEN and preview cache regression

The new host suite initially lacked structural creation support. It then exposed null CAD being accepted into a definition and the legacy geometry exporter attempting an unsupported App::Link path. Creation now rejects null/invalid CAD before mutation; standard export explicitly guards structural selections pending shared routing.

The expanded mixed-shear suite exposed an obsolete raw BRep hash-invariance assertion after moving a mixed affine preview parent. Before/after artifacts showed unchanged local solid volume and mesh coordinates while OCC serialization introduced identity Location nodes and equivalent trimmed pcurve wrappers. The class8 curve serialization stores trimming bounds and its basis curve in the [official OCCT8 source](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V8_0_0/src/ModelingData/TKGeomBase/GeomTools/GeomTools_Curve2dSet.cxx).

A pre-Placement-change observer witness now permits refreshing only the derived display baseline when the old preview was already verified. Payload/child changes revoke the witness. Both edits before moving and edits after the witnessed move reject atomically. Native source geometry/matrices and their signatures remain authoritative. The existing display fingerprint's12-significant-digit token policy is not an exact native geometry oracle; sub-resolution changes, quantization boundaries and broader unobserved cache/property mutations warrant further fixtures.

## Validation

- SDK full C++ module link and subsequent script refresh builds exit0. Both installed Python files match tested source SHA256.
- Rust74 tests across18 suites exit0; fmt --check exit0.
- Native CTest8/8 exit0 in29.46s, including both immutable real-ring geometry fixtures. Merge tests cover new-only and mixed source/new graphs, nested/shared placements, empty definitions, existing native references targeting new definitions, separate namespace remapping, centimeter normalization and strict atomic invalid-input rejection. Reflected/nonuniform and shear examples verify volume576 and mesh coordinates(-4,44,66),(-8,44,66),(-7,53,66); centimeter source/new matrix world point(110,220,330) verifies no double scaling. Ring geometry success does not establish full preservation of opaque resources/classes.
- FreeCAD281 checks across20 suites; every final owned GUI process exit0. New-block29 checks include graph identities, two placements/four expanded items, volume576 and mesh world point(128,309,450), repeated UUIDs, FCStd, reimported native graph, selected closure, CAD edit1440/Undo576/Redo1440, edited imported dependencies, nested imported targets, separate copied instance IDs, preflight document immutability, single Undo/Redo, regrouped world point(1005,2006,3007), empty blocks, singular matrix atomic refusal and definition-container selection refusal. Temporary-source removal uses an owned copy, never either immutable user fixture.
- Mixed-shear11 checks include local solid volume24, unchanged mesh coordinates, exported world bounds(17,60,90), FCStd and both preview-edit ordering refusals. All prior structural, copy, overlay, preview, lifecycle and baseline suites remain green.

|Suite|Checks|Artifact|
|---|---|---|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/7a294a494ed34c0d98f89d52f3891abf/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/f598c151f18b4a60a0b56678b596ec15/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/f543f0c750484c0d8de625e984c3f044/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/08c89b96ed1b4c188b6624531a77bdc7/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/5c4d0fe7e898472ca97899999dfdee0a/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/7b8f5fb6ed7640079ff731beea217838/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/4ab0ac6af56f45899c90453d68858536/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/ac7e8b46807a4310b1019db84bc78f3b/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/c890dfcf4f874ea299ff288b66b378b4/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/552fa18557184c18abcb998a9067639f/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/7d864e2c53aa47e69dd04a478eb619a4/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/07af3ca08d04431f8286628db09c7c45/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/ec3f6b4abb29420aa38e1c737414c6e1/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/fb69ce16a0a942769d463336532f2ed7/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/bd040dc8378549eb8a3a24d3389170a2/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/c0b06652dd6c4fa8ae69b11d5680bfb0/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/71eab7edbd8c4a408f42b799e267b6eb/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/c00b42f0777f4889906807fcb05d76c2/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/313945b776cc4b66b9f77025b77936c8/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/bc5f885116e643adad23b6b639e85592/results.json`|

## Remaining scope

Existing imported host references retargeting newly created host definitions, newly created references added directly to imported definition membership, source-backed/retained member reclassification, whole copied-definition families and explicit legacy owner/source/proxy migration require further implementation and fixtures. Different-source document metadata merge, shared standard/menu/CMD preservation routing, final whole-package review and public primary integration remain pending.

Appearance, annotations, resources/settings/history, safe arbitrary plugin references, complete native class/version coverage and actual Rhino5 application acceptance remain incomplete. No broad coverage-row promotion or full-support claim is made from this fixture-specific package.

## Source fingerprints

- `Gui/ThreeDmMerge.cpp` SHA256 `1c01b077f3c8edcf6f20d6c7610e1ed9984e1d403e33b9bc496632a76a4cec2a`
- `ThreeDmArchiveState.py` SHA256 `c50e2f61dfcddf770c01292eac40fb3661d63367b90b3b64b0c5ae7ccf3db906`
- `ThreeDm.py` SHA256 `4e500b366459255e50199e69c03ae6b61f99828bdf178f85916e7ba3a20744c6`
- `tests/native/three_dm_merge.cpp` SHA256 `d4c3e6496dd9d9f9b3d055b082a099a47489adacffd554597b5762a417dcad3a`
- `tests/three_dm_new_blocks_smoke.FCMacro` SHA256 `712afc85e2ff702695e0a270bf938593a5fd8bbbe0acb66f4600ae819f0c84af`
- `tests/three_dm_mixed_shear_smoke.FCMacro` SHA256 `8b279f0f0943d5c3703405161019325445eb0bbcfa670dfb142a7444d3bfebfe`
- `docs/development/3dm-structural-blocks.md` SHA256 `e39f23b901a75f975382c246d55324a850a0276309f5aa0816822410a66c5e9d`
