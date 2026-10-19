# Associative history contract (G00-05)

## 1. Cái đã có trong OM9

`om9-core::history` (Tolerance, Recipe, HistoryGraph, EvaluationState) và `history_ops`
(`stale_recipes`, `mark_stale`, `plan_recompute`, `mark_clean`, `mark_failed`, `detach`, `freeze`,
`state_counts`). `EvaluationState` phân biệt `Clean/Dirty/Pending/Unresolved/Failed/Detached`.

## 2. Mapping QymCAD ↔ Matrix Builder

| QymCAD | OM9 | Ghi chú |
| --- | --- | --- |
| operation timeline (`qymcad-part`, `regenerate`) | `HistoryGraph` + `Recipe` | mỗi recipe là một bước, thứ tự tất định |
| naming bridge 4 quy tắc (`naming.rs`) | cần port sang `history_ops`/naming service | mảnh split đặt theo face cha; seam theo hàng xóm; còn lại theo vị trí; cạnh theo cặp face |
| `Ref`/`RefError` (`qymcad-core/refs.rs`) | chưa có tương đương; dùng `ObjectId` + `EntityId` hiện tại | MVP dùng explicit id, chưa có query-based ref |
| rebuild toàn bộ khi đổi tham số | `plan_recompute` theo thứ tự `execution_order` | đã có giới hạn `execution_order_bounded` |

## 3. Hỗ trợ MVP và fallback

- MVP: rebuild theo thứ tự, đánh dấu stale, dừng ở recipe lỗi (`Failed`), giữ `Unresolved` thay vì xoá.
- Fallback: nếu một recipe không rebuild được, object giữ hình học cuối và được đánh dấu `Stale/Unresolved`;
  người dùng thấy trạng thái, không bị mất dữ liệu.
- Chưa làm: query-based reference, conflict UI, persistence feature id.
