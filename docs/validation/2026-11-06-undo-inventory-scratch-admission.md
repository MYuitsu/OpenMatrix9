# Undo inventory scratch admission — 2026-11-06

The production coordinator's full Undo inventory built temporary BTree indexes
for document, B-rep source/topology/display and resource identities without a
separate admission check. It could therefore allocate those indexes despite
having no physical headroom. The new fixture initially failed because the old
inventory returned success under a simulated zero-headroom sample.

Before building the indexes, the coordinator now computes a conservative
duplicate-inclusive BTree storage bound, checks the existing managed ceiling,
pending reservations, available physical memory and system commit when their
samplers are configured, and validates retained owners with that scratch bound
deducted. It holds the existing external-document and shared-account locks in
the established order through inventory, so peers cannot spend the same
headroom while the scratch exists. No second budget or preallocated Undo quota
is introduced. A failed refresh marks the cached estimate stale; setting an
Undo policy returns the admission error before replacing the policy.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -p om9-core --lib -j 2`: 353 passed.
- `cargo test -p om9-core --lib undo_inventory -j 2`: eight passed, including
  simulated managed-ceiling, physical-RAM and commit refusal, policy/Redo
  preservation, retry, peer admission serialization and capture-drop races.
  These eight are included in the 353, not additional distinct cases.
- `cargo test -p om9-core --test atomic_transactions -j 2`: 29 passed.
- `cargo test -p om9-app --lib incremental_scene_memory_tests -j 2`: 12
  passed, including the existing Sphere snapshot pressure cases.

SHA-256 at validation: `transaction.rs`
`add8a58bf44b099fcea62fe60ada4dcd51478bfe8fd6f7a70e6db10b7a732cc2`;
`transaction/undo_memory.rs`
`b0c24248f31b98e85be21bb32b886f9cdee85e2023a0aec8f53dce421a22620a`.
The physical and commit pressure cases are simulated; no process RSS, native
OCCT peak or desktop latency was measured. The conservative duplicate-inclusive
bound may refuse an inventory that would fit after deduplication; its effect on
large histories and latency has not been benchmarked. This is a managed Undo
scratch fix, not source/display eviction or a completed Sphere gate. The open
OM9 session and drawing were not changed.

The later indexed dedup change and release timing are recorded in
[Undo inventory scaling](2026-11-06-undo-inventory-scaling.md); the hashes
above refer to the earlier admission-only version.
