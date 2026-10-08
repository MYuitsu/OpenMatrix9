# Verified source members in new native 3dm definitions

The development module now creates new definitions from verified source-backed geometry and existing source instances, in addition to the preceding unsourced geometry/new-instance routes. Full openNURBS remains in progress. This package is tested in the isolated runtime `H:/FreeCAD-src/build/3dm-preservation-sdk`; it has not been integrated or pushed to the public primary checkout. Actual Rhino5 application acceptance remains untested.

## Behavior and protocol

`ThreeDm.create_definition` verifies archive ownership, included snapshot/hash, exact native record and source baseline before its single Undo transaction. It retains original source UUID/record provenance and allocates a separate persistent creation identity for the promoted native member. Actual parent placement is retained when moving a member from an existing definition. Retained geometry without FreeCAD Placement gains an explicit editable OM9BlockMemberPlacement rather than an invented source Placement. The verified TextDot fixture retains text and native attributes; broader retained-class/display coverage still needs evidence.

Unchanged source payloads use an additive strictly boolean member-copy policy `follow_canonical=false`. This clones the original normalized native geometry/attributes rather than following a selected canonical source's later geometry replacement. Promoted members can therefore coexist with original source selections without applying world placement twice. The default `true` continues to follow canonical overlays for existing proxy/dependent-copy routes. False plus independent_overlay rejects as ambiguous, and malformed policy types reject atomically. Both top-level source geometry and original definition-member geometry can be copied into fresh definition-member records.

Promoted member copies receive deterministic separate creation-scoped output IDs. Supported geometry edits stage current local CAD/mesh. Existing source instance members stage their current native target/matrix, including discovered new target graphs; copies retain separate stable native reference IDs. Shared dependencies remain verified current dependencies: original native geometry retention does not claim a whole-component-graph snapshot. Empty new definitions retain their owning source namespace after all members are removed.

Selecting transformed retained geometry alone currently rejects with an instruction to select its placed block. The selected-root protocol lacks a retained native-transform action; the guard prevents silently exporting stale source coordinates. Unsupported payloads, ambiguous/foreign ownership, altered native provenance and invalid inputs continue to reject before destination replacement. Source-member proxy creation and explicit legacy migration remain pending.

Usage: [structural block API](../development/3dm-structural-blocks.md).

## RED to GREEN

- The original source-member creation macro rejected source inputs. Native tests then exposed unsupported promotion of top-level source records. The host creation/binding/copy request and native role/canonical-follow policy now pass native and GUI reread checks.
- A failing host fixture demonstrated that a transformed retained member selected alone exported its original source coordinates. The new explicit guard rejects that request and leaves an existing destination byte-identical.
- Native test oracles compare semantic TextDot text through ON_wString, and actual reread native points/instance matrices. Geometry-only read intentionally rejects retained TextDot; it was not weakened to make a mixed fixture pass. Point CRC alone is not a payload oracle: explicit reread world coordinates verify the promoted and selected representations.
- During GUI testing, concurrent Surface work overwrote the shared SDK binary/scripts. A separate module runtime now isolates OpenMatrix9 outputs while reusing the matching FreeCAD SDK. Final source/install bytes match and the binary hash remained unchanged throughout the GUI regression. Other tasks and their SDK build were not interrupted.

## Validation

- Native merge target build/run exit0. Fresh full native build/CTest8/8 exit0,29.19s, including both immutable user ring geometry fixtures. These ring geometry tests do not establish complete opaque-plugin/resource preservation.
- Isolated SDK module build exit0; final scripts refresh exit0. ThreeDm.py, ThreeDmArchiveState.py and InitGui.py installed bytes match their tested source.
- FreeCAD359/359 checks across24 suites; every owned final process exit0. New source-member suite23 checks and source BRep/mesh suite8 checks cover world/local placement, original source and new-member co-selection, native TextDot payload/attributes, metadata edits, current CAD/mesh edits, reflected scale, copied member/instance identities, new target discovery, closure omission, empty owned definitions, FCStd with source removed, Undo, invalid creation and atomic refusal. All328 preceding GUI checks pass on this final code.
- Rust archive core/tests are unchanged by this package and match their prior ledger SHA256 fingerprints. Prior Rust74 tests/18 suites and fmt exit0 remain applicable; they were not rerun solely to repeat unchanged passing checks.

