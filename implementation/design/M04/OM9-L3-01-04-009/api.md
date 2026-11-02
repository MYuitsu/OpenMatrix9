# Associative Rebuild Geometry Fixtures

**Owner:** `OM9-L2-01-04` · **Milestone:** M04 · **Gate:** AUTO_ONLY

reference ring/gem/settings history regression fixtures.

## Contract

- `crates/om9-core/src/associative.rs: BaselineNode, BaselineEdge, FeatureGraphBaseline, feature_graph_baseline, load_baseline, node_content_key`

## Positive oracle

`fixture graph ring rail->profile->sweep with a gem placeholder` -> deterministic baseline hash across sessions; an edited rebuild matches a fresh build

## Negative oracle

`corrupt fixture JSON, a wrong node index, an edge outside the record, a tampered hash` -> load/verify fails with Io/Invalid('feature baseline node order'/'feature baseline edge')/Stale('feature baseline hash')

## Verification

`crates/om9-core/tests/associative_rebuild.rs`:

- `the_baseline_is_deterministic_across_sessions_and_round_trips`
- `editing_a_parameter_and_rebuilding_matches_a_fresh_build`
- `corrupt_or_missing_fixture_data_fails_to_load`
- `options_are_hashed_by_content_not_by_allocation`

The JSON fixtures in `implementation/fixtures/M04/OM9-L3-01-04-009/` are the machine-readable
record of the same oracles; the Rust tests are the executable form and are the authority.
