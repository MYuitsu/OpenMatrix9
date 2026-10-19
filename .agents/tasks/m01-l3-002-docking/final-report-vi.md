# Báo cáo cuối: OM9-L3-01-01-002 Workbench Docking Regions

## Diễn giải yêu cầu
"M01 - L2-01 L3-02" = milestone M01, L2-01-01 (Febe Application Shell Docking), L3-002 (Workbench Docking Regions) = OM9-L3-01-01-002. Đây là mục khớp duy nhất. Lượt hai ("làm xong các việc chưa xong cho xong M01 L2-01-01 L3-002") là đợt close-out cho 8 việc còn mở sau lượt đầu.

## Các L4 đã hoàn thành (có bằng chứng, chạy trong `standalone`)
- L4-01 Source audit: `implementation/design/M01/OM9-L3-01-01-002/source_crosswalk.json`. QymCAD ghim `746a17f1...`, sạch (`git status --porcelain` rỗng). Không chép mã, không phụ thuộc QymCAD.
- L4-02 Hợp đồng API và thiết kế test: `api.md`, `positive.json`, `negative.json`. RED trước khi code, thoát 101, lỗi assertion, không lỗi build.
- L4-03 Triển khai (Rust an toàn, không ngoại lệ native): `workbench_regions.rs` và nối vào `ui.rs`, `ui/chrome.rs`.
- L4-05 Nghiệm thu tự động: `om9.exe --ui-audit` exit 0, 21 pass (RTX 3060, Vulkan). Ảnh: `build/standalone-validation/ui-artifacts/2026-10-16-m01-l3-002-docking/`.
- Review lượt đầu: APPROVED (`review.json`).

## Đợt close-out (2026-10-16): số liệu thật từ lần chạy cuối
- `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace -j 1`: exit 0; 581 đạt, 0 lỗi, 24 bỏ qua (test native). `om9-app`: lib 62, `workbench_regions` 8, `status_bar_overlap` 4, `workbench_regions_sizes` 10, `shell_docking` 10. (Chạy song song bị lỗi hệ điều hành 1455 lúc link, nên dùng `-j 1`.)
- `python tools/validate_m01_l3_002_ui_matrix.py`: exit 0, 10/10 ca cửa sổ thật PASS. Chiều rộng đầu tiên mà dock phải xuất hiện: 915 pt.
- Việc 1, chữ thanh trạng thái chồng lên nhau ở 800x600: đã sửa trong `ui/chrome.rs::status_bar_ui`. Test đỏ (`closeout-01.red.log`, exit 101, 3 lỗi assertion) rồi xanh (`closeout-01.green.log`); ảnh mới đã xem.
- Việc 3, kiểm toán nguồn công khai:
  - Chạy toàn bộ `python tools/public_source_audit.py`: exit 1, 11721 lỗi (OM9_Milestones 5877, ref 5611, vendor-research 114, QymCAD 114, manuals 4, om9-window.png 1), tất cả nằm ngoài các đường dẫn của mục này. Toàn bộ kiểm toán CHƯA đạt.
  - Chạy thu hẹp trên bản sao chỉ-đọc `build/scoped-audit/2026-10-16-l3-002` (557 tệp, không có ảnh PNG ngoài `Resources/`): exit 0. Ca đối chứng thêm `control.exe` và `control.png` vào bản sao: exit 1 đúng hai lỗi đó; xóa đi thì exit 0 lại. Kết quả thu hẹp chỉ nói về các tệp đã sao chép, không nói về cả repo.
  - Không sửa `tools/public_source_audit.py`. Log: `implementation/evidence/M01/OM9-L3-01-01-002/closeout-03.scoped-audit.log`.
  - Đề xuất để bạn quyết định (chưa làm): thêm `--paths FILE...` (hoặc `--only-tracked` qua `git ls-files`) và danh sách `--exclude` tùy chọn cho `ref`, `manuals`, `OM9_Milestones`, `QymCAD`, `vendor-research`, ảnh ở gốc; hành vi mặc định giữ nguyên. Đánh đổi: dùng `--exclude` làm mặc định sẽ làm yếu tính fail-closed; `--paths` thì không.
