# Existing source references targeting new native definitions

Verified in the development checkout and installed SDK scripts. This supersedes the preceding host guard against existing imported references targeting new host definitions. Full openNURBS remains in progress. The public primary checkout remains clean and unintegrated; no push or actual Rhino5 application acceptance is claimed.

## Supported behavior

Existing rigid App::Link references use their current explicit definition target. Existing affine references use their editable OM9DefinitionTargetUUID while retaining the immutable native instance matrix and source snapshot. Both now accept creation-identified new definitions in the same document. Source instance UUIDs remain stable; new definition/member IDs retain their creation identity. A new target with archive ownership must match the source reference namespace. An unowned new target can be staged separately under two imported source namespaces, with independently remapped output graphs.

Current target identity participates in source signatures, dependency traversal, canonical member overlays, independent copied-member overlays and selected/copy instance requests. New-definition dependencies enter a growing scoped staging worklist alongside new member references. Final native closure includes the current new/imported graph and excludes an obsolete source target. Source attributes/snapshots and native matrices remain provenance; no fabricated source record or flattened preview becomes authoritative geometry.

Source LinkTransform, reflected/nonuniform ScaleVector, current new definition placement and affine rigid deltas compose once. Preview traversal uses current new/imported target graphs and refreshes after creation-identified target edits. Current copied source references may keep a different source/new target from their canonical object. Top-level copies get fresh output UUIDs while sharing the current new definition. Existing source-copy output UUIDs remain the earlier per-export fresh allocation policy; this package does not claim persistent copied-source output IDs.

Native source IDs resolve within the verified owning archive namespace. A creation-identified target must be fresh in that namespace. This distinction allows an unsourced original new definition to coexist with the reimported native definition carrying the same file UUID, without mistaking it for a second source mapping. Multiple new hosts deliberately bound to one effective new UUID still reject as ambiguous. Missing/nil/foreign/colliding target identities and edited previews reject before destination replacement.

Usage: [structural block API](../development/3dm-structural-blocks.md).

## RED to GREEN and scope regression

The new source-to-new target fixture originally failed at source_signature with the same-source ownership guard. Definition identity resolution now distinguishes verified source definitions and creation-identified new definitions. Dependency collection registers new target graphs for native staging rather than indexing them as absent source records.

The fresh new-block regression then failed when importing a just-exported graph into a document still containing its unsourced originals. Artifact diagnostics proved the original creation UUIDs equal the native definition UUIDs as intended. The initial resolver incorrectly mixed those new hosts into the imported source namespace and reported an ambiguous target. The corrected resolver gives verified source records their archive scope; an explicit new link target colliding with that namespace is rejected instead of silently resolving as source geometry. Reimport and subsequent preserved export now pass.

The centimeter fixture is a reflected/nonuniform affine reference, not an App::Link; its test uses OM9DefinitionTargetUUID and retains its original matrix. New millimeter geometry stays unscaled by source-unit normalization. Solid volume1440 and live/native world bounds agree after the affine transform and rigid delta. The source bytes remain unchanged.

## Validation

- SDK scripts refresh builds exit0; installed ThreeDmArchiveState.py and ThreeDm.py match the tested source bytes.
- FreeCAD328 checks across22 suites pass, every owned final process exit0. Source-new-target27 checks cover original source UUID, obsolete closure omission, live/native rigid bounds, stable canonical/new IDs, reflected scale/LinkTransform, affine raw-matrix preservation, CAD/mesh preview and edits, nested and independent copied source targets, new wrappers with imported dependencies, top-level copies, FCStd, Undo/Redo, two namespace remaps, centimeter normalization, foreign/missing/ambiguous/colliding target refusal, protected manual preview and immutable snapshots.
- All21 prior GUI suites pass on the final script code:301 checks, including the new-block reimport regression and newly added instance membership, source retargeting, independent/proxy copies, new geometry, metadata/layer/definition/member overlays, bridge, mixed shear, fresh-process preview reopen and lifecycle/geometry/menu baselines.
- Native C++/Rust sources and SDK binary are unchanged in this host-script package. Gui/ThreeDmMerge.cpp SHA256 matches the preceding block-creation report; Rust core archive and tests match ledger fingerprints. The previous CTest8/8 exit0,29.46s and Rust74 tests/18 suites/fmt exit0 remain applicable native evidence. Those suites were not rerun solely to repeat passing checks. Both immutable ring geometry fixtures remain that previous native evidence; their geometry-only success does not establish full opaque-resource preservation.

|Suite|Checks|Artifact|
|---|---|---|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/caeb4f82612f4b62bac7a8045954dc05/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/8c820e7fdebd4dccab38c47163bc1c01/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/a2271ca4185f4c69bb92530c621b8864/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/eaa74e4ca81d4379a0e880c2120b3e67/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/d2fb2ec7267141caa2c9cf6b0b39d959/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/19032da6740443a8b4d6fd53e1d71cdf/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/7056a00176fe4ee49c7176cdab63ff7d/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/a6d0ac6ec4dc4be9a03b03fdc3f2e9a6/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/a839469088814c90afcbacf29c4494d8/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/b90e9e21a81b47e2bf4ca03207d29051/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/1530d70e9b59408c961d6fbb666b8a26/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/10479b6b30204b9ba1f30c4e1aededbb/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/25c8cee35b5742acb921315bf5920a04/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/8576f62be33743df85496fc60fad1e21/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/bf79e7735fb5446db5b826dce19b74c9/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/32f483855cb64529b49cfc5e081eb2da/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/ddd184a9aa7d4015960adfcdc2f6b9db/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/8962ab7101cf41f39ba5b7853c7c4de1/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/14b14a9a312240fc8d8acdbb05a9de76/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/2de337993754424fb02c59b566eef817/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/2cb9bc0ee31d4c8f9a431161d1f631e7/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/4bc353989afb40718aa3049b2178f49a/results.json`|

## Remaining scope

Creating new definitions from source-backed/retained members, whole copied-definition families, manually inserted geometry identities and broader observer/cache/property fixtures still need implementation and acceptance. Explicit legacy owner/source/proxy migration, distinct-source document metadata merge, shared standard/menu/CMD preservation routing, whole-package review and public primary build/integration remain pending.

Appearance, annotations, resources/settings/history, safe arbitrary plugin references, full native class/version coverage and actual Rhino5 application acceptance remain incomplete. The display fingerprint remains12 significant-digit tokens; sub-resolution changes are not claimed detected, and quantization boundaries/large shared graph performance need broader fixtures. This package establishes the listed reference routes, not full compatibility coverage.

## Source fingerprints

- `ThreeDmArchiveState.py` SHA256 `5df6c5c3b36c766a4506e42d02c7941d84ebbd83110ac0f14c90779e830e5e81`
- `tests/three_dm_source_new_target_smoke.FCMacro` SHA256 `75e4dfe4bcf4b3ab926ad0d298ae7eadf258caa53c2da8e8bcc90cbbf8fc55b8`
- `tests/three_dm_new_blocks_smoke.FCMacro` SHA256 `c71f8357194890607dd910609e7c403fe7d2e47021a4bc971fae8793a3e1a56c`
- `docs/development/3dm-structural-blocks.md` SHA256 `c822a744a4d2fedf8a894b93c0732cdb0763cc29adee0dfef3591c49589a9dd0`
