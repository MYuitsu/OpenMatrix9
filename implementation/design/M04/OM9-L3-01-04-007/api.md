# History Recompute Failure Rollback

**Owner:** `OM9-L2-01-04` · **Milestone:** M04 · **Gate:** AUTO_ONLY

keep the last valid result and report the failed feature.

## Contract

- `crates/om9-core/src/associative.rs: RebuildRequest, RebuiltOutput, RecomputeOutcome, FeatureSnapshot, recompute_feature`

## Positive oracle

`valid ring rail, then a parameter the producer refuses` -> recompute fails, the previous geometry/published result/cache entry are retained, the feature is Failed(reason)

## Negative oracle

`producer returns no output, a different object id, or geometry of the wrong kind` -> Failed('invalid rebuilt output set' / 'invalid typed feature output'); no corrupt committed node

## Verification

`crates/om9-core/tests/associative_rebuild.rs`:

- `a_successful_rebuild_updates_geometry_published_result_and_cache`
- `a_failed_rebuild_keeps_the_last_good_geometry_result_and_cache_entry`
- `a_producer_that_breaks_the_recorded_output_set_creates_no_corrupt_node`

The JSON fixtures in `implementation/fixtures/M04/OM9-L3-01-04-007/` are the machine-readable
record of the same oracles; the Rust tests are the executable form and are the authority.
