# Undo: cache validation proofs for retained immutable owners

## Reference observations

The user's Rhino 5 Ghidra export was inspected read-only around `CRhinoDoc::DeleteObjectHelper` and `ReplaceObject`. The observed path records a deleted object's runtime serial in an active Undo record, places the deleted object on a retained list, and replacement invokes delete and add; the matching Undo dispatcher can undelete by runtime serial. This is bounded static pseudocode, without native disassembly or runtime corroboration. It does not establish the complete B-Rep storage/copy algorithm. No reference source was copied.

Blender's [MemFile structures](https://github.com/blender/blender/blob/main/source/blender/blenloader/BLO_undofile.hh) explicitly allow shared unchanged chunks; its [Undo architecture notes](https://developer.blender.org/docs/features/core/undo/) describe both stateful and differential steps. OM9 does not implement Blender's chunk system or Rhino's journal in this change. The common idea applied here is reuse of evidence for unchanged, retained data.

## Diagnosis and implementation

On a four-cavity debug probe with temporary timing instrumentation, 386 B-Rep validation calls accumulated approximately 2.084 s, and 44 `validate_retained_owners` calls accumulated 1.529 s; these measurements overlap and must not be added. This was an instrumented diagnostic, not a sampled profile. Temporary instrumentation was removed.

`UndoEntry` now holds two bounded, mutex-protected transient validation proofs. A proof is used only when that exact `Arc<Document>` is still owned by that entry and the caller's nine limits all cover the stricter limits under which validation succeeded. Other owners, tighter limits and invalid proofs execute full validation. Validation first attempts to retain half the aggregate headroom for later growth; if that attempt fails for any reason, it retries with the exact caller limits and preserves that result. Owned storage is inventoried and charged on each call; proof memory is part of the accounted UndoEntry size. The cache does not own a document or persist across sessions, and is excluded from semantic snapshot equality. Normal Undo/Redo and invalid-input validation remain enabled.

## Benchmark

Windows x64, default unoptimized debug build, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`. Same 20 mm cube, 21 sphere cuts, face order, MCP controller and start-to-save boundary as `2026-11-02-undo-brep-inventory-optimization.md`. Undo depth checked after each cut, 43 at save; post-timing Undo/Redo returned to 43/0 and the same object ID and B-Rep source digest. The final executable used in the primary run SHA-256 is `dc68a2a4c732aff789fa07b737b6cc1ac6a2af363a72760b93a3a53bbcd03fd1`.

| Single run | Undo | Total through save | Last cavity |
|---|---:|---:|---:|
| Before the first inventory optimization | On | 1212.506 s | 159.848 s |
| After inventory optimization | On | 239.924 s | 27.468 s |
| Validation cache, prior equivalent mutex build | On | 26.438 s | 2.051 s |
| Validation cache, final build | On | 40.038 s | 2.232 s |

The final run was 5.99× faster (83.31% less elapsed time) than the preceding 239.924 s baseline; the earlier equivalent cache build ran in 26.438 s. These are single, non-interleaved runs of different debug binaries; load and other run conditions were not controlled, so the difference is not a confidence interval or a release/desktop guarantee. The earlier no-Undo result was from an older binary and is not a controlled comparison with this cache.

Final, previous baseline and after-Redo native B-Rep source SHA-256 all equal `22ce14cab9bf89e8e5b5a45b806ce1196f8d44b2b16bec5645a0ba4f3783a23f`; displayed mesh volume is 7852.34279186927 mm³. The saved project reopened in the four-view GPU renderer. This verifies the tested path, not independent exact CAD manifoldness or 3D-print readiness.

## Checks and limitations

`cargo test -p om9-core -j 2`: 995 passed, 2 ignored. New focused test covers a matching retained Arc, narrower still-valid memory limit, too-small name limit, corrupted copy-on-write clone and preservation of `Send + Sync`; focused `atomic_transactions` and `transaction_scopes`: 26 passed. Application `mcp_stdio`: 3 passed; `solid_workflows`: 113 passed, 1 ignored; `stl_export`: 10 passed. A first application link attempt failed LNK1318 when drive L had approximately 202 MiB free. Only the two PDBs from that failed link and an earlier task-created obsolete MCP test PDB were removed; sequential focused tests then passed. Final public source audit and artifact pruning are recorded in the live ledger.

Raw receipts: `2026-11-02-dice-validation-cache-timing.jsonl` (final) and `2026-11-02-dice-validation-cache-prior-timing.jsonl`. The reproducible fixture script is `tools/dice_21_undo_validation_cache_benchmark.py`. Output project: `build/dice-21-validation-cache-verified-2026-11-02/dice.om9`. Remaining work: repeated release and desktop measurements, comprehensive UI acceptance, and profiling other geometry and large multi-document histories. The fixed two proof slots are an optimization only; misses still validate normally.
