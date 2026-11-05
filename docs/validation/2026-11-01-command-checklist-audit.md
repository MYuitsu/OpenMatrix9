# Kiểm tra danh sách lệnh — menu, Command Line và hình học

Ngày 2026-11-01. Phạm vi: 26 mục trong danh sách kiểm tra của người dùng. Mỗi mục được kiểm
tra theo ba câu hỏi: có mở được từ menu/icon không, có gõ được từ khung lệnh không, và hình
học tạo ra có đúng không.

## Phương pháp

- Binary `L:\om9-target\debug\om9.exe` dựng từ nguồn hiện tại (`cargo build -p om9-app -j 2`,
  `CARGO_TARGET_DIR=L:\om9-target`), chạy `om9 --mcp`. MCP dùng đúng `Controller` của Command
  Line, nên kết quả đo được là kết quả của lệnh thật, không phải mô phỏng.
- Mỗi dòng: dựng tiền đề (đường/circle/box/surface) và chọn đối tượng, gọi tuyến menu bằng
  feature id qua `om9_dispatch`, gọi tuyến gõ lệnh qua `om9_command`, rồi chạy cùng kịch bản
  điểm/số và đọc `om9_report` để đo bounding box và thể tích B-Rep (tổng có dấu của tam giác
  hiển thị). Script: `tools/om9_command_audit.py`; dữ liệu thô:
  `docs/validation/2026-11-01-command-checklist-audit.json`.
- Snap, layer, gumball, STL không có lệnh Command Line tương ứng nên dùng test Rust chuyên
  biệt (bảng dưới cùng).

## Kết quả 26 mục

| # | Mục | Menu/icon | Khung lệnh | Hình học đo được |
|---|-----|-----------|------------|------------------|
| 1 | solid → box | PASS `OM9-SOLID-012` | PASS `Box` | PASS: hộp 10×20×5, V=1000 mm³ |
| 2 | curve → rectangle | PASS `OM9-CURVE-004` | PASS `Rectangle` | PASS: 4 đoạn, 0..10 × 0..20 |
| 3 | snap → mid | PASS icon Snaps | PASS (M) | PASS: Mid đúng (5,0,0) |
| 4 | snap → end | PASS icon Snaps | PASS (E) | PASS: End đúng (10,0,0); click tạo line đúng (0,0,0)-(10,0,0) |
| 5 | snap → on/off | PASS từng loại snap | PASS (E/M one-shot) | PASS: tắt End thì mất marker, bật lại có marker. **Grid Snap chưa nối** |
| 6 | solid → sphere | PASS `OM9-SOLID-014` | PASS `Sphere` | PASS: tâm (0,0,0), r=5, V sai lệch <2% so với 523,6 mm³ |
| 7 | snaps → gumball | — | PASS `Gumball Snappy` | PASS: Snappy dùng candidate Osnap khi kéo, loại object đang di chuyển; Smooth vẫn là mặc định |
| 8 | top11 → copy | PASS `OM9-TOP11-002` | PASS `Copy` | PASS: bản sao dịch +10 theo Z (placement), bản gốc giữ nguyên |
| 9 | curve → polyline | PASS `OM9-CURVE-001` | PASS `Polyline` | PASS: 2 đoạn, 0..10 |
| 10 | curve → offset | PASS `OM9-CURVE-018` | PASS `Offset` | PASS: `Distance=2` cho đường song song y=±2 |
| 11 | top11 → mirror | PASS `OM9-TOP11-004` | PASS `Mirror` | PASS: bản gương x=-4..-1 |
| 12 | snap → center | PASS icon Snaps | PASS | PASS: Center đúng tâm circle (5,5) |
| 13 | top11 → move | PASS `OM9-TOP11-006` | PASS `Move` | PASS: dịch (+5,-2) |
| 14 | top11 → rotate | PASS `OM9-TOP11-007` | PASS `Rotate` | PASS: xoay 90° quanh Z, đầu mút (0,10,0) |
| 15 | transform → rotate 3d | PASS `OM9-TRANSFORM-008` | PASS `Rotate3D` | PASS: xoay 90° quanh Z, đầu mút (0,10,0) |
| 16 | layer → assign, show, hide | PASS panel Layers | — (không có lệnh gõ) | PASS: arrow gán layer, Hide/Show đổi visibility, current layer và Undo đúng |
| 17 | surface → plane from corner | PASS `OM9-SURFACE-015` | PASS `Plane` | PASS: mặt 0..10 × 0..20 tại z=0 |
| 18 | top11 → split | PASS `OM9-TOP11-009` | PASS `Split` | PASS: circle r=5 cắt tại (0,±5) thành 2 nửa cung trái/phải |
| 19 | surface → offset surface | PASS `OM9-SURFACE-025` | PASS `OffsetSrf` | PASS: mặt thứ hai tại z=±5 |
| 20 | solid → shell | PASS `OM9-SOLID-029` | PASS `Shell` | PASS: hộp 10³, bỏ mặt 0, dày 1, V=424 mm³ |
| 21 | curve → dupEdge | PASS `OM9-CURVE-040` | PASS `DupEdge` | PASS: 12 đường biên trong hộp 10³ |
| 22 | top11 → join | PASS `OM9-TOP11-008` | PASS `Join` | PASS: 1 polycurve, 2 đoạn, 0..10 × 0..10 |
| 23 | top11 → trim | PASS `OM9-TOP11-010` | PASS `Trim` | PASS: trim phần trái, còn đoạn x=0..10; đường dọc còn nguyên |
| 24 | solid → extrude curve straight | PASS `OM9-SOLID-009` | PASS `ExtrudeCrv` | PASS: chữ nhật 10×20 cao 5, V=1000 mm³ |
| 25 | solid → different | PASS `OM9-SOLID-001` (mở và chờ "subtract") | PASS `Difference` | PASS: Boolean Two Objects Difference V=500 mm³; nút Difference đo V=500 trong `solid_workflows` |
| 26 | file → export stl | PASS icon File | PASS `ExportSTL` | PASS: chọn mesh/B-Rep kín, xuất STL nhị phân mm, watertight, ghi file atomic |

