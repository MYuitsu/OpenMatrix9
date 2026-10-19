# GPT-6 Luna — OpenMatrix9 M05 L4 handoff and execution protocol

## Prompt bắt đầu (dán nguyên khối khi có SOURCE checkout và muốn triển khai)

> Bạn là GPT-6 Luna, đang làm OpenMatrix9, **chỉ stage `E10 / M05`**. Đọc lần lượt `planning/README.md`, `planning/M05/README.md`, `planning/M05/RUNBOOK_GPT6_LUNA.md`, `planning/CODEX_TASK_PROTOCOL.md`, `planning/GATES_AND_EVIDENCE.md`, `planning/DEPENDENCY_AND_RELEASE_POLICY.md`, `planning/M00/SOURCE_AUDIT_V012.md` và `planning/M05/M05_L4_EXECUTION_MANIFEST.json`. Không đọc toàn bộ 126 TASK.md trong một context. Kiểm tra G00 và E09 có bằng chứng PASS; nếu chưa thì **không bắt đầu code M05**, chỉ ghi blockers/chuẩn bị audit nếu được yêu cầu. Đối chiếu git HEAD với SHA pinned, AGENTS.md/CLAUDE.md, giấy phép, OCCT/toolchain. Chỉ chọn **một** task L4 có dependency đã PASS; đọc đúng `l4.json`, `TASK.md`, `l3.json`, `module.json`, trang manual đã dẫn và source code liên quan. Làm từng phase: audit + đóng băng contract → BE có tests → FEBE có UI và nghiệm thu. Giữ owner/capability ID, không sao chép thuật toán vào UI. Xuất exact diff, lệnh build/test, kết quả thật, fixtures/oracle, blocker và đường dẫn evidence. Không tự tuyên bố DONE khi thiếu xác nhận người kiểm thử. Dừng sau một task và yêu cầu review trước task kế tiếp.

## 1. Trước khi chọn task (preflight)

1. Run `python tools/validate_l3.py && python tools/validate_v010_plan.py && python tools/validate_m05_l4.py`.
2. Verify actual local source checkout and exact revision; read repo agent instructions. A URL in task metadata does **not** verify a local file.
3. Confirm G00-01..07 PASS with evidence, license obligations cleared; confirm E01..E09 PASS. If any missing, output `BLOCKED_G00_E09` and name exactly which gate/evidence is missing.
4. Inspect the M05 CSV manifest for a task whose `depends_on_task_ids` are all `PASS_EVIDENCED` or accepted `DONE` in **runtime progress records** (not this static planning file).
5. Never use the stage label `E10` as a substitute for preflight checks.

## 2. Context budget / execution size

- Read only **one `TASK.md` and its `l4.json`** per turn. Expand to source and 1–3 direct dependencies as needed; do not ingest entire planning tree, PDFs or all 126 tasks.
- Phase 01 = repository/source audit and frozen API contract. It **must not implement** the feature.
- Phase 02 = one domain/geometry or adapter service change + tests, no UI refactor.
- Phase 03 = bind actual command, preview, cancel, accept, Undo/Redo, save/reopen + manual test protocol, no new CAD mathematics inside UI.
- Split any phase that starts to contain two independent algorithms, many unrelated crates or multiple owners; record split before implementing.
- Keep individual commits or patches reviewable and restartable. Avoid mass `cargo fix` and unrelated dependency upgrades.

## 3. Strict results and errors

- `Result<Output, DomainError>` is **illustrative**: discover actual error conventions from source before defining Rust APIs.
- No geometry placeholder, button performing nothing, or test that only asserts `Ok`.
- Invalid/NaN input, degenerate geometry, deleted reference, wrong units, bad normal, conflicting history and file errors must produce user-visible explainable errors and roll back the transaction.
- OCCT validity/orientation/topology checks required whenever geometry is generated. Compare to independent oracle/fixture using document tolerance established by previous stages; no invented micrometre magic numbers.
- `Cancel` leaves the scene, Undo/Redo and save state unchanged. `Accept` is one transaction; rebuilding a group must not silently re-create IDs or duplicate stones.
- M05 may produce **azure/prong placement markers**, but metal prong geometry belongs to **M06** and cutting metal belongs to **M07**. Defer any proprietary `.mss` interop until format and legal proof.
- No `90-after` dependencies in mainline; no cloning Matrix executable/asset packs/manual pictures to open-source code.

## 4. Phased status / progress checkpoints

Keep a runtime journal at `implementation/evidence/M05/PROGRESS.json` (created when the implementation repo is available), not in the plan ZIP. The journal should include `task_id`, `status`, `completed_dependency_ids`, `source_sha`, `tested_sha`, `log_paths`, `reviewer`, `blocking_reason`, `next_ready_task_id` and timestamp. Before doing work, load journal and reconfirm using the current repository HEAD and actual evidence.

Allowed execution flow:

`PLANNED_BLOCKED_G00_E09 → READY_TO_AUDIT → IN_PROGRESS → PASS_EVIDENCED` (phase 01/02)  
`IN_PROGRESS → READY_FOR_MANUAL_TEST → DONE` (phase 03 only after human signoff).

Use `BLOCKED_SOURCE`, `BLOCKED_CONTRACT`, `BLOCKED_REFERENCE_OR_ORACLE`, `BLOCKED_TOOLCHAIN`, `FAIL_TEST`, `FAIL_GEOMETRY`, `FAIL_MANUAL` as applicable. Never change plan manifest status to PASS just because a source file exists.

## 5. Output REQUIRED after every single L4 task

```text
TASK: <task_id>
STATUS: <PASS_EVIDENCED|READY_FOR_MANUAL_TEST|DONE|BLOCKED_*|FAIL_*>
SOURCE: <repo URL, exact SHA, local paths verified and file:line proof>
OWNER/BOUNDARY: <owner module, reused/adapted/added service, explicit non-owned pieces>
CHANGED FILES: <paths and concise change notes>
COMMANDS: <literal shell commands + exit codes + raw log paths>
FIXTURES: <IDs, exact inputs, unit, tolerance, reference oracle>
RESULTS: <expected vs actual numeric or UI observations>
UNDO/HISTORY/PERSISTENCE: <pass/fail/not run, reasons>
BLOCKERS: <precise list or none>
MANUAL SIGNOFF: <reviewer/date/evidence or NOT_PERFORMED>
NEXT: <one next task whose dependencies really passed; else STOP>
```

Do not present sample commands, screenshots, timings, or PASS results as if executed. If only the planning ZIP is available, say `BLOCKED_SOURCE` and request the real source checkout for implementation.

## 6. QA gate before moving to E11

Every required M05 domain task must pass executed automated tests and numeric reference checks; every M05 FEBE task must pass interactive tests and be signed off. Test gem load, on-curve/surface, edit source path, Halo, pavé, Gem Map/Reporter, unit/history/Save-Open, Undo/Redo, cancel/error rollback. Record compatibility contract for E11 consumers (bezel/head/prong) and E12 cutters, without implementing those modules inside M05. Failures block E10 exit; a planning validator PASS alone is not sufficient.
