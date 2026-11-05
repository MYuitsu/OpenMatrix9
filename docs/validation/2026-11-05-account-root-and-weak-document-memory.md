# RAM metadata: shared account roots, startup và expired document Weak — 2026-11-05

Phạm vi R1: Rust coordinator/account inventory và preflight khởi tạo. Không sửa native adapter, mesher, profile xuất, input/Gumball hay bản vẽ người dùng. [Bàn giao cache trước](2026-11-05-picking-cache-recency-qos.md); [plan còn lại](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md).

## Thiếu sót đã tái hiện

Run đỏ `build/standalone-validation/ui-artifacts/account-roots-20261105-045027`: 101 passed, 3 failed, exit 101. Ba regression mới xác nhận:

1. Metadata tài khoản chỉ tính generation/empty ledger và dynamic indexes, chưa đủ Arc bodies/counters của reservation account, document-index mutex và export gate.
2. World/identity scratch vừa hạn mức nhưng fixed coordinator/account roots không vừa vẫn có thể khởi tạo thành công. Fixture đã chừa world validation scratch; không dùng một limit quá nhỏ vốn đã thất bại trước đoạn thiếu kiểm tra.
3. `Weak<Document>` đã hết strong owner vẫn giữ allocation chứa `Document` và Arc counters, nhưng inventory bỏ qua khi `upgrade()` trả None. Hai Weak trỏ cùng allocation không được tính hai lần. Validation proof hết hạn có thể giữ cùng khối dù không còn dùng được.

Trên Windows x64 hiện tại, khối expired Document được oracle độc lập tính `size_of::<Document>() + 2*sizeof(usize) = 592 byte`; payload heap của Document đã drop và không được tính lại. Đây là lỗi kiểm kê metadata, không phải chứng cứ nguồn hình học đã mất hoặc OCCT chiếm sai một lượng RAM cụ thể.

## Sửa và vòng đời

`ExternalReservations::fixed_metadata_bytes()` liệt kê năm allocation dùng chung: account mutex/body, document-index mutex/Vec body, export atomic, capture-generation atomic và empty peer identity ledger. Phần lớn Arc tiếp tục dùng allowance bốn machine words của inventory hiện có; ledger dùng hai. Đây là layout estimate có phần dự phòng, không tính allocator/RSS overhead. Vec capacity của indexes và lease bodies vẫn được tính riêng; peer/handle clone không tạo khoản fixed root mới trên cùng account.

Khởi tạo coordinator admit fixed Self, account roots và initial identity ledger cùng World/validation scratch trước khi tạo các allocation quản lý đó. World/structural/overflow checks giữ nguyên; không tăng trần bytes hoặc bỏ validation để startup vừa.

External document inventory nay upgrade một lần cho từng Weak identity: upgrade được thì giữ Arc qua lượt tính storage; hết strong owner thì tính fixed Document block. Không có khoảng giữa hai lần kiểm strong/upgrade khiến cả hai nhánh đều bị bỏ qua. Weak trùng địa chỉ bỏ qua trước khi tính nguồn/metadata. Expired validation proof được dọn ngay trong inventory, thay vì giữ lại một block không dùng hoặc tính một peak mà lượt validate sẽ xóa lập tức. Metadata không bị báo đã free chỉ vì geometry payload đã drop.

Một oracle cũ được sửa từ “thả capture trả toàn bộ Document storage” thành equality: payload được trả là `old_storage - size_of::<Document>()`; external Weak vẫn giữ body/header. Các checks chia sẻ B-rep/resource/Undo được giữ. Không thay bằng một bất đẳng thức rộng để che sai số.

Implementation trung gian còn tính expired validation proof ngay trước khi validate ghi đè nó, làm inventory giữ thêm một khối đã trả ở cuối lượt; regression chia sẻ source đã bắt lỗi này. Đã đổi sang dọn proof hết hạn trước khi trả inventory. Lượt xanh cuối dùng sửa này cùng oracle payload/body chính xác, không chỉ sửa ngưỡng test.

## Kiểm chứng

Windows x64, Cargo default features, `CARGO_TARGET_DIR=E:\om9-surface-snap-validation`, `-j 2`, dev debug=0/incremental=0. OCCT 8.0.1 SDK/TBB/jemalloc như các lượt trước; không build/chạy phiên desktop OM9 mới.

Ba regression mới: hai empty World/account setups và một canonical Document fixture (không geometry payload); không gọi ba tests là ba measurement batches CAD. Toàn bộ fixture count của các suites hồi quy chưa được kiểm kê nên không suy số model từ tổng test.

