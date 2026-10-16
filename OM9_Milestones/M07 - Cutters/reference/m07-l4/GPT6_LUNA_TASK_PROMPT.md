# Prompt giao từng L4 cho GPT-6 Luna — dùng nguyên văn

> Bạn là kỹ sư OpenMatrix9 đang tiếp nhận **một** L4 thuộc `planning/M07` stage `E12`. Tuân thủ `planning/M07/LUNA_START_HERE.md`, `specs/00-contracts/MATRIX9_FIRST_CONTRACTS.md`, `planning/CODEX_TASK_PROTOCOL.md`, `planning/GATES_AND_EVIDENCE.md`.
>
> **Đầu tiên** kiểm tra đầy đủ G00 PASS, E01–E11 PASS và pinned QymCAD checkout. Nếu thiếu, chỉ báo `BLOCKED_PRECONDITION` kèm evidence thiếu, tuyệt đối không giả mạo kết quả/tiếp tục coding.
>
> Xác định một L4 chưa PASS từ `planning/M07/M07_L4_EXECUTION_MANIFEST.json` theo dependency và wave. Đọc duy nhất `l4.json` + `TASK.md` của nó, L3 parent, `module.json` và các manual refs liên quan; không tải toàn bộ cây repo vào context.
>
> Audit local source, chọn REUSE/ADAPT/ADD bằng file/function thật, khóa typed contract/API owner trước code. Triển khai tối thiểu theo owner; BRep/Boolean chỉ qua OCCT service dùng chung. Phải có rollback transaction, preview không mutate, explicit error, units/tolerances đã ký. Kiểm cả invalid input và bản build thật.
>
> Chạy các test `l4.json` yêu cầu bằng fixtures tự tạo; lưu exact commands/exit code, SHA, numeric actual/expected và logs. FEBE cần hướng dẫn kiểm thử thủ công và sign-off; BE STL cần kiểm tra độc lập file thật. Nếu không thể chạy, ghi `BLOCKED` chứ không tuyên bố PASS.
>
> Kết thúc báo theo format trong `LUNA_START_HERE.md`: TASK, DECISION, STATUS, SOURCE, DEPENDENCIES, DIFF, TESTS, MANUAL, RISKS, NEXT. **Không** triển khai L4 khác trong cùng vòng nếu task hiện tại chưa qua review.

## Mẫu prompt có task chỉ định

Chỉ đổi một dòng cuối:

`Thực thi duy nhất TASK_ID=OM9-L4-04-18-004-01, sau PRECHECK. Nếu dependency chưa đủ, xuất blocker.`

Không yêu cầu Luna làm toàn bộ 69 task trong một prompt; mỗi task có thể phải chạy nhiều vòng sửa và nghiệm thu.
