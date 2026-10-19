# OM9-L3-01-03-009 — Jewelry Semantic Extension Record (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-core/src/{model,transaction}.rs`. Reuse decision: **ADD the versioned named-slot record; EXTEND Entity and the mutation set**.

Oracle — positive: *Slot om9.gem giữ ObjectId/generation, chỉ tăng attributes_revision; payload giữ nguyên văn bản.*
Negative: *Slot lạ được giữ và báo cáo; payload sai/version 0/tên slot trần bị từ chối.*

## 1. Typed API

```rust
pub const GEM_EXTENSION_SLOT: &str = "om9.gem"; // + rail/head/cutter
pub struct Extension { pub version: u32, pub payload: String }   // payload = JSON object text
impl Extension { pub fn new(version: u32, payload: impl Into<String>) -> Result<Self>;
                 pub fn validate(&self, l: &Limits) -> Result<Usage>; }
impl Entity { pub extensions: BTreeMap<String, Extension>; pub fn extension(&self, slot: &str) -> Option<&Extension>; }
impl Document { pub fn extension(&self, id, slot: &str) -> Result<Option<&Extension>>;
                pub fn unknown_extension_slots(&self) -> BTreeSet<String>; }
Mutation::Extension { source, slot: String, extension: Option<Extension> }   // None clears the slot
```

## 2. Errors (exact strings)

- `Invalid("extension slot")          — tên slot trần như "Gem"`
- `Invalid("extension version")       — version = 0`
- `Invalid("extension payload")       — payload không phải JSON object`
- `Invalid("empty extension payload") — object rỗng`
- `Permission("source edit")          — đối tượng thuộc layer khoá`

## 3. Tests that carry the oracle

- `cargo test -p om9-core --test document_graph l3_009 -> 6 passed`
- `cargo test -p om9-archive --test document_identity_round_trip -> 4 passed`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-009/OM9-L4-01-03-009-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-009/OM9-L4-01-03-009-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-009/OM9-L4-01-03-009-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-009/positive.json`, `negative.json`