- Việc 5, OM9-L2-01-02 và OM9-L2-01-03: ngoài phạm vi, giữ stub (`CommandInputHost`, `ViewportSceneHost`, `AttachedCmdFrame::command_input_ready` luôn true). Trạng thái BLOCKED_DEPENDENCY. Việc cần làm sau: khi OM9-L2-01-02 xong, thay adapter hằng-true `AttachedCmdFrame::command_input_ready` trong `ui.rs` bằng truy vấn CMD host thật và thêm test khung hình khi host chưa sẵn sàng để chạy `RegionError::CommandInputUnavailable`; khi OM9-L2-01-03 xong, thay phần `Workspace` của `ViewportSceneHost` và chạy lại `tests/workbench_regions.rs`. Tới lúc đó `CommandInputUnavailable` chỉ chạm được trong test đơn vị và fixture.
- Việc 6, không chép mã và không phụ thuộc QymCAD: `cargo tree --workspace` (kể cả `--edges all --all-targets`) không có `qymcad`; không có trong 7 `Cargo.toml` và `Cargo.lock` của `standalone`; các nhắc tới `qymcad` trong `standalone/crates/**/*.rs` chỉ là chú thích (`startup.rs:41`, `ui.rs:968`, `window_identity_guard.rs:5`, `workbench_regions.rs:23-24`). Lưu ý `startup.rs` và `window_identity_guard.rs` ghi "Adapted from", thuộc công việc trước đó, không thuộc mục này; trạng thái tái sử dụng của chúng nằm trong cùng quyết định pháp lý đang chờ. `vendor-research/QymCAD` vẫn ở `746a17f127897e921a57118b6caad08968602bf5`, `status --porcelain` rỗng. PENDING_HUMAN_M00_APPROVAL giữ nguyên; phê duyệt pháp lý/M00 là quyết định của người, tôi không duyệt gì.
- Việc 7, `planning_state` giữ nguyên: `l3.json` của L3-002 là `SPEC_ONLY_NOT_IMPLEMENTED`, bốn `l4.json` là `NOT_STARTED`. Lý do: (a) quy ước repo không có giá trị "đã làm nhưng chờ cổng" (đếm lại: L3 586 `SPEC_ONLY_NOT_IMPLEMENTED` / 206 `SPECIFIED_L4_PLANNED_NOT_IMPLEMENTED`; L2 83 / 20; L4 1749 `NOT_STARTED`, 340 `SPEC_ONLY_NOT_EXECUTED`, 522 không có trường), `tools/merge_l4_overlay.py` là nơi duy nhất ghi và chỉ chuyển giữa hai giá trị chưa-triển-khai; (b) `execution_policy` trong `l4.json` khai `NOT_RUN`, `must_not_fake_pass`, cần M00 ký trước, mà M00 là NOT_PASS, pháp lý đang chờ, L2-01-02/03 chưa có; (c) cây kế hoạch là vùng được bảo vệ. `git status` giới hạn ở thư mục L3-002 không có thay đổi nào từ đợt này. Nếu muốn thêm tên trạng thái mới thì đó là đổi quy ước, do bạn quyết định.

## Tệp tạo hoặc thay đổi (cả hai lượt)
- Mới: `standalone/crates/om9-app/src/workbench_regions.rs`, `tests/workbench_regions.rs`, `tests/status_bar_overlap.rs`, `tests/workbench_regions_sizes.rs`, `tools/validate_m01_l3_002_ui_matrix.py` (ngoại lệ Rust-first: chỉ khởi chạy `om9.exe` thật bằng biến môi trường và đọc `ui-receipt.json`, cần cửa sổ GPU thật, không có logic nghiệp vụ), `docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md`, `implementation/{design,fixtures,evidence}/M01/OM9-L3-01-01-002/*`, `.agents/tasks/m01-l3-002-docking/*`.
- Sửa: `src/ui.rs` (kế hoạch vùng, biến audit `OM9_UI_AUDIT_WINDOW`, `OM9_UI_AUDIT_RIGHT_DOCK`, `OM9_UI_AUDIT_ZOOM`, trường đo chữ thanh trạng thái, một câu chú thích BLOCKED_DEPENDENCY), `src/ui/chrome.rs` (`sync_panel_size` không ghi khi có lỗi vùng; sửa thanh trạng thái), `skills/openmatrix9-workspace-contract/SKILL.md` (và bản cài đặt), `docs/openmatrix9-progress.json`.
- Không commit, không stage, không đụng `ref/`, `manuals/`, `OM9_Milestones/`, `QymCAD/`, `vendor-research/`, `om9-window.png`.

