# Gumball nhìn theo sách Matrix9

Ngày: 2026-10-19. Tham chiếu: ảnh gumball trong sách Matrix9 (`matrix8_book_1 Image[45].jpg`).

## Vòng 3 (2026-10-19): khớp hình vẽ trong sách trang 45

Ảnh tham chiếu `m8book1_p45_img02.png` (đã đọc) cho thấy bố cục:

- **Move**: 3 mũi tên có đầu nhọn, X đỏ, Y xanh, Z vàng.
- **Scale**: 3 **ô vuông rỗng** ở phía **đối diện** mũi tên, nối về tâm bằng **đường chấm**.
- **Rotate**: 3 **cung 1/4 vòng** nằm sát tâm, mỗi cung nằm giữa hai trục còn lại (thành "bông hoa" nhỏ).
- **"4" Plane indicator**: icon lưới nhỏ trong góc dương giữa hai trục.
- **"5" Free Move Origin**: ô vuông nhỏ có chấm ở tâm.

Đã sửa theo đúng bố cục đó:

| Thành phần | Trước | Sau |
| --- | --- | --- |
| Rotate | 3 cung 3/4 vòng, bán kính = độ dài trục, lệch pha 120° | **3 cung 1/4 vòng**, bán kính **0,6 ×** trục, mỗi cung giữa hai trục còn lại |
| Scale | ô vuông đặc ở **phía +** trục | **ô vuông rỗng ở phía −** trục, nối bằng **đường chấm 4 px** (như hình sách) |
| Plane | ô vuông đặc | **icon lưới** (ô vuông + 2 đường chéo trong) |
| Tâm | ô vuông đặc, không chấm | **ô vuông rỗng + chấm trắng** |

Hệ quả đã kiểm: kéo ô scale ở phía − vẫn cho factor dương vì `factor = end/start` (tỉ lệ), nên không phải đổi
toán học; test `gumball_viewport` cập nhật điểm bấm theo vị trí mới (3/3 pass).

## Đã đổi

| Thành phần | Trước | Sau |
| --- | --- | --- |
| Trục move | đường thẳng 2 px | **mũi tên** có đầu tam giác 13 px, cạnh 5 px |
| Rotate | 3 vòng tròn đủ 48 đoạn | **3 cung 3/4 vòng** (40 đoạn), mỗi cung lệch pha 120° như hình sách |
| Scale | đường thẳng + chấm tròn | **đường đứt nét** (6 px nét, 5 px hở) + **ô vuông** 7 px ở đầu |
| Move plane | chấm tròn | ô vuông 8 px |
| Tâm gumball | không có | **ô vuông 7 px** nền xám, viền trắng |
| Màu | X đỏ, Y xanh lá, Z xanh dương | **X vàng hổ phách, Y xanh cyan, Z đỏ** như sách |

## Kỹ thuật

- Vẽ theo **nhóm handle** (`GumballView::handles()`), không vẽ từng đoạn rời: cung rotate được vẽ thành một
  `Shape::line` liền nên không lộ mối nối.
- `dashed_line(from, to, 6.0, 5.0)` chia đường thành đoạn nét.
- Hit-test không đổi: vẫn dùng `GumballView::segments()`/`nearest`, nên cung ngắn hơn nhưng vẫn bấm trúng.

## Test

- `om9-scene` lib `gumball_view`: **8 pass** (cập nhật kỳ vọng 3×40 đoạn cho 3 cung).
- `gumball_viewport` 3/3, `gumball_interaction` 5/5, `gumball_placement` 3/3, `gumball_numeric` 4/4,
  `selection_overlay` 7/7; clippy sạch.

## Chưa đối chiếu được bằng ảnh

Máy không cho chụp màn hình (`handle is invalid`), nên khớp về **hình dạng/màu** mới ở mức code + test; cần bạn
nhìn trực tiếp để xác nhận độ giống ảnh sách.


## Vòng 4 (2026-10-19): ba cung rotate ra sau mũi tên

Theo yêu cầu trực tiếp: "đổi 3 vòng rotate ở phía sau mũi tên".

- Ba cung rotate giờ nằm ở **phía âm** của hai trục của chúng (u = −axis1, v = −axis2), nên chúng vòng ra
  **phía sau** cụm mũi tên thay vì cắt qua mũi tên, đúng hình Matrix9.
