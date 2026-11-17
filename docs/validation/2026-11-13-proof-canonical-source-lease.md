# Canonical source lease for topology proofs

Follow-up: the remesh path subsequently gained a scoped source-only rebind; see [attested source remesh rebind](2026-11-13-attested-source-remesh-rebind.md). This record describes the earlier verified build.

2026-11-13. When a topology proof is adopted, it now retains an accounted immutable source/topology lease, without retaining the document or its display. The remesh worker's normal proof capture reuses that same lease. A different lease must match the complete source bytes and topology IDs as well as the existing fingerprint and account checks; proof deduplication and rebind likewise compare exact source/topology content. Leases sharing both immutable canonical allocations take a constant-time path, even when their lease handles differ. The initial native descriptor attestation and existing full display checks remain in force: exact source equality alone is not a native attestation.

On Windows with `CARGO_TARGET_DIR=L:\om9-target`, OCCT 8.0.1 runtime and `-j 2`:

- App `--lib topology_ -- --test-threads=2`: 160 passed, 6 ignored. The new fixture confirms repeated worker captures share the attested source lease, rejects another root, and shows that keeping only the proof preserves source but not document display; releasing the final proof releases that source. The batch includes remesh, history, cache reclamation, stale and cancellation paths.
- Core lib: 386 passed, 1 ignored; the fixture also checks independent lease handles sharing the same canonical allocations.
- Kernel lib: 33 passed, 3 ignored. The kernel printed `File was not written with this version of the topology` after its success summary. Workspace check passed with two existing warnings; no-default-features app check passed with 21 warnings.

Proof lifetime can now extend canonical source lifetime, tracked by the shared account; it does not authorize eviction of a document display. Comparing independent leases can scan the full source and topology on the calling thread; its large-model latency has not been measured. Existing proof lookup/rebind still checks the document-backed display, and the document/history model still owns display. Source-only rebind and cold restoration, NativeBrep disposition, v5 and acceptance remain open. No actual physical OOM, native peak, desktop or large-workload timing is claimed.

SHA-256: `transaction.rs` `9e2d811504612cd3895958b75f60bb9442ee5e474aeb05431ff16444f80c1530`; `topology_capture.rs` `482a1056926509c58c9fac0ade82e6b9525a18cc5b7eccf798801ad98d99f24f`; `session.rs` `c9a2d6fccd4cb6f12ae3519bbda87100da739990f6dd975595133ea869ce6480`.