## Lỗi phát hiện, ghi lại, chưa sửa
- Lưu tài liệu có Line lỗi `Invalid('archive encoding verification')` trong `om9-archive` (lưu Point thì được); test vùng dùng tài liệu Point.
- Dưới khoảng 311 px chiều cao, kế hoạch kẹp panel lệnh về 58 nhưng egui vẫn vẽ panel cao hơn đã lưu (hở 28 px ở 560x286; cửa sổ thật không xuống được vì tối thiểu HĐH là 480).

## Chưa được xác minh
- Cổng M01-UI vẫn MỞ. Máy chỉ chứng minh theo kiểu mô phỏng (ma trận kích thước, 150% giả lập bằng `OM9_UI_AUDIT_ZOOM=1.5` trong không gian point, không phải DPI thật từng màn hình). `reviewer.status = NOT_SIGNED` trong `OM9-L4-01-01-002-05.ui-review.json`, danh sách rút gọn còn 5 mục.
- Quyền tái sử dụng pháp lý: PENDING_HUMAN_M00_APPROVAL; M00 NOT_PASS.
- OM9-L2-01-02 và OM9-L2-01-03: BLOCKED_DEPENDENCY như trên.
- `tools/validate_m01_*`, `OpenMatrix9_Codex_Spec_v2/` và định nghĩa `om9.m01.oracle.v1.5` vẫn không có.

## Thay đổi quy tắc nghiệp vụ: đã ghi vào skill
Bạn đã đồng ý bằng câu trả lời "làm xong các việc chưa xong cho xong M01 L2-01-01 L3-002". Đã thêm mục "Vùng workbench docking (OM9-L3-01-01-002)" vào `skills/openmatrix9-workspace-contract/SKILL.md` kèm dòng đồng ý có ngày 2026-10-16, đúng bốn hành vi và quy tắc dock phải nhường dưới khoảng 900 px:
1. Cửa sổ dưới 560x286 (320x286 khi không có rail trái): hiện lỗi "Layout:" trên thanh trạng thái và giữ kế hoạch tốt gần nhất, không kẹp âm thầm.
2. Dòng lệnh bị đánh dấu ẩn vẫn được giữ chỗ, kèm thông báo `CommandInputKept`.
3. Khi đang có lỗi vùng, đồng bộ kích thước panel và kẹp menu nổi không ghi lại layout.
4. Dock phải nhường hoặc rail trái thu hẹp: ghi vào lịch sử CMD, và vào thanh trạng thái nếu không có lỗi đang hiện.

Bản cài đặt `C:\Users\nguye\.codex\skills\openmatrix9-workspace-contract\SKILL.md` đã đồng bộ; SHA-256 hai bản giống nhau (`453705A0A7EC1791D25BFFD176F778329BE13F3C44B97DF922C4476ED251F69F`). Việc rút gọn chữ thanh trạng thái là sửa trình bày, không ghi vào skill. Không thêm gì khác ngoài các mục trên.

