# OM9-L3-01-03-006 — Layer Selection Visibility Rules (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-core/src/model.rs (Document::layer_permissions, permissions_for_entity)`. Reuse decision: **REUSE the existing permission chain; prove it with the locked/hidden cases**.

Oracle — positive: *Layer ẩn thì không vẽ/không pick; layer khoá vẫn vẽ nhưng không sửa.*
Negative: *Mọi ghi vào đối tượng thuộc layer khoá bị từ chối, tài liệu không đổi.*

## 1. Typed API

```rust
pub struct Permissions { pub visible: bool, pub selectable: bool, pub editable: bool, pub reference_snappable: bool }
impl Document { pub fn permissions(&self, id: ObjectId, snap_locked: bool) -> Result<Permissions>;
                pub fn permissions_for_entity(&self, e: &Entity, snap_locked: bool) -> Result<Permissions>; }
```

## 2. Errors (exact strings)

- `Permission("source edit") — ghi bị từ chối trước khi clone tài liệu, nên không có ghi một phần`

## 3. Tests that carry the oracle

- `cargo test -p om9-core --test document_graph l3_006 -> 2 passed`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-006/OM9-L4-01-03-006-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-006/OM9-L4-01-03-006-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-006/OM9-L4-01-03-006-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-006/positive.json`, `negative.json`
