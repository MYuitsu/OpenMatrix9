# Incremental Recompute And Cache

**Owner:** `OM9-L2-01-04` · **Milestone:** M04 · **Gate:** AUTO_ONLY

only recompute affected features and cache unchanged outputs.

## Contract

- `crates/om9-core/src/associative.rs: CacheEntry, RecomputeCache::{is_valid, record, invalidate, plan}, output_hash (SHA-256 over canonical geometry)`

## Positive oracle

`independent features X,Y and chain A->B; edit the source of A` -> only A and B recalculate; X and Y cache entries (revision and sha256) are unchanged

## Negative oracle

`a cache entry whose recorded input stamp is one revision old; a parameter edit without a revision bump` -> both are cache misses and A is planned even while the graph still calls it Clean

## Verification

`crates/om9-core/tests/associative_rebuild.rs`:

- `only_the_affected_chain_recomputes_and_independent_cache_entries_are_untouched`
- `a_cache_entry_with_an_outdated_input_stamp_is_a_miss_even_when_the_graph_is_clean`

The JSON fixtures in `implementation/fixtures/M04/OM9-L3-01-04-005/` are the machine-readable
record of the same oracles; the Rust tests are the executable form and are the authority.
