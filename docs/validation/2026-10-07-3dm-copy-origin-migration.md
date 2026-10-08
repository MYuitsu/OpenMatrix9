# Explicit origin mapping for legacy 3dm copies

The isolated development module accepts `ThreeDm.migrate_archive(owner, rebuild_previews=False, origins=None)`. `origins` maps verified native source/definition UUIDs to the internal FreeCAD `Name` of the original host object. A copied host sharing a source UUID remains provenance from that archive; a copied definition now gets its own persistent creation UUID and source-backed copied members get separate persistent member UUIDs. Current CAD/mesh geometry and actual App::Link targets remain unchanged. Native source definition descriptions, URLs and safe user strings survive export through the copied-definition protocol.

An explicit origin must belong to the matching identity group in this archive. Unknown UUIDs, mismatched host objects, malformed mappings and contradictions with an existing original host witness refuse before mutation. Ambiguous groups without a witness require their own explicit origin. Original definition host witnesses persist for later migration. All planned property types are checked before one Undo transaction; copies use independent native UUIDs without replacing original source UUIDs/records. Current edited geometry is compared against an independent native reference, never accepted as the original geometry baseline. Existing creation IDs remain stable on repeated migration.

Verified shared proxies keep their canonical target and receive separate member IDs. Their metadata baseline comes from the original native definition membership. Unchanged two-dimensional rational NURBS proxy data retains its exact native payload checksum, including after FCStd. Native and host property/class/version gates still apply to other retained data. Copy definition names are kept distinct; generated suffixes, where needed, do not lock the editable Label.

## Verification

- RED: the new origin fixture failed because the migration API did not accept `origins`. GREEN: 22 checks now cover explicit source/definition ambiguity resolution, malformed/foreign identity and contradictory-witness guards, incompatible member-property preflight, independent native geometry, original definition metadata, Undo/Redo, FCStd, repeated migration and immutable sources.
- Fresh full native CTest8/8, final process exit0,29.20s, including both immutable user ring geometry fixtures. This does not establish opaque plugin/resource preservation or actual Rhino5 application acceptance.
- Fresh isolated FreeCAD483/483 across29 suites; every owned final process exit0. The complete prior461 checks ran again on the final Python files/binary; source/install bytes and binary fingerprints remained stable through the run. Cached SDK script build exit0; this Python-only change requires no changed C++ binary.
- A test assumption used `CurveA` and integer editor modes; native fixture uses `Curve A` and FreeCAD returns an empty mode list for editable properties. The assertion now compares the captured original Label and the actual editor API. No production workaround was needed.
- Rust archive source/tests remain unchanged under matching ledger fingerprints; previous74/18/fmt results remain applicable and were not rerun.

|Suite|Checks|Artifact|
|---|---|---|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/8a40b26b945b4da5b542eb8ef5b57705/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/9c78f13f6f1b4d0e826b8c99c77dde67/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/693cfa6ae8ae46ee9e8fcb9fed8a095a/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/3fe45b0f11074047895dcba8b7e70d6b/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/d852eda87efb4a90a656cc6f98d33279/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/5c92f4bc0100490fa14838013a360873/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/48d7f695dd0a4cf7b0e790ec7fd53ae6/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/0070699ed2ac47ae93a96d942b3ee53f/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/5a0a1337d23742b4a044238abb86e236/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/9b17f7d891b4424f80dfacdb668ee8a4/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/f7acbd46302e486d867f767a1cef426c/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/bd6c17de564e4eb0b67f97b6ced2e808/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/32bda266732e4898b3e44d40ae2b82db/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/47fa157c699b4931bb3e62537eeb1c32/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/c14fce885ef843de9b53f5393d43593d/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/ed4e86c47c33453cbd0e6efa992a1de8/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/2dba35686b9c423aac1cd28eb089cd03/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/bf583a29b4a640f7ab35c8b76b023e14/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/615cc22ceb1a44f8ba3a47b592e5063b/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/ddc187f5fac141eea9ae1192ad7e0371/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/5699bd21964f4da2a03efd80ee4cb445/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/9f6705758a3e443b99b7cfc5f50c2d3f/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/b2c5abe5bbee4ceba262481f3da3d5be/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/242a9cea53c947abb6e389499edca20b/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/f5d80eb8ccb64e329536812610e4921b/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/48c023db472c4af1840c22f60ee75be9/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/5109a3f4c9374d2cb80b98dbfa158ded/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/4933b4c15ac14db3a03bc7a300332579/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/ca182a5f23f646599b9fffebda833315/results.json`|

## Remaining scope

Full openNURBS remains in progress. Explicit origin mapping covers the tested supported legacy layouts and physical references. It does not infer which duplicate definition an affine serialized target UUID should mean; explicit affine target mapping is next. Ordinary recursive copies that duplicate archive owners, historical FCStd representations, missing proxy membership provenance, legacy object upgrades and arbitrary retained/plugin payload migration need further fixtures and implementation.

Linked/external resources, full appearance/annotation/settings/history/reference semantics, arrays/subelements and large graph performance remain incomplete. Standard/menu/CMD preservation routing, different-source document policy, final whole-package review, public integration and actual Rhino5 application testing remain pending. This package stays in `build/om9-dev` with isolated runtime `build/3dm-preservation-sdk`; public checkout has concurrent unrelated work and has not received this package. Native copy quotas are per namespace, not global RSS bounds. Derived display fingerprints still use12 significant digits.

## Synchronized status

Technical progress is synchronized into OM9-FILE-012's Spec v1 implementation notes, development API docs, feature/support docs, README and ledger. The Spec v1 catalog has no OM9-FILE-012 record or IMPLEMENTATION_STATUS.md; source-derived catalog metadata is unchanged. Its MANIFEST entry for the extension spec is refreshed.

## Fingerprints

- `ThreeDm.py` SHA256 `eb5f63f8499a6432112fd8cd2c0dfe7b6db05acaf029b4e7ebcbd54414efaa1c`
- `ThreeDmArchiveState.py` SHA256 `9dc27eb1f4c0506371053f4c48a15c39633da5b500a2f15c9783cd61ac72fcd6`
- `ThreeDmMigration.py` SHA256 `c522576921c3fbdbb6dd288fbd9c632fb84aec9a25c38624ec98693036df71b6`
- `tests/three_dm_copy_origin_migration_smoke.FCMacro` SHA256 `ed200670f5a9375c69c6901ddd4cc8528af720be4a87bd1b01bc3c81651d1bc4`
- `docs/development/3dm-structural-blocks.md` SHA256 `cf21812397e88d7c0f84a05ff05fb01db27ad182d9c7cefd12547ee69d5bd42f`
- `docs/features/OM9-FILE-012.md` SHA256 `a2d1de60d141c9030f7b33a121cd1f8d409d18dc680236d2a2059a16c7a4f94d`
- `docs/features/3dm-support-matrix.md` SHA256 `9f3ef1b8a611c9241d9dd6bbada88aa55697995806438a5c071a068517a3783b`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `daef837efebb222a6d5d4de51394533df32723a5b62583b8a036f026e3cf1a8c`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `867716fa11ba63ccde1f43b9ebabe1fac865184bc564a568368508dad0a8ca51`
- `README.md` SHA256 `cc0660f6e5e0c8ff649ac495e2504dd55af48322bb574807b2dfdd4708657382`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `0c4d8869afeea9e182df0e56bcb96d37b66b5150b3bd87fc8a6d1cfc19c50464`
