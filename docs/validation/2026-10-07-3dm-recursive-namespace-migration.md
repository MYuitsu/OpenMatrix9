# Recursive archive namespace migration

The isolated development runtime supports `ThreeDm.migrate_archive(owner, rebuild_previews=False, origins=None, targets=None, fork_namespace=False)`. Explicit `fork_namespace=True` separates the selected archive owner and its supported owned source/definition graph from another recursive copy. It creates a persistent namespace and records `OM9NamespaceOrigin` plus `OM9NamespaceHostID`. Native source/definition UUIDs, records, matrices and the included archive remain provenance. Current CAD/mesh edits, physical links and placements remain the caller's graph.

Repeated migration on the same verified fork host keeps its namespace. Recursively copying an already forked archive owner changes its host witness and allocates another namespace whose origin is the parent scope. Default migration still rejects conflicting owners. Preservation export now rejects distinct archive owners sharing a namespace before replacing the destination; it cannot silently alias their current dependency edits.

Fork preflight checks owned canonical source bindings, namespace/property types, physical definition targets and native member ownership. Foreign original definitions or source payloads refuse. Every affected affine preview must rebuild successfully in the same Undo transaction; failure rolls back namespace markers, source fields, generated preview objects and geometry. Manual/unverified cache edits still require `rebuild_previews=True`. Missing historical representations and unsupported graphs remain guarded.

Default migration accepts two independently reconstructed native display layouts: the initial native conversion and the reference document's canonical renderer. A one-vertex shape and a one-vertex compound can represent the same pristine native graph. Both baselines come exclusively from the verified included archive, never from current edited geometry. This fixes a reproduced pristine copied affine preview rejection without weakening the manual-cache gate.

Native multi-source export previously used random collision UUIDs. It now derives collision aliases using UUIDv5 from the source namespace, original native UUID and bounded salt. The first unused native ID is retained; up to4096 occupied alias candidates are permitted before refusal. Stability is verified for the same ordered source set. Changing source order or selected subsets can change collision conditions; order-independent identity is not claimed. Component references use the existing native mapping and source archives remain immutable.

## Verification

- Actual FreeCAD `document.copyObject(group, True)` probes distinguish copying an affine root, a nested imported definition and a whole plain group containing archive owner/root/definition. Only the whole graph copy clones the archive owner while retaining its old namespace. The exploratory probe records actual graph structure; it is separate from acceptance counts.
- RED: exporting two distinct owners with the same namespace succeeded and overwrote the sentinel. GREEN: the explicit owner guard refuses and keeps the destination bytes intact.
- RED: repeated native multi-source export produced different geometry UUIDs. GREEN: the native test compares output UUID sets across the same repeated request; the host fixture also verifies stable merged geometry identities and native block closure.
- New FreeCAD acceptance25/25: actual nested recursive copy, explicit namespace/host witnesses, source/native identity retention, independent edited geometry, invalid boolean/property/witness/conflicting namespace rejection, foreign physical target/member rejection and late failed-preview rollback.
- Analytic fixture: original world point(-2,6,12), edited copy(154,377,664). Combined export contains6 unique native records and4 native definitions; copy-only export contains3 records. Native source bytes and both included archive hashes stay unchanged.
- Undo restores the prior namespace and host fields while keeping current geometry edits; Redo restores the fork. FCStd retains the namespace and graph. The copied archive remains exportable after deleting the original owner. Copying an already forked graph allocates its own scope. A pristine affine graph forks without explicit preview rebuilding.
- Full fresh native CTest8/8,27.47s, final process exit0. Both immutable user ring fixtures run again. Isolated FreeCAD540/540 across31 suites, every final owned process exit0. All previous515 checks rerun against the final Python files and rebuilt native module. The runner verifies source/install equality and binary/source hashes before and after every suite.
- Native target ThreeDmMergeTests and full isolated module build completed with exit0. Rust source/tests remain byte-identical to the prior ledger fingerprints; prior74/18/fmt results are retained rather than rerun.

Commands in the MSVC x64/Qt6/FreeCAD SDK environment:

```text
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-tests.cmd ThreeDmMergeTests
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd
rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-recursive-namespace-regressions.py
```

The full GUI runner invokes the owned hidden `run_menu_smoke.ps1` process for each macro and waits for its final exit. Geometry/result files reside beside each listed results.json; this slice verifies graph/geometry programmatically and does not claim a new visual screenshot audit. The advanced checkout's Git registration remains unavailable; the verified checkpoint and fingerprints identify this local source package.

