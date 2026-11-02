# Delete Parent Or Change Input Policy

**Owner:** `OM9-L2-01-04` · **Milestone:** M04 · **Gate:** AUTO_ONLY

behaviour for missing inputs and changed output topology.

## Contract

- `crates/om9-core/src/associative.rs: BrokenInputPolicy, BrokenDependency, DeleteSourceReport, RebuildActivity, delete_source_input, drop_references_of`

## Positive oracle

`parent curve deleted; a child uses its semantic rail reference` -> child marked broken; the report offers KeepBroken/DetachDependents/RemoveDependents and the chosen policy leaves no dangling reference

## Negative oracle

`delete the parent while a child is rebuilding asynchronously` -> Error::Busy and the document is unchanged

## Verification

`crates/om9-core/tests/associative_rebuild.rs`:

- `deleting_the_parent_marks_the_child_broken_and_reports_the_policy_choices`
- `detach_and_remove_policies_leave_no_dangling_reference`
- `a_delete_during_an_async_rebuild_is_refused_not_raced`

The JSON fixtures in `implementation/fixtures/M04/OM9-L3-01-04-008/` are the machine-readable
record of the same oracles; the Rust tests are the executable form and are the authority.
