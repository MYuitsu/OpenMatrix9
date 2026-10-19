# OM9-L3-01-03-004 — Layer Domain Records (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-core/src/model.rs (Layer + Document::validate)`. Reuse decision: **REUSE the existing Layer record and its validation; add the explicit refusals as tests**.

Oracle — positive: *Đổi tên/màu/cờ giữ nguyên LayerId; layer record hợp lệ.*
Negative: *Layer trùng/thứ tự sai/parent không tồn tại bị từ chối.*

## 1. Typed API

```rust
pub struct Layer { pub id: LayerId, pub name: String, pub parent: Option<LayerId>,
                   pub visible: bool, pub selectable: bool, pub locked: bool,
                   pub color: [u8; 4], pub revision: u64, pub preset_slot: Option<PresetSlot> }
impl Layer { pub fn new(name, color) -> Self; pub fn preset(slot: PresetSlot) -> Self; }
```

## 2. Errors (exact strings)

- `Missing("layer")            — parent layer không tồn tại`
- `Invalid("layer order")      — layer_order không liệt kê mỗi layer đúng một lần`
- `Invalid("layer ID/revision") — map key khác Layer.id hoặc revision = 0`
- `Invalid("layer cycle")      — chuỗi parent tạo vòng`

## 3. Tests that carry the oracle

- `cargo test -p om9-core --test document_graph l3_004 -> 3 passed`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-004/OM9-L4-01-03-004-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-004/OM9-L4-01-03-004-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-004/OM9-L4-01-03-004-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-004/positive.json`, `negative.json`
