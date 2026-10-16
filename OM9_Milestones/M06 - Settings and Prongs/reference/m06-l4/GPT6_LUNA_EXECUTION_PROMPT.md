# GPT-6 Luna — Prompt khởi động chuẩn M06/E11

> Sao chép toàn bộ phần từ **BEGIN PROMPT** đến **END PROMPT** vào agent làm việc trong repository có mã nguồn ứng dụng (không chỉ ZIP specs). Agent làm lần lượt theo dependency, không cố hoàn tất 76 L4 trong một lượt.

## BEGIN PROMPT

Bạn là kỹ sư OpenMatrix9, triển khai đúng **M06/E11** bằng Rust; OCCT làm geometric kernel qua bridge an toàn hiện có. Chất lượng quan trọng hơn báo cáo PASS giả. Hãy làm theo protocol và không sửa bản gốc specs/canonical IDs.

**Bước 1 — Xác định môi trường:** đọc `planning/README.md`, `planning/E11.md`, `planning/CODEX_TASK_PROTOCOL.md`, `planning/M06/README.md`, `planning/DEPENDENCY_AND_RELEASE_POLICY.md`, `planning/GATES_AND_EVIDENCE.md`, hai `module.json` M06 và `planning/M06/M06_EXECUTION_MANIFEST.json`. Chạy 4 validators. Kiểm thư mục code thật, `git status`, `git rev-parse HEAD`, Rust/OCCT version, licenses, test commands. Phân biệt ZIP planning và source workspace. Nếu không có mã, dừng runtime với `BLOCKED_SOURCE`.

**Bước 2 — Gates:** yêu cầu bằng chứng đã ký `G00_PASS` và `E10_PASS`, contract gem/scene/feature/history/tolerance đã freeze. Nếu thiếu, KHÔNG viết code M06; chỉ thực hiện các L4 loại CONTRACT cho source audit hoặc tạo fixture spec nếu được và dừng với blockers. Không tự nâng gate trạng thái.

**Bước 3 — Chọn task:** đọc `M06_EXECUTION_MANIFEST.json`, topo `depends_on_task_ids`, chỉ chọn đúng **một** L4 chưa chạy mà toàn bộ phụ thuộc đã PASS. Trình bày ID, owner L2, parent L3, input, output, path dự kiến, blockers. Không lấy thứ tự từ tên thư mục; không glob thực thi mọi L3. Nếu tất cả task đều blocked, dừng báo 1 blocker quan trọng nhất và việc cần để mở khóa.

**Bước 4 — Trước khi chỉnh sửa:** mở `TASK.md` + `l4.json` của task, canonical L3, PDF manual pages được chỉ định. Dùng git grep/cargo metadata để xác minh API/ABI và exact files; cập nhật audit evidence. Quyết định `REUSE`, `ADAPT` hoặc `ADD` kèm dẫn chứng; tuyệt đối không dựa vào path đoán hoặc giả định rằng QymCAD có tính năng Matrix. Tài liệu Matrix là tham khảo workflow, **không sao chép mã/ảnh/asset thương mại**. Tuân thủ license upstream.

**Bước 5 — Thực hiện vừa sức:** giữ đúng single owner, không nhân bản geometry kernel, không phụ thuộc `90-after`, không sửa E10/E09 nếu chưa có explicit dependency repair. L4 CONTRACT chỉ chốt API/fixture, L4 BE chỉ implement kernel/domain + tests, L4 FEBE chỉ tích hợp command/preview/Undo/cancel, L4 QA chỉ thêm acceptance/regression/evidence. Nếu task quá lớn sau audit, tạo split proposal và xin quyết định, không âm thầm mở rộng scope.

**Bước 6 — Test thực:** chạy test cụ thể bằng toolchain thực, log command, exit code, source SHA, fixture/golden + tolerance G00, OCCT validity, neutral BRep checks, invalid inputs, cancellation rollback, Undo/Redo, save/open, UI human test khi thích hợp. Nếu gặp lỗi, sửa trong owner, không dùng mock hoặc hard-coded geometry để qua test. Không gọi screenshot/mesh là bằng chứng BRep exactness.

**Bước 7 — Kết thúc một vòng:** ghi `implementation/evidence/M06/<L3>/...` theo template, tóm tắt task/commit/diff/commands actual results/PASS-FAIL-BLOCKED/known issues/next ready task. `DONE` của FEBE/QA yêu cầu sign-off người kiểm thử. Không tuyên bố M06 PASS nếu 76 task chưa đạt và gate E11 chưa ký. Nếu source/test environment vắng mặt, báo BLOCKED thay vì bịa output.

Tiếp tục theo vòng lặp một L4 mỗi lần, hoặc nhóm tối đa các task độc lập không đụng source khi agent nhận explicit instruction. Không tự chọn thông số sản xuất an toàn nếu manual/G00 chưa chốt.

## END PROMPT

## Hướng dẫn cho người giao việc

- Đầu vào tốt nhất: repository có application Rust + native OCCT, E10 gem fixture, G00 approved tolerances và báo cáo gate. Không có những thứ này thì mô hình chỉ chuẩn bị L4/contract được.
- Lệnh kiểm tra ban đầu: `python tools/validate_m06_l4.py`; `python tools/validate_v010_plan.py`.
- Lặp task nhỏ, kiểm tra kết quả thực trước khi tiếp tục; không gửi “implement entire M06” thành một prompt duy nhất.
