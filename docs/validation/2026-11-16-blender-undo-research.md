# Blender Undo research — 2026-11-16

Scope: official Blender developer documentation and public `main` source read on
2026-11-16. This is architectural research, not a Blender performance benchmark,
an OM9 implementation, or certification of memory accounting. No Blender source
was copied into OM9. Links to `main` can change after this reading.

## Findings

1. Blender has one stack containing different `UndoType` implementations with
   encode/decode/free callbacks. Global memfile Undo and mode-specific mesh,
   sculpt, painting and other steps share that orchestration. Global Undo must
   not be described as merely recording commands or universally storing only
   object deltas. [Developer overview](https://developer.blender.org/docs/features/core/undo/).
2. Global Undo encodes a memory representation through the blend-file writer,
   passing the preceding memfile as a reference. `BLO_memfile_chunk_add` compares
   corresponding chunks with `memcmp`, reuses identical byte buffers, and adds
   newly allocated chunk bytes to the new memfile's size. It indexes IDs by
   session UID to preserve matching when their ordering changes.
   [Encode](https://github.com/blender/blender/blob/main/source/blender/blenkernel/intern/blender_undo.cc),
   [chunk handling](https://github.com/blender/blender/blob/main/source/blender/blenloader/intern/undofile.cc).
3. Some eligible arrays are held directly by an undo step with implicit shared
   ownership instead of serializing a new copy. Shared data is immutable; a
   writer copies shared data before modifying it. Reference counts and version
   information support ownership and change detection.
   [Writer](https://github.com/blender/blender/blob/main/source/blender/blenloader/intern/writefile.cc),
   [sharing primitives](https://github.com/blender/blender/blob/main/source/blender/blenlib/BLI_implicit_sharing.hh).
4. `memfile_undosys_step_encode` stores `undo_size` in `UndoStep::data_size`.
   `BKE_undosys_stack_limit_steps_and_memory` walks step metadata and sums that
   recorded number while enforcing count/memory limits. This particular limit
   pass does not re-inspect the geometry in every historical state. It still
   traverses the step list; it is not an O(1) history-limit algorithm.
   [Memfile step](https://github.com/blender/blender/blob/main/source/blender/editors/undo/memfile_undo.cc),
   [limits](https://github.com/blender/blender/blob/main/source/blender/blenkernel/intern/undo_system.cc).
5. Removing an old memfile step transfers ownership of shared chunks still
   needed by the next memfile, through `BLO_memfile_merge`. Implicit shared data
   releases its reference when the step is freed. This preserves remaining
   history while retiring old owners.
   [Merge/free](https://github.com/blender/blender/blob/main/source/blender/blenloader/intern/undofile.cc).
6. Mesh Edit Mode has deduplicated array storage and a low-priority background
   compaction task pool. It also has explicit waits before certain operations.
   This is not evidence that all Global Undo recording runs asynchronously.
   [Edit-mesh Undo](https://github.com/blender/blender/blob/main/source/blender/editors/mesh/editmesh_undo.cc).

## Important accounting distinction

`BlendWriter::write_shared` explicitly uses an approximate byte charge divided
by `strong_users()` for eligible shared data. Consequently Blender's per-step
Undo limit should not be presented as an exact census of physical RAM or copied
directly as OM9's hard memory-admission proof. Sharing and accounting precision
are separate issues; this estimate does not imply lossy geometry storage.

## Implications for OM9 — proposed, not implemented

Prior-work reconciliation: this is not a new memory/Undo initiative. The existing
[memory plan](../../implementation/design/M01/viewport-export-separation-20261102/PLAN_MEMORY.md)
already calls for shared ownership accounting, reservations for increments and
avoiding whole-history scans every frame. The
[2026-11-02 validation-proof cache](2026-11-02-undo-validated-owner-cache.md)
already referenced Blender and implemented proof reuse, explicitly retaining
storage inventory/accounting on each call. Its limited debug benchmark does not
establish performance of the current 512-sphere release path. The 100 MiB Undo
inventory cadence is separate from repeated aggregate admission checks.

The earlier [bounded Rhino 5 review](2026-11-02-rhino5-undo-architecture-review.md)
observed object/runtime-serial operation records and cached Undo record size/count
accounting in selected pseudocode. Official Rhino APIs distinguish reversible
[Delete](https://developer.rhino3d.com/api/rhinocommon/rhino.docobjects.tables.objecttable/delete)
from [Purge](https://developer.rhino3d.com/api/rhinocommon/rhino.docobjects.tables.objecttable/purge),
which permanently removes the object instead of retaining it for Undo. This
supports an object-level journal explanation for the inspected add/delete paths;
it does not prove every Rhino command avoids scans, how every BRep is copied,
or a matched performance advantage under the OM9 fixture.

OM9 already shares immutable source/display allocations and has an Undo memory
estimate/growth counter. The physical-console measurement instead identifies
repeated retained-owner validation across reservations, transaction preparation,
incremental scenes and proof attachment: sphere 512 logs 47 calls taking at
least 50 ms, totaling 7756.858 ms inside a 7867.289 ms Sphere operation.
[OM9 evidence](2026-11-16-console-sphere-metrics.md).

The useful direction is to cache storage summaries for immutable sources and
documents, and maintain accounting for owner transitions with stable allocation
and generation keys. Reuse still-valid admission evidence across a command;
recheck relevant versions/owners and current limits at atomic publication.
Keep strict accounting for current documents, Undo/Redo, external worker captures,
pending allocations and viewport owners. A per-step `data_size` alone does not
solve repeated whole-retained-state admission in OM9.

Keep canonical exact BRep and all requested Undo steps. Move eligible preparation
off UI only with accounted immutable captures, cancellation and stale-result
checks. Preserve the current quotas and failure/rollback behavior. Native and
concurrency tests must establish correctness before claiming reduced freezes.
No speedup or whole-window maximize fix is established by this research.
