# Builder Preview Vs Committed Features

**Owner:** `OM9-L2-01-04` · **Milestone:** M04 · **Gate:** AUTO_ONLY

discard preview graph state without affecting the saved design.

## Contract

- `crates/om9-core/src/associative.rs: PreviewSession::{begin, base_revision, staged_counts, set_parameter, stage_recipe, stage_entity, stage_result, plan, commit, discard}`
- The preview stores only staged recipes/entities/results: `begin` copies the base identity/revision, not the document

## Positive oracle

`preview rail diameter 18->19->20 without Apply` -> committed document id/geometry/hash unchanged; one Apply yields only the final value

## Negative oracle

`Escape midway; Apply after the committed document moved` -> transient nodes disappear; Stale('preview base document')

## Verification

`crates/om9-core/tests/associative_rebuild.rs`:

- `preview_edits_never_touch_the_committed_document_until_one_apply`

- `escape_discards_transient_nodes_and_a_moved_base_is_refused`

The JSON fixtures in `implementation/fixtures/M04/OM9-L3-01-04-006/` are the machine-readable
record of the same oracles; the Rust tests are the executable form and are the authority.