## Việc / Trạng thái / Ai đóng
| Việc | Trạng thái | Ai đóng |
| --- | --- | --- |
| Chữ thanh trạng thái chồng lên nhau ở 800x600 | Đã sửa, có test đỏ→xanh, ảnh mới đã xem | Máy |
| Ma trận kích thước + 150% DPI giả lập | Đã chạy (10 test kích thước, 10/10 ca cửa sổ thật), nhãn SIMULATED | Máy (không thay thế ký của người) |
| DPI thật, màn hình thứ hai, kéo cửa sổ thật | Chưa làm | Người (M01-UI) |
| Kiểm toán nguồn công khai | Quét thu hẹp exit 0; toàn bộ vẫn exit 1 (11721 lỗi) | Người quyết định đề xuất |
| Ghi quy tắc vào skill | Đã ghi, đã sync bản cài, hash giống nhau | Máy |
| L2-01-02 / L2-01-03 | BLOCKED_DEPENDENCY, thay stub khi module đó xong | Module khác |
| Pháp lý / M00 | PENDING_HUMAN_M00_APPROVAL | Người |
| planning_state | Giữ nguyên, có giải thích | Người nếu muốn đổi quy ước |

## Hành động tiếp theo (theo thứ tự)
1. Chạy danh sách M01-UI rút gọn trên Windows và ký `OM9-L4-01-01-002-05.ui-review.json`.
2. Quyết định phê duyệt pháp lý/M00 cho việc tái sử dụng QymCAD.
3. Quyết định đề xuất thu hẹp phạm vi kiểm toán nguồn công khai.

Sau đó làm tiếp mục anh em kế tiếp: L3-003 (live drag preview) của OM9-L2-01-01.

Không có gì được commit hoặc stage trong đợt này.

---

## Cập nhật 2026-10-16: cửa sổ tối thiểu 1280x720 logic

Chỉ là bằng chứng. Chưa cổng M01-UI, M00 hay pháp lý nào được thông qua. Không commit, không push, không stage.

### Đã đổi gì (trước và sau)

| Mục | Trước | Sau |
| --- | --- | --- |
| Cửa sổ thật (min inner size) | 720x480 | 1280x720 điểm logic, có giới hạn theo màn hình nhỏ (xem bên dưới) |
| Sàn của bộ lập kế hoạch vùng | 560x286 (320x286 khi ẩn rail trái) | 800x400, không phụ thuộc rail trái |
| Ngưỡng dock phải nhường | 900, đo thực tế dock hiện từ 915 | 1200 điểm logic |
| Dưới sàn | lỗi `Layout:` + giữ kế hoạch tốt gần nhất | giữ nguyên hành vi, chỉ đổi số |
| Ma trận cửa sổ thật | 10 ca (720x480 ... dpi150-1280x800) | 9 ca: 1280x720, 1366x728, 1440x720, 1440x900, 1280x900, mô phỏng laptop 1366x768 ở 100%/125%/150%, và 1 ca "unsupported size, error path" (1000x600 bị từ chối khi khởi động) |
| Validator ma trận | chỉ kiểm `status_text_overlap == false` | thêm `status_message_px` và `status_summary_px` phải khác null, kiểm mức tối thiểu đã gửi, và báo lỗi nếu màn hình kẹp nhỏ cửa sổ |

### Xung đột DPI cao và cách xử lý

Laptop 1366x768 thường chạy 125% hoặc 150% DPI, tức khoảng 1093x614 hoặc 911x512 điểm logic, và thanh tác vụ cộng thanh tiêu đề lấy thêm khoảng 80 điểm. Mức tối thiểu cứng 1280x720 logic không vừa các màn hình này, cửa sổ sẽ bị đẩy ra ngoài màn hình.

Cách chọn (mặc định đã đề ra): mức tối thiểu hiệu lực = min(1280x720, vùng dùng được của màn hình theo điểm logic), không thấp hơn sàn 800x400. Trên màn hình bình thường vẫn là 1280x720. Hàm thuần `startup::effective_min_size` có test cho: màn hình lớn, 1366x768 ở 100% (1280x688), 125% (1076.8x534.4), 150% (894.7x432), màn hình quá nhỏ hoặc dữ liệu hỏng (không panic, không bằng 0). Ứng dụng gửi `ViewportCommand::MinInnerSize` ở khung hình đầu và mỗi khi màn hình đổi (chuyển màn hình khác DPI).

