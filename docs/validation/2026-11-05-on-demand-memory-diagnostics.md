# RAM diagnostics theo yêu cầu — 2026-11-05

Phạm vi R1: projection chỉ đọc trên tài khoản RAM hiện có và MCP `om9_memory`. Không đổi quota/Undo, geometry, meshing/export, Gumball hay phiên OM9 đang mở. Nền tảng trước: [account roots/Weak](2026-11-05-account-root-and-weak-document-memory.md), [cache recency](2026-11-05-picking-cache-recency-qos.md). [Checklist còn lại](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md).

## Đường sử dụng và semantics

Tool `om9_memory` có trong catalog headless và catalog của proxy live. Cả hai gọi cùng `Controller`/shared account. New/Open vẫn theo cơ chế hiện hành; không tự tạo document để có report. Arguments là `{}` hoặc `{"additional_pending_bytes": 1048576}`. Chỉ số nguyên u64 không âm được nhận; unknown fields, null/string/fraction/negative bị từ chối. Tool không trả danh sách object, geometry, lịch sử recipe hoặc path file; không đính report vào mọi `om9_state` hoặc frame.

Report gồm configured/effective byte limit và learned 60% ceiling nếu đã có; bộ nhớ coordinator/document/history, external documents/controls/reservations và caller-held riêng. Ba khoản này tạo `tracked_total_bytes`; `external_classes` là chi tiết của khoản external đã tính, không cộng thêm lần nữa. Các class Document/Undo ở bảng external chỉ là reservation external (thường 0), không phải toàn bộ document/history bằng 0. Materialized/pending được tách; Undo policy 1024 MiB không phải allocation cấp sẵn. Undo inventory là cached estimate có growth/capture-change flag, không tự kiểm kê lại hay cắt lịch sử.

Physical và Windows system commit là hai probe/gate độc lập, mỗi gate có margin và headroom cho pending riêng. Trạng thái phân biệt `not_configured`, `failed`, `invalid`, `measured`; không biến probe lỗi thành đủ RAM. Sample age ghi microseconds từ lúc probe trả về đến lúc dựng phần report tương ứng; không bao gồm thời gian truyền/serialize sau đó. `collection_duration_us` ghi thời gian thu thập.

Request tùy chọn chỉ dự báo **new pending bytes** theo các dữ liệu đã biết. Ba kết quả `fits_known_managed_inventory`, `fits_physical_sample`, `fits_commit_sample` độc lập; null là chưa xác minh. Không cộng RAM trống với commit, không reserve, không áp trần mới từ query, không kiểm structural/transaction/native peak hoặc bảo đảm tác vụ sẽ thành công. Pending đang vượt headroom vẫn được báo không vừa ngay cả khi request mới là 0. Overflow hoặc caller inventory thất bại làm totals/assessment phụ thuộc thành unknown, không bịa số 0 hoặc UINT64_MAX làm bộ nhớ thật.

API core dùng lại storage-only projection thay vì validation/admission. Giữ thứ tự locks external-documents → account; không tessellate, tạo index validation, refresh/trim Undo, thu hồi cache, xóa expired proofs hay thay tài liệu. Khi inventory cần tính một expired proof còn được giữ, tính đúng fixed Arc block mà không dọn nó. Validation bình thường vẫn dọn expired proof như bàn giao trước. Aggregate limits được nới chỉ trong phép quan sát để đọc được overage khi policy đã siết; policy/admission thật không bị nới. Overflow vẫn kiểm tra.

`not_measured` ghi rõ native allocator/peak, process RSS/private commit, unregistered auxiliary/peer/retired owners và GPU residency/driver. Known tracked bytes không phải toàn bộ RAM tiến trình. Report hiện có storage walk đồng bộ khi yêu cầu; không gọi mỗi frame hoặc tự ghi log. Không nhận đây là bằng chứng nonblocking UI trên model lớn; đường live hiện xử lý yêu cầu trên UI owner thread và cần đo/giới hạn hoặc chuyển worker trước khi đóng gate đó.

