# Recursive shared CAD/mesh and native NURBS migration

This slice extends actual `document.copyObject(group, True)` acceptance for `ThreeDm.migrate_archive(..., fork_namespace=True)` beyond the nested point fixture. Supported recursive copies preserve one canonical source per native geometry, multiple definition-member proxies, rigid/nested links and mixed affine CAD/mesh previews. Missing legacy proxy metadata baselines restore from the verified native membership graph. Current geometry and proxy transforms stay edits; the fork keeps the included source archive and native matrices immutable.

The tests exposed another source identity defect: ordinary transformed source proxies and independently copied source members received a fresh random output UUID each export. `ThreeDmArchiveState._source_copy_identity` now derives a UUIDv5 from the archive namespace, native source UUID and member host's internal Name. Export adds no properties or mutations. Each proxy of an independently copied canonical payload uses its own host identity, so separate proxy branches do not alias. Explicit new/promoted members continue to use their existing persistent creation identities. Native collision/reference checks remain enforced.

Stable output identities are verified for fixed namespace/native UUID/internal host Name, including save/reopen and repeated export. Label changes do not change the host Name. Internal Name changes, namespace forks, source changes or switching between original and copy member representations can change output identity; broader rename/lifecycle policy remains pending. Multi-source collision remapping retains the previous same-ordered-source-set guarantee.

## Verification

- Actual shared point family: three roots, two definition-member proxies share one copied canonical point. Editing its local geometry and placement produces analytic world points(17,29,11),(-34,27,44),(37,9,11). Original roots keep(11,22,3),(-22,6,12),(31,2,3). Combined export has one canonical ON_Point record per namespace. Sharing and namespace survive FCStd, with immutable external/included source bytes.
- Added native fixture `om9-shared-mixed-shear-block.3dm`: two definitions share the same ON_Brep and ON_Mesh; one root is sheared, the other reflected/nonuniformly scaled. The native fixture writer verifies four expanded members, preserving two CAD and two mesh types.
- Actual recursive mixed copy has two copied canonical payloads and four copied proxies. Current CAD/mesh edits and one proxy branch's placement/scale combine with immutable native matrices and root placement delta. Independent analytic CAD volumes are24/576 for original branches and720/2880 for edited copies. Twelve mesh world vertices match an independently calculated affine composition; the native graph retains BRep/Mesh records and complete definitions.
- Missing proxy metadata baselines restore without splitting shared payloads. Undo restores legacy metadata/namespace while preserving edited geometry. Redo and FCStd restore the same namespace, four proxy uses, native graph and output UUIDs. A current edited mixed cache remains guarded unless rebuilding is explicit.
- RED after correct fixture branch selection: native output UUIDs changed across FCStd export even though namespace, source IDs, geometry, host Names and all four proxy pointers were unchanged. Two copied record UUIDs were random. GREEN after the identity fix: repeated export and FCStd IDs are stable.
- The initial fixture used the copied definition Label to select the transformed branch; FreeCAD changes that Label on copy. The fixture now uses the native definition UUID and immutable shear matrix rather than relying on Label, and its expected volumes/coordinates remain independently derived.
- Two proxies of one independently copied canonical CAD payload emit distinct stable member IDs and analytic volumes48/192, leaving the original copied canonical CAD unchanged. A direct copied member coexists with both proxy branches, adding volume6 and its own stable UUID. Native counts and repeated output sets validate both identity paths.
- Rational NURBS family: two namespaces each retain one ON_NurbsCurve record shared across their definitions. Both native geometry CRCs exactly match the original rational two-dimensional curve; namespace migration avoids a host geometry rewrite. External source and included archive hashes stay immutable.
- New FreeCAD acceptance24/24. Full fresh isolated FreeCAD564/564 across32 suites; all owned final processes exit0. All previous540 checks ran with the final source/scripts/binary, and source/install/hash stability is checked before/after every suite.
- Fresh native CTest8/8,28.32s, final process exit0, includes both immutable user ring fixtures and the new native shared mixed fixture. Native ThreeDmBlockTests rebuilt successfully; cached isolated OpenMatrix9Scripts install exit0. Production native binary is unchanged in this Python-only production fix. Rust source/tests remain byte-identical to the ledger; prior74/18/fmt evidence retained.

```text
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-tests.cmd ThreeDmBlockTests
rtk proxy H:/FreeCAD-src/.pixi/envs/default/Library/bin/cmake.exe --build H:/FreeCAD-src/build/openmatrix9-preservation --target OpenMatrix9Scripts --parallel 20
rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd
rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-recursive-shared-regressions.py
```

The GUI runner invokes the owned hidden run_menu_smoke.ps1 process and waits for its final exit. Geometry/result artifacts reside beside each listed results.json. This slice verifies graph/geometry programmatically; it adds no visual screenshot audit.

