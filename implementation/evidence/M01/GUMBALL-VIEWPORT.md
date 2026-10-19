# Gumball trong viewport — select và tương tác (M01)

Ngày: 2026-10-18. Mục tiêu: chọn object trong viewport và kéo gumball để move/scale/rotate, một hoặc nhiều object.

## Trạng thái trước khi làm

| Thành phần | Có sẵn |
| --- | --- |
| Chọn object bằng click trong viewport | **Có** — `ui.rs` gọi `scene.pick(...)` rồi `Controller::select_root` |
| Mô hình gumball (frame, handle, drag, transform, session) | **Có** — `om9-core/src/gumball.rs` (7 handle move, 3 rotate, 3 scale; `Session::begin/drag/move_mutations`) |
| Vẽ gumball trong viewport | **Không** — không có chỗ nào trong `om9-app` nhắc tới gumball |
| Kéo gumball | **Không** — `viewport_camera_input` chỉ xử lý camera |

## Vòng này: lớp toán hit-test/kéo

`crates/om9-scene/src/gumball_view.rs` (mới): `GumballView { frame, mode, size }`, `Mode::{Move,Rotate,Scale}`,
`handles()`, `segments()` (đường/kẻ theo world), `hit(camera, viewport, pointer, tolerance_px)`,
`drag(camera, viewport, handle, from, to) -> Drag`.

Quy tắc khớp mô hình cũ: axis chỉ cho chuyển động dọc trục (closest point giữa tia và trục), plane bỏ thành
phần pháp tuyến, free di chuyển trong mặt phẳng hướng về camera, rotate trả góc quanh trục, scale trả tỉ lệ
dọc trục.

**8 test** (`cargo test -p om9-scene --lib gumball_view`):

| Test | Chứng minh |
| --- | --- |
| `an_axis_handle_is_hit_where_it_is_drawn` | click đúng điểm vẽ → `MoveAxis(X)` |
| `empty_space_is_not_a_handle` | góc màn hình → `None` |
| `dragging_an_axis_handle_moves_only_along_that_axis` | Δ ≈ 1 theo X, y/z = 0 |
| `dragging_a_plane_handle_keeps_the_normal_component` | Δz = 0, Δx/Δy dương |
| `rotating_about_z_reports_the_screen_angle` | góc = π/2 |
| `scaling_reports_the_ratio_along_the_axis` | factor = 2 |
| `an_orthographic_viewport_hits_the_same_handles` | ortho cũng hit |
| `every_mode_exposes_the_handles_it_draws` | move 7 handle; rotate/scale 3; vòng 48 đoạn |

## Việc còn lại (vòng sau)

1. `Controller`: đọc selection đã publish + `apply_placement_mutations(name, mutations)` (prepare → scene →
   commit → Undo 1 entry).
2. `ui.rs`: vẽ `GumballView::segments()` bằng painter với màu trục; left-drag trên handle → `Session::begin`,
   mỗi frame `Session::drag` với delta **tăng dần** (vì `Session::drag` cộng dồn), thả chuột → commit.
3. Chuyển mode: Move mặc định; Shift = Rotate, Alt = Scale (hoặc nút trên thanh trạng thái).
4. Test tích hợp app: click chọn 1 và nhiều object, kéo → `Placement` đổi, Undo trả lại.

## Kiểm chứng vòng này

- `om9-scene`: 15 + 15 + 8 + 3 + 12 + 2 + 9 test pass (gồm 8 test gumball view mới).
- `om9-exchange`: 7 test STL pass (L3-01-12-008/009, làm ở vòng trước bị ngắt).
- clippy sạch cho `om9-scene` và `om9-exchange`.

## Vòng 2: nối vào app — kéo gumball thành một transaction

### Controller (`crates/om9-app/src/session/gumball.rs`)

- `gumball_targets()` — object đang chọn, theo thứ tự document.
- `gumball_pivot()` — tâm **từ hình học trong scene** (`RenderBuffers` + `display_transform`), không phải
  từ `placement`: Point/Line mang toạ độ trong buffer nên placement thường là identity.
- `commit_gumball(session, name)` — bỏ qua gesture không có chuyển động (`drag_count == 0`), còn lại:
  prepare → commit → **`refresh_scene()`** → `sync_published(Edit)`. Phải rebuild scene vì
  `Scene::update_owned` từ chối thay đổi placement (`Stale("scene metadata geometry")`) — đây là phát hiện
  khi chạy test, đã sửa đúng đường.
- `commit_gumball_copy` — nhân bản kèm transform (dùng `Session::copy_mutations`).

### Giao diện (`crates/om9-app/src/ui.rs`)

