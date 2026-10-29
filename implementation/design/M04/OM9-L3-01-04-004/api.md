# Stable Semantic Reference Linking

**Owner:** `OM9-L2-01-04` · **Milestone:** M04 · **Gate:** AUTO_ONLY

preserve rail/profile/gem attachment under recompute.

## Contract

- `crates/om9-core/src/associative.rs: SubshapeKind, SubshapeSignature, SemanticReference, bind_reference, resolve_semantic_reference, relink_reference, drop_references_of`

- `crates/om9-core/src/history.rs: HistoryGraph.references`
- `crates/om9-core/src/producers.rs: SnapshotTopology` and `crates/om9-kernel/src/associative.rs: OcctTopology` publish edge/face/vertex names from the produced geometry

## Positive oracle

`profile references a named edge; rebuild the rail with the same semantic identifier and a 0.01 mm move` -> the reference resolves to the same intended edge after the allowed edit; the bound id is unchanged

## Negative oracle

`two subshapes with the same name, a changed kind, a move beyond tolerance, a stale revision, a detached owner` -> Invalid('ambiguous semantic reference'), Invalid('semantic subshape kind'), Stale('semantic reference moved'), Stale('semantic reference revision'), Invalid('semantic reference detached feature')

## Verification

`crates/om9-core/tests/associative_rebuild.rs`:

- `a_semantic_reference_survives_an_allowed_edit_and_refuses_ambiguity`

- `a_detached_feature_cannot_keep_a_semantic_reference`

The JSON fixtures in `implementation/fixtures/M04/OM9-L3-01-04-004/` are the machine-readable
record of the same oracles; the Rust tests are the executable form and are the authority.
