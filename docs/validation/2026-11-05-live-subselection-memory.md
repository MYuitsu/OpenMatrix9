# RAM sống của subselection và nested paths — 2026-11-05

R1a: root selection đã có owner ở lượt trước; lượt này thêm owner cho Vec mặt/cạnh/đỉnh và các path lồng nhau. Rust sở hữu bookkeeping, không sửa FFI, geometry/tolerance, topology validation, lệnh, Gumball hoặc phiên desktop người dùng.

## Lỗi và thay đổi

Fixture một instance hai cấp chứa triangle mesh, giữ STL capture trên worker. Chọn 5 subobjects với path dài 2 IDs nhưng cố ý dành capacity 4096 IDs/path. Trước sửa, 329.024 byte payload slots/path theo capacity không xuất hiện trong admission peer: observed growth = 0. Đây là fixture capacity có padding để bắt undercount, không phải tuyên bố 5 face bình thường tiêu thụ từng đó RAM hay đo RSS. Red log được giữ trong `build/standalone-validation/ui-artifacts/subselection-owner-20261105-065126/red-before-owner.log`; worker disconnect phụ trong log là hậu quả test chính panic sau assertion đỏ.

Controller nay giữ một CPU reservation cho subselection; Vec capacity và mỗi path capacity được tính một lần. Caller đã tạo input path trước API, nên phần đó được admit là allocated; chỉ slot mới là pending. Slot spare với path rỗng không tạo allocation/request mới. Khi cần thêm slots, dự trù capacity tăng theo cấp số nhân, giữ old/new slots cùng sống, move các path qua Vec mới mà không clone buffer. Candidate data được thả trước guard khi lỗi; grow failure giữ key/path pointer/revision/document/history cũ.

`ExternalMemoryReservation::transfer_allocated_from` chuyển riêng byte đã materialize giữa hai guard cùng account/class. Hàm không cấp phát, không đổi tổng account hoặc pending và không lấy mẫu mới; metadata pending/probe cũ của receiver được giữ, không biến nó thành bằng chứng mẫu mới. Pending bytes của donor không được chuyển như allocated; sai class/account/amount từ chối trước thay đổi. Việc đổi owner dữ liệu đã sống vẫn được phép khi ceiling đã bị siết, không coi đó là yêu cầu cấp phát mới.

Resize chuyển ownership của các path hiện hành trước publication; old guard còn đúng phần old slot buffer đến khi buffer đó bị thả. Không lặp tính tất cả path mỗi lần thêm vào spare slot: old path total lấy từ owner, chỉ sum lại khi removal/prune thật sự thay đổi dữ liệu. Toggle/clear/prune trả phần path bị bỏ; Vec capacity còn giữ tiếp tục được hạch toán. Drop controller trả owner, capture STL chỉ giữ nguồn/IDs của chính nó. Các khoản đã đăng ký được loại khỏi caller-held auxiliary, không thêm ledger thứ ba hoặc eviction xóa selection.

Input `SelectedSubobject::from_hit`/caller clones có thể cấp phát trước API này; lượt này chỉ quản lý dữ liệu được nhận và lưu, chưa đóng mọi query/input scratch R1b. Allocator capacity lớn hơn request được đối chiếu trước publication hoặc candidate bị bỏ, không chứng minh chặn mọi allocator/native/RSS peak. OS RAM/commit sample vẫn là ảnh chụp.

## Kiểm chứng

Windows x64, default Cargo features, OCCT 8.0.1 SDK/TBB/jemalloc; target `E:\om9-surface-snap-validation`, `-j 2`, dev debug=0/incremental=0. Hai run cuối:

- `build/standalone-validation/ui-artifacts/subselection-owner-20261105-065126`: core 325, gồm 2 test transfer mới.
- `build/standalone-validation/ui-artifacts/subselection-owner-20261105-065328`: app/native groups dưới; không chạy lặp core sau khi chỉ thêm app tests.

