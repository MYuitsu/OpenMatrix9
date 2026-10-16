# Luna bắt đầu tại đây — OpenMatrix9 M08/E13

## Đọc ngữ cảnh có giới hạn

1. `planning/M08/README.md` (toàn cảnh và blocker), `planning/M08/RELEASE_GATE.md` (gate và oracle), `planning/M08/M08_L4_EXECUTION_MANIFEST.json` (lựa chọn task). Không nạp 772 L3 một lần.
2. `planning/CODEX_TASK_PROTOCOL.md`, `planning/GATES_AND_EVIDENCE.md`, `planning/DEPENDENCY_AND_RELEASE_POLICY.md`; hợp đồng `specs/00-contracts/MATRIX9_FIRST_CONTRACTS.md` + `MATRIX_WORKBENCH_UI.md`.
3. Chỉ khi chọn task: mở `l4.json` + `TASK.md`, canonical `l3.json`, `module.json`, exact manual pages. Source code QymCAD phải được audit từ local checkout tại pinned SHA; đường dẫn source trong L4 chưa được xác minh.

## Quy trình một task/một lượt, không tự chạy toàn bộ 221

**(A) CHECK GATES** — G00-01..07 PASS + pháp lý, E01–E12 PASS thực, `G-MVP` ký chính thức, signed numerical tolerances, pinned source checkout build. Thiếu bất kỳ mục nào: chỉ trả `BLOCKED_PRECONDITION` với đường dẫn evidence cần có. Có thể đề xuất audit tài liệu (source-free), nhưng không coding hay đánh dấu PASS.

**(B) PICK** — chọn task có `execution_wave` nhỏ nhất chưa PASS mà tất cả `depends_on_task_ids` đã có evidence PASS. Danh sách CSV chỉ là gợi ý thứ tự, điều kiện DAG mới là chuẩn. 22 transform adapter phải chờ owner services: ví dụ `OM9-L4-04-01-011-02` chờ `OM9-L4-04-05-002-03`; tuyệt đối không đưa thuật toán Smart Flow vào menu binding.

**(C) AUDIT/CONTRACT** — ghi `git rev-parse HEAD`, Cargo/OCCT versions, tatsäch source files, license/AGENTS, mapping REUSE/ADAPT/ADD; chốt API typed, owner, callbacks, errors, transaction preview/commit, geometry tolerances, topology and history. Chưa chứng minh gọi được từ QymCAD thì trả `BLOCKED_SOURCE`.

**(D) IMPLEMENT** — Rust app/domain, nhân OCCT dùng C++ FFI; không viết geometry qua mesh/render GPU. Dựng BRep có test, không chỉ màn hình đẹp. Enforce unit mm, orientation, trimmed UV seams, rigid gems, history update, rollback/Undo, Save/Open; giữ baseline G-MVP.

**(E) VERIFY** — unit/contract/OCCT golden + negative tests; record actual numeric differences, exact commands and exit codes, memory/FFI failures, deterministic seed. Chạy desktop build cho FEBE, ghi kịch bản thao tác, screenshots/log, reviewer người thật. Không báo PASS khi chưa có manual/evidence đầy đủ.

**(F) HANDOFF** — trả kết quả và NEXT task (khi đủ dependencies), không tự nâng L3 hoặc task từ planned lên executed ở `planning/`. Kết quả thực thi lưu riêng `implementation/evidence/M08/<task-id>/`.

## Phân biệt lỗi quan trọng

- **Smart Flow:** Autobase/Target UV, normals; rigid transform giữ nguyên kích thước của gem, history recompute theo DAG; Break giữ hình học hiện tại, không đồng nghĩa Delete.
- **Deformations:** Bézier/OCCT validity, projective frame, axis singularity, cage release vs deleting cage, live mirror tránh vòng dependency.
- **Smart Pattern:** rows/columns, Group Output vs Output Mesh, Towards attractor and UV seam; preview cache không trở thành nguồn BRep.
- **Rope:** Cap=None thực sự mở, không được đánh dấu watertight; Flat/Round phải kiểm topology.
- **Libraries:** root path sandbox, atomically save, corrupt asset, safe import/open, preservation of user files, rights to reuse content.

## Format Luna buộc trả sau mỗi task

```
TASK: OM9-L4-...
GATES: actual G00/E12/G-MVP evidence paths, signers, or MISSING
DEPENDENCIES: complete PASS task IDs and reviewed artifacts, or BLOCKER
DECISION: REUSE | ADAPT | ADD | BLOCKED_SOURCE
STATUS: BLOCKED | IN_PROGRESS | READY_FOR_REVIEW | READY_FOR_MANUAL_TEST | PASS | FAIL
SOURCE: repository, local SHA, license, paths/functions actually inspected
DIFF: changed files, owner justification, API compatibility
TESTS: command, exit code, fixture, expected/actual numerical measurements, regression
MANUAL: tested build, tester/date, steps, screenshots, sign-off or NOT_RUN
RISKS: geometry/UV/history/license/library IO, issue IDs
NEXT: first runnable task ID, or precise reason no task is runnable
```

Không tự chọn alias lệnh từ tên UI khi `source_command_candidate_from_l3` chưa xác minh. Không tự chốt ngưỡng độ chính xác, dung sai sản xuất hoặc tính hợp pháp của tài nguyên.
