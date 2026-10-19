# M01 L3-01-01-001 — Application Startup And Main Window (hoàn chỉnh)

**Ngày:** 2026-10-16 (UTC) · **Phạm vi:** `OM9-L3-01-01-001` (module `OM9-L2-01-01`, 4 L4)
**Nguồn ghim đã kiểm:** `vendor-research/QymCAD` @ `746a17f127897e921a57118b6caad08968602bf5`, HEAD khớp, cây sạch.

Bản ghi này đóng **bốn L4** của L3-001 theo đúng artefact mà gói kế hoạch yêu cầu. Không có
`planning_state`/status L3/L4 nào trong `OM9_Milestones/` được nâng.

## 1. L4-01 Source audit → `implementation/design/M01/OM9-L3-01-01-001/source_crosswalk.json`

Bốn anchor theo kế hoạch đều tồn tại thật, cộng thêm các tệp liên quan tìm được trong lần đọc này:

| QymCAD (pin 746a17f1) | Bằng chứng | Quyết định | Chủ sở hữu OM9 |
| --- | --- | --- | --- |
| `crates/qymcad/src/gui.rs` | `:178 impl App`, `:1391 impl eframe::App for App`, `:1401 fn ui(...)` | ADAPT | `om9-app/src/ui.rs` (`MatrixApp::show`, `impl eframe::App::ui`) |
| `crates/qymcad/Cargo.toml` | `:24 eframe 0.35 (persistence, wgpu)`, `:25 egui 0.35` | ADAPT | `om9-app/Cargo.toml` (eframe 0.36.2 wgpu, không persistence) |
| `crates/qymcad-shell/src/lib.rs` | `:59 struct Place`, `:104 struct Shell`, `:111 impl Shell` | NOT_APPLICABLE cho startup | mô hình dock/menu của OM9 là của riêng (`dock_layout.rs`) |
| `crates/qymcad-ui-state/src/lib.rs` | `:87 cmd_val`, `:594 pressed_chord`, `:615 hotkey_action` | NOT_APPLICABLE cho startup | không dùng cho đường khởi động |
| `crates/qymcad/src/gui/startup.rs` | `:164 launch()`, `:174 NativeOptions`, `:175 ViewportBuilder`, `:176 with_inner_size`, `:180 with_title`, `:184 Renderer::Wgpu`, `:194 run_native`, `:201-215` adapter/glow, `:216-225` status line, `:229-236` failed start được ghi lại, `:256` `choose_the_adapter_ourselves`, `:264` native_adapter_selector | ADAPT | `startup.rs` (`AppConfig`, `StartupFailure`, `adapter_note`) + `ui::launch_with` |
| `crates/qymcad/src/window_identity_guard.rs` | `:14 OPENING = "egui::Window::new("`, `:17 windows_without_id`, `:52` guard toàn crate | ADAPT | `om9-app/src/window_identity_guard.rs` (guard đã port) + `.id()` cho F6/float menu |
| `crates/qymcad/src/crash.rs` | `:66 install`, `:98 begin_journal`, `:248 note_failed_start` | ADAPT | `StartupFailure::write_receipt/report` + `main.rs` |
| `crates/qymcad/src/diagnostics.rs` | `:51 note_gpu`, `:59 note_adapters`, `:98 drawing_on_the_processor`, `:112 note_start_failure` | ADAPT | `status_surface.rs` + adapter ghi vào receipt |
| `crates/qymcad/src/app_identity.rs` | `:31` test "cùng một tên ở mọi nơi" | ADAPT | `AppConfig.app_title` + fixture khởi động |

Thêm vào OM9 (không có ở upstream): `StartupFailure` có kiểu + receipt, mô hình dock/menu, bề mặt status,
audit receipt của chính app, store layout theo schema. **Không sao chép mã QymCAD**; QymCAD là AGPL-3.0-or-later
nên việc sao chép vẫn cần phê duyệt G00-06 (ghi rõ trong crosswalk).

