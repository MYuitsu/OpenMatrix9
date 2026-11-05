# RAM sống của root selection trong lúc STL giữ capture — 2026-11-05

R1a, phần root selection: quản lý bộ nhớ set/order đang sống trên account chung để tác vụ STL thấy thay đổi sau capture. Không đổi geometry, profile/tolerance STL, thuật toán picking, hành vi Gumball hoặc native adapter; không thao tác phiên OM9 người dùng.

## Tái hiện và sửa

Fixture 101 object (một triangle mesh và 100 point): chụp STL với một root được chọn, rồi chọn 101 root. Trước sửa, phần selection theo công thức hạch toán tăng 7680 byte nhưng detached admission peer thấy tăng 0. Regression trong `selection-owner-20261105-062948/red-before-owner.log` ghi lỗi này; không nhận tăng 7680 là số đo allocator/RSS.

Root BTreeSet và order Vec nay có reservation CPU sống trên account hiện có. Candidate được admit trước khi tạo tree/order, giữ old/new overlap; plugin selection chuẩn bị owner trước transaction commit. Các đường replace, add/Shift, toggle/Ctrl, layer, prune/Delete, import-clear và clear cập nhật cùng vòng đời. Thêm ID dùng tree hiện có, chỉ thay Vec khi hết capacity, không clone toàn tree mỗi Shift-click. Guard nằm sau data trong thứ tự drop; candidate lỗi cũng thả data trước guard.

Sau publication, phần root selection không cộng lại vào `auxiliary_usage` hoặc capture-time held estimate. Worker thấy owner mới qua registry hiện có; không thêm ledger hoặc quét history mỗi frame. Bỏ chọn trả phần tree, giữ charge cho Vec capacity còn tồn tại; khi controller drop trả owner root. IDs riêng của capture STL tiếp tục có lease Export riêng và không giữ selection hiện hành sống thêm.

Phân loại CpuCache dùng cho trạng thái dẫn xuất CPU; root selection không đăng ký OptionalMemoryCache, nên pressure reclamation không được xóa selection người dùng. Vec tính theo capacity; BTree dùng upper allowance của `Usage::btree` cho Rust profile đã pin, không nhận là capacity allocator đo chính xác. Grow/shrink phase và physical/commit gates tái dùng core. Nếu allocator cung cấp capacity vượt request, code đối chiếu/grow trước publication hoặc bỏ candidate; đây không phải trần cứng mọi allocation/RSS.

## Kiểm thử và giới hạn

Bốn regression mới: hai fixture 8 point cho order/revisions/Shift/Ctrl/removal và low-budget atomicity; fixture plugin 2 point cho staged selection/Undo/Redo; fixture capture 101 object dùng hai luồng và channel barriers. Worker đo trước tăng, sau tăng, sau clear và sau controller drop; nguồn capture giữ IDs cũ. Refusal giữ selection, revision, document, Undo/Redo và owner cũ; pinned picking readers khiến test pressure không được cứu bằng eviction dữ liệu đang đọc.

Đợt đầu dùng nhầm class Document vốn không cho external reservation, regression báo Invalid; đã chuyển sang class CPU hợp lệ. Một oracle hover cũ chờ tổng CpuCache về baseline trước khi tạo selection nên thất bại khi selection có owner thật. Sửa oracle bằng đúng BTree allowance + one-ID Vec của fixture; giữ equality, timeout và stale-hit assertion. Không nới timeout, bỏ assertions hoặc gán completion của worker chỉ từ trạng thái command.

Windows x64, default features, OCCT 8.0.1 SDK/TBB/jemalloc; `E:\om9-surface-snap-validation`, `-j 2`, dev debug=0/incremental=0. Script `build/run-selection-owner-20261105.ps1` chạy theo nhóm, dừng khi fail/zero tests. Production cuối không đổi giữa các run dưới; sau run đầu chỉ tăng kiểm chứng STL thành hai luồng và sửa oracle hover.

