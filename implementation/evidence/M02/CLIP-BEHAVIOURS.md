# Hành vi trong clip `F:\20261018-2003-15.8824342.mp4` và cách OM9 làm lại

Clip: 20,57 s, 486×482, 30 fps, một viewport Perspective. Đã trích **41 frame** ở 2 fps và 3 contact sheet
(4×4) để đọc tuần tự; mỗi ô trong sheet = 0,5 s.

## Danh sách thao tác đọc được

| Thời điểm | Thao tác | Quan sát |
| --- | --- | --- |
| 0,0–0,5 s | **Click vào đường (line)** | Gumball xuất hiện tại object; highlight |
| 1,0 s | **Click vùng trống** | Gumball biến mất, bỏ chọn |
| 2,0 s | **Chọn lại** | Gumball hiện lại |
| 2,5–7,5 s | **Orbit camera** (kéo chuột phải) | Lưới xoay, gumball vẫn dính object, kích thước màn hình không đổi |
| 6,0–8,0 s | **Zoom** (lăn chuột) | Khoảng lưới đổi, gumball vẫn ~90 px |
| 8,0–10,5 s | Orbit tiếp | Gumball đổi hướng theo trục thế giới, selection giữ nguyên |
| 11,0–14,0 s | **Kéo cung rotate** | Object quay; bản ghi cũ để lại **vệt mũi tên** (lỗi build cũ) |
| 14,0–15,5 s | Rotate/orbit tiếp | Gumball giữ tâm |
| 16,0–20,5 s | Nhìn gần trục | Chỉ còn **2 mũi tên** + ô vuông scale (trục còn lại chiếu vào camera) |

## OM9 làm lại — test tương ứng

| # | Hành vi | Test |
| --- | --- | --- |
| 1 | Click chọn → gumball hiện | `clip_behaviours::clicking_selects_and_empty_space_deselects_with_the_gumball_following` |
| 2 | Click trống → bỏ chọn, gumball mất | cùng test trên |
| 3 | Orbit khi gumball đang hiện, gumball vẫn ở pivot | `clip_behaviours::orbiting_and_zooming_leave_the_gumball_attached_and_the_same_screen_size` |
| 4 | Zoom bằng lăn chuột, gumball giữ ~90 px | cùng test trên |
| 5 | Kéo cung rotate quay object, pivot không đổi | `gumball_viewport::shift_dragging_a_ring_rotates_the_selection` + `preview_drag` |
| 6 | Kéo ô vuông scale theo một trục | `gumball_viewport::alt_dragging_an_axis_scales_the_selection` |
| 7 | Kéo khung chọn nhiều object, Esc bỏ chọn | `clip_behaviours::a_window_band_selects_the_set_and_escape_clears_it` |
| 8 | Preview khi kéo hiện ở **cả 4 viewport** | `preview_drag::dragging_a_handle_updates_the_preview_then_commits_on_release` |
| 9 | Không vệt khi kéo | preview dùng model matrix + pass clear; đo 0,677 ms/frame so với 7,695 ms/frame khi dựng lại (`measure_preview_frame_cost`) |

## Về "vệt mũi tên" trong clip

Frame 11–14 của bản ghi cho thấy nhiều mũi tên chồng nhau. Đó là hành vi của build **trước** khi có preview
bằng model matrix: mỗi frame kéo lại dựng lại cảnh, ảnh cũ không được thay kịp. Bản hiện tại:

- Preview chỉ ghi `set_transform` cho object đang chọn (không cấp phát buffer).
- Pass scene luôn `LoadOp::Clear`.
- Cache ảnh theo `preview_revision` nên mọi viewport vẽ lại đúng một lần mỗi frame.

Không tái hiện được vệt trên build hiện tại trong test (không có GPU trong test), nên bằng chứng là cơ chế +
đo frame cost; cần một clip mới để xác nhận bằng mắt.

## Chạy

`crates/om9-app/tests/clip_behaviours.rs` — **3/3 pass**; các suite gumball/preview liên quan:
`gumball_viewport` 3/3, `gumball_interaction` 5/5, `preview_drag` 2/2, `selection_overlay` 8/8.


## Vòng 2 (2026-10-19): đọc lại clip ở 10 fps và hai lỗi còn thiếu

Người dùng chỉ ra: (1) thiếu **animation mờ khi rotate**, (2) **không chọn được một ô vuông để scale**,
(3) trước đó tôi đọc sót nhiều frame.