Giới hạn trung thực: egui/winit không cung cấp vùng làm việc thật (cần gọi Win32, tức unsafe hoặc FFI, trái quy tắc Rust an toàn), nên vùng dùng được chỉ là ước lượng = kích thước màn hình trừ (16, 80) điểm. Tôi tính cả thanh tiêu đề vì 720 là chiều cao vùng client. Nếu bạn muốn chỉ trừ thanh tác vụ, đổi một hằng `SHELL_RESERVE_PT`. Chưa chứng minh được việc khởi động ở trạng thái phóng to trên máy 1366x768 thật ở 125%/150%.

### Ngưỡng dock phải: 1200 điểm logic

Với rail mặc định (trái 300, phải 300, khe 2), mỗi viewport rộng (W - 602) / 2: 156,5 điểm ở ngưỡng cũ 915, 299 điểm ở 1200. Một viewport cần khoảng 300 điểm để còn dùng được, nên 1200 là mốc tròn nhỏ nhất giữ được bốn viewport hữu ích cạnh hai rail. 1200 thấp hơn mức tối thiểu 1280 nên dock luôn hiện ở mọi cỡ được hỗ trợ trên màn hình thường, và nhường ở ca 125%/150% (viewport rộng 395 và 305 điểm). Có test biên 1199 / 1199,99 / 1200 và quét khung hình 1150..1250 (dock đầu tiên ở 1200).

Bằng chứng bố cục (điểm logic, dock phải bật): 1280x720 -> viewport 339x260, có dock; 1366x728 -> 382x264, có dock; 1093x585 -> 395,5x192,5, dock nhường; 911x480 -> 304,5x140, dock nhường; 1920x1080 -> 659x440; 5120x1440 -> 2259x620. Số liệu: `implementation/evidence/M01/OM9-L3-01-01-002/min-window-layouts.json`.

### Assertion đã đổi (và lý do)

Mọi assertion dùng 560x286, 320x286, 559x286, 560x285, 719x479, 900 hay 915 được thay bằng assertion tương đương ở 800x400, 1280x720 hoặc 1200; không nới lỏng để cho qua. Chi tiết từng dòng nằm ở bảng "Assertions changed" trong `docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md`. Các ca cỡ nhỏ bị bỏ; giữ đúng một ca dưới mức tối thiểu `unsupported_size_error_path` (799x600 và 800x399: lỗi `Layout:`, giữ kế hoạch cũ, không đổi layout/cảnh/tài liệu).

### Skill (đã được bạn đồng ý đúng thay đổi này)

Trước:
- "Rail trái, bốn viewport và ô nhập lệnh luôn được giữ; dock phải nhường khi cửa sổ hẹp hơn khoảng 900 px."
- "Cửa sổ nhỏ hơn mức tối thiểu (560x286, hoặc 320x286 khi không có rail trái) hiện lỗi `Layout:` trên thanh trạng thái và giữ kế hoạch tốt gần nhất, không kẹp âm thầm."

Sau:
- Dòng đồng ý mới: "Người dùng đồng ý ngày 2026-10-16: quyết định cửa sổ tối thiểu 1280x720 logic, ngưỡng dock phải giao cho agent chọn (chỉ cập nhật hai dòng số tối thiểu và ngưỡng dock bên dưới)."
- "Rail trái, bốn viewport và ô nhập lệnh luôn được giữ; dock phải nhường khi cửa sổ hẹp hơn 1200 điểm logic."
- "Mức tối thiểu được hỗ trợ là 1280x720 điểm logic, áp ở cấp cửa sổ thật; trên màn hình nhỏ hơn mức đó tính theo điểm logic (ví dụ 1366x768 ở DPI 125% hoặc 150%) mức tối thiểu bị giới hạn theo vùng dùng được của màn hình. Cửa sổ nhỏ hơn mức sàn kế hoạch (800x400) hiện lỗi `Layout:` trên thanh trạng thái và giữ kế hoạch tốt gần nhất, không kẹp âm thầm."

Các dòng khác (CommandInputKept, đồng bộ bị tạm dừng khi có lỗi vùng, thông báo CMD/thanh trạng thái) giữ nguyên. Bản cài tại `C:\Users\nguye\.codex\skills\openmatrix9-workspace-contract\SKILL.md` đã đồng bộ; SHA-256 hai bản bằng nhau: `B1D2305AEA2FC69BE5698C3261D7A1F0A6DCEA5B57D0CF225E13968616A58541`.

