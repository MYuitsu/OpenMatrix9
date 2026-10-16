# M01 L2-01-01 — triển khai phần thiếu của shell docking (Rust, standalone)

**Ngày:** 2026-10-16 (UTC) · **Phạm vi:** `OM9-L2-01-01` (L3-001, L3-002, L3-003, L3-004, L3-005, L3-007, L3-008)
**Tiền đề:** [audit mã sẵn có](2026-10-16-m01-l2-01-01-source-audit.md) — người dùng yêu cầu kiểm tra code trước khi làm.

Không có `planning_state`, `execution_policy.status` hay status L3/L4 nào trong `OM9_Milestones/` được nâng.
Đây là bằng chứng **triển khai standalone**, tách khỏi gate M00 và gate thủ công M01-UI.

## 0. Sửa hướng theo tài liệu tham chiếu (quan trọng)

Bản đầu tiên của phiên này đặt **Project Manager + Builder slot vào dock phải** — đó là suy diễn của tôi, **sai**.
Người dùng chỉ ra ảnh Matrix9 thật và yêu cầu đọc `Matrix.rui`. Kết quả kiểm tra:

- `Matrix.rui` (bản cài riêng `C:\Program Files (x86)\Matrix90\Matrix.rui`, 10.7 MB, XML) chỉ định nghĩa **toolbar/menu của Rhino**:
  119 `tool_bar`, 20 `tool_bar_group`, 20 `dock_bar_info`, 14 `sidebar`; `sidebar_one = 42b36785-…` trỏ tới group **"Main"**.
  **Không có mục "Projects"** trong đó (chuỗi "project" chỉ xuất hiện trong tên lệnh như *Project to CPlane*).
  `Config/MainMenu.ini` chỉ định nghĩa nhóm MAIN MENU (Menu1…Menu18). `Matrix.tb` (ws6) không chứa các tên mục sidebar.
- Nguồn đúng là **tài liệu tham chiếu chính thức của chính module này** —
  [`m8book1__p11-13__a-tour-of-the-interface.md`](../../OM9_Milestones/M01%20-%20CAD%20Shell,%20OCCT%20Bridge,%20Document,%20Undo,%20GPU%20and%20Viewport%20Foundation/L2-01-01%20-%20Febe%20Application%20Shell%20Docking/reference/text/m8book1__p11-13__a-tour-of-the-interface.md) (Matrix8 book 1, tr. 11–13):

  > "divided into sections by the gray title bars indicating **"Icon History", "Main Menu", "Display", "Snaps", "Info & Settings", "Layers", and "Projects"**"
  > "In the space **below the Project Manager**, the Builder interface for Matrix tools will appear"
  > "Left-click the gray menu bar once to **"roll up"** or temporarily hide the contents … Some (**Snaps, Layers**) keep important controls … on their closed title bars"
  > "Click and drag … above or below another menu to **change the order** … drag it out into the viewports to **"float"** … drag it to the far right-hand side … it will **"dock"** there … a 300-pixel-wide menu that mirrors its counterpart on the left"
  > "View … **Matrix Menus › Set as Default** … **Reset Menus** puts them back in the default mode, while **Reload Menus** loads them back the way the program opens"

Vì vậy mô hình đã được viết lại cho đúng: 7 mục sidebar theo thứ tự tài liệu, **Builder nằm ngay dưới Projects trong cột trái**, cột phải chỉ là nơi neo menu do người dùng dock (~300 px), và ba lệnh menu theo đúng tên tài liệu.
Bố cục cũ schema 1 (có `right-dock.project-manager`) bị **từ chối và reset về mặc định** kèm chẩn đoán — đã thấy trên hồ sơ thật của máy khi chạy audit.

## 1. Quyết định phạm vi dựa trên audit

Audit cho thấy shell/CMD/viewport/layers/files đã có thật trong `standalone/crates/om9-app` (21 tệp test).
Vì vậy phần được viết mới **chỉ là phần thiếu**, không dựng lại giao diện:

