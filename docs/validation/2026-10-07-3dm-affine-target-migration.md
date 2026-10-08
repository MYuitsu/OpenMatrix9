# Explicit affine target mapping during 3dm migration

The isolated development runtime supports `ThreeDm.migrate_archive(owner, rebuild_previews=False, origins=None, targets=None)`. `targets` maps the internal Name of a supported affine source reference to the internal Name of its intended definition. The target can be an original verified definition, a legacy copied definition receiving a fresh UUID in this same migration, or an existing new definition in this namespace. An unowned new definition is scoped to the source namespace. Physical App::Link references keep their existing object pointers and refuse this serialized-target option.

`origins` still identifies original hosts sharing native source/definition UUIDs. `targets` records the caller's explicit current graph choice; original source UUIDs, native instance matrices, native records, geometry edits and placements remain intact. The planned new definition UUID becomes the affine reference's current target, in the same single Undo transaction as provenance restoration and preview refresh. No target is inferred from a duplicated UUID or a Label.

Malformed maps, missing/wrong host types, foreign namespace source/targets, colliding geometry/component UUIDs, ambiguous existing new definitions and incompatible target/status property types reject before mutation. Current native-matching display data can refresh without replacement. Manually edited/unverified previews still require explicit `rebuild_previews=True`. Every mapped preview must rebuild successfully; a cyclic/unsupported graph rolls back restored fields, fresh definition/member IDs, current target, placements, geometry payloads and replaced preview objects. Cycle errors include the object Name and actual graph failure. The temporary reference closes on both success and rejection.

## Verification

- RED: the new affine target fixture initially failed because the public API did not accept `targets`. GREEN:32 checks pass, covering simultaneous copied-definition identity migration and target assignment, mixed CAD/mesh under shear and definition/root/member placements, preview independence, native graph closure, original source immutability, Undo/Redo, FCStd and repeated migration.
- Existing new/unowned and original imported targets are accepted. Both mixed App::Part and Part::Feature affine representations have evidence; native mesh remains ON_Mesh in export while any faceted host preview remains derived display.
- Malformed/missing/wrong-type/foreign maps, source geometry/native component UUID collisions, ambiguous new identities, incompatible target/status properties and physical-link relinking refuse. The cycle fixture verifies actual cyclic failure, complete field/property/group/placement/geometry fingerprint restoration, object identity/count preservation, active-document restoration and observer/transaction cleanup.
- The rollback test initially compared document.Objects iteration order. FreeCAD restores deleted objects at the end of that iteration after abort, while names, fields, ordered Group membership and payloads are restored. The test now compares complete snapshots by Name and independently checks the actual cycle error. It does not require incidental document iteration order.
- An integer legacy preview status exposed a late TypeError during preview refresh. Migration now checks that field's string type before opening its transaction.
- Fresh native CTest8/8,42.27s, final process exit0, includes both immutable user ring geometry fixtures. Those checks do not prove arbitrary plugin/resource preservation or actual Rhino5 application acceptance.
- Fresh isolated FreeCAD515/515 across30 suites, all final owned processes exit0. All previous483 checks ran again with the final Python files/binary. Source/install bytes and binary hashes were stable before/after every suite. Cached SDK script build exit0; this Python-only change leaves the native module unchanged.
- Rust archive source/tests fingerprints match the ledger; earlier74/18/fmt evidence remains applicable and was not rerun.

