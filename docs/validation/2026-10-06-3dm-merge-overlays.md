# Selected preservation export: metadata, copies and affine blocks

Verified in the `codex/rhino5-3dm` module worktree; primary source is not integrated. SDK OpenMatrix9Gui and Python bindings were rebuilt from this worktree. This report does not establish full openNURBS support.

## Verified behavior

- Native overlays preserve Unicode names, object colors, visibility and lock. Changing lock preserves visibility; the existing `OpenMatrix9.Locked` user string retains hidden locked state. Invalid metadata types, color ranges and unsupported layer overlays reject without changing the destination.
- Edited copies receive fresh native UUIDs before geometry or instance overlays. Original/copy BRep volumes24/210 and instance points(101,202,303)/(71,82,93) remain independent. Copies share unchanged block dependencies. Deleted original selections are not restored; archive ownership survives FCStd reload.
- Reflected/nonuniform instance previews support rigid placement deltas. Export composes `currentPlacement * baselinePlacement.inverse() * sourceMatrix`, preserving the native definition and affine linear transform. Test points after translation and rotation are(18,56,72) and(34,28,72); an unchanged copy remains(-22,6,12). Modified preview geometry rejects atomically.
- Preview signatures remove root placement before copying geometry. Actual source BRep signatures use a fresh topology copy to avoid mutable OCC validation/cache flags producing false edit reports. Geometry edits remain detected by regression tests.
- Dedicated mixed BRep/mesh shear fixture now passes native reading and FreeCAD preservation export. The preview contains one Part feature and one Mesh feature; box volume24 and sheared mesh vertex(11.5,23,30) are verified. A90-degree rotation plus translation(40,50,60) leaves preview content signatures unchanged and preserves the native instance, definition and both member types. Exported box minimum bounds(17,60,90) and FCStd reopening are checked. Edited mesh preview rejection leaves existing destination bytes unchanged.

## Evidence

- Mixed/shear host9/9, process0: `build/three_dm_mixed_shear_smoke-1/1ce2c7a44ac44eb384babc07c945ece3/results.json`. Initial run used stale installed Python scripts and failed container binding; refreshing SDK scripts with `build/3dm-preservation-module.cmd` completed exit0 and the fresh test passed without changing implementation behavior.
- Refreshed structural host23/23, process0: `build/three_dm_block_structure_smoke-1/45cc9178f7bf4d01bed96e5e9b5b1d08/results.json`.

- Native CTest8/8, exit0,33.14s; includes both immutable user ring geometry fixtures. Merge target additionally verifies metadata acceptance and atomic invalid metadata rejection.
- Structural host23/23, process0: `build/three_dm_block_structure_smoke-1/ffece46aa2dc42ec8ce8f7b167dcf526/results.json`.
- Source signature14/14, process0: `build/three_dm_source_signature_smoke-1/c87b531332e44e12a13d2d64c74ba7a6/results.json`.
- Preservation export12/12, process0: `build/three_dm_preserved_export_smoke-1/592ac5e022c249969250c0c64469e0d4/results.json`.
- Preservation regression19/19, process0: `build/three_dm_preservation_smoke-1/183a0b931812493faef63f41a5c2aa62/results.json`; rollback, Undo/Redo, included snapshots and explicit geometry-only export remain verified.
- SDK build completed with exit0 after forced native inventory relink. An earlier loaded SDK module lacked the worktree's structural instance fields; fresh runtime acceptance above checks actual structural host binding.

## Remaining scope

Only the explicit `ThreeDm.export_preserved` API currently uses this writer. Standard/menu/CMD routes retain their guard. Layer assignment and canonical member edits were subsequently implemented and verified in `2026-10-06-3dm-member-layer-overlays.md`; definition membership/container changes and affine preview regeneration remain pending. Unsourced new CAD, distinct archive metadata merge, older documents without verified baselines and arbitrary opaque plugin/resource/history semantics remain pending. Real ring preservation export still rejects unknown plugin payloads; passing ring geometry tests does not establish preservation roundtrip. Actual Rhino5 application acceptance, full package review and primary-source integration remain pending. The private development checkout's Git metadata points to removed pre-public-history worktree registration; source files remain present and were tested directly, with no history reconstruction or public-source overwrite in this slice.
