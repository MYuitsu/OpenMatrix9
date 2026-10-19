# OM9-L3-01-03-005 — Entity To Layer Membership (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-core/src/model.rs, transaction.rs (Mutation::Attributes)`. Reuse decision: **REUSE Attributes.layer + the existing atomic Attributes mutation**.

Oracle — positive: *Đổi layer giữ nguyên EntityId, chỉ tăng attributes_revision.*
Negative: *Xoá layer không để lại membership treo; không tự remap.*

## 1. Typed API

```rust
pub struct Attributes { pub layer: Option<LayerId>, pub visible: bool, pub hidden: bool,
                        pub selectable: bool, pub locked: bool, pub color: Option<[u8; 4]> }
Mutation::Attributes { source: SourceStamp, attributes: Attributes }  // bumps attributes_revision only
```

## 2. Errors (exact strings)

- `Missing("entity layer")       — entity trỏ layer không tồn tại`
- `Permission("source edit")      — ghi vào đối tượng/layer bị khoá`
- `Permission("assigned layer")   — layer đích không cho ghi`

## 3. Tests that carry the oracle

- `cargo test -p om9-core --test document_graph l3_005 -> 2 passed`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-005/OM9-L4-01-03-005-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-005/OM9-L4-01-03-005-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-005/OM9-L4-01-03-005-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-005/positive.json`, `negative.json`
