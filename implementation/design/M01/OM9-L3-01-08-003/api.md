# OM9-L3-01-08-003 — Document Absolute Relative Tolerances (typed API)

Owner module `OM9-L2-01-08` (units, tolerances, basic parameters; stage `E01`, gate **AUTO_ONLY**).
Module contract: `implementation/design/M01/OM9-L2-01-08/api.md` §3.
Implementation owner: `crates/om9-core/src/history.rs, model.rs, transaction.rs`. Legacy decision: **ADAPT**.

Oracle — positive: *Tài liệu có một policy tuyệt đối/tương đối/góc lưu kèm tài liệu (admitted ở schema 4); tài liệu mới dùng policy template tường minh; mutation là một Undo, không đổi context_revision.*
Negative: *Giá trị sai bị từ chối theo tên trường; tài liệu không khai báo policy trả Missing, không bịa mặc định.*

## 1. Typed API

```rust
impl Tolerance { pub fn try_new(absolute_mm: f64, relative: f64, angular_rad: f64) -> Result<Self>;
                 pub fn admit_feature(&self, size: LengthMm) -> Result<()>; }
Document.tolerance: Option<Tolerance>; Document::{tolerance, effective_tolerance}
Mutation::Tolerance { document, expected_revision, tolerance }   // one Undo, context_revision unchanged
```

## 2. Errors (exact strings)

- `Invalid("absolute tolerance") / Invalid("relative tolerance") / Invalid("angular tolerance")`
- `Invalid("feature at or below document absolute tolerance")`
- `Missing("document tolerance") — no default is invented`
- `Stale("document tolerance revision")`

## 3. Legacy `rust/` audit

- `rust/src/tolerances.rs`
- `rust/src/units.rs (Context.absolute_mm/relative_ratio)`

Legacy keeps tolerance inside the unit Context; OM9 makes it a document-level policy with its own mutation so it is auditable and undoable.

## 4. Tests that carry the oracle

- `cargo test -p om9-core --test document_tolerance`
- `cargo test -p om9-archive --test document_identity_round_trip a_document_tolerance`
- `cargo test -p om9-core --test l2_01_08_precision`

## 5. Evidence

- `implementation/evidence/M01/OM9-L3-01-08-003/OM9-L4-01-08-003-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-08-003/OM9-L4-01-08-003-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-08-003/OM9-L4-01-08-003-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-08-003/positive.json`, `negative.json`
