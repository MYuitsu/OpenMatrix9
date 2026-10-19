# W7 — port duy trì phụ thuộc tất định (thuộc M04 `OM9-L2-01-04`)

**Vị trí milestone:** M04 – Builder and History, L2 `OM9-L2-01-04` (be-associative-dependency-rebuild).

| Phần W7 | L3 trong M04 | Trạng thái |
| --- | --- | --- |
| Feature record + cạnh phụ thuộc có kiểu | `OM9-L3-01-04-001` Feature Record And Typed Dependency Edges | `Recipe.inputs`/`outputs` đã có; kernel dùng chúng làm cạnh |
| Dirty propagation + thứ tự topo | `OM9-L3-01-04-003` Dirty Propagation And Topological Order | **kernel mới** (`mark_stale`, `plan_recompute`) |
| Recompute thất bại | `OM9-L3-01-04-007` History Recompute Failure Rollback | kernel ghi `Failed(reason)`, rollback vẫn là việc của transaction |
| Xoá cha / đổi input | `OM9-L3-01-04-008` Delete Parent Or Change Input Policy | `Unresolved("source missing")` + `detach`/`freeze` |

## Đã port

| Legacy | Hành vi giữ lại | Target |
| --- | --- | --- |
| `builder_history.rs` (dirty/rebuild) | phát hiện recipe cũ so với document, lan truyền dơ theo cạnh output→input, kế hoạch recompute theo thứ tự topo | `history_ops::{stale_recipes, mark_stale, plan_recompute}` |
| `history.rs` (state) | `Clean`/`Dirty(reason)`/`Unresolved(reason)`/`Failed(reason)`/`Detached`, revision tăng khi recompute sạch | `history_ops::{mark_clean, mark_failed, state_counts}` |
| `prepared_result.rs` (lifecycle) | detach giữ output nhưng bỏ ownership; freeze khoá recompute | `history_ops::{detach, freeze}` |

Bốn quyết định thiết kế:

1. **"Nguồn mất" ≠ "nguồn đổi".** `Document::resolve` trả `Missing` cho object đã xoá và `Stale` cho tem cũ;
   kernel map thành `Unresolved("source missing")` và `Dirty("source changed")`. Gộp hai cái sẽ làm mất
   thông tin người dùng cần (một cái phải sửa input, một cái chỉ cần recompute).
2. **Chỉ chuyển trạng thái mới tính là thay đổi.** Gọi `mark_stale` hai lần trên cùng document trả `[]`,
   nên vòng lặp UI không quay vô ích.
3. **`Unresolved` không bị ghi đè thành "downstream".** Recipe có nguồn mất giữ nguyên câu trả lời của nó,
   kể cả khi producer của nó cũng dơ.
4. **`Unresolved`/`Failed` không nằm trong kế hoạch recompute.** Chúng chỉ chạy lại sau khi người dùng sửa
   input hoặc xử lý lỗi — tự động chạy lại chỉ tạo vòng lặp thất bại.

## Test (7)

`crates/om9-core/tests/history_ops.rs` — chuỗi hai bước `source --first--> middle --second--> top`:

- graph tươi: không stale, kế hoạch rỗng, `execution_order` = [first, second];
- đổi `geometry_revision` của source → `first` stale ("source changed"), `mark_stale` trả cả hai theo thứ
  tự topo, `second` = "downstream of dirty recipe", kế hoạch = [first, second], lần gọi thứ hai trả `[]`;
- xoá source → `first` = `Unresolved("source missing")`, `second` vẫn downstream dirty, kế hoạch chỉ còn
  `second`;
- detach: idempotent, `policy.record/update` = false, `owner(middle)` = None, nguồn đổi **không** làm nó
  stale;
- `mark_clean` tăng revision và về `Clean`; `mark_failed` giữ lý do, không được plan, lý do rỗng bị từ chối,
  recipe không tồn tại → `Missing`;
- `freeze` bật/tắt `policy.lock` → `graph.locked(output)`; detach + freeze → `Invalid("detached recipe
  freeze")`;
- `state_counts` phản ánh trạng thái; `ensure_document` từ chối graph của document khác.

## Lỗi tự gây khi làm (đáng ghi)

- Fixture lấy id bằng `document.entities.values().last()` — **BTreeMap không theo thứ tự chèn**, nên chuỗi
  phụ thuộc bị đảo và 5/7 test sai. Sửa bằng cách giữ id trước khi insert. Đây là lỗi test, không phải code.
- `mark_stale` bản đầu trả lại tập cũ ở lần gọi thứ hai; đã thêm so sánh trạng thái (idempotent) + bảo vệ
  `Unresolved`.

## Còn lại của W7

- **`reference_path.rs`** (`OM9-L3-01-04-004` Stable Semantic Reference Linking): cần **model** biết "con của
  một result" (subshape/edge/index) trước khi kernel có thể phân giải đường dẫn; target hiện chỉ có
  `Selection` (Whole/Indices) chứ chưa có cây kết quả. Đây là điều kiện dữ liệu, không phải thiếu hàm.
- **Cache tăng dần** (`OM9-L3-01-04-005`): cần key cache theo (recipe revision, input stamps) — làm được khi
  có producer thật đầu tiên.
- **Preview vs committed** (`OM9-L3-01-04-006`): `HistoryPolicy` đã có `record/update/lock`; cần session M04
  quyết định preview ghi hay không.
- **`solid_result.rs`/`edit_result.rs`** (20 KB): hình học kết quả solid/edit → phụ thuộc `OM9-L2-01-05`
  (OCCT), đang `BLOCKED`.
- `reference_path_ffi.rs`, `native_builder_capability.rs`: `LEGACY_HOST`/native, không port ranh giới.