Tổng: 21/21 dòng chạy được bằng MCP đều PASS ở cả menu và Command Line; 38 test chức năng
PASS (0 fail), cộng các bộ hồi quy đầy đủ 152/195/44 và các bộ mới (STL 7, gumball 5, MCP 2).
Hai mục từng thiếu nay đã có sản phẩm: gumball Snappy và export STL.

## Lỗi đã sửa trong lần kiểm tra này

1. **Top11 không nhận feature id.** Trước: `om9_dispatch("OM9-TOP11-006")` trả
   "unsupported command has no accepted standalone operation"; chỉ icon toolbar và tên gõ
   tay mới chạy. Sau: thêm `Top11Action::from_feature` và tuyến `dispatch_effect` gọi đúng
   `top11_invoke` mà icon dùng, nên menu và Command Line dùng chung một implementation.
   Test: `tests/top11.rs::feature_dispatch_reaches_the_same_action_as_the_toolbar`.
2. **Gõ `Cancel` không hủy lệnh solid.** Trước: prompt Sphere/Box/Cylinder/Cone có chữ Cancel
   nhưng gõ Cancel bị đọc thành tọa độ và báo lỗi ("sphere center needs three coordinates").
   Sau: `submit_solid_pending` xử lý Cancel/Esc một lần trước mọi handler.
   Test: `tests/solid.rs::typed_cancel_ends_a_solid_prompt_without_mutation`; MCP xác nhận
   `running=false` cho cả bốn lệnh.

## Đã nối trong bản cập nhật này (sau khi đọc lại hướng dẫn)

1. **File ▸ Export Selected STL** (`OM9-FILE-001`, codec `OM9-L3-01-12-008`, thực thi theo đích M07):
   - `Controller::export_stl_selected` xử lý mesh, B-Rep sửa được (tái tessellate qua
     `om9_kernel::brep::read_brep`), **retained `NativeBrep`** (payload ON_Brep qua
     `OcctBrepBridge`, không dùng display mesh) và **NurbsSurface** (lưới thích nghi 8–128 bước
     theo sai số chord); áp placement, weld vertex, bỏ tam giác suy biến ở cực cầu, ghi STL nhị
     phân mm atomic. Bề mặt hở ghi non-strict kèm `open_edges`/warning thay vì từ chối.
   - Cùng implementation cho icon File, `File ▸ Export Selected STL…`, lệnh `ExportSTL` và MCP
     `om9_export_stl`; dialog native `.stl` đã được test thật bằng HWND + probe IDOK.
   - Test: `tests/stl_export.rs` 10 ca, `mcp_stdio` 2 ca, trimesh độc lập
     ([record](2026-11-01-stl-independent-viewer.json)), GPU render 4 view.
