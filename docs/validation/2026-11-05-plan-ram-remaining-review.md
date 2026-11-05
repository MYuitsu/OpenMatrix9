# Rà soát việc còn lại và RAM — 2026-11-05

Theo yêu cầu cập nhật plan: bỏ việc đã xong khỏi checklist triển khai, bổ sung điểm RAM chưa hợp lý. Lần này chỉ sửa tài liệu và đối chiếu code/log hiện có; không sửa Rust/native, chạy lại test runtime hoặc thao tác phiên OM9. Không nâng trạng thái milestone từ việc viết plan.

## Phạm vi đã đối chiếu

- [Plan chính thức](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md), alias, live ledger, bằng chứng Undo, owners, shared peer và bỏ scene high-water.
- `crates/om9-core/src/budget/memory.rs`: managed 60%, physical Available reserve `max(256 MiB, 10% RAM tổng)`, Windows commit reserve 256 MiB; pending và allocated tách biệt.
- `crates/om9-core/src/transaction.rs::constrain_managed_ram`: ceiling hiện không tăng, giữ mức nhỏ nhất đã quan sát. Đây không phải cơ chế tự refresh tăng/giảm hoàn chỉnh.
- `crates/om9-core/src/transaction/undo_memory.rs`: default minimum 1, quota 1024 MiB, configuration clamp 50% RAM, inventory 100 MiB; `trim_undo_memory_with_reservations` giữ minimum ngay khi vượt quota mềm.
- `crates/om9-app/src/session/layers.rs::auxiliary_usage`: root/subselection có owner được loại khỏi auxiliary, nhưng workspace/recent/repeat/pending solid/layer dialog/contract data vẫn được kiểm kê tại caller. `session/stl.rs` vẫn giữ `caller_held_at_capture` cho phần chưa registered.
- `file_lifecycle.rs`, `session/files.rs` và test Open/presentation ở `gpu_viewport.rs`: FileState owner đang sửa, không đồng nghĩa đã đạt mọi hồi quy.

Không duyệt lại decompile hoặc tài liệu Blender/Rhino trực tuyến trong lượt này: yêu cầu là cập nhật tiến độ và chính sách RAM của OM9 theo code thực tế. Các báo cáo tham khảo cũ được giữ nguyên và không được xem là trạng thái runtime hiện hành.

## Giữ việc đã xong ngoài checklist

Undo Options/default/50%/inventory, đường tương tác đã chứng minh vượt legacy 256 MiB, scene/hover/picking/Weak owners trong phạm vi báo cáo, shared STL peer, root/subselection và bỏ legacy scene high-water tiếp tục là ràng buộc hồi quy. Không giao làm lại. Gumball giữ xác nhận hết lỗi của người dùng.

Các gate chưa đủ bằng chứng vẫn giữ: RAM auxiliary/native, display cache độc lập/v5, GPU admission/batching, STL UI worker, sai số/topology/picking/3DM, checkpoint và release/desktop. Không xóa gate chính xác chỉ vì một số fixtures đã pass.

## FileState: code có nhưng tích hợp chưa xong

Nguồn bằng chứng hiện có:

- [Năm test mới](../../build/standalone-validation/ui-artifacts/file-memory-20261105-073852/file-memory.log): 5 passed, 0 failed. Phủ destination growth/cancel, pre-write ownership/failure, admission refusal, Save/Snapshot/Open và diagnostic capacity của candidate fixture. Candidate diagnostics không là bằng chứng một file migration thật.
- Lần chạy tiếp [replacement](../../build/standalone-validation/ui-artifacts/file-memory-20261105-074055/replacement.log): 8 passed; [lifecycle](../../build/standalone-validation/ui-artifacts/file-memory-20261105-074055/lifecycle.log): 12 + 20 + 9 passed, 6 ignored; [STL](../../build/standalone-validation/ui-artifacts/file-memory-20261105-074055/stl.log): 63 passed; [root](../../build/standalone-validation/ui-artifacts/file-memory-20261105-074055/selection-memory.log): 2 passed; [subselection](../../build/standalone-validation/ui-artifacts/file-memory-20261105-074055/subselection.log): 3 passed.
- [Presentation](../../build/standalone-validation/ui-artifacts/file-memory-20261105-074055/presentation.log): 12 passed, **1 failed**. `open_registers_new_buffers_before_viewport_and_preserves_old_worker_owner` tại dòng 895 lúc đối chiếu: actual 2187, expected 2096 byte. Oracle hiện cộng buffer/scene nhưng chưa cộng file payload mới; đây là giả thuyết giải thích chênh 91 byte, cần xác minh bằng owner thực trước sửa assertion.

