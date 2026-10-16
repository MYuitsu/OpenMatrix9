# Báo cáo cuối: OM9-L3-01-01-002 Workbench Docking Regions

## Diễn giải yêu cầu
"M01 - L2-01 L3-02" = milestone M01, L2-01-01 (Febe Application Shell Docking), L3-002 (Workbench Docking Regions) = OM9-L3-01-01-002. Đây là mục khớp duy nhất.

## Các L4 đã hoàn thành (có bằng chứng, chạy trong `standalone`)
- L4-01 Source audit: `implementation/design/M01/OM9-L3-01-01-002/source_crosswalk.json`. QymCAD ghim `746a17f1...`, sạch (`git status --porcelain` rỗng). Không chép mã, không phụ thuộc QymCAD.
- L4-02 Hợp đồng API và thiết kế test: `api.md`, `positive.json`, `negative.json`. RED trước khi code: `cargo test -p om9-app --lib workbench_regions` và `--test workbench_regions` thoát 101 (lib 3 đạt 18 lỗi; tích hợp 1 đạt 6 lỗi), lỗi assertion, không lỗi build.
- L4-03 Triển khai (Rust an toàn, không ngoại lệ native):
  - `cargo test -p om9-app --lib workbench_regions`: 21 đạt, 0 lỗi
  - `cargo test -p om9-app --test workbench_regions`: 8 đạt, 0 lỗi
  - `cargo test -p om9-app --test shell_docking`: 10 đạt (test cũ không sửa)
  - `cargo test -p om9-app` và `cargo test --workspace`: exit 0
  - `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`: exit 0
- L4-05 Nghiệm thu tự động: `om9.exe --ui-audit` chạy hai lần, exit 0, 21 pass (RTX 3060, Vulkan). 1600x1000: rail trái, 4 viewport, dock phải. 800x600: dock phải nhường (`RightDockYielded/NarrowWindow`), ô nhập lệnh và 4 viewport được giữ, không chồng lấn. Ảnh: `build/standalone-validation/ui-artifacts/2026-10-16-m01-l3-002-docking/`.
- Review: APPROVED (`review.json`).

## Tệp tạo hoặc thay đổi
- Mới: `standalone/crates/om9-app/src/workbench_regions.rs`, `standalone/crates/om9-app/tests/workbench_regions.rs`, `docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md`, `implementation/{design,fixtures,evidence}/M01/OM9-L3-01-01-002/*`, `.agents/tasks/m01-l3-002-docking/*`.
- Sửa: `standalone/crates/om9-app/src/ui.rs` (kế hoạch vùng mỗi khung hình, biến audit `OM9_UI_AUDIT_WINDOW`, `OM9_UI_AUDIT_RIGHT_DOCK`), `src/ui/chrome.rs` (`sync_panel_size` không ghi khi đang có lỗi vùng), và `docs/openmatrix9-progress.json` (thêm `final_reconciliation` vào mục `m01-l3-01-01-002-...`).
- Không đổi `planning_state` trong `l3.json` (giữ `SPEC_ONLY_NOT_IMPLEMENTED`) và bốn `l4.json` (giữ `NOT_STARTED`). Quy ước repo không có giá trị "hoàn thành", cổng M00 là NOT_PASS, nên không có cơ sở nâng trạng thái. Không commit, không stage, không sửa skill.

## Kiểm toán nguồn công khai
`python tools/public_source_audit.py` chạy lại: exit 1, 11721 lỗi, số lượng không đổi. Tất cả nằm trong `ref/`, `manuals/`, `OM9_Milestones/`, `QymCAD/`, `vendor-research/`, `om9-window.png`. Không có lỗi ở đường dẫn của mục này. Không sửa vì AGENTS.md cấm đụng tham chiếu riêng tư. Không tuyên bố là đã đạt.

## Chưa được xác minh
- Cổng M01-UI đang MỞ: danh sách 7 mục chờ người kiểm tra trên Windows (gồm 150% DPI và màn hình thứ hai). Vị trí: `.agents/tasks/m01-l3-002-docking/verification.md` (mục "L4-05 manual checklist") và `implementation/evidence/M01/OM9-L3-01-01-002/OM9-L4-01-01-002-05.ui-review.json` (`reviewer.status = NOT_SIGNED`).
- Quyền tái sử dụng pháp lý: PENDING_HUMAN_M00_APPROVAL.
- OM9-L2-01-02 và OM9-L2-01-03 chưa triển khai: `CommandInputHost` và `ViewportSceneHost` chỉ được test với stub; `command_input_ready` luôn trả true (BLOCKED_DEPENDENCY).
- `tools/validate_m01_*` và `OpenMatrix9_Codex_Spec_v2/` không có, định nghĩa `om9.m01.oracle.v1.5` không có; file `*.evidence.json` theo runbook chưa ghi được.
- Chữ trên thanh trạng thái tự chồng lên nhau ở 800x600 (có từ trước, thuộc L3-007).

## Thay đổi quy tắc nghiệp vụ: cần sự đồng ý của bạn
Trước và sau:
1. Cửa sổ dưới 560x286 (320x286 khi không có rail trái). Trước: tự kẹp (clamp) âm thầm. Sau: hiện lỗi "Layout:" trên thanh trạng thái và giữ kế hoạch tốt gần nhất.
2. Dòng lệnh bị đánh dấu ẩn. Trước: có thể mất. Sau: vẫn được giữ chỗ, kèm thông báo `CommandInputKept`.
3. Khi đang có lỗi vùng. Trước: layout lưu có thể bị ghi lại. Sau: đồng bộ kích thước panel và kẹp menu nổi không ghi lại layout.
4. Thông báo dock phải nhường hoặc rail trái bị thu hẹp. Trước: không có. Sau: ghi vào lịch sử CMD, và vào thanh trạng thái nếu không có lỗi đang hiện.

Đề xuất nội dung skill (thêm vào `skills/openmatrix9-workspace-contract/SKILL.md`):
> Vùng workbench: rail trái, bốn viewport và ô nhập lệnh luôn được giữ. Dock phải nhường khi cửa sổ hẹp hơn khoảng 900 px. Cửa sổ nhỏ hơn mức tối thiểu hiển thị lỗi "Layout:", giữ kế hoạch cuối và không ghi lại layout. Dòng lệnh không thể bị mất bằng cách ẩn.

Bạn có muốn ghi các thay đổi này vào skill không? Tôi chưa sửa file skill nào.

## Hành động tiếp theo
Người kiểm tra chạy checklist M01-UI và ký `OM9-L4-01-01-002-05.ui-review.json`, đồng thời quyết định phê duyệt pháp lý M00. Sau đó làm L3-003 (live drag preview) của OM9-L2-01-01.
