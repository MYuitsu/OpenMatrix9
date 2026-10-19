> Continuation (2026-10-19): the user moved the active foundation to M01. Native BRep source/topology is now implemented and checked; [current evidence](2026-10-19-brep-selection.md) supersedes the missing-policy/native-BRep-pending statements below. Earlier checkpoint facts remain historical. External UI review is still pending.

> Correction 2026-10-19: the L2-01-07 completion claims below are historical and too broad. Native BRep and external acceptance remain open. Current measured scope: [video/implementation checkpoint](2026-10-19-selection-video-review.md).

# M02 — trạng thái `OM9-L2-01-07` và `OM9-L2-03-03`

Ngày: 2026-10-18. Bảng này đối chiếu từng L3 với code và test đang có.

## `OM9-L2-01-07` — Picking, Selection, Generic Handles

| L3 | Nội dung | Trạng thái | Bằng chứng |
| --- | --- | --- | --- |
| -001 | Screen ray và hit test | **xong** | `Camera::ray`, `Scene::pick`, `crates/om9-scene/tests/world_picking.rs` |
| -002 | Selection set core | **xong** | `SelectionState`, `Controller::{select_root, select_objects, toggle_selection}`, test `gumball_interaction`, `selection_overlay` |
| -003 | Subobject face/edge/vertex | **xong** | `PickKind::{Object,Vertex,Edge,Face}` + `Scene::pick_kind`; `Subelement::{MeshVertex, MeshEdge, MeshFace}`; test `subobject_pick` 4/4 |
| -004 | Selection window và crossing | **xong** | kéo trái→phải = window (toàn bộ nằm trong), phải→trái = crossing (chạm là lấy); test `the_band_is_a_window_left_to_right_and_crossing_right_to_left` |
| -005 | Disambiguation khi chồng nhau | **xong** | `Scene::pick_all` + popup `Select object`; chọn xong select đúng object; test `pick_disambiguation` 2/2 |
| -006 | Highlight và hover | **xong** | highlight xanh trong suốt trên GPU + khung trắng quanh vùng chọn; hover object có khung trắng (cache theo vị trí con trỏ + camera revision); test hover |
| -007 | Generic translate/rotate/scale handles | **xong** | gumball 3 nhóm cùng lúc, hit-test chọn gần nhất; `gumball_view` 8 test + `gumball_viewport` 3 test |
| -008 | Drag transaction, commit, cancel | **xong** | commit 1 transaction; Esc huỷ không ghi gì; test `escape_cancels_a_gumball_drag_without_committing` |

## `OM9-L2-03-03` — Viewport Object Selection And Gumball

| L3 | Nội dung | Trạng thái | Bằng chứng |
| --- | --- | --- | --- |
| -001 | Matrix click / Shift / Ctrl | **xong** | click chọn 1; Shift mở rộng; Ctrl đảo (`toggle_selection`), test `ctrl_click_toggles...` |
| -002 | Matrix left/right drag | **xong** | trái = band (window/crossing), phải = orbit/pan camera |
| -003 | Popup khi chọn mơ hồ | **xong** | dùng chung `pick_all` + popup; 2 test `pick_disambiguation` |
| -004 | Gumball axis drag | **xong** | `GumballView::drag` + test `dragging_an_axis_handle...`, `a_click_selects_and_a_gumball_drag_moves_the_line` |
| -005 | Gumball alignment và relocate | **xong** | `GumballOrigin x,y,z` / `GumballReset` / `GumballWorld` / `GumballObject`; test `gumball_placement` 3/3 |
| -006 | Gumball numeric edit | **xong** | `MoveX/Y/Z`, `Move x,y,z`, `RotateX/Y/Z`, `Scale`, `ScaleX/Y/Z`; test `gumball_numeric` 4/4 |

## Đã làm trong vòng này

- Khung chọn: hình chữ nhật **trắng** quanh vùng chọn nhiều object (`selection_frame`), vẽ **sau** ảnh GPU nên
  không bị ảnh che; test `a_multi_selection_draws_a_frame_around_itself`.
- Gumball hiện khi có selection: trước đây gumball bị vẽ **trước** ảnh GPU nên bị che; đã chuyển sang vẽ sau
  ảnh; test `any_selection_shows_the_gumball` (số đoạn thẳng trong viewport tăng ≥10).
- Band kiểu Matrix9: **window** (trái→phải) và **crossing** (phải→trái).
- Ctrl+click đảo một object trong selection.
- Esc huỷ gumball drag, không tạo transaction.
- Gumball hiển thị **cả ba nhóm** move/rotate/scale cùng lúc, hit-test chọn handle gần con trỏ nhất.

## Test

`crates/om9-app/tests/selection_overlay.rs`: **6 test pass** (frame, gumball, clear, ctrl toggle, window/crossing, Esc).


## Vòng 3 (2026-10-18)

- **`Scene::pick_all`**: mọi object dưới con trỏ, gần trước; `pick` cũ refactor thành wrapper để hành vi không đổi
  (19 test `om9-scene` xanh).
- **Popup chọn mơ hồ**: click chỗ có ≥2 object (và không Shift/Ctrl) → hiện `Select object` với từng dòng;
  chọn dòng nào thì `select_objects` đúng object đó. Test: 2 object chồng nhau → 2 ứng viên, gần trước; chọn
  index 1 → object xa được chọn; click 1 object → không popup.
- **Hover highlight**: object dưới con trỏ có khung trắng (nới 4 px để điểm đơn vẫn thấy khung). Pick chỉ chạy
  lại khi con trỏ di >3 px hoặc camera/scene đổi. Test: hover → có khung; rời đi → mất khung.
- Test hiện tại: `selection_overlay` **7/7**, `pick_disambiguation` **2/2**; clippy sạch.

**Còn lại**: `-003` edge/vertex picking, `-005` gumball alignment/relocate, `-006` numeric edit.


## Vòng 5 (2026-10-18)

- **Subobject picking**: chọn **vertex** (`MeshVertex(i)`), **edge** (`MeshEdge{a,b}`), **face** (`MeshFace(f)`)
  và object. `Scene::pick_kind` trả hit gần nhất theo loại đang chọn; test 4/4 (vertex index, edge hai đầu, face,
  object).
- **Gumball relocate/alignment**: `GumballOrigin 0,0,0` đặt gốc, `GumballReset` trả về tâm selection,
  `GumballObject` căn trục theo object đầu tiên đang chọn (45° → x = (0.707, 0.707, 0)), `GumballWorld` trả lại
  trục thế giới; test 3/3.
- Sửa thêm: **epsilon barycentric** khi bắn tia vào tam giác (trước đây tia trúng đúng cạnh chung của hai tam giác
  có thể trượt cả hai).

## Giới hạn còn biết (không chặn hai L2)

- Tia trúng **đúng đường chéo chung** của hai tam giác ở một số trường hợp vẫn không hit (đã thêm epsilon nhưng
  còn trường hợp biên); ghi ở ledger.
- Popup liệt kê **object**, chưa liệt kê **mặt** khi hai mặt của cùng object chồng nhau.
- Numeric edit áp cho **object**, chưa cho subobject đang chọn.