| Nhóm | Passed / ignored |
|---|---:|
| Core | 325 / 0 |
| Subselection ownership mới | 3 / 0 |
| STL unit / root selection / plugin | 61 + 2 + 6 / 0 |
| Subobject/selection/layer contracts | 19 / 0 |
| Document/file/ordinary RAM | 12 + 20 + 9 / 6 |
| Replacement | 8 / 0 |
| B-rep picking/Gumball/overlay | 19 + 7 + 7 + 14 / 1 |
| GPU offscreen publication | 29 / 0, không SKIP |
| STL integration dùng adapter thật | 60 / 0 |

**601 distinct passed, 7 ignored, 0 failed**. Hai core tests nằm trong 325; ba app fixtures cùng dựa trên nested triangle scene. Không coi test count là fixture count hay phần trăm hoàn thành. 6 large-memory cases và 1 responsiveness case chưa chạy. Không đo release/private commit/RSS/desktop mới, không có chứng nhận sai số STL toàn miền.

Oracle mới: worker quan sát tăng/remove/clear/drop theo channel barriers; 5 path pointers giữ nguyên qua slot resize; clear chỉ giữ slot bill. Probe mô phỏng Available đúng bằng reserve margin xác nhận path đã tạo không bị yêu cầu cấp physical RAM lần nữa khi dùng spare slots. Pressure fixture pin picking readers, từ chối growth giữ state/owner/pointers rồi retry thành công. Core checks tổng/pending/probe, transfer dưới ceiling đã siết và từ chối sai account/class hoặc lấn vào pending. Các reservations pressure không cấp phát payload tương ứng, không là thử OS OOM thật.

Elapsed của các nhóm cuối gồm build: core 21,70 s, subselection 25,08 s, STL unit 1,75 s, root selection 0,93 s, plugin 0,83 s, contracts 30,40 s, lifecycle 9,81 s, replacement 4,79 s, picking/Gumball/overlay 19,80 s, GPU 14,70 s, native STL 115,65 s (runtime 111,53 s). Không suy FPS hoặc tốc độ thao tác từ pointer-identity tests. GPU offscreen/synthetic input không thay desktop hay WPR. Không đo batch 3DM/openNURBS mới hoặc đóng exchange packages; tổng batch tương lai chưa xác định.

## Bàn giao và còn mở

Plan/alias, README, live ledger và runtime owners được cập nhật đúng phần đã kiểm chứng. Selection semantics giữ nguyên, không sửa skill nghiệp vụ. R1a còn workspace/path/recent/pending-command và các auxiliary khác, cùng legacy scene high-water. R1b input/query/peer/journal/container/retired accounting, R2 native peak, UI STL worker, P1/v5, GPU batching, độ chính xác STL/3DM, checkpoint và release/desktop vẫn mở.

Tiếp theo: owner sống cho auxiliary còn lại rồi đối chiếu/bỏ legacy high-water khi đủ bằng chứng. Không giao lại root/subselection hoặc primitive transfer đã kiểm chứng.

Source fingerprint SHA-256 nối repo path UTF-8, NUL, bytes file theo thứ tự dưới: `2f0f06baf4c205a9f0e57691ba7d228a40bcf83b5b27b86dc19181cd05e539e8` (không phải Git commit/toàn checkout).

- `crates/om9-core/src/transaction.rs`
- `crates/om9-app/src/session.rs`
- `crates/om9-app/src/session/layers.rs`
- `crates/om9-app/src/session/subobjects.rs`

Public-source audit exit 0: 15.192 source files, 594 icon bindings, 593 SVG, errors rỗng. Fingerprint, terminal summaries, 40 local Markdown links, JSON ledger và 5 owner files đã kiểm; `git diff --check` trong scope đạt. Prune qua tool dự án sau kiểm tra resolved targets trong checkout đã bỏ 1 run không còn được dẫn, giữ 10 mới nhất mỗi root cùng các run được hồ sơ dẫn; không xóa target ngoài checkout. Log `build/standalone-validation/subselection-owner-prune-20261105.log`. Không chạy lại Rust sau thay đổi chỉ ở hồ sơ. [Plan](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md).
