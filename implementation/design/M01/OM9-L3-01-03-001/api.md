# OM9-L3-01-03-001 — Document Entity IDs (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-core/src/{identity,model,transaction}.rs`. Reuse decision: **EXTEND the typed-UUID ledger; ADD the typed feature record**.

Oracle — positive: *Hai entity và một feature nhận ID duy nhất; Undo/Redo và save/reopen giữ nguyên ID.*
Negative: *ID đã nghỉ không được cấp lại; feature trỏ tới output không tồn tại bị từ chối.*

## 1. Typed API

```rust
pub struct FeatureId(pub Uuid);  pub struct GroupId(pub Uuid);      // identity.rs, in the existing identifier! list
pub enum FeatureKind { Annotation, Dimension, Construction, Semantic }
pub struct Feature { pub id: FeatureId, pub label: String, pub kind: FeatureKind,
                      pub outputs: Vec<ObjectId>, pub revision: u64 }
impl Document { pub fn feature(&self, id: FeatureId) -> Result<&Feature>; pub features: BTreeMap<FeatureId, Feature>; }
Mutation::FeatureInsert { document, expected_revision, feature }   // one Undo entry, ledger-protected ID
```

## 2. Errors (exact strings)

- `Invalid("allocated feature UUID")  — một FeatureId đã nghỉ bị cấp lại`
- `Missing("feature output")          — output không phải object đang sống`
- `Missing("feature")                 — tra feature không tồn tại`

## 3. Tests that carry the oracle

- `cargo test -p om9-core --test document_graph l3_001           -> 3 passed`
- `cargo test -p om9-archive --test document_identity_round_trip -> 4 passed`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-001/OM9-L4-01-03-001-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-001/OM9-L4-01-03-001-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-001/OM9-L4-01-03-001-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-001/positive.json`, `negative.json`