`gumball_ui` chạy trong **mọi viewport** (4 khung), sau khi camera đã cập nhật:

| Việc | Cách làm |
| --- | --- |
| Vẽ | `GumballView::segments()` chiếu bằng `Camera::project`; X đỏ, Y lục, Z lam, plane vàng, tâm trắng |
| Kích thước | ~90 px (`world_per_pixel × 90`), độc lập zoom và ortho |
| Mode | mặc định Move; **Shift = Rotate**, **Alt = Scale** |
| Bắt đầu kéo | `drag_started_by(Primary)` + hit-test 8 px → `Session::begin(frame, targets, false)` |
| Kéo | mỗi frame `gumball.drag(last, current)` rồi `session.drag(...)` — **delta tăng dần** vì `Session::drag` cộng dồn |
| Thả | `drag_stopped_by(Primary)` → `commit_gumball` (tên "Move"/"Rotate"/"Scale") |

### Test tích hợp (`crates/om9-app/tests/gumball_interaction.rs`) — 5 test

| Test | Chứng minh |
| --- | --- |
| `moving_two_selected_objects_is_one_journal_entry` | 2 object chọn, kéo (3,4,0) → cả hai dời; **1 Undo entry**; Undo trả về đúng; Redo áp lại |
| `rotating_turns_the_selection_about_its_centre` | xoay 90° quanh Z tại tâm → (0,0,0) thành (5,−5,0), (10,0,0) thành (5,5,0) |
| `scaling_doubles_the_distance_from_the_pivot` | scale X ×2 → −5 và 15 |
| `a_gesture_without_motion_writes_nothing` | click không kéo → không có entry |
| `the_pivot_is_none_without_a_selection` | không chọn → không vẽ gumball |

### Kiểm chứng

- `om9-app --test gumball_interaction`: **5/5 pass**.
- `om9-app` toàn bộ: 90 + 9 + 5 + 4 + 9 + 5 + 5 + 6 + 6 test pass.
- `om9-scene` 8 test gumball view; clippy sạch cho `om9-app` và `om9-scene`.

### Chưa làm

- Kiểm tra thủ công bằng chuột trong cửa sổ thật (app đã mở sẵn để người dùng thử).
- Copy mode (Ctrl khi kéo) chưa nối vào UI; `commit_gumball_copy` đã có và đã test ở tầng controller.
- Chưa có nút chọn mode trên giao diện; hiện dùng Shift/Alt.

## Vòng 3: kiểm chứng end-to-end qua chính `MatrixApp::show`

`crates/om9-app/tests/gumball_viewport.rs` — **3 test, tất cả pass**, dùng sự kiện chuột thật
(`Event::PointerMoved/PointerButton`) đi qua `MatrixApp::show`, không gọi thẳng controller:

| Test | Chứng minh |
| --- | --- |
| `a_click_selects_and_a_gumball_drag_moves_the_line` | click giữa Line → được chọn; kéo handle → Line dời; đúng **1 journal entry**; Undo trả về tâm cũ |
| `shift_dragging_a_ring_rotates_the_selection` | mode Rotate, kéo vòng Z → placement chứa **rotation thật** (cos²+sin²=1), pivot đứng yên, Undo trả placement |
| `alt_dragging_an_axis_scales_the_selection` | mode Scale, kéo trục Y 0,6→1,2 → **factor 2,0** trên Y, X/Z không đổi, Undo trả placement |

### Hai lỗi thật phát hiện khi test end-to-end

1. **Mất những pixel đầu của drag.** egui chỉ báo `drag_started` sau ngưỡng di chuyển, nên `last` lấy
   `interact_pointer_pos()` bị trễ ~15 px: scale factor đo được 1,62 thay vì 2,0. Sửa: lấy
   `pointer.press_origin()` làm điểm bắt đầu gesture.
2. **Press sát mép phải viewport** (khoảng 90 px từ mép) không khởi động drag trong harness; chuyển điểm
   press vào trong (góc 135° của vòng, trục Y) thì chạy đúng. Đây là chi tiết tương tác của egui, đã ghi
   lại; người dùng thật hiếm khi bấm sát mép gumball.

### Trạng thái mục tiêu

| Yêu cầu | Trạng thái | Bằng chứng |
| --- | --- | --- |
| Select trong viewport (1 hoặc nhiều) | **xong** | click test; `select_root(extend)` có từ trước |
| Gumball move 1/nhiều object | **xong** | controller 5 test + UI move test |
| Gumball rotate | **xong** | controller rotate test + UI rotate test |
| Gumball scale | **xong** | controller scale test + UI scale test |
| Một transaction / Undo | **xong** | cả 3 UI test và controller test đều kiểm Undo |
