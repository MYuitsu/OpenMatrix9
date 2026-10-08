# Explicit migration of legacy 3dm provenance

The development module now reconstructs supported legacy owner/source/definition/proxy baselines from verified included native archives. Full openNURBS remains in progress. Tested runtime: `H:/FreeCAD-src/build/3dm-preservation-sdk`; this package has not integrated/pushed to the public primary checkout. Actual Rhino5 application acceptance remains untested.

## Behavior

`ThreeDm.migrate_archive(owner, rebuild_previews=False)` validates the active editable project and rejects an open user transaction. It verifies source schema/mode, hash, unit scale and namespace; identifies unambiguous live source bindings; compares existing native record fields with a fresh snapshot inventory; and checks property types. Missing owner schema/hash and namespace with unambiguous existing owner/source links can be reconstructed.

A hidden temporary reference document is bound from independently prepared native snapshot data. Original converted geometry, placements, matrices, metadata, definitions and proxy relationships produce original baselines. Current edited CAD/mesh is never used as an original baseline. Current canonical geometry, metadata, placements, instance targets and definition membership remain live edits. Ownership, record/class/capability, signatures, metadata/placement baselines and host provenance are restored in one Undo transaction. Display capability comes from the reconstructed host representation, rather than conflating it with the native record's retention capability.

The default checks affine display geometry and non-name child metadata against the independently reconstructed native display payload before initializing a cache baseline, then refreshes verified caches from current canonical members. Unverified/changed caches require `rebuild_previews=True`: generated display data is explicitly replaced from native reference data and rebuilt from the current verified graph. The rigid root placement delta and actual CAD/mesh edits stay intact. Failed rebuild rolls back source fields, cache replacement and generated objects. Undo restores original cache objects and missing legacy fields; Redo and FCStd persist the rebuilt provenance. The reference document closes on success/error, with the original project active again.

Import's read-only native preparation/conversion moved into `_prepare_import` so public import and the migration reference share conversion/validation. Public import still calls guarded native `commit3dm` for its transaction. The temporary reference uses private binding after validating the user's original project; it does not call or weaken the public commit guard. CMake includes the new `ThreeDmMigration.py` script in the module.

API details: [structural blocks and migration](../development/3dm-structural-blocks.md).

## RED to GREEN and regression scope

- The explicit migration API was initially missing. The legacy fixture now reconstructs exactly the original pre-edit source signatures while exporting edited points, current block placement and retained TextDot metadata/user text.
- Calling public import commit on a hidden reference correctly hit the active-project guard. Shared read-only preparation/private reference binding fixes the reference workflow while preserving the public guard.
- Existing property allocation initially failed on an existing instance matrix; compatible existing properties now receive values while new fields are allocated separately. All known source/proxy/preview property types are checked before mutation.
- A fixture exposed loss of `display-retained` host capability by restoring the native record capability. Migration now restores the independently reconstructed host capability.
- An incompatible integer preview signature initially raised a late type error; it now refuses during preflight. A separate missing-current-target rebuild fixture proves full transaction rollback, including original cache names, fields, object count, active project and observer state.
- The default native-matching preview fixture exposed a missing current-cache refresh/status. Verified default caches now refresh through the current graph as well as explicitly rebuilt caches.
- Original proxy retargeting was investigated independently and already worked. The new six-check native reread/Undo/Redo/FCStd regression proves current point membership/world coordinates without changing the existing definition-signature implementation.
- Native fixture adds an unused alternative point definition to the shared two-dimensional rational curve archive, allowing selection exclusion and retarget checks while preserving the original curve's exact native checksum.

## Verification

- Fresh isolated SDK module/scripts build exit0. A direct CMake regeneration attempt lacked the MSVC/Ninja environment; the existing vcvars-based isolated helper reconfigured/built successfully. Final source/install bytes match for all four Python files, with source/binary fingerprints stable through the complete GUI run.
- Fresh full native CTest8/8 exit0,37.65s, including both immutable user ring geometry fixtures. Those geometry fixtures do not prove opaque plugin/resource retention or actual Rhino5 acceptance.
- FreeCAD461/461 checks across28 suites; every owned final process exit0. Migration35 checks cover embedded archive use after external-file removal, exact original baseline recovery, edited geometry/retained metadata export, namespace recovery, transaction/hash/record/property/identity guards, explicit affine CAD/mesh rebuild, failed-rebuild rollback, native matrices/world coordinates, source proxy migration/promotion, Undo/Redo, FCStd and unchanged two-dimensional rational NURBS native checksum retention.
- Proxy retarget6 checks and all420 preceding GUI checks pass on this final script/binary combination. Rust archive code/tests remain unchanged; prior74/18 suites/fmt evidence remains under matching fingerprints.
- Legacy fixtures simulate old documents by removing supported properties from a modern import and retain real user-like geometry/metadata/placement edits. They establish those field layouts and guards, not every historical FCStd version or legacy object representation.

