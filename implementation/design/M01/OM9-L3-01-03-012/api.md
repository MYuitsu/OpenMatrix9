# OM9-L3-01-03-012 — Document Snapshot And Restore (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-archive/src/snapshot.rs, crates/om9-app/src/session/contract.rs`. Reuse decision: **ADD a snapshot envelope over the existing container codec; no second codec**.

Oracle — positive: *Restore tái tạo đúng ID và bit hình học; Undo reset về mốc snapshot; scene dựng lại.*
Negative: *Snapshot tài liệu khác/bị sửa/version lạ bị từ chối; restore khi đang chạy lệnh là Busy.*

## 1. Typed API

```rust
pub struct DocumentSnapshot { /* format, version, document_id, epoch, sha256, payload */ }
impl DocumentSnapshot { pub fn capture(&Document, &Limits) -> Result<Self>;
                        pub fn restore(&self, &Limits) -> Result<Document>;
                        pub fn to_json(&self) -> Result<String>; pub fn from_json(&str, &Limits) -> Result<Self>;
                        pub fn document_id(&self) -> DocumentId; pub fn epoch(&self) -> u64; pub fn sha256(&self) -> &str; }
impl Controller { pub fn capture_snapshot(&self) -> Result<DocumentSnapshot>;
                  pub fn restore_snapshot(&mut self, &DocumentSnapshot) -> Result<SnapshotOutcome>; }
```

## 2. Errors (exact strings)

- `Invalid("snapshot document")  — snapshot của tài liệu khác`
- `Invalid("snapshot checksum")  — payload bị sửa/rút ngắn`
- `Unsupported("OM9 schema version; no implicit migration") — version lạ`
- `Busy                          — đang có lệnh chạy hoặc file operation`

## 3. Tests that carry the oracle

- `cargo test -p om9-archive --test document_snapshot -> 4 passed`
- `cargo test -p om9-app --test document_contract l3_012 -> 3 passed`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-012/OM9-L4-01-03-012-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-012/OM9-L4-01-03-012-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-012/OM9-L4-01-03-012-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-012/positive.json`, `negative.json`
