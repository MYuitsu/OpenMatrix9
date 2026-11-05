# RAM: provenance của external reservation bị từ chối — 2026-11-05

Tiếp R1 sau [báo cáo RAM theo yêu cầu](2026-11-05-on-demand-memory-diagnostics.md). Phạm vi: các entrypoint `reserve_external_memory*` và phần tăng thêm của `grow_external_memory_with_held`, qua wrapper production dùng chung. Không đổi geometry, quota Undo, mesher, file format hoặc phiên OM9 người dùng. [Plan](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md) vẫn giữ các cổng còn thiếu.

## Bằng chứng lịch sử khác forecast

`om9_memory` thêm `last_external_refusal` cùng authority/epoch của coordinator đang đọc. Event chứa stage/class, error kind/context thật, requested tổng/allocated/pending, caller-held, prior external-reserved/pending, configured/learned/effective limit và headroom physical/commit đã dùng tại attempt bị từ chối. Counter/operand lấy trong chính account lock của attempt; không lấy mẫu lại để dựng câu chuyện sau lỗi. Probe chưa được áp dụng/chưa đến lượt/lỗi được phân biệt bằng flags và headroom null. Không tính global retained total tại failure bằng cách quét lại nguồn; không bịa số đó từ samples mới.

Chỉ có một event trên account, nội dung là static strings, IDs và số liệu; không giữ Document/shape/Undo snapshot, không thêm heap log/ring buffer hay file I/O. Fixed body mới thuộc inventory `size_of::<ExternalReservations>` đã có, kể cả khi event là None; peer dùng chung không tính một lần nữa.

Mỗi public reservation có thể thử lại sau eviction. Các attempt thất bại nội bộ chỉ ở stack; nếu public call cuối thành công thì chúng không được công bố thành một refusal mới. Nếu call cuối thất bại, chỉ lưu event của final attempt có cùng error kind/context. Một lỗi callback reclaim khác không được thay bằng operands của admission lỗi trước. Historical event cũ được giữ qua các request thành công; nó không có nghĩa lệnh CAD hiện tại thất bại. Fallback sang một request nhỏ hơn có thể thành công sau khi một public reservation lớn bị từ chối; event lớn vẫn là một refusal thật, không phải verdict của toàn export/command.

`Instant` được ghi khi attempt thất bại; khi các caller công bố kết quả khác thứ tự thực thi, event có timestamp cũ hơn không ghi đè event mới hơn. Age tăng từ event thật, độc lập tuổi probe mới trong `om9_memory`. Authority/epoch của caller tại event cũng độc lập coordinator hiện hành; New/Open/worker có thể dùng cùng account nên không gắn event cũ thành lỗi của document mới. IDs này chỉ ở runtime, không vào native file.

`last_external_memory_refusal()` là API đọc rẻ: một account lock và clone metadata cố định, không walk storage/OS probe/eviction. Full `om9_memory` vẫn có walk storage theo yêu cầu như report trước; gate nonblocking UI trên model lớn chưa đóng. Event giữ riêng dữ liệu lịch sử và report physical/commit mới. Không tự phát log mỗi frame hoặc đổi thông báo UI trong lượt này.

## Kiểm chứng

Windows x64, default Cargo features, target riêng `E:\om9-surface-snap-validation`, `-j 2`, dev debug=0/incremental=0. OCCT 8.0.1 SDK/TBB/jemalloc như trước; không đổi native adapter.

Năm unit tests mới dùng năm empty World/account setups, không phải năm model CAD hoặc batch openNURBS:

1. Physical headroom thực tại attempt là 123 bytes, request 124 bị từ chối; commit chưa được gọi. Đọc cheap API sau đó đặt probes panic vẫn thành công và source/history không đổi.
2. Commit headroom 123/request 124 từ chối; sau đổi probe sang dư và cấp request khác thành công, historical event vẫn giữ 123, trong khi full report có sample mới lớn hơn.
3. Grow lease dưới managed ceiling thất bại giữ nguyên bytes/pending của lease; công bố lại event cũ sau event mới không đổi class/operands của event mới.
4. Probe configured nhưng unavailable/invalid không thành verified headroom, error kind giữ đúng.
5. Class/bytes/allocated request không hợp lệ không tạo event giả là áp lực RAM.

Bổ sung oracle vào test pressure cold/warm picking hiện có: request thành công sau thu hồi quota/arena vẫn không có historical refusal mới. Những assertions này dùng đường production, không xóa event để test xanh. Các test diagnostic/MCP hiện có kiểm serialization/routing/read-only; source module mới không thêm command hoặc mutation API.