|Suite|Checks|Artifact|
|---|---|---|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/72f13fac6abb459891f4dac8b018b3f5/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/a0258cbdc4f44bfc9aae615d0be334b6/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/5b469dcdfd394560aaf13b6812f5fafe/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/cd3043ef433f41f981fed43d774baf55/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/35b62d2c32b244508ad2d5fda09e4999/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/0a41cdc2d46543d9b459758449b390e8/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/d22ea468775b4651bb9bb1034ca7a7ed/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/d048b7d8e0544a5eaa5719e0e591d6d7/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/77f1fbf6be474c618355f460cb398b7d/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/a465717eeeef4c14a5d5e06b4435582c/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/8b708129129644eb89d848527a8e71bf/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/6dc8fc7797a24e8da2837c8b7252f227/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/68c6abad9206480ea203bc7d575186d6/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/5be8427d8a8e4a07973f143a251cf8eb/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/f7be3d22f8ff4280850f381a08dec1ea/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/cc494cf4bb4642f2be7fce56c2ad3642/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/48c5eb4589484f36b1058ea674ea1a9c/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/d78c3380376943f198e578e07a478aaa/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/def741a9af404343852d4f1c9044f284/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/a7dcde9bd1344c9e8b263bb7c6240568/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/bf4cbf04f07c4390a952c47071e04971/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/061fb8f564144e4594663abe347fe939/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/eeee8cb279494555b13d6899e96a5125/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/5a7f691d82c047f195c5cb2c3bf3f5e4/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/c7d7304cacf1485c985e07af3797b831/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/8e981a2d584045d0b6bb7a796ee1d7cc/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/65d29226260741a7a4bc64a1fbd813ec/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/aa7eefe5952e49878620ad40ed0a7027/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/72aa109327c14b688d66b1549116e2ab/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/78e541a245134ad597a526aa5919c32f/results.json`|

## Remaining full-goal scope

Full openNURBS remains in progress. This migration coverage uses supported simulated legacy layouts, with explicit native snapshot provenance. Recursive copies that duplicate archive owners, nested copied ownership sharing, real historical FCStd representation upgrades and missing proxy membership provenance remain pending. Unknown/opaque plugin identity references are still gated.

Linked/external resources, full appearance/annotation/settings/history/reference semantics, array/subelement handling and large graph performance need further implementation and fixtures. Different-source document policy, shared standard/menu/CMD preservation routing, final whole-package review/public integration and actual Rhino5 application acceptance remain pending. The public checkout has concurrent unrelated work; this package stays in `build/om9-dev` with isolated runtime `build/3dm-preservation-sdk`. Native copied-data limits remain per namespace, not global RSS guarantees. Derived display fingerprints use12 significant digits.

## Technical status synchronization

OM9-FILE-012 Spec v1 implementation notes, feature/support docs, README and ledger record this scope. The extension has no FEATURES catalog record or IMPLEMENTATION_STATUS.md in this package; source-derived catalog metadata remains unchanged. The extension spec's MANIFEST entry is refreshed.

## Fingerprints

- `ThreeDm.py` SHA256 `d9ba68ffd862677f9c07d12b1e3aa4cb054a605288729758280849c667cc776a`
- `ThreeDmArchiveState.py` SHA256 `9dc27eb1f4c0506371053f4c48a15c39633da5b500a2f15c9783cd61ac72fcd6`
- `ThreeDmMigration.py` SHA256 `5f7c673410758b7fe733d26264f11deff952ecdaf8e278a92c2771f4e8622dc7`
- `tests/three_dm_affine_target_migration_smoke.FCMacro` SHA256 `308078854970d0752d962f437c6251d6f9f969f966dbb5da0f11c98b0b1df1df`
- `docs/development/3dm-structural-blocks.md` SHA256 `cff629bacdcf5270c394a4eb12901026a483904cf4b29ad22e50441a0edd6b4d`
- `docs/features/OM9-FILE-012.md` SHA256 `86498d5b93e4c7f71963363e5d26cbb14a211b79bbc39e8c1511cb0892406cbf`
- `docs/features/3dm-support-matrix.md` SHA256 `2e7b19eb3e975730c33a66aa8a62ed3c1ac0582260ffdc6edc48e97ca1617c2e`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `789cacf54df95bd4fe9104f7bf5721dd824625a8ba5ab24a02558bf57b75b988`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `ff62380d9307b76a0ea60c757eed6db346beda88a63805f27eed14bd58c91c31`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `3ab5944169922a88decbeb24803852a61000d8a339136fb8e2a5cb5bb3ceeefc`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `ebb3307abd6781233b46a59eefc5a7408bf1b03c4c22edb8ee0238b79200fb8b`
- `README.md` SHA256 `375c24134b6d18a0647d5e76d697667b2049ab09d52a3dc4b4f95ffd2f3416bc`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `0c4d8869afeea9e182df0e56bcb96d37b66b5150b3bd87fc8a6d1cfc19c50464`
