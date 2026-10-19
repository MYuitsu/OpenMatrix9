# OM9-L3-01-03-010 — Selection State Document Contract (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-core/src/selection.rs, crates/om9-app/src/session/contract.rs`. Reuse decision: **ADD the typed selection contract over the existing ObjectId set in the controller**.

Oracle — positive: *Selection là identity có kiểu, thứ tự tất định; prune khi đối tượng mất.*
Negative: *Selection cũ/khác document là Stale; index cũ không được map sang đối tượng lân cận.*

## 1. Typed API

```rust
pub struct SelectionState { pub document: DocumentId, pub epoch: u64, pub session_generation: u64,
                             pub objects: BTreeSet<ObjectId>, pub subshapes: BTreeSet<SubshapeRef>, pub revision: u64 }
impl SelectionState { pub fn new(&Document) -> Self; pub fn validate(&self, &Document) -> Result<()>;
                      pub fn ordered(&self) -> Vec<ObjectId>; pub fn resolve_index(&self, published: &[ObjectId], index: usize) -> Result<ObjectId>;
                      pub fn prune(&mut self, &Document) -> bool; }
impl Controller { pub fn selection_state(&self) -> SelectionState;
                  pub fn validate_selection(&self, &SelectionState) -> Result<()>;
                  pub fn prune_selection(&mut self) -> bool; }
```

## 2. Errors (exact strings)

- `Stale("selection document") — khác document hoặc khác owner session`
- `Stale("selection epoch")    — tài liệu đã publish lần khác`
- `Stale("selection index")    — danh sách publish bị rút ngắn/đổi thứ tự`
- `Invalid("selection revision"), Missing("selection object")`

## 3. Tests that carry the oracle

- `cargo test -p om9-core --test selection_state -> 7 passed`
- `cargo test -p om9-app --test document_contract l3_010 -> 3 passed`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-010/OM9-L4-01-03-010-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-010/OM9-L4-01-03-010-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-010/OM9-L4-01-03-010-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-010/positive.json`, `negative.json`