### Kết quả lệnh thật (chạy trong `h:\OpenMatrix9\standalone`, nơi có workspace Cargo; không có Cargo.toml ở gốc repo)

- `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace -j 1`: exit 0; 596 passed, 0 failed, 24 ignored (test native bị khóa). `om9-app`: lib 73, window_minimum 5, workbench_regions 8, workbench_regions_sizes 9, status_bar_overlap 4, shell_docking 10.
- `python tools\validate_m01_l3_002_ui_matrix.py`: exit 0, 9/9 PASS. Thư mục ảnh mới: `build/standalone-validation/ui-artifacts/2026-10-16-m01-l3-002-min-window/`. Đã xem ảnh `min-1280x720`, `laptop-1366x728`, `lap125-1366x680`, `lap150-1366x700`: thấy rail trái, bốn viewport, khung lệnh, dòng trạng thái không chồng chữ; dock hiện ở 1280/1366 và nhường ở ca 125%/150%.
- `python tools\public_source_audit.py` toàn bộ: exit 1, 11721 lỗi, tất cả nằm trong cây riêng tư (có sẵn từ trước). KHÔNG tuyên bố là đã qua. Bản sao phạm vi hẹp (565 tệp): exit 0; ca đối chứng exit 1 đúng 2 lỗi; chạy lại exit 0.

Phát hiện cần biết: màn hình thử nghiệm lần này báo 1440x900 (không phải 2560x1440 như kế hoạch), hệ điều hành kẹp cửa sổ lớn hơn. Vì vậy cửa sổ thật 1600x1000/1920x1080/2560 không chạy được ở đây; validator giờ báo lỗi nếu cửa sổ bị kẹp, và các cỡ đó được phủ bằng test khung hình và bảng bố cục tính toán.

### Tệp đã đổi

`standalone/crates/om9-app/src/{startup,ui,dock_layout,workbench_regions}.rs`; test `tests/{window_minimum (mới),workbench_regions_sizes,status_bar_overlap}.rs`; fixture `implementation/fixtures/M01/OM9-L3-01-01-002/{positive,negative}.json`; bằng chứng `implementation/evidence/M01/OM9-L3-01-01-002/` (`min-window-01.red/green.log`, `min-window-layouts.json/.log`, `min-window.scoped-audit.log`, `ui-review.json`); `tools/validate_m01_l3_002_ui_matrix.py`; `skills/openmatrix9-workspace-contract/SKILL.md`; `docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md`; `docs/openmatrix9-progress.json`; `verification.md`; báo cáo này.

### Lỗi đã biết

- Lỗi khung lệnh lệch chiều cao (~311 px) nay nằm ngoài phạm vi được hỗ trợ: đã ghi nhận, không sửa.
- Lưu tài liệu có Line báo `archive encoding verification` trong `om9-archive`: không liên quan, chưa sửa.

### Chỉ con người làm được

- Ký M01-UI: chạy checklist 5 mục trên máy 1366x768 thật ở 100%/125%/150% DPI, màn hình thứ hai, kéo cạnh cửa sổ để kiểm tra mức tối thiểu (kéo trực tiếp vẫn thuộc L3-003). `reviewer.status = NOT_SIGNED`, cổng M01-UI vẫn OPEN.
- Phê duyệt pháp lý M00 (`PENDING_HUMAN_M00_APPROVAL`).
- Quyết định phạm vi kiểm toán mã nguồn công khai.
- Quyết định mới: chỉ trừ thanh tác vụ hay trừ cả thanh tiêu đề; và theo ảnh Matrix9 không có dock phải, bạn có muốn giữ dock phải tùy chọn không.
- L2-01-02/03 vẫn BLOCKED_DEPENDENCY; không đổi `planning_state`.

### Việc tiếp theo

Bạn chạy checklist 5 mục và quyết định các điểm trên; sau đó làm OM9-L3-01-01-003 (xem trước kéo dock trực tiếp).
