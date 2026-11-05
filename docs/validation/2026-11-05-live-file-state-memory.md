# FileState: bộ nhớ sống trên account chung — 2026-11-05

Phạm vi R1a.1 trước đây, đã đạt các kiểm tra runtime dưới đây: heap payload của `Controller::FileState` gồm active path, diagnostics và overwrite destination. Dùng reservation CPU hiện có; không thêm dependency/native code, không đổi file format, geometry, dung sai hoặc hành vi Save/Open. Phiên OM9 và bản vẽ đang mở giữ nguyên. Cập nhật kết quả từ log có sẵn ngày 2026-11-05; không chạy lại test khi sửa hồ sơ này.

## Thiếu sót và thay đổi

Trước sửa, file payload nằm trong `auxiliary_usage` tại thời điểm capture. Một detached peer giữ nguồn STL không thấy destination mới sau capture. [Red fixture](../../build/standalone-validation/ui-artifacts/file-memory-20261105-073852/red-before-owner.log) quan sát tăng **0 byte**, trong khi destination thật giữ **89 byte capacity**. Đây là thiếu hạch toán trên account dùng chung của fixture; không phải phép đo RSS hay nguyên nhân đã chứng minh của một lần OOM desktop.

- `FileState` giữ reservation sau các trường dữ liệu, nên path/diagnostics/operation drop trước lease. Bill dùng PathBuf capacity, Vec<String> capacity và capacity từng String. Inline operation/token/hash không là heap allocation riêng: bỏ phantom `FileOperation + 1024 byte` trước đây.
- Destination/path có owner trên account chung trước khi nhận phase hoặc chạy writer. Đường Save hạch toán path đã được cấp phát là allocated, không yêu cầu physical RAM cho cùng bytes lần nữa. Sau khi filesystem chấp nhận, chuyển owner cùng account/class sang FileState; không cần admission mới sau khi file đích đã được ghi.
- Khi cancel, từ chối ghi đè, Save thành công hoặc thay path, payload cũ được drop trước khi shrink bill. Lỗi writer/admission giữ path cũ, history và file đích; token retry giữ destination đã có thay vì take rồi cấp lại.
- Candidate Open/Snapshot nhận file owner **sau** adoption vào account cũ, tách từ reservation staging đã có. Loại file bytes khỏi phần candidate chuyển giao còn lại để không đếm hai lần. Candidate thất bại được thu hồi cùng owner; payload sống được tính đến Controller cuối giữ nó.
- `FileState::usage` chỉ trả phần chưa registered. Vì vậy path đã có owner không bị cộng lại trong capture-time auxiliary, còn worker/peer thấy tăng giảm qua account chung.

## Kiểm chứng

Năm test mới trong `session::files::file_memory_tests`:

1. Triangle scene, root đã chọn; capture nguồn STL rồi giữ trên thread khác. Channel barriers kiểm destination growth, từ chối overwrite, cancel và Controller drop; shared peer thấy đúng delta capacity, auxiliary không cộng hai lần, source epoch giữ nguyên và sentinel destination không đổi.
2. Writer callback thấy path đã có owner; trả Canceled rồi kiểm bill, file đích và Redo không đổi.
3. Pin picking readers và dùng reservation mô phỏng thiếu headroom; từ chối trước writer, giữ document/token/receipt/Redo/destination. Reservation gây áp lực không cấp phát payload tương ứng, không mô phỏng OOM thật của OS.
4. Save native rồi Snapshot Restore và Open; file owner khớp live payload, pending bằng 0, usage chưa registered bằng 0; Controller drop trả hết CPU file/scene owner trong fixture.
5. Candidate cùng đường publication với archive decoder, diagnostic có capacity 64 KiB dù nội dung ngắn; ownership transfer giữ capacity và drop trả bill. Đây là candidate fixture, không là file migration có warning thật.

Hồi quy cũ có bốn oracle trước đây chỉ cộng scene sau Save/Open. Ba oracle trong replacement và một trong presentation cần cộng thêm file payload đang sống. Red presentation: **2187 so với 2096 byte**, chênh **91 byte**. Test hiện lấy capacity của absolute input path và kiểm active path; không lấy delta counter làm expected. Các assertions về reader cũ, failed Open và geometry giữ nguyên. Không nới timeout hoặc bỏ equality để đạt xanh.

### Cấu hình và log

Windows x64, Cargo default features, OCCT 8.0.1 SDK/TBB/jemalloc, target `E:\om9-surface-snap-validation`, `-j 2`, dev debug=0/incremental=0. Đây là debug/offscreen regression, không là đo release/desktop.