## 2. L4-02 Hợp đồng + fixture

- `implementation/design/M01/OM9-L3-01-01-001/api.md`: `AppConfig` (title, document, window/min, maximized, audit dir,
  layout path, dialog policy), `MatrixApp::start(&AppConfig) -> Result<MatrixApp, StartupFailure>`,
  `StartupStage { Document, Renderer, Native }`, `StartupFailure` (bound 512 byte, receipt `om9.startup.failure.v1`,
  atomic replace), `adapter_note`, `should_report_dialog`, quy tắc ownership và bảng lỗi.
- `implementation/fixtures/M01/OM9-L3-01-01-001/positive.json` và `negative.json` (4 ca âm).
- Test điều khiển bằng fixture: `standalone/crates/om9-app/tests/startup_contract.rs` đọc fixture từ đĩa lúc chạy,
  nên hợp đồng lệch khỏi kỳ vọng đã ghi sẽ đỏ ngay (3/3 pass).

## 3. L4-03 Triển khai → `implementation/evidence/M01/OM9-L3-01-01-001/OM9-L4-01-01-001-03.*`

- `startup.rs`: thêm `AppConfig` (+ `interactive`, `window_title`, `validate`) và test đơn vị.
- `ui.rs`: `MatrixApp::start(&AppConfig)`, `launch_with(app, &config)` (title/size/min/maximized/audit/layout từ record),
  `launch_classified`/`launch_file_classified` chạy trên record; F6 nhận `.id("om9-command-search")`.
- `window_identity_guard.rs` (mới): guard theo khuôn QymCAD — quét `src/**` và bắt mọi `egui::Window::new(` thiếu `.id(`.
- Log: `…-03.diffstat.txt` (git status/diff --stat + HEAD OM9), `…-03.target-test.log`
  (`--lib startup` 7/7, `--lib window_identity_guard` 2/2, `--test startup_contract` 3/3, exit 0).

## 4. L4-05 Acceptance → `…-05.regression.log`, `…-05.ui-review.json`

| Kiểm | Kết quả |
| --- | --- |
| `cargo test --workspace` | 88 nhóm "test result: ok", 0 FAILED, exit 0 |
| `cargo fmt --check` · `cargo check --workspace --all-targets` · `cargo clippy -D warnings` | exit 0 · exit 0 · exit 0 |
| `om9.exe --ui-audit` (app thật) | exit 0, `passes=21`, adapter thật **NVIDIA GeForce RTX 3060 / Vulkan / driver 610.88**, 4 viewport, 8 mục menu trái đúng tài liệu, `status_kind=Ready`; ảnh `workspace.png` SHA-256 `7f33d63c…4a492` |
| Lỗi khởi động (`--ui <missing>.om9`, `OM9_STARTUP_DIALOG=0`) | exit 1, stderr `OpenMatrix9 could not start (document): I/O: … os error 2`, receipt `om9.startup.failure.v1`/`stage=document` (SHA-256 `38d78661…b91384`), tài liệu không đổi, layout không ghi ⇒ **không thoát im lặng** |
| Adapter thiếu (fixture `no-hardware-adapter`) | `adapter_note(None)` → `software-fallback` + `GPU adapter unavailable…` |

`…-05.ui-review.json` ghi cả checklist thủ công cho người review (5 bước, gồm DPI 150% và hai màn hình) với
`reviewer.status = NOT_SIGNED` — theo kế hoạch, agent không được tự ký nghiệm thu.

## 5. Trạng thái và việc còn lại

- Gate **M01-UI vẫn MỞ** (chờ người review bằng mắt, đặc biệt DPI/multi-monitor); M00 vẫn chưa ký; không promote status nào.
- Chưa có bước "hạ cấp đồ hoạ tự động giữa các lần chạy" như `safe_graphics` của upstream (ghi ở crosswalk là ADD).
- Việc kế tiếp trong module: **L3-006** (chuyển đổi tài liệu) và phần còn lại của L3-003 (preview khi kéo menu).
