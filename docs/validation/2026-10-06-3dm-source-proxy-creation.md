# Source proxy creation and definition family copying

The isolated development module now creates native definitions from verified shared source-member proxies and copies their supported families with independent canonical geometry. Full openNURBS remains in progress. Runtime: `H:/FreeCAD-src/build/3dm-preservation-sdk`. Public primary integration/push and actual Rhino5 application acceptance remain pending.

## Behavior

`ThreeDm.create_definition(document, proxies, name)` verifies the linked canonical native record, included snapshot/hash, ownership, source baseline and proxy metadata baseline before one Undo transaction. Canonical geometry stays in place with its original source identity. Promoted wrappers move into the new definition with current world placement and receive fresh persistent member UUIDs. Current canonical geometry/metadata and proxy placement/scale/LinkTransform/metadata overrides compose once. Foreign namespace combinations, invalid CAD/mesh, chained proxies, missing baselines and singular/nonfinite transforms refuse before creation mutation or destination replacement.

Unchanged canonical geometry uses the original native payload. Edited supported CAD/mesh stages independent geometry, while instance references use current native matrix/target overlays. The new source-member callback registers dependencies through the canonical object, and affine preview traversal recognizes explicit proxy members in new definitions.

`copy_definition` verifies proxy payloads, clones each canonical object once across the copied family, and gives each copied wrapper a fresh persistent member UUID. Copied uses share that independent canonical payload: one edit updates their copied uses without changing the original. Deep copying retargets canonical instance references to copied target definitions. Existing affine display payload guards remain active; copied owned preview children still require verified payload equality. Details: [structural block API](../development/3dm-structural-blocks.md).

## RED to GREEN

- Source proxy creation initially rejected ownership because the wrapper correctly has no native source identity. The new route resolves and verifies canonical provenance without inventing source identity on the wrapper.
- A shared-family fixture exposed one independent clone per proxy. The family-wide canonical-copy map now preserves shared uses with one independent payload.
- A small invertible transform was wrongly rejected by an arbitrary determinant tolerance. Validation now follows the native finite/nonzero determinant contract; the small-scale native coordinates pass.
- A two-dimensional rational NURBS fixture exposed unnecessary host/native replacement changing the native payload checksum. Unchanged proxies now use direct native copies. Class/checksum remain identical on promoted, copied and FCStd-restored geometry; the curve checksum covers native dimension, rational control data and knots. This is specific fixture evidence, not proof for every class or opaque plugin payload.
- An invalid canonical CAD fixture exposed a missing proxy creation preflight. Null/invalid CAD and empty mesh now reject before the creation transaction.

## Verification

- Fresh native target build/run and full CTest8/8 exit0,30.29s, including both immutable user ring geometry fixtures and the new rational shared-curve fixture.
- Isolated SDK native module build exit0 for the combined allocation guard; final script build exit0 for proxy implementation. Source/install Python bytes match, and source/native binary hashes stay stable throughout the complete GUI regression.
- FreeCAD420/420 checks across26 suites, every owned final process exit0. New proxy suite34 checks covers promotion/copy, identity and native coordinates, reflected scales, LinkTransform override, shared canonical reuse, independent edits, FCStd, Undo/Redo, small invertible scales, missing-baseline/mixed-namespace/invalid-CAD refusal, two-dimensional rational native payload retention, mixed CAD/mesh and affine preview observer refresh, and deep nested instance target copying.
- All386 preceding GUI checks pass on this final script/binary combination. Rust archive code/tests remain unchanged; prior74/18 suites/fmt evidence is retained under matching fingerprints.
- Native definition metadata and member geometry/attributes share the existing per-source-namespace512MiB serialized data budget, with large/combined copy atomic refusal fixtures: [allocation report](2026-10-06-3dm-copy-budget.md).