2. **Gumball Snappy + settings** (theo guide vận hành gumball):
   - `Gumball Snappy`/`Gumball Smooth`; Snappy dùng candidate Osnap, loại object đang di chuyển,
     và với tay nắm Move dùng **world 3D** (axis/plane vẫn chiếu theo ràng buộc), không chỉ mặt
     phẳng kéo. Marker Osnap vẽ trong lúc kéo.
   - `Gumball DragStrength=50` (1–200%), `Gumball RestoreDefaults`; cửa sổ **Gumball settings** mở
     từ icon alignment: axes, Smooth/Snappy, strength, reset pivot, restore defaults.
   - Snappy/strength/alignment được **lưu qua phiên** tại
     `%APPDATA%/OpenMatrix9/gumball.json` (hoặc `OM9_GUMBALL_SETTINGS`), ghi atomic khi đổi và
     đọc lại lúc startup; pivot Relocate vẫn chỉ nhớ trong phiên.
   - Acceptance **OS mouse trên GPU**: `tools/gumball-os-audit.ps1` mở cửa sổ wgpu thật, kéo
     gumball bằng `SetCursorPos`/`mouse_event` sau khi nhận foreground, ghi placement
     `[0,5,0]-[10,5,0]` và screenshot framebuffer GPU; 3 lần chạy PASS.
   - Test: `gumball_viewport` 7 ca + settings window + persistence reload.

Chi tiết bổ sung: [2026-11-01-stl-gumball-completion](2026-11-01-stl-gumball-completion/README.md).

## Giới hạn còn lại

- **STL**: gate M09 G-MVP-013 vẫn cần independent **viewer GUI** được định danh và reviewer ký;
  trimesh chỉ là reader độc lập. Lưới surface thích nghi theo chord 0,01 mm nhưng vẫn đều mỗi
  chiều (8–128 bước), chưa phải quadtree T-junction; retained NativeBrep phụ thuộc bridge OCCT.
- **Gumball**: Snappy đã được kiểm bằng OS mouse trên cửa sổ GPU thật
  (`tools/gumball-os-audit.ps1`, record `docs/validation/2026-11-01-gumball-os-gpu.json`); còn
  human review/mouse vật lý và settings application-scope (chưa theo profile).
- **Grid Snap**: icon `GridSnapON` và `GridSnap` trên thanh tiêu đề hiển thị nhưng disabled;
  model chưa có grid-snap. Snap theo loại đối tượng và Ortho/Planar/Project vẫn hoạt động.
- **Pre-existing test failures**: bộ `om9-app --lib` hiện có 3 ca fail không thuộc thay đổi này
  (2 ca icon-size oracle 31 vs 20 trong `ui/icon_size_audit.rs`, 1 ca Split widget trong
  `ui/chrome.rs`); các file đó đã có thay đổi chưa commit từ trước phiên này.

## Bằng chứng test bổ sung