Đã trích lại clip ở **10 fps** → **13 contact sheet 4×4** (mỗi ô 0,1 s; mỗi sheet 1,6 s). Đọc sheet 8
(t≈11,2–12,8 s) và sheet 10 (t≈14,4–16,0 s): đúng là quanh tâm có **chùm mũi tên mờ** toả ra, và chúng
**mờ dần rồi biến mất** trong ~1–2 s — tức đây là **animation vệt mờ khi quay**, không phải lỗi nén video.

### (1) Animation mờ khi rotate

- Trong lúc kéo handle Rotate, mỗi frame ghi `(accumulated, time)` vào `gumball_trail`; giữ tối đa **24** bản
  trong **0,6 s**.
- Mỗi bản vẽ lại **cung rotate + mũi tên** với khung đã quay theo delta (`rotate_frame`: áp phần tuyến tính của
  ma trận, tâm giữ nguyên) và alpha giảm dần (`(1 - age/0.6) * 0.35`).
- Vẽ **dưới** gumball hiện tại; xoá trail khi bắt đầu gesture mới, khi thả, khi Esc.
- Test: `trail_tests::a_rotation_delta_turns_the_frame_axes_about_the_pivot` (quay 90° quanh Z: x→y, y→−x,
  tâm không đổi).

### (2) Ô vuông scale phải bấm riêng được

- Đường chấm scale chạy suốt trục nên test khoảng cách–đoạn thẳng không phân biệt được ô vuông; thêm
  `GumballView::box_at` — hit-test **đúng tâm ô vuông** trong bán kính **14 px**, ưu tiên trước hit-test thường.
- Test `box_tests::a_scale_box_is_hit_and_a_drag_of_it_scales`: bấm đúng tâm ô X → `Scale(X)`; kéo từ −size ra
  −2size → `Drag::Factor ≈ 2.0`.

### Ghi chú

- Không chụp được màn hình máy này nên hiệu ứng mờ và độ nhạy của ô vuông cần bạn nhìn trực tiếp xác nhận.
- Nếu trail trong clip dày/mảnh hơn ý bạn, chỉnh 3 tham số: `0.6 s` (thời gian giữ), `0.35` (alpha lớn nhất),
  `24` (số bản).


## Vòng 3 (2026-10-19): animation thuộc về **đối tượng**, không phải gumball

Người dùng làm rõ: khi move/rotate/scale, **đối tượng** để lại một **vật màu xám ở vị trí cũ** và chính đối tượng
chuyển động (animation), không phải gumball.

### Bỏ

- Toàn bộ `gumball_trail` (vệt mờ của gumball) và test `trail_tests` đã bị xoá — sai đối tượng.

### Thêm: bóng xám của đối tượng tại vị trí cũ

- `build_ghost(device, queue, scene, selected)`: dựng một `SceneBatch` riêng gồm **các object đang chọn ở
  `object.transform`** — tức placement của document, **không** gồm preview — tô xám `(0.45, 0.45, 0.45, 0.55)`.
- `world_mesh_at(object, transform)`: biến thể của `world_mesh` nhận transform tường minh, nên bóng dùng
  placement còn vật sống dùng `display_transform` (= preview ∘ placement).
- Vẽ **sau** cảnh, có depth test (`render_over_into`), nên bóng chỉ hiện ở chỗ vật sống không che; khi thả chuột
  hoặc **Esc** thì `ghost = None`.
- Áp dụng chung cho **move, rotate, scale** vì preview mang cả ba loại transform; bóng luôn nằm ở vị trí bắt đầu.

### Test

| Test | Nội dung |
| --- | --- |
| `ghost_tests::the_ghost_uses_the_placement_and_the_live_mesh_uses_the_preview` | placement x=2 → bóng ở 1.5; preview +5 → vật sống ở 6.5 |
| `preview_drag::the_drag_leaves_the_placement_untouched_so_the_ghost_can_stay_at_the_old_spot` | trong lúc kéo, `transform` không đổi, `display_transform` đổi; clear trả lại |

### Giới hạn

- Bóng xám chỉ có trên **đường GPU**; đường CPU (khi không có GPU) không vẽ bóng.
- "Animation" ở đây là vật sống bám con trỏ mượt (preview 0,677 ms/frame) + bóng xám đứng yên; không phải
  hiệu ứng mờ dần của vật.

### Ghi chú môi trường

`cold_layers` (spawn tiến trình con) fail với `0xC0000135` khi build kèm `--features gpu-sphere`; chạy không
feature thì 4/4 pass. Đây là vấn đề DLL của tiến trình con trong cấu hình feature, không phải logic mới.