| L3 | Trước | Sau |
| --- | --- | --- |
| 001 Application Startup And Main Window | cửa sổ có, nhưng lỗi khởi động thoát im lặng (`eprintln` trên exe không console) và không có nhánh fallback GPU | `StartupFailure` có kiểu (stage `document`/`renderer`/`native`), receipt `startup-failure.json`, hộp thoại native chỉ khi thật sự mở UI, adapter thiếu → chẩn đoán `GPU adapter unavailable` |
| 002 Workbench Docking Regions | chỉ rail trái + 4 viewport | cột trái có kiểu theo tài liệu + cột phải ~300 px chỉ xuất hiện khi có menu được dock, tự nhường khi cửa sổ hẹp |
| 003 Reorder Collapse And Float | thu gọn nhóm sidebar rời rạc, F6 nổi có Pin | roll-up từng title bar, kéo title bar để đổi thứ tự / dock phải / float, cứu menu ra ngoài màn hình |
| 004 Docking Layout Default Restore | không có | `DockLayout::reset_to_default` + **View ▸ Matrix Menus ▸ Reset Menus** |
| 005 Docking Layout Persistence | không có store nào | `DockLayout::save/load` có version; **Set as Default** ghi cấu hình, **Reload Menus** đọc lại; mặc định ở `%APPDATA%/OpenMatrix9/ui-layout.json` (`OM9_UI_LAYOUT` override) |
| 007 Global Status And Error Surface | lỗi chỉ nằm trong transcript | `StatusSurface` + thanh trạng thái luôn hiển thị, giữ lỗi chưa xác nhận |
| 008 Builder Panel Docking Slot | không có | mục **BUILDERS** giữ chỗ ngay dưới PROJECTS trong cột trái, không tương tác tới khi M04 tới |

## 2. Mã mới

- `standalone/crates/om9-app/src/dock_layout.rs` — chủ sở hữu Rust cho **vùng panel** (left rail, command line, toolbar, cột phải, F6) và **mục menu** (`SectionId`: IconHistory, MainMenu, Display, Snaps, InfoSettings, Layers, Projects, BuilderSlot) với `visible/collapsed/dock/order/height/float`;
  `schema_version = 2`; mặc định theo tài liệu (`BuilderSlot` ngay sau `Projects`); `move_section`, `dock_section`, `float_section`, `normalize`, `clamp_floats`, `resolve(window)` thuần, `from_json`/`load`/`save`.
  Load là hàm **toàn phần**: thiếu tệp → mặc định; hỏng/quá lớn/version lạ (kể cả schema 1) → mặc định + chẩn đoán.
- `standalone/crates/om9-app/src/status_surface.rs` — `StatusKind` (Ready/Info/Warning/Error), `report/acknowledge/complete_success`,
  giới hạn 512 byte theo biên ký tự, tail 8 mục. Lỗi chưa xác nhận **không** bị lệnh thành công sau đó xoá.
- `standalone/crates/om9-app/src/startup.rs` — `StartupStage` + `StartupFailure` (giới hạn 512 byte, gấp newline,
  `to_json`/`write_receipt` ghi nguyên tử qua tệp staging), `adapter_note()` (adapter thật vs `software-fallback` + chẩn đoán),
  `should_report_dialog()` (chỉ `OM9_STARTUP_DIALOG=0/false` mới tắt hộp thoại).
- `standalone/crates/om9-native-owner/src/windows.rs` + `lib.rs` — `report_startup_failure(title, message)`: `MessageBoxW` trong
  unsafe có SAFETY comment, chuỗi bị chặn 512 ký tự trước khi vào Win32; bản non-Windows trả `false` và không giả vờ đã hiện.
- `standalone/crates/om9-app/src/ui/chrome.rs` — `section()` vẽ title bar theo mô hình (click = roll-up, kéo = đổi thứ tự/dock/float),
  `section_bar_controls()` (Snaps giữ Grid/Ortho/Planar, Layers giữ tên layer hiện tại trên thanh đã roll-up),
  `sidebar_ui()` render đúng thứ tự mô hình, `sidebar_section_ui()` tách thân từng mục, `projects_ui()`, `builder_slot_ui()`,
  `right_dock_ui()` (cột phải + nút ▲▼ ◀ ⧉), `float_menu_windows()`, `status_bar_ui()`, menu **View ▸ Matrix Menus ▸ Set as Default / Reset Menus / Reload Menus**.
