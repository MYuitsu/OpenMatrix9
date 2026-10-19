> Continuation (2026-10-19): the user moved the active foundation to M01. Native BRep source/topology is now implemented and checked; [current evidence](2026-10-19-brep-selection.md) supersedes the missing-policy/native-BRep-pending statements below. Earlier checkpoint facts remain historical. External UI review is still pending.

> Correction 2026-10-19: the L2-01-07 completion claims below are historical and too broad. Native BRep and external acceptance remain open. Current measured scope: [video/implementation checkpoint](2026-10-19-selection-video-review.md).

# `OM9-L2-01-07` — traceability L4 → code → test

Ngày: 2026-10-19. Nguồn yêu cầu: `OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking
Selection Generic Handles` (l2.json, 8 × l3.json, 45 × l4.json) và các tài liệu/ảnh tham chiếu trong
`reference/text`, `reference/images` của từng L3.

Oracle (nguyên văn, từ `l4.json`):

| L3 | Positive | Negative |
| --- | --- | --- |
| 001 | "Box tại tâm tia, điểm click ở viewport P trúng BodyId của box" | "Ray không giao đối tượng trả danh sách rỗng, không chọn" |
| 002 | "Click ID A rồi Ctrl click ID B trả set {A,B}, toggle A trả {B}" | "ID đã xóa hoặc layer khóa bị loại và không lưu stale ID" |
| 003 | "Chọn đúng face/edge/vertex của box và lưu khóa topology bền" | "Click vùng trống hoặc face không còn sau rebuild báo miss, không đổi sang mặt gần đó" |
| 004 | "Cửa sổ kéo trái→phải chỉ chọn box hoàn toàn nằm trong; phải→trái chọn giao cắt" | "Object nằm ngoài không được chọn; locked/hidden đều bỏ qua" |
| 005 | (disambiguation: chọn đúng một object khi nhiều object dưới con trỏ) | không chọn theo hình học gần giống khi mơ hồ |
| 006 | highlight selection + hover object dưới con trỏ | hover không đổi selection |
| 007 | move/rotate/scale bằng handle, đúng trục | kéo ngoài handle không đổi gì |
| 008 | drag commit thành **một** transaction, Undo hoàn tác | Esc huỷ, không ghi document |

## 1. Source audit (L4-…-01)

QymCAD checkout: `H:\OpenMatrix9\QymCAD`. Các `source_anchors` của l4.json trỏ vào repo QymCAD, không
phải gốc OM9; đã đọc bản local:

| Anchor (QymCAD) | Symbol đọc được | Quyết định |
| --- | --- | --- |
| `crates/qymcad-pick/src/lib.rs` | "WHAT THE CURSOR CATCHES" — hàm thuần theo document/view/screen, trả vertex/edge/face/body | **ADAPT**: `crates/om9-scene/src/picking.rs` port cùng ý tưởng hàm thuần; OM9 không phụ thuộc QymCAD lúc build |
| `crates/qymcad-pick/src/body_hits.rs` | `body_bbox_hit` — cull bằng bbox trước khi đi cạnh | **ADAPT**: `Scene::pick_all` lọc `visible/selectable` trước; chưa có bbox cache riêng |
| `crates/qymcad-ui-state/src/selection.rs` | `enum Sel { Mesh, Face(mesh, face), Contour, … }` | **ADAPT**: `BTreeSet<ObjectId>` + `Subelement::{MeshFace, MeshEdge, MeshVertex}`; face giữ **chỉ số face của document**, không phải triangle index |
| `crates/qymcad-core/src/refs` | tham chiếu hình học bền | **ADAPT**: `PickHit { source: SourceStamp, world_epoch }` + `revalidate_in_view` |

Không có API nào của QymCAD bị gọi trực tiếp; mọi thứ nằm trong crate OM9 (Rust-first).

## 2. Traceability theo từng L4

### L3-001 — Screen Ray And Hit Test Query
| L4 | Trạng thái | Bằng chứng |
| --- | --- | --- |
| 001-01 audit | DONE | bảng trên |
| 001-02 contract | DONE | `Scene::pick/pick_all/pick_kind` (screen, camera, viewport, tolerance, intent) → `Result<Option<PickHit>>`; lỗi `Invalid("pixel pick tolerance")`, `Stale("pick camera/publication")` |
| 001-03 implement | DONE | `crates/om9-scene/src/picking.rs`; test `acceptance_l2_01_07::a_ray_through_the_box...`, `world_picking` |
| 001-04 integrate | DONE | `crates/om9-app/src/ui.rs` click → `pick_all` + `revalidate_in_view`; test `pick_disambiguation` |
| 001-05 regression | DONE | `twenty_identical_picks_return_the_same_hit` (20 lần), tolerance âm → `Err` |
| 001-06 acceptance | DONE | positive + negative trong `acceptance_l2_01_07` |

### L3-002 — Object Selection Set Core
| L4 | Trạng thái | Bằng chứng |
| --- | --- | --- |
| 002-01 audit | DONE | `Sel` của QymCAD → `BTreeSet<ObjectId>` |
| 002-02 contract | DONE | `select_root`, `select_objects`, `toggle_selection`, `add_objects`, `remove_objects`; lỗi `Permission`, `Stale`, `Budget` |
| 002-03 implement | DONE | `crates/om9-app/src/session/layers.rs`; test `acceptance_selection_set` 5/5 |
| 002-04 integrate | DONE | click/Shift/Ctrl và band trong `ui.rs`; test `acceptance_viewport_selection` 3/3 |
| 002-05 regression | DONE | `selecting_never_writes_document_history` (undo_len + revision không đổi) |
| 002-06 acceptance | DONE | `acceptance_selection_set` (positive {A,B}→{B}; negative locked/hidden/không tồn tại) |