|Suite|Checks|Artifact|
|---|---|---|
|source_members|23/23|`build/three_dm_source_members_smoke-1/ed79a6180e034607b5b06d7fb486da4c/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/84ebfaa244be44b78b63518cec7b363d/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/d06aa9ab2e37443aa2381df31e5a9d42/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/ff5acae3e6d64d13a1f816b6b0be42fa/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/8f0579f6e599427a9096719d3f891535/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/b42b05030d114a6392b66b33c093cf22/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/7631bc6c08614cf5bbb82a92d70d0373/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/79e9ce6b6b6c434481965482d8d36ffa/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/d9002c523dfe42be93bc02488c9c45ba/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/4ea8036b2d4b4b0392f1954836a642f4/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/a9fb34bd869b443fac76c48797516f52/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/52f44e0f5d5340ec9b116f58f06451f1/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/5beb96221fba4ef4a31ea69f19d5a139/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/2cac7e700bd34203bb31db9b5ff56e31/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/e09eaf3272d04b6a9464112653235926/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/d5e1bb43ef3140e199a9451179436ba8/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/43833ca909194b9e842c0bdecdb679c1/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/d6329d01119d44eb82773464ca4d70f6/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/63512b163aa54ed2a6ddd074fcb808f3/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/b87179004d1d4287b4ccdc6231d41e15/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/cf36c5f026414206ac2dfbc6befde908/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/03af77b9bf1e4bf78c35692ea52ef26b/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/d31c2869165f4ae5a659b9955a8913f8/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/880d1e9bd40348d2bc2647ae9aaf42e6/results.json`|

## Remaining full-goal scope

Copied whole-definition families, manually inserted member identity/observer cases, source-member proxies, explicit legacy owner/source/proxy migration, different-source document metadata policy and shared standard/menu/CMD preservation routing remain pending. Retained source selections with transforms need a dedicated native action and acceptance rather than bypassing the guard. Wider class-specific retained display/edit/target-version fixtures remain pending.

Appearance, annotations, resources/settings/history, safe arbitrary plugin references, full native class/version coverage, final whole-package review, public integration and actual Rhino5 application acceptance are incomplete. Derived display fingerprints remain12 significant-digit tokens; sub-resolution changes and large shared-graph performance need wider fixtures. The development checkout's Git registration remains absent; tested source is preserved in verified checkpoints.

## Fingerprints

- `Gui/ThreeDmMerge.cpp` SHA256 `1e7c1cc4c71704776432752a72a3e47647bf763a3a0f1c3f6d025694e68107a9`
- `ThreeDmArchiveState.py` SHA256 `8aa657087ae7349e2f4f4d4d518773e0d7d1cc4b2542ea1ad1e20182a2d373e7`
- `ThreeDm.py` SHA256 `4e500b366459255e50199e69c03ae6b61f99828bdf178f85916e7ba3a20744c6`
- `tests/native/three_dm_merge.cpp` SHA256 `bdbdfd2c344bf24acbd8cf3ae642ff500977e52b4c1f3e0465adfe6ef50ac965`
- `tests/three_dm_source_members_smoke.FCMacro` SHA256 `cc83d0149e8d77cd5c99d2c36a065c0b440975858af6e065d6985627562914a8`
- `tests/three_dm_source_member_geometry_smoke.FCMacro` SHA256 `a10963ebe73958dc5121756d2857052de14e1bcebcde4ab55d6182733990a1b9`
- `docs/development/3dm-structural-blocks.md` SHA256 `a2c605d4be27b4caf93c8291dc7130e549390cb1837f0f0a51a45484e8059750`
- Isolated `bin/OpenMatrix9Gui.pyd` SHA256 `3f177dc09dc7d4ab065a404979306fe6b8009ac23dcdab48b5c5ed655b61b221`