|Suite|Checks|Artifact|
|---|---|---|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/40963ffff59f4c3ab01204d8b55d8e5d/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/41baa479414c443189ec863327ee099c/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/b9211c3c41074efcaf711a6c3372fa07/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/e174cc474e1a4e2abb3e8bb60287e1d6/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/6407fd7d91b74d0aaf33cbb6659af035/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/4d2b43061f8b4404a840a6d0c6c72844/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/a414f2b094cc4ce2b8bb67f64174cfc6/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/37e76970d32a4ae6ba47c7ebe3a9bc41/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/6ec517ff7c7d4017a1439fd4d80420c1/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/b841c90f8d4243a581a9d0865d185346/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/8f384856270f458896a04030f53e81d7/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/54ec7b1116c04961be28e6a415f98e5b/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/63664d5cac2c445c9d70d035b18886b3/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/0267f5eb46834e68a5e71c2816032f52/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/f6fb96051a0c47c8a17e3c7b9621be67/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/2cb84607c72f4cebb3c2d78b6f658807/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/bd2dd2e1c13a46ccae4085d68480a5c5/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/223d4e46590f49a7a013bbae38138b1f/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/16d7ac12059b4c6899ac0d4768cc1c04/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/357a79b70800426180f46972f707af1c/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/a276e117d62f4289b16284de19e35cc5/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/bfdbd207c40048639b7322d18421f942/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/f7466865eed44e369e0f59b850357bfe/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/79ebc39112b94a0ab298e2fb84870492/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/40b43400f7f24a59881a35a20ce93c0d/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/b7f5c52165ea48f0b838621f01fc1593/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/90f41ce7b93641b287a2c5a027d93d7b/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/49b35cec56704d2ba31d2d35c84658d4/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/38b2a9db6c4744a3a6498b31de59317f/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/6c9d4cfe44fa4f52b93efada8ab03efb/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/e72428c5015540e3ac3044c1aaff1a62/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/ef3520b76251436da2a60527054ba73b/results.json`|

## Remaining full-goal scope

Full openNURBS remains in progress. These fixtures establish supported recursive shared point, CAD/mesh and rational NURBS layouts; they do not establish arbitrary retained/plugin proxies, missing historical membership provenance or representation upgrades. Linked/external resources, complete appearance/annotation/settings/history/plugin references, arrays/subelements, class/reference/version coverage and large graphs remain pending. No matrix cell is promoted from aggregate test counts alone.

Different-source document policy, shared standard/menu/CMD preservation routing, final whole-package review/public integration and actual Rhino5 application acceptance remain pending. Advanced source stays in `H:/FreeCAD-src/build/om9-dev`, runtime in `H:/FreeCAD-src/build/3dm-preservation-sdk`. The Git registration remains unavailable; fingerprints and the SHA256-verified checkpoint identify this local package. This slice does not publish or integrate concurrent public checkout work.

## Technical status synchronization

OM9-FILE-012 Spec v1 implementation notes/checklists, API/support docs, README, approved plan and ledger record this scope. Source behavior/catalog metadata is preserved and the extension MANIFEST entry refreshed. No extension FEATURES catalog row or IMPLEMENTATION_STATUS.md exists in this package. No reusable skill business rule is changed.

## Fingerprints

- `ThreeDmArchiveState.py` SHA256 `a3f8f5df139a370035b6c22c1901fe7af5392a0f2234347f6b9729c0bfccd6bd`
- `tests/native/three_dm_blocks.cpp` SHA256 `0b43e296e55bbdf9ea0afc7730f39550b88426ecbaeba5e8682499b96b95f9c3`
- `tests/three_dm_recursive_shared_migration_smoke.FCMacro` SHA256 `dc810d018be202a7edb1790513e733fe0e9e6c3e66c42b6bc1333cca03940a67`
- `docs/development/3dm-structural-blocks.md` SHA256 `16d758f91a2178c48273aa47e298b3338794fd5cc7a3d7972077dd12fdadd6c7`
- `docs/features/OM9-FILE-012.md` SHA256 `f65318e42c262270285ebac82166e372ce606952cce061064ac7c2dce34141e4`
- `docs/features/3dm-support-matrix.md` SHA256 `179ee5a45275ca8be77157ae53065a77f101b352bed08e30c0348fdcc028cf8e`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `d1c6a880cfb76af942abc39fe18c1483992264da7d3b825efb7dad2ddace667c`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `94673d3503eb7adfb5875cf37668b63efd2f00681785fe39c34984ab54f4f5c3`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `f3c017d7c7d93b68edfcb427e670141d8ad8a1715377cfbbc0a9f1dee9611cc5`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `df5149e2429037defb49185aeffe14877d763d426d65b25c627f6251ec979cfc`
- `README.md` SHA256 `8ca1aa1f29961d80edf52db21a3e26e3bf240fca4ae9ba2d91a8ff187b43faaa`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `5f58083c0fffe8d86ada1c9a49d2abedd1fdcea89b8f692f9b700a3495121c46`
