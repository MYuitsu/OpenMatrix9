# Shared NativeBrep payload

`NativeBrep.payload` now uses `Arc<Vec<u8>>`, so document clones and Undo/Redo versions can share retained native 3DM bytes when only placement or other unrelated state changes. The public Rust field type changes; its existing chunked serialized representation does not. Display meshes remain separately cloned, and mutating the payload through copy-on-write still creates a separate allocation.

On the external E: target with `-j 2`, core library, atomic transaction and undo-memory tests passed (269, 24 and 8 respectively). New tests check allocation identity through a move, Undo and Redo using a placeholder payload, and check chunked serde. Five document snapshot tests include a 70,000-byte synthetic payload round trip; seven project codec acceptance tests also passed. Scene osnap and surface broad-phase tests passed. Selection overlay and B-rep picking suites passed 12 and 19 tests respectively. A real retained OCCT sphere fixture produced identical STL for full and partial display meshes. Workspace test targets typechecked; there was an existing unused-mut warning.

The independent kernel 3DM test requires `OM9_BREP_3DM_PAYLOAD` and was ignored without it. An initial invocation also failed to link without the required `3dm-brep` feature; the corrected feature-enabled invocation linked. The synthetic snapshot is not evidence of geometric 3DM reimport, nor does the STL comparison certify global error.

## Native geometry snapshot round trip

An additional app integration test generated an OCCT sphere of radius 5 mm, converted it to a native ON_Brep payload, stored and restored that payload through the document snapshot JSON, and reconstructed it using the OCCT 3DM adapter. The payload bytes and entity ID survived, the reimported volume was within 1e-5 mm³ of the analytic sphere, and the reconstructed topology reported one solid and one shell. The test passed using OCCT 8.0.1 and the external E: target. This is one generated sphere and the snapshot path, not a full .om9 file migration, a complete 3DM export/reimport test, a general topology oracle or a global deviation proof.

## Undo inventory follow-up

Undo's estimated-memory inventory now counts the allocation once when history is its only tracked owner and excludes it while the current document or a tracked capture owns it. A 1 MiB synthetic payload regression failed before this change and passed after it: three moves, a tracked capture, deletion, capture drop, Undo and Redo exercise the ownership transitions. This does not validate native geometry.

After this follow-up, 269 core library, 25 atomic transaction and eight undo-memory tests passed on the E: target with `-j 2`.

## Document inventory follow-up

Document storage and validation now charge a shared NativeBrep allocation once across its entities and definition members. A red/green synthetic 1 MiB fixture compares two aliases with two separate but byte-identical allocations, verifies a memory limit between them, and exercises aliased history through Undo/Redo. Undo's cross-document inventory was adjusted to avoid subtracting document-internal aliases twice. Every geometry continues to be validated. The scans are quadratic in entity count and are not performance-validated for large documents.

On the E: target with `-j 2`, all 269 core library, 26 atomic transaction and eight undo-memory tests passed after this change.

## Retained-owner admission follow-up

The World, retained history and external capture inventories now deduplicate native payload allocations across document versions, including allocation-free fallback and indexed admission. The temporary index is included in its preflight budget. A 1 MiB regression initially failed with duplicate charges and passes after the change; it compares indexed and fallback results and exercises a tight limit. Additional synthetic tests cover two documents sharing an allocation, multiple detached captures, and equal but independently allocated payloads. On the E: target with `-j 2`, 272 core library, 26 atomic transaction and eight undo-memory tests passed.

This does not complete 60% RAM accounting: other owners, allocation peaks, display derivatives and the interactive 256 MiB ceiling remain open. Allocation-free scans can be quadratic and have not been benchmarked on large documents. This is partial P0/P1 work, not completion of either gate.

## Streaming payload chunks

NativeBrep serialization now visits existing payload chunks without allocating a vector of chunk references. Deserialization appends each decoded chunk before requesting the next, rather than retaining the entire nested chunk vector alongside the final payload. Legacy irregular chunk sizes remain accepted; the serialized 32 KiB chunk format is unchanged. A large single legacy chunk can still allocate that whole chunk, and vector growth/allocator overhead can still increase peaks. No process-memory or performance measurement is claimed.

On the external E: target with `-j 2`, 276 core library, 26 atomic transaction and eight Undo tests passed; the entire archive suite, a generated OCCT sphere snapshot/reconstruction test and 20 file-lifecycle tests passed. Tests cover 32 KiB boundaries, irregular legacy chunks and a 70,000-byte snapshot. The first build reported an unused import; it was removed and the two focused core tests passed again. This reduces transient owners but does not close the managed RAM, native-file compatibility or 256 MiB gates.