| Test | Số test | Chứng minh |
|------|---------|------------|
| `om9-scene --test osnap` | 11 | End (10,0,0), Mid (5,0,0), Center, Quadrant đúng và được rank trong bán kính màn hình |
| `om9-app --test osnap_ui` | 3 | Click trong bán kính snap tạo line đúng điểm snap; tắt End mất marker; Escape xóa guide |
| `om9-app --test snap_panel_modes` | 4 | Bật/tắt snap persistent và one-shot |
| `om9-app --test layer_controller` | 9 | Gán layer lazy, current layer, toggle nhóm, layer khóa từ chối và Undo |
| `om9-app --test layer_widgets` (hide/show) | 1 | Nút Hide/Show đổi visibility, giữ preference hidden/selectable |
| `om9-app --test layer_current_and_assign` | 3 | Panel và model đồng ý về current layer và assignment |
| `om9-app --test gumball_viewport` | 7 | Kéo/rotate/scale cũ; Snappy mặc định, strength 50%, defaults, settings icon, 3D snap |
| `tools/gumball-os-audit.ps1` | 1 | OS mouse trên cửa sổ GPU: line [0,0,0]-[10,0,0] → [0,5,0]-[10,5,0], screenshot GPU, 3 lần PASS |
| `om9-app --test solid_workflows` (immediate boolean) | 1 | Nút Difference qua menu + pick đúng V=500, một Undo |
| `om9-exchange --test stl_export_policy` | 3 | Writer STL đúng layout/đơn vị/winding/từ chối |
| `om9-app --test stl_export` | 10 | Route menu/CMD, selection, mixed skip, sphere watertight, native payload, surface thích nghi/hở, mm, sai đuôi, ghi đè |
| `om9-native-owner --lib real_stl_save_dialog... -- --ignored` | 1 | Dialog Save thật của Windows, probe IDOK trả đúng `audit.stl` |
| `tools/om9_stl_independent_viewer.py` (trimesh) | 10 checks | Box/cầu: triangle count, bounds, volume, watertight, winding, 0 degenerate |
| `om9 --render-4` | 1 ảnh | GPU render box+cầu trong bốn view (1280×800 RGBA) |
| `om9-app --test top11` (feature dispatch) | 1 | Dispatch `OM9-TOP11-006` mở đúng Move như toolbar |
| `om9-app --test solid` (typed Cancel) | 1 | Cancel kết thúc prompt solid, không sửa document |
| `om9-app --test mcp_stdio` | 2 | Giao thức MCP và tool `om9_export_stl` |

Bốn bộ hồi quy đầy đủ của các module bị sửa cũng chạy lại và pass:

| Bộ | Kết quả |
|----|---------|
| `om9-app --test top11` | 152 passed, 0 failed (gồm test dispatch mới) |
| `om9-app --test solid` | 195 passed, 0 failed, 1 ignored (gồm test Cancel mới) |
| `om9-core --test top11` | 44 passed, 0 failed |
| `om9-app --test mcp_stdio` | 2 passed, 0 failed (thêm tool `om9_export_stl`) |

## Giới hạn

- Đo hình học solid bằng tam giác hiển thị (deflection mặc định), nên thể tích cầu sai lệch
  trong 2%; box/shell/extrude/boolean là mặt phẳng nên khớp chính xác. Trimesh độc lập xác nhận
  cầu sai lệch 0,15% và mọi kiểm tra watertight/winding.
- Đã kiểm dialog Win32 thật, egui mouse, GPU render tĩnh và kéo gumball Snappy bằng OS mouse
  trên cửa sổ wgpu thật (3 lần PASS); đây là input tổng hợp, chưa có human review. Trimesh là
  reader độc lập, chưa phải GUI viewer/reviewer cho gate M09.
- Các thay đổi top11 khác đang có trong working tree từ trước phiên này vẫn giữ nguyên.

## File thay đổi và SHA-256

