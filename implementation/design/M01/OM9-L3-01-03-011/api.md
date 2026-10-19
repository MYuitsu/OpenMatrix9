# OM9-L3-01-03-011 — Document Change Notification (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-core/src/changes.rs, crates/om9-app/src/session/contract.rs`. Reuse decision: **ADD the pure diff and the observer host; publish from the existing commit sites**.

Oracle — positive: *Mỗi publish sinh tối đa một batch đúng thứ tự, revision tăng; tài liệu không đổi thì không có batch.*
Negative: *Observer panic/lỗi bị gỡ sau một diagnostic; observer không thể mutation tái nhập.*

## 1. Typed API

```rust
pub enum Change { CurrentLayerChanged(Option<LayerId>), LayerOrderChanged, LayerCreated(LayerId),
                   LayerModified(LayerId), LayerRemoved(LayerId), EntityAdded(ObjectId), EntityRemoved(ObjectId),
                   EntityModified(ObjectId), ExtensionChanged(ObjectId), GroupCreated/Modified/Removed(GroupId),
                   FeatureAdded/Removed/Modified(FeatureId) }
pub struct ChangeBatch { pub document: DocumentId, pub from_revision: u64, pub to_revision: u64,
                         pub cause: ChangeCause, pub changes: Vec<Change> }
pub fn diff(before: &Document, after: &Document) -> Result<Vec<Change>>
pub trait DocumentObserver { fn document_changed(&mut self, batch: &ChangeBatch) -> Result<()>; }
impl Controller { pub fn add_document_observer(&mut self, Box<dyn DocumentObserver>) -> usize;
                  pub fn is_recording_changes(&self) -> bool; pub fn observer_diagnostics(&self) -> &[ObserverDiagnostic];
                  pub fn document_changes(&self) -> impl Iterator<Item = &ChangeBatch>; }
```

## 2. Errors (exact strings)

- `Invalid("change document") — diff hai tài liệu khác nhau`
- `Observer trả Err hoặc panic: bị gỡ, ghi một ObserverDiagnostic, các observer khác vẫn chạy`

## 3. Tests that carry the oracle

- `cargo test -p om9-core --test document_changes -> 6 passed`
- `cargo test -p om9-app --test document_contract l3_011 -> 4 passed`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-011/OM9-L4-01-03-011-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-011/OM9-L4-01-03-011-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-011/OM9-L4-01-03-011-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-011/positive.json`, `negative.json`