- [File owner mới](../../build/standalone-validation/ui-artifacts/file-memory-20261105-073852/file-memory.log): 5 passed, 0 failed.
- Run `file-memory-20261105-074055`: replacement 8; lifecycle 12 + 20 + 9 passed, 6 ignored; STL unit 63; root 2; subselection 3. [Presentation đỏ được giữ](../../build/standalone-validation/ui-artifacts/file-memory-20261105-074055/presentation.log); run fail-fast này không có các nhóm phía sau.
- Run tiếp `file-memory-20261105-075042` đã có terminal summaries: [presentation](../../build/standalone-validation/ui-artifacts/file-memory-20261105-075042/presentation.log) **13 passed**; [hover](../../build/standalone-validation/ui-artifacts/file-memory-20261105-075042/hover.log) **14 passed**; [GPU](../../build/standalone-validation/ui-artifacts/file-memory-20261105-075042/gpu.log) **29 passed**, không SKIP; [picking/Gumball/overlay](../../build/standalone-validation/ui-artifacts/file-memory-20261105-075042/gumball.log) **19 + 7 + 7 + 14 passed, 1 ignored**; [native STL](../../build/standalone-validation/ui-artifacts/file-memory-20261105-075042/stl-integration.log) **60 passed, 0 failed**, runtime 121,41 giây.

Tổng các nhóm kết quả cuối trên: **285 distinct passed, 7 ignored, 0 failed**; không cộng retry hoặc các oracle đỏ đã thay thế. Sáu large-memory cases và một overlay responsiveness case vẫn ignored; không coi chúng đã được chạy. Các nhóm trước run cuối dùng cùng production implementation; thay đổi sau đó chỉ sửa oracle trong presentation test. Source fingerprint dưới vẫn khớp lúc cập nhật hồ sơ. Log đỏ được giữ để giải thích lịch sử, không còn là việc cần sửa lại.

## Phạm vi còn mở

FileState slice không đóng toàn R1a: workspace/recent/repeat/pending solid/layer dialog/contract auxiliary còn được chụp tại caller. UI modal text buffers, path-copy/canonicalization scratch, archive decoder scratch, cache/container/peer/journal/retired owners thuộc R1b/R2 theo plan. Không coi tất cả allocation của File UI hoặc Controller đã có owner, không gọi đây là đo toàn RSS/native peak.

Native OCCT pre-copy envelope, UI STL worker, P1/display cache/v5, GPU admission/batching, accuracy/topology/picking/3DM, checkpoint và release/desktop vẫn mở. Không có batch đo openNURBS/3DM mới, không đóng exchange packages; tổng batch tương lai chưa xác định. Tái dùng regression FileState đã đạt, không giao viết lại ownership này.

Source fingerprint SHA-256 nối path UTF-8, NUL và raw bytes theo thứ tự dưới: `6f45eb650b37fa05eaaddce178fb8cbcbac27a31c9abd97739118fc5b1ff84a2`.

- `crates/om9-app/src/file_lifecycle.rs`
- `crates/om9-app/src/session/files.rs`
- `crates/om9-app/src/gpu_viewport.rs`

Đây là fingerprint ba file của slice, không phải commit hoặc hash toàn checkout. Thay đổi hành vi hạch toán kỹ thuật giữ nghiệp vụ Save/Open; không sửa skill nghiệp vụ. [Checklist chính thức](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md).

## Đồng bộ plan chống loop

Theo yêu cầu người dùng ngày 2026-11-05, sửa cùng plan/alias/ledger/README: bỏ FileState/Open khỏi checklist còn làm, thêm quy tắc chống lặp và gom test theo mốc chức năng. Không tạo báo cáo rà soát plan mới, không sửa code/skill, không build hoặc chạy Rust/GPU/native/release/desktop mới. Goal triển khai vẫn paused; không nâng trạng thái hoàn tất của cả P0 hoặc milestone từ các kết quả slice này.

Thứ tự sau khi được tiếp tục là mốc A: auxiliary/native admission cần thiết của export và worker UI trong cùng một bàn giao dùng được; không chia thành batch hồi quy rộng cho từng allocation. Các gate chính xác, topology, picking, 3DM và an toàn dữ liệu không bị bỏ.

Kiểm tra tài liệu đạt: 47 local links, JSON ledger/current entry duy nhất, whitespace, các ràng buộc chống loop/độ chính xác và source fingerprint ba file còn khớp. Public source audit exit 0: 15.195 files, 594 icon bindings, 593 SVG, errors rỗng; scoped `git diff --check` đạt. Không có build/validation runtime batch mới cần prune trong lượt này. Milestone runtime statuses không được nâng từ việc sửa tài liệu.
