# OM9-L3-01-03-002 — Typed Scene Entity Records (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-core/src/geometry.rs, crates/om9-archive/src/container.rs`. Reuse decision: **EXTEND the existing GeometryKind with tag/from_tag; EXTEND the container decode error mapping**.

Oracle — positive: *Sáu loại entity giữ nguyên kind qua serde và qua archive.*
Negative: *Kind không dựng được phải là Unsupported, không bao giờ hạ cấp âm thầm.*

## 1. Typed API

```rust
impl GeometryKind { pub fn tag(self) -> &'static str;
                    pub fn from_tag(tag: &str) -> Result<Self>; }   // "Brep"/"BRep"/"B-rep" -> Unsupported("brep geometry")
impl Document { pub fn geometry_kind(&self, id: ObjectId) -> Result<GeometryKind>; }
```

## 2. Errors (exact strings)

- `Unsupported("brep geometry")   — Brep/BRep/B-rep cho tới khi có OM9-L2-01-05`
- `Unsupported("geometry kind")   — tag lạ, không hạ cấp thành Mesh/Cloud`
- `Unsupported("document variant") — container gặp variant không nhận ra`
- `Invalid("typed document metadata/schema") — dữ liệu sai cấu trúc (khác với variant lạ)`

## 3. Tests that carry the oracle

- `cargo test -p om9-core --test document_graph l3_002 -> 2 passed`
- `cargo test -p om9-archive --test document_identity_round_trip -> 4 passed`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-002/OM9-L4-01-03-002-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-002/OM9-L4-01-03-002-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-002/OM9-L4-01-03-002-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-002/positive.json`, `negative.json`