### L3-003 — Subobject Face Edge Vertex Picking
| L4 | Trạng thái | Bằng chứng |
| --- | --- | --- |
| 003-01 audit | DONE | `Sel::Face(mesh, face)`; khóa = (object id, face index) |
| 003-02 contract | DONE | `PickKind::{Object,Vertex,Edge,Face}` + `Scene::pick_kind` |
| 003-03 implement | DONE | vertex/edge/face branches trong `picking.rs`; test `subobject_pick` 4/4 |
| 003-04 integrate | DONE | **Ctrl+Shift+click** trong `ui.rs` (đúng tài liệu Rhino p37); `app.sub_object()`; test `ctrl_shift_click_picks_the_face_under_the_pointer` |
| 003-05 regression | DONE | `a_face_that_disappears_after_a_rebuild_is_a_stale_miss` (không rematch mặt gần) |
| 003-06 acceptance | DONE | `the_three_subobject_kinds_report_their_own_identifiers`, `a_face_hit_is_the_document_face_not_the_triangle_index` |

### L3-004 — Selection Window And Crossing
| L4 | Trạng thái | Bằng chứng |
| --- | --- | --- |
| 004-01..03 | DONE | `objects_in_band(window)`: window = mọi đỉnh chiếu được phải nằm trong **và** không đỉnh nào bị clip; crossing = ít nhất một đỉnh trong |
| 004-04 integrate | DONE | hướng kéo chọn window/crossing; **Shift = thêm, Ctrl = bớt** (tài liệu Rhino p34) |
| 004-05/06 | DONE | test `the_band_is_a_window_left_to_right_and_crossing_right_to_left`, `shift_adds_with_a_band_and_ctrl_removes`; locked/hidden bị bỏ vì `objects_in_band` đòi `visible && selectable` |

### L3-005 — Object Selection Disambiguation
`pick_all` + popup `Select object`, test `pick_disambiguation` 2/2 (gần nhất xếp trước, chọn index 1 → object xa).

### L3-006 — Selection Highlighting And Hover
Highlight xanh trong suốt trên GPU (batch riêng) + khung trắng vùng chọn; hover có khung trắng, test
`selection_overlay` 7/7 (khung, gumball hiện khi chọn, clear, ctrl toggle, window/crossing, Esc, hover).

### L3-007 — Generic Translation Rotation Scale Handles
Sách Matrix 8 tr.45–47: "Move X … Red arrow, Move Y … Blue arrow, Move Z … Yellow arrow", "4 = Axis Plane
Indicator", "5 = Free Move Origin".
Đã khớp: **X đỏ, Y xanh dương, Z vàng** (sửa lại sau khi đọc ảnh tr.45), plane indicator + free-move ở tâm,
scale = ô vuông cuối đường đứt nét, rotate = 3 cung lệch pha.
Test: `gumball_view` 8, `gumball_viewport` 3, `gumball_interaction` 5.

### L3-008 — Drag Transaction And Commit Cancel
Commit = **một** transaction (`commit_gumball`), Undo một bước; Esc huỷ không ghi;
test `escape_cancels_a_gumball_drag_without_committing`, `a_numeric_move_is_one_transaction`.

## 3. Khoảng trống còn lại (không giấu) — cập nhật vòng 2

1. **Manual gate M02-UI chưa ký**: cần người thật nhìn UI; AI không tự duyệt (quy tắc repo). **Đây là mục
   duy nhất còn chặn "hoàn thành"**.
2. ~~20 lần lặp deterministic~~ **đã bù**: `twenty_identical_picks_return_the_same_hit` (L3-001),
   `twenty_face_edge_vertex_picks_are_identical` (L3-003), `twenty_band_passes_leave_the_same_selection`
   (L3-004), `twenty_selection_round_trips_are_identical` (L3-002).
3. ~~Tolerance theo DPI~~ **đã bù**: `pick_tolerance(ui) = 6 pt × pixels_per_point`; test
   `pick_tolerance_scales_with_the_pixel_ratio` (10 px trượt ở 1×, trúng ở 2×).
4. ~~Subobject selection set~~ **đã bù**: `Controller::{select_subobject, toggle_subobject,
   clear_subselection, subselection, prune_subselection}` với `sub_revision`; test `subobject_selection` 6/6
   (thứ tự pick, chống trùng, toggle, owner lạ bị từ chối, không đụng object selection, prune khi object bị xoá).
   Thêm `Controller::delete_objects` (một transaction, dọn selection + subselection) để test prune chạy thật.
5. Tia trúng **đúng đường chéo chung** của hai tam giác vẫn có trường hợp biên không hit (đã thêm epsilon).

## 3b. Cập nhật vòng 2

- **Ctrl+Shift+click** giờ **thêm** subobject vào bộ (không chỉ lưu một cái gần nhất); bấm lại cùng một
  subobject thì **bỏ** nó (toggle).
- Tolerance pick **theo DPI** ở cả 3 chỗ: click thường, Ctrl+Shift+click, hover.
- Test mới: `subobject_selection` **6**, scene acceptance **7**, app acceptance **5**, selection set **6**.

## 4. Test chạy trong đợt này

| Suite | Kết quả |
| --- | --- |
| `om9-scene` `acceptance_l2_01_07` | 6/6 |
| `om9-app` `acceptance_selection_set` | 5/5 |
| `om9-app` `acceptance_viewport_selection` | 3/3 |
| `om9-app` `selection_overlay` | 7/7 |
| `om9-app` `rubber_band`, `gumball_viewport`, `gumball_interaction` | 2/2, 3/3, 5/5 |
| `om9-scene` `subobject_pick` | 4/4 |
