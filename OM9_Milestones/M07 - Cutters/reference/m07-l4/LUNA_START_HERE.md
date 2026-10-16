# GPT-6 Luna — hướng dẫn tiếp nhận M07/E12

## 0. Quy tắc bất biến

1. Không được nhầm **viết kế hoạch** với **đã triển khai hoặc kiểm thử**. Toàn bộ E12 hiện `PLANNED_NOT_EXECUTED`.
2. Đọc ít ngữ cảnh có chủ đích: `planning/M07/README.md`, `planning/M07/GPT6_LUNA_TASK_PROMPT.md`, task `l4.json` + `TASK.md`, L3 gốc, L2 owner, hợp đồng `specs/00-contracts/`, M00 audit. Không tải toàn bộ 772 L3 vào một lần.
3. Mỗi vòng chỉ **một L4** hoặc tối đa hai L4 độc lập thật sự nếu đã review files touched và không cùng ownership/geometry/history contract. Một task = một diff nhỏ và evidence riêng; không giải quyết mọi module cùng lúc.
4. Chọn task runnable từ manifest JSON, `execution_wave` tăng dần, khi **mọi** `depends_on_task_ids` PASS. Thứ tự hàng CSV không thay gate. Không được đánh dấu PASS từ việc viết code mà không có build/test.
5. `source_revision_candidate` là SHA v0.12 đã đọc remote. Không khẳng định đã checkout/build tại local; lấy SHA thực `git rev-parse HEAD` và so khớp. Không bỏ qua luật license AGPL/copyleft, OCCT và tài sản Gemvision.
6. Mọi API/filename proposed được phép sửa sau khi kiểm source, nhưng **không** đổi `l3_id`, `capability_id`, owner, G00/E12, hoặc chuyển geometry kernel sang UI. Mọi thay đổi contract phải có ADR/reviewer.
7. Không copy source QymCAD sang archive mới mà thiếu attribution/terms; không dùng tài nguyên thư viện cutter thương mại không có phép.
8. OpenMatrix9 logic bằng Rust, nhân OCCT và FFI C++ là thành phần bắt buộc theo feasibility; **không** viết lại OCCT bằng Rust để đáp ứng khẩu hiệu Rust-only.

## 1. Từng vòng Luna (thủ tục quyết định)

- **PRECHECK:** chạy validators planning; đọc `implementation/evidence/G00/` và các E01–E11 sign-offs. Nếu không có: thông báo `BLOCKED_PRECONDITION`, liệt kê chính xác evidence thiếu; chỉ triển khai source-free planning audit khi có chỉ thị.
- **SELECT:** lấy task đầu tiên chưa PASS thỏa dependencies. Nếu chưa có task runnable, báo dependency blocker thay vì chọn đại.
- **AUDIT:** trước sửa code, mở `module.json`, `l3.json`, manual refs, source pinned; ghi local file/function, license, `REUSE` / `ADAPT` / `ADD`, test baseline hiện có.
- **DESIGN:** chốt typed request/response, owner/caller, topology references, units/tolerance, preview snapshot, error behavior, rollback. Với thay đổi API chéo module, yêu cầu contract sign-off trước khi code.
- **IMPLEMENT:** backend service từ owner trước, adapter menu/F6 dùng chung registry sau. Không giả định source sẵn có. Không dùng UI preview làm geometry oracle.
- **VERIFY:** test seed fixtures tự tạo; unit/integration/golden OCCT; tests invalid/cancel/Undo/Redo; save/open; independent STL verification nếu liên quan; chạy regressions.
- **HANDOFF:** lưu `implementation/evidence/M07/<task_id>/...` với tested SHA, OS/toolchain/OCCT, logs, numeric errors, screenshots/tester khi manual; chỉ ghi status sau review.

## 2. Thứ tự nội bộ

- **Phase 01:** local source audit & typed contract. Có thể làm nhiều contract độc lập, nhưng không bao giờ gắn `REUSE` như đã verified từ screenshot/README.
- **Phase 02:** integrate owners. Các task có phụ thuộc chéo được sắp theo `execution_wave` (manifest JSON). Không viết lại Boolean từ E07 hoặc gem settings từ E10/E11.
- **Phase 03:** UI/external acceptance. `febe` bắt buộc app thực và tester, `be` STL test bằng parser/viewer độc lập, không tự suy diễn từ hình ảnh.
- **Phase 04:** adversarial/gate evidence cho 9 capability rủi ro cao. Làm rõ report nào không thể tự ký.

## 3. Cấu trúc phản hồi bắt buộc của Luna sau mỗi task

```
TASK: OM9-L4-...
DECISION: REUSE | ADAPT | ADD | BLOCKED_SOURCE
STATUS: BLOCKED | IN_PROGRESS | READY_FOR_REVIEW | READY_FOR_MANUAL_TEST | PASS | FAIL
SOURCE: repository, SHA, license, paths/functions verified locally
DEPENDENCIES: task IDs + PASS evidence
DIFF: changed files and why they belong to owner
TESTS: exact commands, exit codes, fixture IDs, units, numeric expected/actual
MANUAL: tester/build/date/evidence OR NOT_RUN
RISKS: source/geometry/history/STL/legal concerns and issue IDs
NEXT: first runnable task ID (or blocker)
```

Luna chỉ cập nhật `status=PASS` trong evidence sau khi thực tế vượt criteria, có reviewer thật; trạng thái L4 kế hoạch trong `planning/` vẫn là baseline planning cho tới khi có cơ chế đồng bộ bằng chứng riêng. Không ký thay user/QA, không thực hiện fake sign-off.

## 4. Stop conditions

- `BLOCKED_SOURCE`: không có pinned checkout hoặc missing files.
- `BLOCKED_DEPENDENCY`: G00/E11/source contract hoặc dependency task chưa PASS.
- `BLOCKED_REFERENCE`: thiếu source manual/capability/fixture có thể kiểm toán.
- `FAIL_BLOCKER`: invalid geometry, wrong mm, STL not manifold, license conflict, broken Undo/Save, silent failure.
- `READY_FOR_MANUAL_TEST` **không** được ghi thành `DONE`.
