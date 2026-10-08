# Shared definition-member copies and retained members

Implemented and verified in the development checkout. This package extends selected preservation export; full openNURBS remains in progress. The primary public source checkout has not been integrated or pushed.

## Behavior and scope

- A transformed shared-member App::Link receives its own native member UUID. Its geometry comes from the current canonical source member after geometry, placement, instance and metadata overlays. Unchanged shared branches continue using the canonical UUID. Copies of source points, BReps, meshes and nested instance references are covered by native fixtures; point/nested-instance and mixed BRep/mesh paths are covered in FreeCAD.
- The copy matrix composes proxy Placement and ScaleVector. LinkTransform=False compensates for the canonical Placement already included in local host geometry; True retains it. Proxy metadata differences use a persisted baseline. Ordered definition overlays reference the new member IDs; selecting a copy-only branch omits the original canonical record while still using its current edited geometry as copy input.
- Native validation rejects malformed/projective/singular matrices, duplicate or colliding identities, unreachable copies, ambiguous canonical inputs and unsafe identity-bearing userdata. Native source records, object attributes and block graphs are checked after writing Rhino5 version5/50, before atomic destination replacement. Failed requests leave destination bytes and source snapshots unchanged.
- Rust bounds the sum of serialized geometry plus object-attribute copy payloads to512MiB with checked arithmetic. Native preflight checks before allocating physical copies, then checks again after canonical overlays. The request manifest remains bounded32MiB. This is a serialized-copy payload budget, not a bound on total process memory. A40000-control-point NURBS curve requested600times verifies rejection without cloning the whole family.
- The document observer now notices proxy edits despite App::Link forwarding source-owner properties. Affine previews follow both proxy transformations and canonical edits. Mesh preview construction bakes the canonical topology Placement exactly once: FreeCAD Mesh.transform changes only its kernel, whereas Mesh.Topology includes object Placement. The former copy/transform/reset path lost Placement and disagreed with native export; the new display mesh agrees with the exported coordinates.
- Retained definition members without a host Placement now import and keep stable signatures. The TextDot fixture proves native UUID/geometry CRC and definition role retention, Unicode name overlays, FCStd reopening, source immutability and atomic refusal of an altered retained payload signature. Native TextDot display/editability is not claimed.

## RED to GREEN evidence

Native member-copy requests initially failed because edited definitions referenced unknown member UUIDs. Copy records are now allocated before graph closure and refreshed after canonical overlays. Copy-only edited mesh exposed a reread check that expected an intentionally omitted canonical record; checks now verify the output copy instead. Rust copy-budget tests first failed with the absent API, then passed with bounded arithmetic.

FreeCAD shared-proxy tests exposed missing canonical placement in links and missing observer notifications. Both were fixed and tested against live world points. The mixed mesh/BRep test then exposed mesh preview coordinates differing by canonical Placement. It now compares preview and native output coordinates and passes. The retained-member test initially raised AttributeError for a missing Placement during import; it now passes without inventing a host geometry transform.

## Validation

- SDK module/scripts build completed with exit0. Installed ThreeDmArchiveState.py SHA256 matches the tested source.
- Rust full suite:74 tests across18 suites, exit0. cargo fmt --check exit0.
- Native CTest:8/8 suites, exit0,29.08s. Merge tests include reflected/nonuniform copy transforms, canonical replacement and copy-only closure, nested native references, exact double mesh vertices, metadata, immutable snapshots, malformed requests and the large-copy rejection fixture. Both immutable real-ring geometry suites pass. Real-ring geometry regression does not establish preservation acceptance for every retained resource/class in those files.
- FreeCAD:164 passing checks across14 suites, each final process exit0. The fresh-process FCStd suite ran after the current affine-refresh fixture was saved.

|Suite|Checks|Artifact|
|---|---|---|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/4b5033cf8073412381e9c2557e96422b/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/3b16f19d9668493286dd3ba2c7166025/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/ad4e150dae534d7c8332213b9133c3ce/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/d6f0893089334f17bd6a73a46e4eea61/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/4efc024e904a4d39bfb6c69b29c65139/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/b2f25a68b60f4538b55b338a3c995dd2/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/7df59d3b7d3e46698e657c305c81b78d/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/af4822c611af4b7081857bc0497b0109/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/fcd56eec029948709e57b0bce4207cf0/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/171e7c7f0aea476584600609540a864d/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/3e9832ccc3fe4a4b9d78bd2c969a2391/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/09d02019fc344a20aeb64ed83b14eb14/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/959ec8755c2d450198b0ed9b4c2e6a82/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/6641d1338dab49d2ab1408e18444e249/results.json`|

Five initial concurrently launched FreeCAD regression processes did not reach their macros and timed out at300s without result/config files. Their exact process IDs and unique task command lines were verified before stopping only those owned test processes. Sequential fresh reruns passed, including structural blocks, placement, member overlays, lifecycle and affine refresh. The startup timeout is recorded separately from feature assertions; no root cause is claimed.

## Remaining work

The explicit ThreeDm.export_preserved API is tested; standard export/menu/CMD preservation routing remains guarded. Copies of independently edited source-backed definition members still require an explicit independent overlay and reject ambiguity. Native copy-to-copy sources are deliberately rejected. Old shared proxies without verified metadata baselines require migration rather than assuming current edits are unchanged.

New standalone definitions and definition-reference edits, legacy provenance migration, distinct-source document policy, final package review and primary public-source integration remain pending. Appearance/resources/history, safe arbitrary plugin references, complete class/version coverage and actual Rhino5 application acceptance remain incomplete. The goal stays active. Generic class coverage rows are not promoted to full support by these fixture-specific results.

## Source fingerprints

- `Gui/ThreeDmMerge.cpp` SHA256 `8c56b39dd7924731b96132d2cc4c2ce3f6a07d9dc1b8d9eca4608d96800ca429`
- `Gui/RustBridge.h` SHA256 `294b99c2de6d32b593e1ea7e9183228cfdcd1ca696ed2b795d6de9e948bab99f`
- `ThreeDmArchiveState.py` SHA256 `a139d99a3b704004748f536e08d484b5b80f37231807f8f98203a8d9be4dd756`
- `rust/src/core_3dm_archive.rs` SHA256 `096c83cad37a13de6fe39b22d89f686aa824966771c0b6e6f9bb117376030465`
- `rust/tests/core_3dm_archive.rs` SHA256 `07462be2479f20b4a152057e36849101af0a4f5dede32b12ee07e7248a1e9941`
- `tests/native/three_dm_merge.cpp` SHA256 `ba62458e8f39cba862012d22db3cad706b88c7d9af94468be2561a4157828586`
- `tests/native/three_dm_blocks.cpp` SHA256 `c1bf474b1da48c29be86969ad5809fc932e919e05ecf2aea27fb35d087ba4bf8`
- `tests/three_dm_shared_proxy_smoke.FCMacro` SHA256 `5277dc185c98e43e917796a0a15df3204494a318412702e87fd225cfac41114a`
- `tests/three_dm_shared_proxy_geometry_smoke.FCMacro` SHA256 `89790981ed4ae95a4e6fc0deedc03d30a1ab08f827c3e309b610816fcb64c0c3`
- `tests/three_dm_retained_member_smoke.FCMacro` SHA256 `954f8e6cdf3418e3b438b68737611a107e3da9dd02b8150af0c8a6097955fa2c`