|Suite|Checks|Artifact|
|---|---|---|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/109bed9b89014d95835ac3083804f228/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/79ec7912d3ba43c494007402893c3489/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/312b67bf97cc47759aaef4731578a8f7/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/90b0f755d8a046188f971d47aaec827b/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/4877adaef5cb47d2a9cb8d8bbf851806/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/b4321d2e960e43f4afa459f099fc39a1/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/2792a9b6ea624bcbb266b74409dca8e3/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/7b063d8f78a24f47b6392df04acbb7eb/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/75ac9577ad2b4208a811306f067e8a08/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/70de9f5d9f2c4ac09f0263726bdf325c/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/9f9ededb419143f1bc43a7f4e1f1c6ee/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/fc42896806a741a8acebebfcae0dec63/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/5ac0db5bea1c494d9689c5086894f24c/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/3b16d86d50c344fea580d88b2f132ba2/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/ecb86e9ea05f4c7192774fe82718846b/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/4c790a25b48842578a22761f098d5c14/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/a50160022fb942478d09dd7f1da8c438/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/91f89f55e71b431e8fcd9a05d8df513e/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/50247e2cc014444997d5c0bf450b7f82/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/ee0e79b69aa74848891e7ba3d0386fb0/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/dd2f9574b9d4409c847606245c5e52bd/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/2990fa43cff9414a87bd0dfa23c026fe/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/933b1996d68d46e2baae9b947959b749/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/6acd4dec957f4244a10037ddbd48bfbe/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/7e3367d71b7d4e76986c96f156646a44/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/35e578ff1973488981775cff620ad7b3/results.json`|

## Remaining full-goal scope

Legacy source/owner/proxy baseline migration remains explicit and unimplemented; ordinary recursively copied imported definitions still need provenance migration. Use the explicit family copy API. Arbitrary retained/plugin members, App::Link arrays/subelements, internal Name changes, wider manual identity mutations and large graph performance are not established by these fixtures. Derived display fingerprints still use12 significant digits, while original native/source geometry remains unquantized.

Linked/external definitions/resources, broader appearance/annotations/settings/history/reference semantics and full class/version coverage remain incomplete. Different-source document metadata policy, shared standard/menu/CMD preservation routing, whole-package review, public integration and actual Rhino5 application acceptance are pending. Serialized copy quotas are per source namespace and do not establish total multi-source RSS bounds. The development Git registration is absent; verified file checkpoints preserve current work.

## Fingerprints

- `Gui/ThreeDmMerge.cpp` SHA256 `f5d0962e6016776524c40f7c0dcbfefe35caafa3efa52d437f62d8fe212e33a5`
- `Gui/ThreeDmInventory.cpp` SHA256 `fdebce5e299fa49f46cf92245ba98706d09c09d5e6f7ff3293e4da75285e4955`
- `ThreeDm.py` SHA256 `59552e70b0d40165de878111497fc99df77acb0a5d3ec70e5f665e92bad68b3f`
- `ThreeDmArchiveState.py` SHA256 `81f83c5e866b7eb7236b90314453b0f7d85e4ecc9cb9b2b10c72a68f48736401`
- `tests/native/three_dm_merge.cpp` SHA256 `9ce34ed07dc16cd62ba52cce32fd1f9dfb3d1f2327e7b99b66b25cff85da6535`
- `tests/native/three_dm_blocks.cpp` SHA256 `a10acd3b9eadfd891d78f9d82e8627ddf3409e7639c429dcec58ec9aae6d321e`
- `tests/three_dm_definition_copy_smoke.FCMacro` SHA256 `3ca7508b17e22822cd00741cb9a31cfc1273427822bb4a1125630f3100b18bd7`
- `tests/three_dm_source_proxy_creation_smoke.FCMacro` SHA256 `5df586ffe9d742f07c13819b8c8c7873470161bf93d6079918e55b54dfbabbaf`
- `docs/development/3dm-structural-blocks.md` SHA256 `5a0c515bb6178a6d3f5efc58b7b2bb359490c30b19564eba043c58dfc2b23c1e`
- Isolated `bin/OpenMatrix9Gui.pyd` SHA256 `a67dc12a30bb5b30df61c02d4bffebcacf9030530ac2a2c09c33c5fbd772c854`