| Nhóm | Passed / ignored |
|---|---:|
| Root memory / STL unit / plugin unit | 2 + 61 + 6 / 0 |
| Document / file / ordinary RAM | 12 + 20 + 9 / 6 |
| Replacement | 8 / 0 |
| Selection, layer, subobject contracts | 34 / 0 |
| B-rep picking / Gumball / overlay | 19 + 7 + 7 + 14 / 1 |
| Presentation / hover / scene memory | 13 + 14 + 7 / 0 |
| GPU offscreen publication | 29 / 0, không SKIP |
| Native STL integration | 60 / 0 |

**322 distinct passed, 7 ignored, 0 failed** trong các nhóm cuối; không cộng lại các lượt trùng. 6 large cases và 1 responsiveness case chưa chạy; không suy fixture count hoặc phần trăm hoàn tất từ test count. Không đo release/private commit/RSS/desktop mới, không chứng minh tốc độ Shift-click hay tổng chính xác STL/3DM. Native STL regression giữ profile/output invariants hiện có; không thêm chứng nhận sai số toàn miền. GPU offscreen không thay desktop.

Log theo phạm vi:

- `build/standalone-validation/ui-artifacts/selection-owner-20261105-062948` — earlier
- `build/standalone-validation/ui-artifacts/selection-owner-20261105-063339` — unit
- `build/standalone-validation/ui-artifacts/selection-owner-20261105-063443` — rest
- `build/standalone-validation/ui-artifacts/selection-owner-20261105-063836` — tail

Build/runtime elapsed của các nhóm cuối gồm build: root-memory 26,96 s, STL unit 1,69 s, plugin 0,79 s; lifecycle 29,32 s, replacement 5,21 s, selection/layers 17,00 s; picking/Gumball/overlay 40,97 s, presentation 0,95 s, GPU 14,97 s, native STL 115,95 s (runtime 111,83 s), hover 9,79 s, scene-memory 1,58 s. Summaries nằm trong từng log; elapsed không là số đo FPS hoặc tốc độ riêng của owner. Evidence run đầu của overlay đã fail được giữ, không gộp vào kết quả cuối.

## Bàn giao và còn mở

R1a vẫn còn subselection paths, workspace/path/recent/pending-command và các auxiliary khác, cùng legacy `scene_high_water`; không gọi root owner là giải xong toàn stale capture. R1b vẫn còn scratch/peer/journal/container/retired inventory. Native peak R2a-STL, UI worker, P1/v5, GPU admission/batching, độ chính xác STL/3DM, checkpoint và release/desktop vẫn mở. Không đo batch 3DM/openNURBS mới hoặc đóng exchange packages; tổng batch tương lai chưa xác định.

Plan/alias, README, live ledger và runtime owner records được cập nhật theo phạm vi root selection, không nâng toàn module. Đây là sửa kỹ thuật bảo toàn nghiệp vụ; không sửa skill. Tiếp theo: quản lý các auxiliary còn lại bằng owner sống, rồi bỏ high-water khi hết dữ liệu chưa đăng ký; không giao lại root selection.

Fingerprint SHA-256 nối repo path UTF-8, NUL, bytes file theo thứ tự dưới: `f8970f25fdfe8dcb75e90357a3ab39745c5b12c4b07373b2944bda582e344eca`. Không phải Git commit hay hash toàn checkout.

- `crates/om9-app/src/session.rs`
- `crates/om9-app/src/session/selection_memory.rs`
- `crates/om9-app/src/session/layers.rs`
- `crates/om9-app/src/session/plugins.rs`
- `crates/om9-app/src/session/contract.rs`
- `crates/om9-app/src/session/subobjects.rs`
- `crates/om9-app/src/session/files.rs`
- `crates/om9-app/src/session/stl.rs`
- `crates/om9-app/tests/selection_overlay.rs`

Public-source audit exit 0: 15.191 source files, 594 icon bindings, 593 SVG, errors rỗng. Fingerprint, terminal summaries, JSON ledger và 5 owner files, 40 local Markdown links đã kiểm tra; `git diff --check` trong scope đạt. Prune qua tool dự án sau kiểm tra resolved targets trong checkout đã bỏ 1 run không còn được dẫn, giữ 10 mới nhất mỗi root cùng các run được hồ sơ dẫn; không xóa target ngoài checkout. Log `build/standalone-validation/selection-owner-prune-20261105.log`. Không chạy lại Rust sau thay đổi chỉ ở hồ sơ. [Plan](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md).