Batch cuối `build/standalone-validation/ui-artifacts/memory-refusal-20261105-052819` exit 0: **440 distinct passed, 0 ignored/failed**, GPU không có SKIP. Không cộng lại 5 refusal và 6 diagnostics focused vào toàn core 323:

| Nhóm | Passed | Elapsed gồm build (giây) |
|---|---:|---:|
| Core đầy đủ (gồm 5 refusal + 6 diagnostics) | 323 | 7,23; focused 15,33 và 0,18 |
| Managed picking pressure/lifetime | 15 | 16,63 |
| MCP stdio | 6 | 44,87 |
| Live-controller memory | 1 | 26,44 |
| STL phase/session unit | 58 | 1,70 |
| Replacement budget | 8 | 4,60 |
| GPU offscreen | 29 | 14,90 |

Binary headless acceptance `build/standalone-validation/ui-artifacts/memory-diagnostics-cli-20261105-053141`: build dev default `-j 2`, `om9.exe --mcp` exit 0, 9 replies, một Point fixture, state trước/sau bằng nhau. JSON có additive coordinator fields và `last_external_refusal: null` khi không có refusal được entrypoint này ghi. Windows physical/commit measured; total physical 68.635.316.224, learned ceiling 41.181.189.734, tracked bytes 58.263, pending 376. Đây là smoke serialization/routing với probes thật; các refusal thật được kiểm bằng core tests đã nêu, không ép máy thiếu RAM hoặc kết luận native/RSS từ 58.263 bytes.

Source SHA-256 `6fe63b8fc98a7ff1f0461a84ae572cb02caa1a24031e61282537e060be567cf4`: nối theo thứ tự UTF-8 path + NUL + raw bytes của năm file sau; không phải Git commit hoặc fingerprint toàn workspace:

```text
crates/om9-core/src/transaction.rs
crates/om9-core/src/transaction/memory_refusal.rs
crates/om9-core/src/transaction/memory_diagnostics.rs
crates/om9-scene/tests/managed_picking_cache.rs
crates/om9-app/src/mcp/tools.rs
```

Binary dev đã chạy `E:\om9-surface-snap-validation\debug\om9.exe`, SHA-256 tại acceptance `4d26d8722d2ac0c01381f59c198f7c430341828ce48f43de87ba4256b2ce54ff`; target có thể được build tiếp, không phải binary đóng băng hoặc bản đang mở đã thay. Plan/alias, README, live ledger và runtime owner M01 L2-01-02/03/06 đồng bộ phạm vi. Audit/JSON/links và prune chốt bên dưới. Không chạy release/desktop hoặc native STL accuracy matrix mới cho thay đổi metadata này.

`tools/public_source_audit.py` exit 0: 15.186 source files, 594 icon bindings, 593 SVG, errors rỗng. JSON, source/binary fingerprints, terminal/CLI receipts và 52 local Markdown links đã kiểm tra. `git diff --check` không lỗi whitespace trong scope, chỉ cảnh báo CRLF/LF sẵn có. Prune qua `tools/prune_build_artifacts.py --apply` sau kiểm tra đích trong checkout, giữ 10 run mới nhất mỗi root cùng các run được dẫn, không xóa target ngoài checkout. Không chạy lại Rust sau thay đổi chỉ ở hồ sơ.

## Phần chưa bao phủ

- Không bắt mọi failure của retained prepare/publication, `reserve_remaining_external_memory*`, lazy reservation recheck, allocator/native/codec, registry index growth hoặc OS OOM. Null chỉ là chưa có final refusal được entrypoint này ghi trong account, không phải chứng minh không từng có lỗi.
- Scope/class không phải tên đầy đủ lệnh STL/Save/kernel hoặc canonical source receipt. Chưa có source/job/codec provenance tổng quát, attribution owner đầy đủ hoặc Undo inventory age.
- Không kết luận lỗi Budget `bytes` luôn là 60% RAM; giữ nguyên context và stage, không thay bằng nguyên nhân global đoán. Các structural constraints/overflow vẫn hoạt động.
- Không đo RSS/native peak, release/model lớn/desktop/WPR mới. Không cập nhật binary trên phiên người dùng; `om9_memory` của phiên cũ chưa có field này.
- Không có batch đo openNURBS/3DM mới; package exchange và P1–P5 vẫn mở; tổng batch tương lai chưa xác định.

Bước tiếp: mở rộng provenance đúng caller/codec còn thiếu và attribution/ownership R1, rồi native envelope R2 và UI STL worker; dùng cheap API khi cần tránh full inventory trên UI. Không coi report này đóng toàn R1 hoặc toàn kế hoạch.
