# Copying native 3dm definition families

The development module now explicitly copies supported new/imported definition families with independent identities, current geometry and native definition metadata. Full openNURBS remains in progress. Tested runtime: `H:/FreeCAD-src/build/3dm-preservation-sdk`. This package is not integrated/pushed to the public primary checkout; actual Rhino5 application acceptance remains untested.

## Supported routes

`ThreeDm.copy_definition(definition, name=None, copy_targets=True)` validates the active document, explicit native definitions, ownership, source snapshot/hash/record baselines, current supported members and target graph before one Undo transaction. It creates fresh persistent definition/member/reference IDs. Shared nested targets are copied once; false retains current targets while copying reference identities. Original source snapshots/UUIDs/records remain provenance. Local definition/member placements remain separate and compose through the existing instance matrix rules.

New CAD/mesh families also have evidence through FreeCAD recursive copyObject. Imported families use the explicit API: ordinary recursively copied source definitions cannot be silently assumed to have a fresh native identity. Explicit copies retain the verified original source owner; archive containers are not recursively duplicated. Source member/instance copies stage current supported edits independently, with separate native member identity. Copies of copied families retain their original native definition provenance.

The additive native new-definition field `source_definition_uuid` clones an original verified source ON_InstanceDefinition. A fresh native UUID/name and current member graph replace its structural identity while preserving static definition description, URL, URL tag and safe user text. Inventory now reports these three descriptive fields, and the writer's existing target-version semantic comparison checks them after Rhino5 reread for retained and copied definitions. The native source must be an original definition in the verified namespace, not a new placeholder or unrelated geometry. Linked definition resources/settings and identity-bearing or opaque user data remain gated.

Copied affine BRep and mixed BRep/mesh display caches are independent owned objects. FreeCAD DuplicateLabels=false forces new preview child Labels even on explicit assignment; those Labels participate in the display fingerprint. Copy initialization verifies the original cache before mutation and compares copied types, placements, non-name metadata and display geometry recursively. Only a matching owned copy gets its initial display baseline. Existing or later-edited caches never get a fabricated baseline. Recompute follows independent copied member geometry/targets, and manual preview edits reject export and further copy atomically.

Usage: [structural block API](../development/3dm-structural-blocks.md).

## RED to GREEN and evidence

- Native source-definition copy initially rejected the unknown protocol field; host copy initially lacked the explicit API. The new native/host routes now pass reread and identity/metadata checks.
- Mixed affine copy initially failed preview integrity. Diagnostics proved original children unchanged, independent copied children, identical non-name metadata, and forced distinct child Labels. The verified-copy initializer fixes this without disabling the later edit guard. Both export of an edited copied cache and copying a family with an edited original cache reject before mutation/output replacement.
- A test expecting source definition placement initially omitted LinkTransform. The fixture now explicitly enables it before comparing the intended world points; production composition was already correct.
- Native merge target build/run exit0; fresh full native build/CTest8/8 exit0,28.89s, including both immutable user ring geometry fixtures. Ring geometry tests do not prove complete opaque-plugin/resource preservation.
- Isolated SDK module and final script builds exit0; source/install Python match, binary hash unchanged during the complete GUI regression.
- FreeCAD386/386 checks across25 suites, every owned final process exit0. Copy suite27 checks covers new/imported families, deep/shared targets, independent edits, original graph exclusion, static native metadata, recursive copies, fresh persistent graph IDs, FCStd with external source removed, creation/member Undo/Redo, nested reflected source matrices, mixed shear preview ownership/edit propagation, TextDot native attributes/local transform and cache/invalid-policy refusal. All359 preceding GUI checks pass on this final code.
- Rust archive code/tests are unchanged and match prior ledger fingerprints. Prior Rust74/18 suites/fmt exit0 remain applicable; unchanged Rust checks were not rerun just to repeat evidence.

|Suite|Checks|Artifact|
|---|---|---|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/8b5c33e0be9a48e78543cb4fa5b1a12d/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/2d2bbe145c1546bdad7d09587dbe3071/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/431bcb0fc1a14242bced1829ab6490ea/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/e58ab0e346bc4f3c8b7fcec9ac87301a/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/883d767395224f4eac6febcd629d945a/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/1c60b7c8642b416b897f1221357129b8/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/299e2d5a498e4a948f8665cf5fdb2d7b/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/c4064b58b1a4420aa89522472d557fc8/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/b77543e3c93c4284894aa6c2a6cb8683/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/9191e4f8025a414fb7e98fe580c08ca7/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/7520c4d5e8fb476bb3b3abee54acb64a/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/3991f748026f407d86d1c6b725067b28/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/0a797ca6abdb4158b2453d3a5296e084/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/41ee2da03b9f4791ade6aa7b2ee6a4ad/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/b20c896a2c444855802db352ea308ef4/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/0f0160030ac54c179331e14ca90328e4/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/b469629888ba47a3bd5025533e1ecd1c/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/a8ef12991f944dc9ba2f74fcca951790/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/9fdd1bd3d10f48d687d58a7e8bb97736/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/ba93d6fcc83544a1a4d1ba207a145215/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/0acf46f6494043298e6e9a4ebee3ecda/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/bb256d1b520b4839bf1dfd8fc2676d6c/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/408d08224dfb4f48b955ab13a501f4e1/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/5507c63265ce4d54bf526d326ec34008/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/b36f9d5510e743259e1465beab859302/results.json`|

## Remaining full-goal scope

Source-member proxies and legacy owner/source/proxy migration, ordinary copied imported-definition provenance migration, manually inserted identities and internal Name changes require further work. Linked definition copying, broader class/resource reference semantics, large copied-definition metadata allocation bounds and graph performance need implementation/acceptance before claiming complete copy coverage. Static tested family behavior does not establish those paths.

Different-source document metadata policy and shared standard/menu/CMD preservation routing remain pending, as do appearance/annotations/resources/settings/history, arbitrary plugin reference handling, full class/version coverage, whole-package review, public integration and actual Rhino5 acceptance. Retained source selections with transforms still require their dedicated native action; the current placed-block route/guard remain. Derived display fingerprints use12 significant digits; sub-resolution changes remain outside detection claims. The development checkout's Git registration is absent; verified checkpoints preserve tested files.

## Fingerprints

- `Gui/ThreeDmMerge.cpp` SHA256 `12bbceb789d86d2268a7de4a35269474548ed3879829bfed65d07cece178e3c0`
- `Gui/ThreeDmInventory.cpp` SHA256 `fdebce5e299fa49f46cf92245ba98706d09c09d5e6f7ff3293e4da75285e4955`
- `ThreeDm.py` SHA256 `59552e70b0d40165de878111497fc99df77acb0a5d3ec70e5f665e92bad68b3f`
- `ThreeDmArchiveState.py` SHA256 `2a0a26b3644e0340cf13c81d9352e95a0c08e59ed2a5851f28338a9c9b48f42e`
- `tests/native/three_dm_merge.cpp` SHA256 `bca809a0e3fa9cd4ca57ef97ac9bf07ec8276f97c7c55f2d19963d2ae35b6e08`
- `tests/three_dm_definition_copy_smoke.FCMacro` SHA256 `3ca7508b17e22822cd00741cb9a31cfc1273427822bb4a1125630f3100b18bd7`
- `docs/development/3dm-structural-blocks.md` SHA256 `cbe97bac9634e5d787ef41cb4ac4ae83b456347686dd2f59f1d97531ea8d224f`
- Isolated `bin/OpenMatrix9Gui.pyd` SHA256 `7092d9a45b29dc5cdc627ffa087023b1c77589c1acc935050e96d9afabd580bd`
