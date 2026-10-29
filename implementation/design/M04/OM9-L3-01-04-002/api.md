# Feature Provenance From Command

**Owner:** `OM9-L2-01-04` · **Milestone:** M04 · **Gate:** AUTO_ONLY

record the generating command and the user parameters for each result.

## Contract

- `crates/om9-core/src/associative.rs: FeatureProvenance, RecordedInput, ReopenFeature, feature_provenance, reopen_feature, replay_environment`

## Positive oracle

`builder RingRail diameter_mm=18.0 with its originating command parameters` -> re-open the exact command, typed inputs and unit preserved

## Negative oracle

`command payload with a bare Number parameter, a nil required input id, a duplicate role or an empty selection` -> Invalid('feature parameter unit/type'), Invalid('feature input identity'), Invalid('feature input role'), Invalid('feature input selection')

## Verification

`crates/om9-core/tests/associative_rebuild.rs`:

- `provenance_reopens_the_exact_command_parameters_inputs_and_unit`
- `provenance_refuses_a_bare_number_a_missing_input_id_and_a_duplicate_role`

The JSON fixtures in `implementation/fixtures/M04/OM9-L3-01-04-002/` are the machine-readable
record of the same oracles; the Rust tests are the executable form and are the authority.
