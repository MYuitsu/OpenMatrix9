# New instance members inside imported definitions

Verified in the development checkout and installed SDK scripts. This extends the preceding [native block creation package](2026-10-06-3dm-new-structural-blocks.md) by accepting explicitly created new reference members in imported definitions. Full openNURBS remains in progress. Public primary source remains clean and unintegrated; no push or actual Rhino5 application acceptance is claimed.

## Supported behavior

An imported definition may include a ThreeDm.create_instance reference targeting either a new explicit definition or another verified imported definition in the same archive namespace. The writer retains that member as ON_InstanceRef and includes its new or source-backed dependency graph. Current member placement, reflected/nonuniform ScaleVector, LinkTransform and definition placement compose once. No flattening or invented source baseline is used.

New member identities persist through repeated export, FCStd and membership Undo/Redo. Copies get distinct stable native UUIDs while sharing the same definition. Removed new references disappear from selected closure. A new reference can retarget from an unsourced definition to a verified imported definition without modifying its provenance. A reference with an existing archive binding can target a new unsourced definition; the writer verifies/includes its bound snapshot context even when the current target has no archive owner. Foreign bindings/targets still reject.

Source dependency callbacks may discover further new references inside imported targets. A growing scoped member worklist stages those graphs once; native closure remains responsible for final reachability, nesting and cycles. A reference selected as both top-level and a definition member rejects atomically. FreeCAD's link graph rejects the tested direct cyclic target at property assignment; native cycle validation remains covered by the unchanged native suite. These are separate guard layers, not a claim that the GUI cycle fixture reaches the native writer.

Imported affine previews traverse current new/imported definition graphs. A shared new-instance matrix helper keeps display/export composition consistent. Creation-identified new CAD, new definitions and reference edits schedule verified affine refresh; modified preview geometry remains protected and is not overwritten by subsequent new-member edits. This changes derived display data only.

## RED to GREEN

The first new-membership test failed with Definition members must belong to the same source archive. Explicit new-reference classification and staging now distinguish a new native member from a malformed source record. Export then passed but affine preview traversal failed; preview traversal now handles new references and definitions. Later RED tests exposed missing source ownership for a previously unsourced new reference retargeted to an imported definition and an unknown document namespace for a source-bound new root retargeted to an unsourced definition. Ownership resolution now verifies the target for unbound new instances, preserves an existing bound namespace and includes its verified snapshot context.

The preview comparisons use the closed solid's bounds rather than the complete compound bounds, which also contain the source point. The initial point/solid compound comparison was an invalid test oracle; corrected tests compare matching native/exported solids and verify volumes576 and1440.

## Validation

- SDK script refresh builds exit0; installed ThreeDmArchiveState.py and ThreeDm.py match the tested source bytes.
- FreeCAD301 checks across21 suites pass; every owned final process exit0. New-member20 checks cover ordered native membership, reflected transforms/LinkTransform, new CAD/Placement preview updates, copies/shared target and stable UUIDs, imported targets and callback-discovered references, retargeting new references in both directions, FCStd, membership deletion/Undo/Redo, dual-role selection refusal, cyclic-link and foreign-namespace atomic refusal, manual-preview protection and immutable source archive.
- All20 prior GUI suites freshly rerun after the final script change:281 checks, including reference edits, independent/proxy copies, geometry/metadata/layer/definition overlays, native bridge, affine mixed shear and fresh-process reopen, lifecycle and the33 geometry/menu baseline checks.
- Native C++/Rust sources and SDK binary are unchanged in this host-script package. The preceding package's CTest8/8 exit0,29.46s (both immutable ring geometry fixtures), Rust74 tests/18 suites and fmt exit0 remain the applicable native evidence. These were not rerun solely to inflate the current host result. Geometry-only ring success still does not establish full preservation of opaque resources/classes in those files.

|Suite|Checks|Artifact|
|---|---|---|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/f4a37b036d6f4898914a19a19b85951e/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/e64c618f53f54297993bc1ea6c485b5f/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/c3f960673e0344c68f83d06a758970eb/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/ffb294d9823d4f1080b3433587916ee8/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/c88ef30ce1ec4743843ed8185013f1fb/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/2c3b52ce8d564fcca37c6e6c76d3df24/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/8c4fc0c1b5a6481dbf845fa918b7cce6/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/4241ad93f43d469097af6df577a8727c/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/f83fecc8da1d4cb193491d9f1e9cb289/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/88dab37fc48f463bafe5dd6d4d17bb0a/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/ea82b427314749429c37a002288dda1b/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/749c392770cb4eb8970a6346cd9b298c/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/3b4652f03988458a9c0a986566518cf1/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/0818fb450cd44d0c9b86a22d0f6cb50d/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/d22afd3810ac439b93d583f5fb39283a/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/92e139ea2431428b89b0489fa169ca00/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/493d0d88a04d4421955531efee20319b/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/baf4f053147946218c1cc48c6d84b3d6/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/52b2316c60a44d8d8ff13a6d43408261/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/b8bbf2bad51547469d930c83d97546b3/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/c01e253e48fc49ff98fec4fb0e84453c/results.json`|

## Remaining scope

Existing imported host references targeting new host definitions, source-backed/retained member reclassification, whole copied-definition families, manually inserted geometry identity/observer fixtures and legacy owner/source/proxy migration remain pending. New instance retargeting does not imply that every existing imported reference supports the same route.

Different-source document metadata merge, shared standard/menu/CMD preservation routing, whole-package review and public primary integration are still pending. Appearance, annotations, resources/settings/history, safe arbitrary plugin references, complete native class/version coverage and actual Rhino5 application acceptance remain incomplete. No broad coverage promotion follows from these fixture-specific checks. Refresh currently schedules verified owners when creation-identified unsourced objects change; large shared graphs and broader observer/property scenarios need further performance/integrity fixtures.

## Source fingerprints

- `ThreeDmArchiveState.py` SHA256 `06cf462cbf623d96cccdcc0a2a8fe52a66b71cd18238622e14f9952fb4142945`
- `tests/three_dm_new_member_instance_smoke.FCMacro` SHA256 `29214547487607c942aededfd93ec3497dc4df737eaab5540a6fb4c14db12109`
- `docs/development/3dm-structural-blocks.md` SHA256 `db10294cc396a995579ebf6452b3e911371f8ed68c116e3bc4e5c0006fb41698`
