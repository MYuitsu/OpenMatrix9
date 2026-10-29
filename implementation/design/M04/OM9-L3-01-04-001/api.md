# Feature Record And Typed Dependency Edges

**Owner:** `OM9-L2-01-04` · **Milestone:** M04 · **Gate:** AUTO_ONLY

stable feature graph node with typed inputs and outputs.

## Contract

- `crates/om9-core/src/associative.rs: PortKind, FeaturePort, record_feature, typed_edges, validate_typed_ports`
- `crates/om9-core/src/history.rs: Recipe.input_ports / Recipe.output_ports (serde default, skipped when empty)`

## Positive oracle

`nodes rail->profile->sweep; typed GeometryRef vs ScalarMm` -> 3 nodes, edges follow typed IO; a curve fed into ScalarMm is rejected

## Negative oracle

`feed CurveRef into ScalarMm input; a port role or output kind that does not match` -> Invalid('object bound to scalar input'), Invalid('typed input port role'), Invalid('typed feature output')

## Verification

`crates/om9-core/tests/associative_rebuild.rs`:

- `typed_edges_follow_the_declared_io_and_link_producers`
- `a_curve_in_a_scalar_input_and_a_wrong_output_type_are_refused`
- `a_legacy_recipe_without_ports_is_reported_untyped_never_guessed`
- `record_feature_enforces_identity_document_and_output_ownership`

The JSON fixtures in `implementation/fixtures/M04/OM9-L3-01-04-001/` are the machine-readable
record of the same oracles; the Rust tests are the executable form and are the authority.