## Kiểm chứng

Windows x64, default features, `CARGO_TARGET_DIR=E:\om9-surface-snap-validation`, Cargo `-j 2`, dev debug=0/incremental=0, OCCT 8.0.1 SDK/TBB/jemalloc runtime như trước. Không khởi động hoặc đổi binary phiên desktop hiện tại.

Sáu test core mới kiểm:

1. Reserved 5000/allocated 2000/pending 3000 và caller-held 100 tạo đúng tổng; quota Undo không cộng 1 GiB. Physical và commit mỗi bên chỉ còn 1000 bytes: request 1000 vừa, 1500 không vừa ở cả hai, không cộng thành 2000.
2. Probe thiếu/cấu hình lỗi/invalid không báo verified headroom.
3. Policy bị siết dưới live usage vẫn đọc được overage; callbacks reclaim được đặt panic nếu gọi; source/history, account ceiling và expired proof giữ nguyên.
4. Storage projection khớp admission trên nguồn có Undo và tracked capture; cached Undo inventory không refresh.
5. Caller inventory lỗi hoặc cộng overflow trả unknown; request UINT64_MAX không wrap thành vừa.
6. Tuổi mẫu nằm trong thời gian thu thập; pending cũ vượt physical/commit headroom không bị che bởi request 0.

Fixtures core: năm empty World/account setups và một canonical Document có một Point; captures/history là owner/version bổ sung. Hai MCP tests mới dùng một document Point và một document rỗng, kiểm catalog/JSON nhỏ hơn 8192 bytes, error/unknown arguments và state trước/sau bằng nhau. Live-controller test dùng placeholder như host thật và một document đang nhận Point: query không đổi command prompt, world/history hoặc trạng thái đang chạy. Đây là routing trực tiếp Controller, chưa phải thao tác cửa sổ desktop/OS focus.

Lượt đầu core 6 và MCP 2 pass, live test fail do fixture Server chưa khởi tạo placeholder mà `handle_live` yêu cầu. Đã sửa fixture theo setup hiện có; không gọi đó là lỗi trên phiên người dùng hoặc che bằng bỏ test.

Batch cuối `build/standalone-validation/ui-artifacts/memory-diagnostics-20261105-051147` exit 0, **452 distinct passed, 0 ignored/failed**, không GPU SKIP:

| Nhóm | Passed | Elapsed gồm build (giây) |
|---|---:|---:|
| Core đầy đủ, đã gồm 6 diagnostics tests | 318 | 7,10 (lượt 6 focused riêng 15,33) |
| Scene integration: admission/managed picking/shared/broad phase | 32 | 20,17 |
| MCP stdio đầy đủ, đã gồm 2 tests mới | 6 | 0,99 (lượt 2 focused riêng 44,76) |
| Live-controller memory test | 1 | 26,01 |
| Replacement budget | 8 | 4,62 |
| STL phases/session unit | 58 | 1,67 |
| GPU offscreen | 29 | 14,39 |

Không cộng lại 6/2 focused tests thành test riêng. Không chạy lại toàn native STL accuracy matrix hoặc large RAM batch vì thay đổi chỉ thêm diagnostics và tách storage helper dùng chung, không đổi mesher/export. Những gate đó vẫn theo phạm vi bằng chứng đã có, không được nhận là nghiệm thu mới ở lượt này.

CLI acceptance `build/standalone-validation/ui-artifacts/memory-diagnostics-cli-20261105-051556`: build default dev binary tại target riêng (`-j 2`), process `om9.exe --mcp` exit 0, 9 JSON-RPC replies, một Point fixture, typed state trước/sau bằng nhau. `memory.json` và request/response JSONL được giữ trong run. Hai lần thử cùng fixture definition: script đầu dùng Int32 để lookup request ID Int64 nên assertion đọc nhầm null; response thực đã có Point. Đã sửa khóa script sang string và chạy lại; source/binary sản phẩm không đổi.

