# Rà soát plan viewport/export và RAM, lần hai — 2026-11-05

Yêu cầu: bỏ việc đã xong khỏi kế hoạch còn lại, bổ sung phần RAM chưa hợp lý. Lần này chỉ sửa plan, alias và ledger; không sửa runtime, không chạy thêm Rust/GPU/release, không thao tác phiên OM9 đang mở. Không nâng trạng thái tính năng hoặc milestone.

Follow-up triển khai sau lần rà soát: [scene/picking và Save theo phase](2026-11-05-scene-picking-save-phases.md) đã chốt focused regressions còn mở bên dưới và thay cơ chế giữ toàn headroom của Save. Các đoạn bên dưới giữ trạng thái tại thời điểm rà soát; checklist hiện hành theo PLAN.md và báo cáo mới.

## Phần đã đưa ra khỏi việc cần triển khai

Giữ nguyên bằng chứng đã công bố: Undo 1024 MiB/tối đa 50% RAM/kiểm kê 100 MiB, đường tương tác tổng hợp >256 MiB, publication buffer R0, phase pending/allocated trên tài khoản chung, metadata snapshot viewport/hover và gate commit Windows. Bảng hồi quy trong [plan](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md) liên kết các báo cáo tương ứng. Xóa phần tường thuật R0 lặp lại khỏi mục công việc, không xóa bằng chứng lịch sử.

Các kiểm tra STL và chia sẻ nguồn/display đã có vẫn là ràng buộc hồi quy. Không gọi cache display đã độc lập, native v5 đã có, UI STL đã chạy nền hoặc GPU batching đã hoàn tất.

## Code mới khác với trạng thái cũ của checklist

Đọc trực tiếp các tệp:

- `crates/om9-scene/src/scene/picking_cache.rs`: quota metadata + arena, transfer/reserve, phase thực cấp phát, kiểm lại pending trước cold allocation, preview giữ storage, eviction và re-admission.
- `crates/om9-scene/src/spatial.rs`: capacity index và workspace sắp xếp được tính riêng.
- `crates/om9-app/src/session.rs`, `session/files.rs`, `gpu_viewport.rs`, `ui.rs`: các đường nhận/chuyển allowance cho scene và snapshot.
- `crates/om9-core/src/transaction.rs`, `crates/om9-native-owner/src/system_memory.rs`: phase và probe physical/commit.

Không còn giao việc “viết owner metadata scene chính/lazy BVH từ đầu”. Tuy nhiên chưa bỏ cổng kiểm chứng cuối: log focused có thất bại lịch sử và source tiếp tục được sửa sau đó. Cần lưu kết quả cuối đúng source; source/test tồn tại không tự chứng minh pass.

## Log đã đọc, không phải test chạy lại trong lần sửa tài liệu

Thư mục `build/standalone-validation/ui-artifacts/commit-integration-20261105-024737` có terminal summaries:

| Log | Kết quả ghi trong log |
|---|---|
| selection-overlay.log | 14 pass, 1 ignored |
| picking-gumball.log | 19 B-rep precision, 7 Gumball interaction, 7 Gumball viewport pass |
| gpu-publication.log | 28 pass; không thay bằng chứng desktop |
| lifecycle.log | 12 document, 20 file lifecycle, 9 ordinary RAM pass; 6 large ignored trong lượt này |
| real-resource-270mib.log | 1 fixture tài nguyên 270 MiB pass riêng; không thay các fixture CAD lớn khác |
| stl-export.log | 60 pass |

Thư mục `build/standalone-validation/ui-artifacts/commit-metadata-20261105-024241`: core budget 4, retained/account 101, native memory 5, presentation 10, hover 14, scene memory 7 pass. `replacement.log` ghi 3 pass/4 fail. Các phép đối chiếu owner đã có thay đổi sau log này; không coi log integration khác suite là kết quả thay thế cho focused replacement hoặc STL lifecycle cuối. Các suite này vẫn là việc cần xác nhận, không mở rộng nhiệm vụ sửa tài liệu thành đợt build mới.

Không dùng số test làm phần trăm hoàn thành hoặc ước lượng số giờ. Không xác nhận hiệu năng release, RSS/native upper bound hay độ mượt desktop từ các log này.

## RAM: điểm chưa hợp lý và phần bổ sung

1. **Giữ trước quota cache tùy chọn:** scene đang dự trù arena chưa dùng; nhiều snapshot có thể giữ nhiều phần pending. Đây là reservation có chủ sở hữu, vẫn có thể gây thiếu ngân sách giả. Plan yêu cầu đo và giải phóng quota không cần khi có áp lực, re-admit khi dùng; không đánh đồng quota với allocation đã tồn tại.
2. **Eviction mới có API:** tìm caller `evict_picking_cache` trong crates hiện chỉ thấy tests. Cần nối chính sách thu hồi theo request vào app, giới hạn retry, tránh cache churn và tôn trọng owner/fence. Đánh dấu invalid không phải đã giải phóng.
3. **Save/STL giữ toàn headroom:** production `session/files.rs::save_archive_for_batch_guard` và hai đường captured/detached STL trong `session/stl.rs` vẫn dùng `reserve_remaining_external_memory_with_held`. Ưu tiên R2a chuyển từng đường sang reservation theo phase có căn cứ; đo old/new/scratch/native overlap, không thay bằng hằng số hoặc hệ số tùy ý.
4. **Không chặn sửa thực dụng bằng kiểm toán quá rộng:** chốt focused regressions scene/picking rồi làm R2a, đồng thời giữ audit query scratch/auxiliary/high-water/native ở các bàn giao hữu hạn. Các phép đo Boolean/fillet/loft thuộc R2b, không là điều kiện tiên quyết cho sửa reservation Save/STL.
5. **Ba giới hạn khác nhau:** managed 60% RAM tổng, Available physical sau dự phòng, commit headroom Windows. Không cộng Available với commit, không tính lại allocation đã sống như request pending. Không hứa trần RSS/OOM tuyệt đối.
6. **Undo không cấp sẵn:** 1024 MiB là quota lịch sử bên trong tổng 60%, không là RAM đã dùng hay hạn mức toàn OM9. Giữ minimum Undo count và từ chối an toàn khi giao dịch không vừa; không phá Redo/Undo để cứu giao dịch thất bại.
7. **Chẩn đoán và gate:** thêm owner, live/pending, tuổi mẫu physical/commit, phần chưa đo; low/unknown probe, concurrent requests, nhiều snapshot, cold/warm/evicted cache và failure/cancel. Thông báo thường vẫn ngắn. Giữ các guard cấu trúc/overflow và codec theo đúng mục đích.

## Kiểm tra tài liệu

Kiểm tra đã chạy: liên kết local của plan/alias/biên bản hợp lệ; ledger parse JSON và ID lần rà soát xuất hiện đúng một lần; hai thư mục log được dẫn còn tồn tại. Public-source audit: 15.176 source files, 594 icon bindings, 593 SVG, không có lỗi. Không chạy lại Rust cho thay đổi tài liệu và không tạo batch validation runtime mới. Phần việc tiếp theo: kết quả focused cuối scene/picking → R2a Save/captured STL/detached STL → nối thu hồi cache và audit owner còn thiếu; P1–P5 còn mở.
