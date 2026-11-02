# Dirty Propagation And Topological Order

**Owner:** `OM9-L2-01-04` · **Milestone:** M04 · **Gate:** AUTO_ONLY

invalidate dependent outputs and recompute in DAG order.

## Contract

- `crates/om9-core/src/history_ops.rs: stale_recipes, mark_stale, mark_document_stale, plan_recompute, plan_document_recompute`
- `crates/om9-core/src/history.rs: HistoryGraph::execution_order (Kahn, cycle refusal)`

## Positive oracle

`DAG A->B->C and A->D; edit the source of A` -> dirty set {A,B,C,D}, producer before consumer, no duplicate evaluation

## Negative oracle

`cycle C->A` -> Invalid('History cycle') from execution_order, plan_recompute and typed_edges

## Verification

`crates/om9-core/tests/associative_rebuild.rs`:

- `editing_the_source_dirties_b_and_d_then_c_in_topological_order`
- `a_cycle_is_refused_instead_of_being_ordered_or_planned`

The JSON fixtures in `implementation/fixtures/M04/OM9-L3-01-04-003/` are the machine-readable
record of the same oracles; the Rust tests are the executable form and are the authority.