- `crates/om9-core/src/top11.rs` `72ee058d053e2b4728cab2cf021edd1d8b06ecc4367e7bf77462c470c7d5dc88`
- `crates/om9-app/src/session.rs` `912d91cc9676948146aaae97daa71e861cb06efacf51b9204f0fedb6d6fd247e`
- `crates/om9-app/src/startup.rs` `cf6f19095a7e782cef16d8121b2ce08c550d6029af23753f957446d87d7582c7`
- `crates/om9-app/src/session/solid.rs` `fb23b4c744b71a2c6eccee9c6176657a28dc0693a5f993e39a5224d8381c5cd1`
- `crates/om9-app/src/session/stl.rs` `b6989059143ac1ba4cc1b098a1b96ecc6b2565733c9a959d4126ef60dca6ae1b`
- `crates/om9-app/src/session/gumball.rs` `61799eb40a78e99d420abdd4fec4e6c21f44de3260cd7aa339182a27ed67d599`
- `crates/om9-app/src/session/text_input.rs` `9675293056c85d167e188e840289624adc80a993efa35669650a843d8fd16f5c`
- `crates/om9-app/src/session/registry.rs` `1a2ddb6e2e2bdd5d56b346db9aeb36cf9afe175c243ae73015c83db6610f5968`
- `crates/om9-app/src/ui.rs` `def6836dc7172af6d862369ec56dd60ea75d1356a42e97d508edbd09837def19`
- `crates/om9-app/src/ui/os_audit.rs` `6a4a045d94e8f75b66a13735f23d6ea07ce86efe10ea1439ac6f97e702e89876`
- `crates/om9-app/src/ui/files.rs` `e4d55a97f563298513341e8ef27018dbb162fb103acaea47341ac938740fe772`
- `crates/om9-app/src/ui/chrome.rs` `fbd6dc91cd26deca8d90bbb3e39dd3171be277fb794cb56b36807bdbb0e8c950`
- `crates/om9-app/src/ui/sidebar_palettes.rs` `6e029d604657e3c52d1753e9f242b4c7fec81b949678251e7f82028b6a9188f1`
- `crates/om9-app/src/native_picker.rs` `912891fc3e177fb86a3380f851ced62b0a2d6de5e77d639208ab5542764dec95`
- `crates/om9-native-owner/src/file_dialog.rs` `f464a0b508be3755a34a942506c1d9d31b0d4fa4a189c2b4382a347ddea37eaa`
- `crates/om9-native-owner/src/dialog_probe.rs` `a68dc1ecbdbc653ffc409d7e6cbf2f60cb29249232ebb450fdd3e47c77d87e2d`
- `crates/om9-native-owner/Cargo.toml` `9944f253eb2be39491d079503129b41fcc027c1864c807cf3519b48a7402ac45`
- `crates/om9-app/src/mcp.rs` `9fcea713bcddd12a0613661ebf2483a3a6a989dc70ee4438a592c1c226443f70`
- `crates/om9-app/src/mcp/tools.rs` `1d65739d8fdaf13ca14b3899e97624abd017957f47676dc7ad2d4f2982c1e206`
- `crates/om9-app/src/mcp/document.rs` `ed45a34b0ff75173090a40ea32d8c9b2c21b4e0fb73d8cd97de16d53b24efaa1`
- `crates/om9-app/src/mcp/command.rs` `ea44a60080e099ec28c32a609e99a945e059d57ef346262a39f3992380f09073`
- `crates/om9-app/tests/stl_export.rs` `cbd420d86531b59d5da8747eeaba4acec052c1f0d3b1cf965c4807994c61a9fc`
- `crates/om9-app/tests/gumball_viewport.rs` `a93b2ab9d8c33afbb94b274595434ccf5cfb1eda6827505b090060414eef8646`
- `crates/om9-app/tests/mcp_stdio.rs` `ac9475accc24f38603e9b75c2447abe1ed48a802c127bfadbea3b2d0427bbcd3`
- `crates/om9-app/tests/startup_contract.rs` `7e5556a2c520ad294c71376759a8d1e57b3cad04548e6fdc6616e35b28beddcc`
- `crates/om9-app/tests/top11.rs` `6e830265487930c6a9238bd1d6cc93e2a9a21d0f05bc5614c48c3e10d2855013`
- `crates/om9-app/tests/solid.rs` `bf39fc285460a847d4049958fe67d9398dc9fd168c65c7d49a3406b496c778d9`
- `tools/om9_command_audit.py` `e18a0e979691178bc2611c66fd533c89553f7e574e610821cad27118378d30e9`
- `tools/om9_mcp_client.py` `cf1fa03f4354772ac230a7c779cd5eafbb440bfa027e370c951f984a29f2ac57`
- `tools/om9_stl_independent_viewer.py` `91ca7943912f6e613a94cff9ed33603e1f37ae6c1d700279a296039c4d09d37f`
- `tools/om9_write_audit_record.py` `44e61dc634f54b37aee3a2c53ed84b3113b7a991da687061081b3eb36efdb2f8`
- `tools/gumball-os-audit.ps1` `f3a19e30d01c1f0aa7a0ff8f4b9cedda2398a208af03f298d015d3337b0483ed`
- `tools/om9_blank_project.py` `b8319eaa8d1389c3d7e0e073c9aaae3d618b2d4be2d38f00e16b30090a4613bc`
- `tools/om9_write_os_acceptance.py` `e396d8692234e42f18ca49455160d2b74a8d9fab95f133360826572454415324`
- `docs/validation/2026-11-01-stl-independent-viewer.json` `6cb0dc55caa018620496d761a8683c35388645f5650b2e8c5523a6c745aafbb1`
- `docs/validation/2026-11-01-gumball-os-gpu.json` `660131d8d6c88f6729139a381036731b871073d5bf7b01d14e832cf753804605`
