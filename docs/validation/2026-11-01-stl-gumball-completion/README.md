# STL export, gumball settings and acceptance — 2026-11-01

Bổ sung sau khi người dùng yêu cầu làm hết các giới hạn còn lại của đợt kiểm tra lệnh.

## STL

### Nguồn hình học
- Mesh, editable OCCT B-Rep (tái tessellate qua `om9_kernel::brep::read_brep`), **retained
  `NativeBrep`** (payload ON_Brep được đưa lại qua `OcctBrepBridge` để tessellate canonical —
  không dùng display mesh làm hình học thay thế) và **NurbsSurface** (chia lưới thích nghi
  ước lượng sai số chord, 8–128 bước mỗi chiều thay vì cố định 32×32).
- Vertex được weld, tam giác suy biến ở cực cầu bị loại. B-Rep/mesh kín ghi strict
  watertight; surface/mesh hở ghi non-strict kèm `open_edges` và cảnh báo, không giả báo kín.
- File thay thế atomic; receipt có `triangles`, `bytes`, `open_edges`, `watertight`, `skipped`,
  `warnings`; MCP `om9_export_stl` trả cùng các trường này.

### Independent viewer
- `tools/om9_stl_independent_viewer.py` dùng **trimesh** (không dùng writer OM9) đọc lại file
  STL, kiểm tra triangle count, bounds, thể tích, watertight, winding và tam giác suy biến.
- Kết quả: `docs/validation/2026-11-01-stl-independent-viewer.json` — box 10 mm: 12 tam giác,
  V=1000 mm³, bounds 0..10, watertight/winding OK; cầu r=5 mm: 5006 tam giác, V=522,09 mm³
  (sai lệch 0,15% so với 523,60), watertight/winding OK, 0 tam giác suy biến. Tất cả check PASS.
- Đây là **thư viện đọc độc lập**, chưa phải viewer GUI bên thứ ba; gate M09 G-MVP-013 vẫn cần
  reviewer và viewer độc lập được định danh theo quy trình.

### Hộp thoại Windows thật
- `om9-native-owner` thêm `DialogFilter::Stl` / `choose_stl` và `DialogKind::SaveStl`.
- Test `real_stl_save_dialog_accepts_an_owned_fixture` mở dialog Save thật với owner HWND tạo
  trên thread test, probe bấm IDOK, nhận lại đường dẫn `audit.stl` đúng thư mục. Test được đánh
  dấu `#[ignore]` vì cần desktop tương tác; đã chạy riêng:
  `cargo test -p om9-native-owner --lib real_stl_save_dialog_accepts_an_owned_fixture -- --ignored`
  → **1 passed**.

### GPU render
- Tạo project bằng MCP (box 10³ + sphere r=5), render 4 viewport bằng
  `om9 --render-4 build/command-audit/gpu-acceptance.om9 build/command-audit/gpu-acceptance.png 640x400`.
- Ảnh local 1280×800 RGBA, 238.492 bytes, SHA-256
  `040ae89006d02df3e4d4856ae40ff7971957f4c2728166617d2b2c57aa643da1`; cả bốn view hiển thị box
  và sphere. Đây là GPU renderer thật (cùng `Renderer` của viewport), không phải painter.
  Ảnh nằm trong `build/` (gitignored) theo quy tắc public-source: không commit raster chưa review.
- Chưa có automation kéo gumball bằng OS mouse trên GPU; tương tác chuột được kiểm bằng egui
  widget test và dialog Win32 thật.

## Gumball

### Drag strength
- Thêm `gumball_drag_strength` (mặc định 1.0), lệnh `Gumball DragStrength=50` /
  `DragStrength=50` (phần trăm 1..200, ngoài khoảng bị từ chối).
- Strength chỉ áp cho chuyển động con trỏ; snapped/numeric input giữ chính xác.
- Test egui: kéo 48 px ở 50% → pivot dời 24 px.

### Settings menu
- Icon `InfoSettingsGumballAlignment` nay mở cửa sổ **Gumball settings**: World/Object/CPlane,
  Smooth/Snappy, Drag strength 1–200%, Reset pivot, Restore defaults.
- Test: click icon index 19 mở menu; cửa sổ render một frame không panic và giữ trạng thái mở.