- `standalone/crates/om9-app/src/ui.rs` — nối layout vào frame thật: `sync_dock_window`, `sync_panel_size`, `settle_section_drag`,
  `float_section_at`, `move_menu_section`, `dock_menu_section`, `float_menu_section`, `set_menus_as_default`, `reset_menus`,
  `reload_menus`, `menus_have_unsaved_changes`, `menu_summary`, `section_rect`, `launch_classified`/`launch_file_classified`, `report_startup_failure`.
- `standalone/crates/om9-app/src/main.rs` — `run()` trả `StartupFailure`; lỗi được ghi stderr + receipt, hộp thoại native
  chỉ khi lệnh thực sự mở cửa sổ (`--ui`, `--ui-audit`, hoặc không tham số).
- `standalone/crates/om9-app/src/lib.rs` — đăng ký `dock_layout`, `status_surface`, `startup`.

Mặc định theo tài liệu: rail trái 300 px (240–420), command line 90 px (58–400), toolbar 67 px, cột phải **ẩn** (300 px, 200–420),
thứ tự trái `Icon History → Main Menu → Display → Snaps → Info & Settings → Layers → Projects → Builders`;
ngưỡng hẹp 900 px và canvas tối thiểu 320 px.

## 3. Bằng chứng chạy thật

| Lệnh | Kết quả |
| --- | --- |
| `cargo test -p om9-app --lib dock_layout` | 13/13 pass (thứ tự tài liệu, Builder dưới Projects, reorder, dock phải, float, schema 1/99 → mặc định, cứu float) |
| `cargo test -p om9-app --lib status_surface` | 4/4 pass |
| `cargo test -p om9-app --lib startup` | 5/5 pass (stage/detail, bound + gấp newline, receipt thay thế lần lỗi trước, adapter fallback, công tắc hộp thoại) |
| `cargo test -p om9-app --test shell_docking` | 10/10 pass (thứ tự cột trái + Builders dưới Projects, roll-up giữ trạng thái qua phiên, dock phải rồi dock lại, float giữ rect, cửa sổ hẹp, Reset/Reload Menus, schema 1, builder bất khả tương tác, status giữ lỗi, biên rail/command) |
| `cargo test -p om9-app` | 36 lib + toàn bộ test tích hợp pass, 0 regression |
| `cargo test -p om9-native-owner` | pass (18 lib + 8 process owner + 1 teardown + 8 verified file) |
| `cargo test --workspace` · `cargo fmt --check` · `cargo clippy --workspace --all-targets -- -D warnings` | pass / exit 0 / exit 0 |

Test UI chạy qua đúng khung `MatrixApp::show` như các test shell hiện có (không mock, không sửa tài liệu trực tiếp).

### 3b. Chạy app thật (không phải test)

`om9.exe --ui-audit <dir>` chạy đúng executable standalone, replay CMD qua đường input thật rồi tự đóng cửa sổ:

- `ui-receipt.json` (copy tại [`2026-10-16-m01-l2-01-01-docking/ui-receipt.json`](2026-10-16-m01-l2-01-01-docking/ui-receipt.json)):
  `passes = 21`, `objects = 1`, `undo = 1`, `screenshot_saved = true`, checks `CMD line accepted once`, `actual CMD Undo`,
  `actual CMD Redo`, `Esc canceled draft without publication`; `view_rects` có đủ 4 khung;
  khối `dock`: `left_sections = [icon-history, main-menu, display, snaps, info-settings, layers, projects, builder-slot]`,
  `right_sections = []`, `floating_sections = []`, `right_dock_visible = false`, `builder_slot_visible = true`,
  `status_bar_px = 20`, `status_kind = "Ready"`.
  `diagnostics` ghi đúng ca chuyển schema thật: `"layout version 1 is not supported; restored the Matrix default"`
  (hồ sơ `%APPDATA%` còn tệp schema 1 do bản đầu của phiên này ghi) — tức đường version-gate chạy thật, không phải mô phỏng.
  `adapter` là adapter phần cứng thật (`NVIDIA GeForce RTX …`).
