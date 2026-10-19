# OM9-L3-01-03-008 — Block Definition And Instance Graph (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-core/src/{model,transaction}.rs, crates/om9-scene/tests/shared_instances.rs`. Reuse decision: **REUSE Definition/member_transform/cycle detection; ADD one atomic DefinitionInsert**.

Oracle — positive: *Hai occurrence dùng chung một definition, placement độc lập; một Undo xoá cả definition và các occurrence.*
Negative: *Self-cycle bị từ chối không ghi một phần; vòng lặp lồng nhau bị validate bắt.*

## 1. Typed API

```rust
impl Document { pub fn instance(&self, definition: DefinitionId, placement: Transform,
                            layer: Option<LayerId>) -> Result<Entity>;
                pub fn member_transform(&self, root: ObjectId, path: &[ObjectId]) -> Result<Transform>; }
Mutation::DefinitionInsert { document, expected_revision, def: Box<Definition>, instances: Vec<Entity> }
```

## 2. Errors (exact strings)

- `Invalid("definition cycle")     — definition chứa chính nó hoặc vòng lặp lồng nhau`
- `Missing("instance definition")   — occurrence trỏ definition không tồn tại`
- `Invalid("duplicate definition ID"), Invalid("duplicate object ID")`
- `Invalid("allocated definition UUID"), Invalid("allocated object UUID")`

## 3. Tests that carry the oracle

- `cargo test -p om9-core --test document_graph l3_008 -> 5 passed`
- `cargo test -p om9-scene --test shared_instances -> 4 passed`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-008/OM9-L4-01-03-008-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-008/OM9-L4-01-03-008-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-008/OM9-L4-01-03-008-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-008/positive.json`, `negative.json`