### Snappy 3D
- Snappy không còn chỉ chiếu lên mặt phẳng kéo: với tay nắm Move, delta dùng trực tiếp
  `SnapCandidate.world` qua `Drag::WorldDelta`; `gumball::transform` vẫn chiếu theo ràng buộc
  axis/plane nếu tay nắm là axis/plane. Vì vậy free move đặt pivot đúng điểm 3D, axis/plane
  giữ đúng ràng buộc.
- Test `snappy_drag_follows_the_object_snap_marker` khẳng định pivot bằng đúng điểm 3D
  `[10,10,0]` (không chỉ trùng hình chiếu màn hình), cộng test strength 50%.

### Lưu settings qua phiên
- `GumballSettings` (schema 1: snappy, drag_strength, alignment) lưu tại
  `%APPDATA%/OpenMatrix9/gumball.json` (hoặc `OM9_GUMBALL_SETTINGS`), ghi atomic và chỉ ghi khi
  giá trị đổi. Pivot Relocate không lưu, đúng guide (chỉ nhớ trong phiên).
- Startup đọc file và áp vào Controller; file thiếu/hỏng không chặn mở app (quay về defaults),
  lỗi ghi chỉ báo một lần.
- Test `ui::tests::gumball_settings_persist_and_reload`: đổi Snappy + strength 50% + Object,
  chạy một frame, mở app mới đọc lại đúng giá trị.

### OS mouse trên GPU
- `tools/gumball-os-audit.ps1` mở **cửa sổ eframe/wgpu thật** với project rỗng; app audit tạo
  line 10 mm + Point snap tại `[5,5,0]`, chọn line, bật Snappy, đợi layout ổn định rồi ghi
  `ready.json` (toạ độ client của pivot và snap point).
- Driver lấy HWND, `AttachThreadInput` + Alt để nhận foreground, `SetCursorPos` + `mouse_event`
  kéo thật từ pivot tới snap point; app nhận drag qua OS input queue, commit placement, ghi
  `result.json`, chụp framebuffer GPU rồi đóng.
- Ba lần chạy liên tiếp PASS: before `[[0,0,0],[10,0,0]]` → after `[[0,5,0],[10,5,0]]`;
  `drag_started=true`, `app_focused=true`, `done.status=ok`, ảnh GPU
  `build/gumball-os-audit/gumball-os-audit.png` (SHA-256
  `91872306…c2d3e`). Record: [2026-11-01-gumball-os-gpu.json](../2026-11-01-gumball-os-gpu.json).
- Audit mode bỏ qua điều kiện `input.focused` khi `OM9_GUMBALL_OS_AUDIT` bật, vì desktop tự động
  không cấp foreground theo cách một người dùng thật làm; mọi đường sản phẩm khác vẫn giữ điều
  kiện focus.

## Kiểm thử

| Bộ | Kết quả |
|----|---------|
| `om9-app --test stl_export` | 10 passed (native payload, surface thích nghi, surface hở, watertight) |
| `om9-app --test gumball_viewport` | 7 passed (thêm strength, defaults, settings menu icon) |
| `om9-app --test mcp_stdio` | 2 passed |
| `ui::tests::gumball_settings_window_renders_and_stays_open` | 1 passed |
| `ui::tests::gumball_settings_persist_and_reload` | 1 passed |
| `tools/gumball-os-audit.ps1` | 3 consecutive runs PASS: OS drag lands on the 3D snap point, GPU screenshot saved |
| `om9-native-owner --lib real_stl_save_dialog... -- --ignored` | 1 passed |
| Independent trimesh record | all_passed=true |
| GPU `--render-4` | ảnh 1280×800 hợp lệ |

`om9-app --lib` vẫn còn 3 fail có sẵn từ working tree trước phiên này (2 icon-size oracle
31 vs 20, 1 Split widget); không thêm fail mới từ các thay đổi này.

## Giới hạn còn lại

- M09 G-MVP-013 cần independent **viewer GUI** được định danh/version và reviewer ký; trimesh
  chỉ là reader độc lập.
- Surface chia lưới thích nghi theo ngưỡng chord 0,01 mm nhưng vẫn là lưới đều mỗi chiều
  (8–128 bước), chưa phải quadtree T-junction; retained NativeBrep phụ thuộc bridge OCCT.
- Acceptance OS mouse trên GPU đã tự động hoá bằng Win32 input + audit env; đây là input tổng hợp
  qua OS queue, chưa phải mouse vật lý và chưa có human review.
- Settings lưu theo application-scope (`%APPDATA%`), không theo từng document/profile.