|Suite|Checks|Artifact|
|---|---|---|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/91d2f61d7dc94061ab097250df017316/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/5b1ebd19c1534e48848048af7c5bf283/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/11d0674e58184ebdae29b62d2eabf656/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/003c7921b340484a852568de89820d65/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/6e2752fa39c147e2bafee0124e8a570a/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/38c116bfb9b146d1956333859730e807/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/cb517eaf1fda4ed39286b179d00f4531/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/4da4e52b15644cefa0cce81fbfabad62/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/a556c07446be4aa2a2032cf54969a686/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/a350c5dba78948299d5771884b96b90b/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/5e05501cacc242fead502dedaa52cdb0/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/5c23798f7c9b4b11b571b52507c0d4c8/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/fd6a6cc5a02147b1b9acfe2e66ae3c06/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/25932a7480f343c39041206d3b0b85c2/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/72f7858929fa4275b99dc6a5182603b9/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/80bf31f115d140cbbce66ba4c08659e2/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/fe2bddfa68364072b31f0ae4ac6d7b01/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/d6c1659a0d0e4e109daecf35dbf219de/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/5334c6f2f632487587e509d35821df73/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/5229f3ee0666475ea01bbab6a01d81ad/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/0c273b0f9342436183f3d01d4e729451/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/f57d0834c07a40eb82a1e7cf9efd18be/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/c5f59f6a68e641aa9c1e768ec64a155c/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/ec174665ec9d420f843bfe28c13c828e/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/00830b137d8f4b7c94bb7920c420353a/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/248d0da8f5c24fc4af1a41b420517d80/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/4741efc75a4744acb7addc7cff7430f0/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/ebaea1d8a8f4494f9fff092818e0d6fe/results.json`|

## Remaining full-goal scope

Ambiguous copied source/definition identities require an explicit origin mapping/copy migration tool; the current API refuses guessing. Legacy object type upgrades, missing proxy membership provenance and broader real historical/retained/plugin migration fixtures remain pending. App::Link arrays/subelements, manually inserted identities/internal Name changes, retained transformed root actions and large graph performance still need work. Derived display fingerprints use12 significant digits; original source/native geometry remains unquantized.

Linked/external resources, broader appearance/annotations/settings/history/reference semantics and full class/version coverage remain incomplete. Different-source document metadata policy, shared standard/menu/CMD preservation routing, final whole-package review, public integration and actual Rhino5 application acceptance are pending. Copy quotas are per source namespace, not global RSS guarantees. Development Git registration remains absent; verified checkpoints preserve tested source.

## Fingerprints

- `CMakeLists.txt` SHA256 `85acd11cb266e474b35b09afe2d6acc70e7d7a510215cf985587ed1fc9fcac88`
- `ThreeDm.py` SHA256 `6365a10d7733e4ae91a2acecfd24b351fc1f5b42e43d800a9a7d10c3568a98c8`
- `ThreeDmMigration.py` SHA256 `d226f735ebf70aea9a12cf17e3bb8d950cf9b2e4ac3344b55edea2d9646572a5`
- `ThreeDmArchiveState.py` SHA256 `81f83c5e866b7eb7236b90314453b0f7d85e4ecc9cb9b2b10c72a68f48736401`
- `Gui/ThreeDmMerge.cpp` SHA256 `f5d0962e6016776524c40f7c0dcbfefe35caafa3efa52d437f62d8fe212e33a5`
- `Gui/ThreeDmInventory.cpp` SHA256 `fdebce5e299fa49f46cf92245ba98706d09c09d5e6f7ff3293e4da75285e4955`
- `tests/native/three_dm_merge.cpp` SHA256 `9ce34ed07dc16cd62ba52cce32fd1f9dfb3d1f2327e7b99b66b25cff85da6535`
- `tests/native/three_dm_blocks.cpp` SHA256 `d17f7806e7cc26c621dcf2ea0af9e2284d1d2b72c7124751cfd506ed4bc84e2c`
- `tests/three_dm_legacy_migration_smoke.FCMacro` SHA256 `3e87c7d76932e8d7ae00d12db727f7f3e90997619e1ddfb1c4b2bb0bd955968a`
- `tests/three_dm_proxy_retarget_smoke.FCMacro` SHA256 `ce157cd3506c662de0946b06ef4bc018921b6f501bccf604512380423711bca6`
- `tests/three_dm_source_proxy_creation_smoke.FCMacro` SHA256 `5df586ffe9d742f07c13819b8c8c7873470161bf93d6079918e55b54dfbabbaf`
- `docs/development/3dm-structural-blocks.md` SHA256 `0d0b966f3ea19e623b2e6b38ab3d2c97429431ff6abe14e56d19b03aa640bd9c`
- Isolated `bin/OpenMatrix9Gui.pyd` SHA256 `0c4d8869afeea9e182df0e56bcb96d37b66b5150b3bd87fc8a6d1cfc19c50464`