- **Thứ tự vẽ**: `Rotate` → `Scale` → `Move`. Cung ở dưới cùng, đường chấm scale ở giữa, **mũi tên vẽ trên
  cùng** nên mũi tên che phần đường chấm đi qua nó, như hình sách.
- Đường chấm scale giờ chạy **suốt cả trục**: từ mũi tên (+size) qua tâm đến ô vuông (−size), thay vì chỉ
  nửa đường từ tâm.
- Test `gumball_viewport` cập nhật điểm bấm cung về góc −45°: **3/3**; `gumball_interaction` 5/5,
  `selection_overlay` 7/7; clippy sạch.


## Vòng 5 (2026-10-19): handle nào nhìn thấy mới vẽ (lỗi "3 ô vuông ở single view")

Lỗi người dùng báo: ở view song song (single view) có **3 ô vuông scale** chồng nhau quanh tâm. Nguyên nhân:
trục của view đó **chiếu thẳng vào camera** nên mũi tên và ô vuông của trục ấy co lại thành một điểm ngay trên
pivot, cộng với 2 ô của hai trục kia → thành cụm 3 ô.

Sửa trong `GumballView`:

- `visible_segments(camera, viewport)`: bỏ handle **move/scale** khi `|dot(axis, view_dir)| > 0.93` (trục chỉ
  vào camera); bỏ **plane indicator** khi `|dot(normal, view_dir)| < 0.5` (nhìn nghiêng); bỏ **cung rotate**
  khi cung chiếu ngắn dưới 1 px.
- `nearest_in_view(...)`: hit-test chỉ trên các handle đang vẽ, nên **không grab được handle vô hình**.
- App dùng hai hàm này khi vẽ và khi bấm.

Kết quả: **top/left/front view chỉ còn 2 ô vuông** (đúng như Matrix9), perspective vẫn đủ 3.

Test mới trong `gumball_view::view_tests`:

| Test | Ý nghĩa |
| --- | --- |
| `a_single_view_hides_the_axis_pointing_at_the_camera` | top view: chỉ còn trục X, Y và plane XY |
| `a_single_view_keeps_two_scale_boxes_not_three` | top view: **2** ô vuông, không phải 3 |
| `a_perspective_view_keeps_all_three_axes` | perspective: đủ 3 trục |
| `a_hidden_handle_cannot_be_grabbed` | bấm đúng tâm ở top view không lấy được hộp Z vô hình |

`gumball_view` 12/12, `gumball_viewport` 3/3, `gumball_interaction` 5/5.


## Vòng 6 (2026-10-19): mỗi handle mang màu trục của nó

Lỗi người dùng báo: "ô scale ... sao dùng màu vàng hết vậy".

Nguyên nhân thật: **ba icon plane indicator** (icon lưới "4") bị vẽ cùng một màu vàng nhạt cho cả ba mặt
phẳng, nên nhìn như một cụm ô vuông vàng. Bản thân ô scale đã theo màu trục, nhưng icon plane thì không.

Sửa:

- `axis_colour(axis)`: X đỏ `(205,60,50)`, Y xanh dương `(60,150,200)`, Z vàng `(235,180,50)`.
- `gumball_colour(handle)`: mọi handle của một trục (move/scale/rotate) dùng **cùng** màu trục đó.
- `plane_colours(plane)`: icon lưới vẽ **viền** bằng màu trục thứ nhất, **hai đường trong** bằng màu trục thứ hai
  — đúng câu trong sách "the two axises are indicated by their color":
  - XY → viền đỏ (X), trong xanh (Y)
  - YZ → viền xanh (Y), trong vàng (Z)
  - ZX → viền vàng (Z), trong đỏ (X)

Test mới (`crates/om9-app/src/ui.rs`, `mod gumball_colour_tests`):

| Test | Ý nghĩa |
| --- | --- |
| `the_three_axes_have_three_different_colours` | X đỏ, Y xanh, Z vàng và ba màu khác nhau |
| `every_scale_box_carries_its_own_axis_colour` | `Scale(X)`/`Scale(Y)`/`Scale(Z)` = đúng màu trục, không cùng màu |
| `a_plane_icon_shows_its_two_axes` | ba mặt phẳng trả đúng cặp màu |

3/3 pass; clippy sạch.


## Vòng 7 (2026-10-19): bỏ khung trắng một-object và preview trực tiếp khi kéo

### 1. Bỏ khung trắng khi chỉ chọn một object

