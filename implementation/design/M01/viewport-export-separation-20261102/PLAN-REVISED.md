# Kế hoạch OM9 đã rà soát

Checklist chính thức duy nhất: [PLAN.md](PLAN.md).

Đã kiểm chứng: Undo/cap tương tác, scene/picking owners, phase Save/STL, shared peer, root/subselection, bỏ scene high-water, FileState và diagnostics theo bảng hồi quy. Gumball giữ xác nhận hết lỗi của người dùng; không giao làm lại những phần này.

FileState/Open đã xử lý khoản 91 byte; các nhóm kết quả cuối đạt 285 test distinct, 7 ignored, gồm native STL 60/60. [Bằng chứng hiện có](../../../../docs/validation/2026-11-05-live-file-state-memory.md). Không suy ra toàn bộ RAM/native/UI export hoặc accuracy gates đã xong.

Ưu tiên khi tiếp tục: **mốc A — gom R1a/R2a-STL cần thiết với P3b để bàn giao xuất STL nền dùng được trong UI**. Metadata nhỏ có bound được tách R1b; không buộc UI chờ audit mọi allocation hoặc đo mọi kernel operation. P1/v5, GPU batching, độ chính xác STL/3DM, checkpoint và release/desktop còn mở.

Áp dụng phần **Quy tắc chống loop** và **Gom test theo mốc** trong PLAN.md: test hẹp theo thay đổi; một batch tích hợp khi chốt candidate, chỉ chạy lại nhóm lỗi/bị ảnh hưởng. Không mở lại việc đã đạt hoặc nhận lại next_action lịch sử. Cập nhật cùng hồ sơ mốc; không tạo thêm báo cáo cho từng allocation.

Giữ managed RAM 60% RAM tổng; Undo 1024 MiB/cấu hình tối đa 50% RAM nằm trong tổng đó. Minimum steps hiện có thể vượt quota mềm; cần kiểm chứng và thông báo rõ. Phân biệt legacy cap 256 MiB, dự phòng OS và GPU fallback. GPU tối đa 50% VRAM/budget OS; không cam kết trần RSS/native. **Triển khai vẫn tạm dừng; lượt này chỉ cập nhật tài liệu, không build/test ứng dụng.** Phiên OM9 người dùng giữ nguyên.
