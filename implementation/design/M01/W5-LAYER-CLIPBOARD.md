# W5 — port kernel layer clipboard / exchange / codec (thuộc M01)

**Vị trí milestone:** M01 – CAD Shell, Viewport. Lớp gần nhất trong cây kế hoạch:

| Phần W5 | L3 gần nhất trong M01 | Trạng thái |
| --- | --- | --- |
| Layer records + entity↔layer membership | `OM9-L2-01-03` (`L3-004`, `L3-005`, `L3-006`) | **đã nghiệm thu trước đó** — W5 không mở rộng phạm vi L2 này |
| Codec/wire form của layer payload | `OM9-L2-01-12 L3-002` Scene And Layer State Encoding | kernel đã có, L3 chưa đóng (cần runbook M01) |
| Chọn object theo layer | `OM9-L2-03-14 L3-004` Select All Objects By Layer | membership kernel đã có (`exchange_contents`) |
| **Clipboard/exchange layer** | **không có L3 nào sở hữu** | **lỗ hổng kế hoạch** — xem dưới |

## Lỗ hổng kế hoạch (ghi lại, không tự nhận)

`layer_clipboard.rs` / `layer_exchange.rs` của legacy làm việc *copy layer nội bộ tài liệu* và *chuyển object
giữa layer*, nhưng cây kế hoạch hiện tại **không có L3 nào** cho clipboard layer (L2-03-14 chỉ có palette,
current layer, visible/locked, select-by-layer, persistence visual). Giống lỗ hổng "copy 3DM" đã ghi trước
đó: cần một L3 chủ sở hữu (hoặc gộp vào `L2-03-14`) trước khi có thể đóng nghiệm thu. Kernel dưới đây là
phần portable, chờ chủ sở hữu.

## Đã port

| Legacy | Hành vi giữ lại | Target |
| --- | --- | --- |
| `layer_clipboard.rs` (`prepare_clipboard`/`receive_clipboard`) | nội dung một lần copy layer: layer + **toàn bộ sublayer**, entity trên chúng, thứ tự xác định | `layer_ops::copy_layer` + `LayerClipboard` |
| `layer_exchange.rs` (`plan_receive`) | paste đổi **toàn bộ id** (layer + entity), re-parent sublayer, có thể gắn dưới layer đích | `layer_ops::paste_layer` + `PastePlan` |
| `layer_exchange.rs` (transfer scope) | chuyển object giữa hai layer: chỉ đổi membership, không sửa record layer | `layer_ops::exchange_contents` |
| `layer_codec.rs` | wire form có version, kiểm **byte budget trước khi parse** | `layer_ops::{encode, decode}` |

Ba quyết định thiết kế (đều bám invariant sẵn có của target, không tự đặt luật mới):

1. **Paste phải tạo layer trước, rồi mới insert entity.** Entity sau khi paste trỏ vào layer *mới*, nên
   commit entity trước khi layer tồn tại bị admission từ chối `Missing("layer")` — có test âm chứng minh.
   Đây là lý do `PastePlan` tách hai nửa: `layers` (đi qua `LayerPlan`) và `entities` (đi qua transaction).
2. **Bản copy không thừa hưởng provenance.** Entity copy giữ geometry/attributes/placement nhưng namespace
   import và retained source vẫn thuộc object gốc (luật L2-01-03: chuỗi phải đầy đủ mới là provenance).
3. **Không thêm mutation mới.** Paste dùng `Mutation::Insert`, exchange dùng `Mutation::Move` — cả hai đã
   được test; transaction invariants (`storage == validate`) không bị chạm.

## Test (10)

`crates/om9-core/tests/layer_ops.rs`:

- copy gom đúng layer + sublayer + entity của chúng (không lấy layer không liên quan), thứ tự theo id;
- copy layer không tồn tại → `Missing("layer")`; copy layer rỗng vẫn tạo payload hợp lệ;
- paste đổi toàn bộ id layer/entity, `revision` về 1, root giữ top-level hoặc gắn dưới layer đích,
  sublayer theo đúng cha mới, entity trỏ vào layer mới;
- paste với layer đích không tồn tại → `Missing("layer")`;
- **commit entity trước khi có layer → `Missing("layer")`** (admission), và sau khi áp nửa layer thì commit
  thành công: +2 entity, **một** Undo entry, bản gốc còn nguyên;
- exchange chỉ sinh `Move` cho entity thuộc layer nguồn, không đụng record layer; `from == to` →
  `Invalid("layer exchange")`; layer lạ → `Missing("layer")`;
- wire form round-trip bằng nhau; schema khác → `Unsupported("layer clipboard schema")`; rác →
  `Invalid("layer clipboard wire")`; vượt `max_bytes` → `Budget("layer clipboard bytes")` ở **cả** encode
  và decode (kiểm trước khi parse).

## Còn lại của W5

- **Chủ sở hữu L3 cho clipboard layer** (quyết định kế hoạch) — chặn nghiệm thu, không chặn code.
- **Định dạng clipboard hệ thống** (OS clipboard, nhiều format, backup budget của legacy): cần lớp host;
  kernel chỉ lo payload + codec.
- **Layer retained/legacy mapping** (`layer_retained.rs`, `layer_legacy.rs`): giữ liên kết nguồn khi layer
  đến từ import và map layer legacy Matrix — thuộc M03 `L2-01-13 L3-004` (3dm layer/attribute mapping).
- **UI**: palette/current/visible/locked (`L2-03-14`) — thuộc M01 UI, chưa làm.
- `layer_ffi.rs` (55 KB): `LEGACY_HOST`, không port ranh giới.