Script fail-fast dừng ở presentation; hover/GPU/Gumball/native STL phía sau không có log trong run này. Không coi chúng đã chạy hoặc mượn kết quả trước để tuyên bố source đang sửa đã xanh toàn bộ. Phiên chạy cũ không còn truy cập được qua process session; các terminal summaries trên disk là bằng chứng lần rà soát này. Không tạo retry hoặc cộng retries vào tổng test hoàn tất.

Do đó plan giữ FileState là một bàn giao đang làm với gate cụ thể: xác minh path charge, giữ assertions old-reader/failure, chạy phần bị ảnh hưởng và phần bị dừng rồi audit. Không bắt đầu lại toàn bộ thiết kế ownership và không chấp nhận bỏ assertion để có test xanh.

## RAM được làm rõ và bổ sung

1. 60% tính trên RAM vật lý **tổng**, không phải RAM hiện trống. Undo 1024 MiB là quota bên trong tổng managed, không là ngân sách toàn app hoặc 1 GiB được cấp sẵn. Máy 16 GiB có managed ceiling khoảng 9,6 GiB, không phải 10,6 GiB sau khi cộng Undo.
2. Undo hiện có quota mềm vì minimum steps. 50% là trần cấu hình, chưa là chứng minh actual history không bao giờ vượt 50%. Bổ sung test một bước quá lớn, requested/effective/actual và từ chối transaction nếu vượt trần managed. Đổi minimum hoặc quota mềm thành cứng cần quyết định nghiệp vụ riêng; lượt sửa plan này không đổi hành vi.
3. Không xóa mọi hằng 256 MiB: tách legacy local default, physical/commit reserve và GPU fallback. Caller codec/packet/depth/overflow vẫn cần giới hạn đúng mục đích.
4. Admission phải tính đỉnh tăng thêm trước allocation: candidate, native result, Rust copy, topology, staging và old/new overlap. Source chia sẻ được tính một lần; quota chưa dùng không thành bytes đã cấp. Available và commit là hai gate riêng, không trừ lại live allocations khỏi mẫu OS.
5. Cấu hình theo RAM tổng không bị cắt theo Available thay đổi nhanh. Kiểm chứng non-widening ceiling hiện có, probe lỗi/phục hồi và file cấu hình chuyển máy; không mô tả chúng như đã hoàn thiện. Không bỏ owner charge khi task/candidate còn sống.
6. Native OCCT pre-copy peak vẫn chưa có hard envelope. Giữ hạn chế job nặng và reservation đến native return/join, đo phase cùng profile; không lấy STL file size hoặc hệ số tùy ý làm upper bound. Allocation failure cần đường lỗi an toàn, preflight không bảo đảm toàn tiến trình tránh OOM.
7. RAM ledger chỉ cam kết phạm vi managed đã kiểm kê. Đối chiếu commit/working set và công bố phần chưa giải thích, không buộc counter bằng RSS. Metadata nhỏ có bound được bàn giao R1b để không biến audit thành điều kiện chờ vô hạn cho UI STL. Không tạo bộ quản lý RAM thứ ba.

Ưu tiên giữ phạm vi cụ thể: FileState regression → auxiliary thực trên đường export → native/scratch STL → UI worker. Accuracy gates hiện hành luôn áp dụng; P1/P3a/P2/P3c/P4 và desktop vẫn mở, Mesh Repair giữ AFTER riêng.

## Kiểm tra tài liệu

Kiểm tra đạt: 46 local links trong plan/alias/biên bản, JSON ledger parse được với review ID duy nhất, whitespace và các invariant của plan. Public source audit exit 0: 15.194 source files, 594 icon bindings, 593 SVG, errors rỗng. `git diff --check` trong phạm vi tài liệu đạt; file mới/untracked được kiểm tra whitespace trực tiếp.

Không tạo batch đo runtime/openNURBS/3DM mới, không đóng exchange package; tổng batch tương lai chưa xác định. Không có build batch mới phải prune trong lượt sửa tài liệu này. Log của phần đang sửa được dẫn cụ thể để bảo toàn bằng chứng, kết quả thất bại được giữ nguyên.