Kết quả cuối trên production source không đổi giữa hai lượt:

- `build/standalone-validation/ui-artifacts/account-roots-20261105-045311`: core retained 104 passed; toàn core 312 passed (đã gồm 104), scene 55, presentation 10, hover 14 và scene-memory 7 passed. Harness sau đó lọc nhầm hai nhóm ra 0 tests rồi dừng vì một target sai tên; toàn run exit 101, không ghi là run hoàn chỉnh pass. Hai nhóm 0 test bị loại khỏi bằng chứng.
- Đã sửa tên đúng từ source/harness trước, thêm guard không chấp nhận 0 executed tests; chạy phần còn lại tại `build/standalone-validation/ui-artifacts/account-roots-20261105-045540`, exit 0. Replacement 8, STL unit 58, document/file/ordinary RAM 41, GPU 29, picking/Gumball/overlay 47 và STL integration 60 passed.

**641 distinct passed, 7 ignored, không còn test failure trong các nhóm cuối được chọn.** Sáu large RAM cases và một overlay responsiveness measurement vẫn ignored; không cộng 104 retained lặp trong toàn core thành test riêng. GPU log không có SKIP.

| Nhóm | Passed / ignored | Elapsed gồm build (giây) |
|---|---:|---:|
| Toàn core (104 retained nằm trong 312) | 312 / 0 | 7,13; lượt retained riêng 21,14 |
| Scene unit và bốn integration groups | 55 / 0 | 21,45 |
| Presentation | 10 / 0 | 49,30 |
| Hover | 14 / 0 | 9,75 |
| Incremental scene memory | 7 / 0 | 1,58 |
| Replacement budgets | 8 / 0 | 4,96 |
| STL unit | 58 / 0 | 1,73 |
| Document/file/ordinary RAM | 41 / 6 | 28,92 |
| GPU offscreen | 29 / 0 | 14,88 |
| Picking/Gumball/overlay | 47 / 1 | 19,93 |
| STL integration qua adapter cấu hình | 60 / 0 | 114,91 |

Fingerprint SHA-256 `ebbefa6b709c1291988ec15d1f0924afb9dea69c5316e5ad6df176c0e4ee62b2`: UTF-8 `crates/om9-core/src/transaction.rs` + NUL + raw file bytes. Đây là source scope của thay đổi lượt này, không phải Git commit hoặc fingerprint toàn workspace. Các native/runtime assumptions giữ nguyên như trên.

Plan/alias, README, live ledger và runtime owner M01 L2-01-02/03/06 được đồng bộ theo bằng chứng. `tools/public_source_audit.py` exit 0: 15.182 source files, 594 icon bindings, 593 SVG, errors rỗng. JSON, fingerprint, summaries và 46 local Markdown links đã kiểm tra. `git diff --check` không lỗi whitespace trong scope (chỉ cảnh báo CRLF/LF sẵn có). Prune qua `tools/prune_build_artifacts.py --apply` sau xác minh đích trong checkout: xóa 4 run không còn được dẫn, giữ 10 mới nhất mỗi root và các run được hồ sơ dẫn, không xóa target ngoài checkout. Không chạy lại Rust sau thay đổi chỉ ở hồ sơ; không nâng R1/P1–P5 thành hoàn thành.

## Giới hạn và bước tiếp

- Fixed roots của account hiện hành được charge một lần; đây chưa phải audit tất cả coordinator/worker stack, retired account handles còn sống, peer/journal owners và mọi auxiliary/path/query scratch.
- Weak ngoài các registries/proof được quản lý vẫn nằm ngoài phạm vi inventory này. Metadata estimates và tổng managed không phải RSS, private commit hay native allocation certificate; OS Available/commit vẫn là các gate độc lập.
- Không xử lý bằng cách nới 60% RAM, cắt Undo/Redo để cứu giao dịch thất bại hoặc giảm độ chính xác hình học. Native peak trước Rust callbacks, shared UMA/GPU fence/batching và v5 vẫn mở.
- Chưa có release/model lớn/desktop/WPR mới và không chạy lại large RAM cases tại bước này. Chưa có batch đo openNURBS/3DM mới; các gói exchange không đóng, tổng batch tương lai chưa xác định.

Tiếp theo: diagnostics phân biệt managed/Undo/physical/commit/codec cùng attribution của owner còn thiếu; tiếp tục audit peak cũ/auxiliary trước native envelope R2 và UI STL worker. Toàn R1/P1–P5 vẫn mở.