|Suite|Checks|Artifact|
|---|---|---|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/fa963d0d947f4184b39fc51336b924d9/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/3b0df72680b649cf9d792fe29f4ea4bd/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/e1c058ba5eef46b6bff966c74d0e036a/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/f5b1f6426e4b40f69fcc4a0c2cceb17c/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/6bad50843c794572b57805b31fb291eb/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/dd3ebcdc002d41c1b3dde3ad480c82ed/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/0e72ea537bcd48ccad4f4c87dc521c9c/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/0cf05bfbbe6c45b78ab5bf90ecc0a042/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/2216147ae3c54157a305c1d0651b0fd4/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/9917fa0af9144ec49c5146960b4f3f96/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/157f8598d8704e75b52a3367e4903c07/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/dd2f55e719f8468eb3046f8275c8d46a/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/06555d4bce5a40439e661985d7c834e8/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/507671ae29cd48439bc3f6b00d659646/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/fa43549704e0477181c5946d91e62261/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/62f6ddcde73b4cf3aad10174d0164aac/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/0d4a946c6d594dbf897c4486908716ff/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/26afca62219a497f89c89a44b85c202a/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/a8f9d61ec1bd44dd93114c79a617b08f/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/1c4b852157b542cd82a81bfb6f1cd158/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/00c5df29a6a7431889d5abeeeb478cfc/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/093f3685ea26460e8baafcec3d7af37f/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/3c3a8634fb2a4c1f80fb3516532c387f/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/3c7efa97861543d88ad374d1931974b5/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/633991eb1458424ea310f814f2d2e4ea/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/c2b6a90d21e04470a8f5a0cf000735ce/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/b60a6801d0ee4f15bc5c9bf7de959b11/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/4d08c6f1d56e414082873467a4eddc68/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/2daefd376acf4ebabb98db61ff0ebb62/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/530bcffa66c04da38004b79d33cd9992/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/5ebabc842acd402e94bd848a6cdd3ced/results.json`|

## Remaining full-goal scope

This acceptance fixture covers a supported nested point/physical-reference/affine-reference archive graph and simulated legacy provenance layouts. Arbitrary recursive shared proxy/mixed CAD/mesh families, real historical representation upgrades, missing membership provenance and manually inserted objects need broader fixtures. No coverage-matrix cell is promoted solely from these aggregate test counts.

Full openNURBS remains in progress: linked/external resources, complete appearance/annotation/settings/history/plugin reference semantics, arrays/subelements, class/reference/version compatibility and large graphs still need implementation and evidence. Different-source document merge policy, shared standard/menu/CMD preservation routing, final whole-package review/public integration and actual Rhino5 application acceptance remain pending.

The authoritative advanced source is `H:/FreeCAD-src/build/om9-dev`; the isolated runtime is `H:/FreeCAD-src/build/3dm-preservation-sdk`. Concurrent public checkout work is separate. This checkpoint does not publish or integrate the package. Native copied-data limits remain per namespace rather than total process RSS guarantees; display fingerprints retain the existing12-significant-digit policy.

## Technical status synchronization

OM9-FILE-012 Spec v1 implementation notes/checklists, feature/support docs, README, plan and ledger record this exact scope. The extension has no FEATURES catalog row or IMPLEMENTATION_STATUS.md in this package. Source behavior/catalog metadata is preserved; the extension MANIFEST entry is refreshed.

## Fingerprints

- `ThreeDm.py` SHA256 `bdb00857d337527fc2fa4762847161c317f2f2c294ee645b7dc4d4d4a9471573`
- `ThreeDmArchiveState.py` SHA256 `297c953a1d2c90a78770dcc7fc750c10faee5b811d877968372611c2daa2e36c`
- `ThreeDmMigration.py` SHA256 `86d107f3bb50581f3b32b5c441abbd4373dd7609f1b03954cecdaae948bb639e`
- `Gui/ThreeDmMerge.cpp` SHA256 `9546383e5e04e0581c5b9a2d0b76ae7f8f4187922e6e2cbde5d03f68ee52d805`
- `tests/native/three_dm_merge.cpp` SHA256 `732d6aaf491f37f663b7fc6a6ec14fad92d64772f989ad452cb38773a3cb206f`
- `tests/three_dm_recursive_namespace_migration_smoke.FCMacro` SHA256 `7bd1303302c3b3526aa486246ecade8adbb6ba278468080723a77c93e02a3cb6`
- `tests/three_dm_recursive_copy_probe.FCMacro` SHA256 `f04c30a85dfa619ab6fb562c3c21d1442c78558165995911cacb365c7db80688`
- `docs/development/3dm-structural-blocks.md` SHA256 `1e5878fa394b86fe8d8a9de5d2150e448c9b76a42fdc54ee4b92521eabf61c97`
- `docs/features/OM9-FILE-012.md` SHA256 `b78748f1cbe57d8aa21d9dc92c3582e12f572f9f48218f7194f8d933fc812b8c`
- `docs/features/3dm-support-matrix.md` SHA256 `264b49e68a06d8768635934fb783e5908256f9139519ee614175a5e6332ad232`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `138d5f38d0b3c7a45ea545b594df8bd1bc1011ada0ab00928940bc7769f86873`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `82ff4d152ea5e77fafc018cd1c3fda47b82778a0e2ca70a61c975ed3c1defc29`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `3b902035f8745ec2a8b84ab940884b63335f79995f4b90c7be313f0c7b164734`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `a23ffa0419bc9dd92c2f2b18324506b1e623cb1574353a288004bf500cac570c`
- `README.md` SHA256 `97d8fdadc584777987947d57d5951eae67517c18fac893702605fab91336f1ca`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `5f58083c0fffe8d86ada1c9a49d2abedd1fdcea89b8f692f9b700a3495121c46`