Số quan sát thực từ lượt CLI thành công:

| Khoản | Byte |
|---|---:|
| RAM vật lý OS báo | 68.635.316.224 |
| Learned ceiling `floor(3 × total / 5)` | 41.181.189.734 |
| Tracked tổng / materialized / pending | 58.015 / 57.639 / 376 |
| Coordinator retained / external aggregate / caller-held | 52.095 / 2.365 / 3.555 |
| Physical Available / dự phòng | 24.740.933.632 / 6.863.531.622 |
| Commit limit / used / dự phòng | 77.023.924.224 / 49.417.502.720 / 268.435.456 |

Physical sample age 114 µs, commit age 46 µs, collection 118 µs trên **fixture một Point**. Undo policy 1024 MiB, hardware cap 32.727 MiB. Đây không phải RAM/RSS của cả app hoặc benchmark model thật, latency desktop hay FPS. Không triển khai binary vào phiên OM9 đang mở và chưa kiểm client connector đang chạy nhận tool mới.

Fingerprint source SHA-256 `7f4b4d7c7e5fae648ec2fc08958867f100e017174f8bf7ac7924107b18948c46`: nối lần lượt UTF-8 path + NUL + raw bytes của bảy file dưới đây; đây không phải Git commit hoặc fingerprint toàn workspace:

```text
crates/om9-core/src/transaction.rs
crates/om9-core/src/transaction/memory_diagnostics.rs
crates/om9-app/src/mcp.rs
crates/om9-app/src/mcp/state.rs
crates/om9-app/src/mcp/tools.rs
crates/om9-app/src/mcp/live.rs
crates/om9-app/tests/mcp_stdio.rs
```

Binary dev được chạy: `E:\om9-surface-snap-validation\debug\om9.exe`, SHA-256 `ded81c7172d51a31cc6872b645a716c85d7891d33f944953a6269dc55e8e339c` tại thời điểm acceptance. Binary target có thể được build lượt sau; hash này là receipt lịch sử, không nói bản đang mở đã được thay. Plan/alias, README, live ledger và runtime owner M01 L2-01-02/03/06 đã đồng bộ phạm vi. Audit/JSON/links và prune chốt bên dưới; không nâng toàn R1 thành complete.

`tools/public_source_audit.py` exit 0: 15.184 source files, 594 icon bindings, 593 SVG, errors rỗng. JSON, source/binary fingerprints, terminal/CLI receipts và 50 local Markdown links đã kiểm tra. `git diff --check` không lỗi whitespace trong scope, chỉ cảnh báo CRLF/LF sẵn có. Prune qua `tools/prune_build_artifacts.py --apply` sau kiểm tra đích trong checkout, giữ 10 run mới nhất mỗi root cùng các run được dẫn; không xóa target ngoài checkout. Không chạy lại Rust sau thay đổi chỉ ở hồ sơ.

## Việc còn lại

- Gắn actual failed-request context/codec bounds vào báo lỗi, attribution Document/Undo/owner còn thiếu, tuổi cached Undo inventory chưa đo; report dự báo không phải bản ghi nguyên nhân của một lỗi đã xảy ra.
- Normal UI messages ngắn/phân biệt nguyên nhân, log opt-in có giới hạn và đường diagnostics không block UI trên model lớn chưa đóng. Không tự log toàn report mỗi gesture/frame.
- Audit auxiliary/peer/retired/query/path owners và historical peak, native envelope R2, worker UI STL P3b, display/v5/GPU/accuracy/checkpoint và release/desktop vẫn mở.
- Không có batch đo openNURBS/3DM mới; không đóng package exchange. Large RAM/release/WPR chưa chạy lượt này; tổng batch tương lai chưa xác định.

Không thay đổi skill nghiệp vụ hoặc đánh dấu toàn R1/P1–P5 hoàn thành.