- `workspace.png` 1600×1000, SHA-256 `7f33d63c445c78075a832e358a722940d2b5fb12df611648c0e71c644644a492`:
  framebuffer thật cho thấy cột trái đúng tài liệu — ICON HISTORY, MAIN MENU, DISPLAY, SNAPS, INFO & SETTINGS, LAYERS,
  **PROJECTS** (Project tiles + Master/Creation/Parts/Render/Output), rồi **BUILDERS** với `Builder (reserved)` và placeholder gem —
  cùng thanh trạng thái `left: 295px, command: 83px, right: 0px · left [Icon History, Main Menu, Display, Snaps, Info & Settings, Layers, Projects, Builders] · right []`.
  Ảnh nằm trong artifact phiên (không commit raster): `build/standalone-validation/ui-artifacts/2026-10-16-m01-l2-01-01-docking/workspace.png`.
- Cấu hình menu **không** tự ghi ra đĩa khi chỉ kéo/thu gọn: đúng ba lệnh tài liệu. Audit xác nhận `ui-layout.json` **không** được tạo (`False`).
- Đường lỗi khởi động (L3-001), chạy thật với `OM9_STARTUP_DIALOG=0` và `OM9_UI_LAYOUT` trỏ vào thư mục tạm,
  `om9.exe --ui <missing>.om9`: exit `1`, stderr `OM9: OpenMatrix9 could not start (document): I/O: The system cannot find the file specified. (os error 2)`,
  và receipt [`startup-failure.json`](2026-10-16-m01-l2-01-01-docking/startup-failure.json) với `schema = om9.startup.failure.v1`,
  `stage = "document"`, `detail` đúng lỗi I/O. Nhánh hộp thoại native chỉ chạy ở lần mở UI thật của người dùng nên **không** được
  kích hoạt trong kiểm thử tự động (tránh hộp thoại modal treo tiến trình); phần quyết định hiện/không hiện đã có test đơn vị.

Ghi chú kiểm tra nguồn công khai: `tools/public_source_audit.py` vẫn exit 1 vì 5877 raster tham chiếu trong
`OM9_Milestones/` và 4 PDF manual **đã có từ trước** (không do thay đổi này); bản ghi mới của phiên này
không còn nằm trong danh sách lỗi.

### 3c. Ghi chú về `Matrix.rui`

Đã đọc trực tiếp `Matrix.rui` trong bản cài riêng (không sao chép nội dung vào repo; chỉ ghi cấu trúc dẫn xuất
trong `build/matrix-rui-*.json`). Kết luận: `Matrix.rui` là định nghĩa UI **của Rhino** (toolbar/group/dock_bar_info/sidebar),
`sidebar_one` trỏ tới group "Main"; **không chứa mục PROJECTS**. Vì vậy vị trí Projects/Builder được chốt theo tài liệu
Matrix8 book 1 tr. 11–13 ở mục 0 — khớp với ảnh Matrix9 thật mà người dùng cung cấp.

## 4. Giới hạn còn lại (không tuyên bố đạt)

- L3-006: ứng dụng vẫn một tài liệu, chưa có chuyển tài liệu (phụ thuộc hợp đồng `OM9-L2-01-03`).
- Kéo-thả menu đã có đủ mô hình + thả để đổi thứ tự/dock/float, nhưng **chưa có preview trực tiếp khi đang kéo** và chưa hỗ trợ
  kéo sang màn hình khác; thanh title bar đã roll-up mới giữ controls rút gọn cho Snaps/Layers.
- Hộp thoại lỗi khởi động native mới được kiểm bằng test quyết định + build Windows, chưa có ảnh chụp thao tác người thật.
- Chưa có ảnh chụp DPI/multi-monitor trên Windows cho module này, nên **gate M01-UI thủ công vẫn mở**.
- M00 vẫn chưa ký: mọi L3/L4 giữ `SPEC_ONLY_NOT_IMPLEMENTED`.

## 5. Việc kế tiếp

1. Preview kéo-thả trực tiếp + thả menu sang màn hình khác (hoàn thiện L3-003).
2. Kiểm thử thủ công Windows (DPI/multi-monitor) cho cột trái/cột phải/status bar, đóng gate M01-UI ở phạm vi module.
3. Chuyển trạng thái tài liệu (L3-006) khi có nhiều tài liệu, sau khi L2-01-03 chốt.