Khung trắng là khung **tập chọn nhiều object**. Một object đã được nhận ra nhờ highlight + gumball, nên khung
quanh nó chỉ che mất gumball (đúng như ảnh người dùng gửi). Nay `selection_frame` chỉ vẽ khi
`selected().len() >= 2`, và có margin 3 px để tập chọn thẳng hàng vẫn thấy khung.

### 2. Kéo gumball ở một view cập nhật cả bốn view

Trước đây transform chỉ đi vào document lúc **thả chuột**, còn ảnh GPU của mỗi viewport được cache, nên ba
viewport kia không đổi cho tới khi thả.

Đã thêm preview tạm thời:

- `Controller::preview_gumball(delta)`: gán `SceneObject::preview = delta` cho các object đang chọn, tăng
  `preview_revision`. Document **không** đổi (revision giữ nguyên).
- `Controller::clear_gumball_preview()`: trả preview về identity (dùng khi Esc).
- `GpuViewport::show` nhận thêm `preview_revision` và đưa vào cache key, nên **mọi** viewport dựng lại ảnh khi
  preview đổi.
- `gumball_ui` mỗi frame kéo gọi `preview_gumball(session.accumulated())` — dùng transform **tích luỹ** của
  gesture, không phải delta từng frame.
- CPU path cũng đọc `display_transform()` (= `preview ∘ transform`) nên cũng cập nhật.

### Test

| Test | Nội dung |
| --- | --- |
| `the_preview_moves_the_selected_objects_without_touching_the_document` | preview đổi `display_transform`, document revision/placement không đổi; clear trả lại |
| `dragging_a_handle_updates_the_preview_then_commits_on_release` | revision tăng khi kéo, transform vẽ đổi, document chờ đến lúc thả, và preview == transform commit sau khi thả |
| `a_single_selection_has_no_frame_but_two_do` | 1 object: 0 khung; 3 object: có khung |

`selection_overlay` 8/8, `preview_drag` 2/2; toàn bộ `om9-app` 0 failed; clippy sạch.


## Vòng 8 (2026-10-19): kéo gumball mượt — đọc clip `F:\20261018-2003-15.8824342.mp4`

Đã trích 21 frame (ffmpeg, 1 fps, 486×482). Hành vi đọc được:

1. Gumball ở Perspective: 3 mũi tên, 3 cung, 3 ô vuông, icon lưới, ô tâm; có khung trắng (bản cũ).
2. Camera **orbit/zoom** trong lúc gumball đang hiện (lưới xoay/đổi cỡ giữa các frame) — không được vỡ hình.
3. Khi kéo **cung rotate**, frame 12–14 cho thấy **vệt mũi tên** (ghost/trail) và cảm giác giật: mỗi frame lại dựng
   lại cảnh và ảnh cũ không được xoá sạch/không kịp vẽ lại.

Sửa để mượt:

- **Preview bằng model matrix, không dựng lại buffer**: `GpuScene` giữ index của các object đang chọn
  (`selected_objects`) và bản highlight; mỗi frame kéo chỉ gọi `set_transform(index, delta)` — vài lần ghi
  uniform, không upload lại đỉnh.
- `preview_model(delta)`: đổi `Transform` (row-major, f64) sang ma trận `f32` cho renderer; có test
  `the_preview_matrix_keeps_the_row_major_origin`.
- Ảnh GPU **vẫn cache** nhưng cache key có `preview_revision`, nên mọi viewport vẽ lại mỗi frame kéo, và pass
  scene luôn `LoadOp::Clear` → không còn vệt.
- **Cảnh gộp (merged static)**: cờ `merged` — khi selection đổi trên cảnh đã gộp, batch được dựng lại để object
  được chọn tách khỏi static range (nếu không, preview không thể tách nó ra).
- Đo (`measure_preview_frame_cost`, 2 000 cube, 320×180):

| Đường | mỗi frame |
| --- | --- |
| Preview (set_transform + render) | **0,677 ms** |
| Dựng lại cảnh + render | **7,695 ms** |

→ nhanh hơn ~11×, và không cấp phát buffer mỗi frame nên hết giật.

## Vòng 9 ghi chú

- Clip được ghi lúc 20:03 ngày 18/10, trước bản gumball hiện tại; nếu bạn vẫn thấy vệt khi ghi lại, gửi clip mới
  để tôi đối chiếu đúng build.
- Việc còn có thể làm tiếp cho mượt: nội suy vị trí con trỏ theo thời gian khi máy chậm, và giới hạn tần số
  dựng lại highlight (hiện mỗi lần selection đổi là một lần).
