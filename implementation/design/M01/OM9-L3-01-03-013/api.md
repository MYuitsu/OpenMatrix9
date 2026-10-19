# OM9-L3-01-03-013 — Import Source Provenance Metadata (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-core/src/provenance.rs`. Reuse decision: **ADD a read-only provenance query over the already-retained import metadata (no 3DM decision)**.

Oracle — positive: *Object nhập báo source name/sha256/native id/index/family/disposition; summary đếm đúng.*
Negative: *Object tạo native không có provenance; thiếu một mắt bằng chứng thì None; namespace lạ là Missing.*

## 1. Typed API

```rust
pub struct Provenance { pub object, source, resource, source_name, source_sha256, archive_version,
                        namespace, import_mode, kind, original_uuid, original_index, original_family, disposition }
pub struct ImportSummary { pub sources, namespaces, records, bound, retained_only, deleted, unsupported, objects_with_provenance }
impl Document { pub fn provenance(&self, id: ObjectId) -> Option<Provenance>;
                pub fn has_source_proof(&self, id: ObjectId) -> bool;
                pub fn import_summary(&self) -> ImportSummary;   // one pass, not per object
                pub fn namespace_summary(&self, id: ImportNamespaceId) -> Result<ImportSummary>; }
```

## 2. Errors (exact strings)

- `Missing("import namespace") — namespace không tồn tại`
- `Không lỗi khác: thiếu bằng chứng nguồn thì trả None, không gắn nhãn .3dm native nửa vời`

## 3. Tests that carry the oracle

- `cargo test -p om9-core --test provenance -> 5 passed (includes the 2000-object x 1000-record scale guard)`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-013/OM9-L4-01-03-013-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-013/OM9-L4-01-03-013-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-013/OM9-L4-01-03-013-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-013/positive.json`, `negative.json`
