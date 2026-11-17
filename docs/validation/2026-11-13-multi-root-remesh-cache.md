# Multi-root remesh representation cache

2026-11-13. The existing optional, document-independent remesh cache now retains up to four proven source/profile representations instead of one. The fixed-capacity inline metadata is covered by the cache owner's reservation. Within this bounded transitional cache, hits update recency, duplicate source/profile/proof entries reuse the existing owner, and pressure or insertion reclaims the oldest unpinned representation. If every slot is pinned, the new optional cache insertion is refused; scene publication still succeeds. A hit still requires current-source proof validation and freshness checks before publication. Display and buffer owners retain their existing reservations; a scene or reader pin prevents reclaiming them.

## Verification

On Windows with `CARGO_TARGET_DIR=L:\om9-target`, OCCT 8.0.1 runtime and `-j 2`:

- `cargo test -p om9-app --lib remesh_cache -- --test-threads=2`: 6 passed. Two distinct attested Box/Sphere roots reuse their own profiles through the product remesh path, without changing document or history. A reader pin survives pressure after its representation leaves the active scene. The fixture's reported release equals the reduction in managed reservations. A five-profile fixture checks recency eviction, and another keeps all four slots pinned while a sixth profile is published.
- App `--lib remesh -- --test-threads=2`: 78 passed, 1 ignored before the final extra all-pinned test and strengthened pin assertion; those final affected tests passed in the six-test run above. This includes stale, cancellation, proof refusal and GPU remesh fixtures.
- Workspace `cargo check -j 2` passed with two existing warnings. The public source audit reported no errors; artifact pruning removed no runs and retained 169 older cited runs.

The four slots are a bounded transition, not a general representation store or a final capacity policy. Ordinary display is still owned by Document/Undo, the cache only captures attested remesh overrides, and it does not yet serve Open or cold Undo/Redo. No real OOM, process/GPU peak, desktop interaction, native opaque payload migration or multi-document cold restoration is established. C1–C5 and `.om9` v5 remain open.

SHA-256: `topology_remesh.rs` `fff6d74ae97e859a0d39d2ad7e39e9598dbd6bef687ab9367fb2886b80d41bb6`; `session.rs` `0ec57e7f6f44ba69650a7a3735bdb7c098995b1a38b22729a574317f6806c0dd`.
