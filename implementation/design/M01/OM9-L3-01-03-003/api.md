# OM9-L3-01-03-003 — Geometry Handle And Attributes (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-core/src/model.rs`. Reuse decision: **ADD GeometryHandle/DisplayAttributes queries over the existing Entity record**.

Oracle — positive: *Handle và display attributes truy vấn được riêng, không sửa bản ghi.*
Negative: *Handle cũ/treo là Stale, không bao giờ trỏ vào đối tượng đang sống.*

## 1. Typed API

```rust
pub struct GeometryHandle { pub object: ObjectId, pub generation: u64, pub geometry_revision: u64 }
pub struct DisplayAttributes { pub object: ObjectId, pub name: String, pub color: Option<[u8; 4]>,
                                pub visible: bool, pub selectable: bool, pub editable: bool, pub layer: Option<LayerId> }
impl Entity { pub fn geometry_handle(&self) -> GeometryHandle; }
impl Document { pub fn geometry_handle(&self, id) -> Result<GeometryHandle>;
                pub fn resolve_handle(&self, h: &GeometryHandle) -> Result<&Entity>;
                pub fn display_attributes(&self, id) -> Result<DisplayAttributes>; }
```

## 2. Errors (exact strings)

- `Stale("geometry handle") — handle không khớp bản hiện tại hoặc đối tượng đã bị xoá`
- `Missing("object")          — tra đối tượng không tồn tại`

## 3. Tests that carry the oracle

- `cargo test -p om9-core --test document_graph l3_003 -> 2 passed`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-003/OM9-L4-01-03-003-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-003/OM9-L4-01-03-003-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-003/OM9-L4-01-03-003-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-003/positive.json`, `negative.json`
