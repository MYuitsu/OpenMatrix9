# OM9-M09-L4-GMVP-014 — Kiểm đầu vào lỗi, recovery và hiệu năng

**Gate:** `G-MVP` · **Loại:** `ROBUSTNESS_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Đo khả năng chịu lỗi, sử dụng bộ nhớ và ngưỡng GPU/CPU dựa trên tiêu chí đã chốt ở G00.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Dùng corpus malformed 3DM/project/STL input nếu in-scope; kiểm bounded errors, crash, no harmful side effects.
4. Test context loss GPU, minimize/resize, fallback nếu được tuyên bố; quan sát ứng dụng/geometry cache sau phục hồi.
5. Chạy soak benchmark theo machine profile và baseline G00: startup, 4-view redraw, memory, OCCT operation time; báo p50/p95 nếu có dữ liệu.
6. Sử dụng ngưỡng được ký tại G00; nếu thiếu, để BLOCKED_THRESHOLD thay vì tùy tiện ra ngưỡng.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-06-001` · `OM9-L2-01-06` · `E02` · `cap.01-06.gpu-device-and-render-loop`
- `OM9-L3-01-05-001` · `OM9-L2-01-05` · `E01` · `cap.01-05.occt-ffi-safe-handle-lifecycle`
- `OM9-L3-01-12-001` · `OM9-L2-01-12` · `E02` · `cap.01-12.native-project-container-format`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-004`
- `OM9-M09-L4-GMVP-006`
- `OM9-M09-L4-GMVP-008`
- `OM9-M09-L4-GMVP-009`

## Acceptance

**Oracle cụ thể:** Malformed data and GPU recovery cases have explicit safe outcomes; all G00-approved stress/performance budgets are met on declared target devices.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Crash/data corruption/leak hay timeout vượt ngưỡng đã duyệt: FAIL.
- Hệ thống không có test GPU target: BLOCKED_ENVIRONMENT.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-014-01` (ACCEPTANCE): Dùng corpus malformed 3DM/project/STL input nếu in-scope; kiểm bounded errors, crash, no harmful side effects.
- `M09-TC-GMVP-014-02` (ACCEPTANCE): Test context loss GPU, minimize/resize, fallback nếu được tuyên bố; quan sát ứng dụng/geometry cache sau phục hồi.
- `M09-TC-GMVP-014-03` (ACCEPTANCE): Chạy soak benchmark theo machine profile và baseline G00: startup, 4-view redraw, memory, OCCT operation time; báo p50/p95 nếu có dữ liệu.
- `M09-TC-GMVP-014-04` (ACCEPTANCE): Sử dụng ngưỡng được ký tại G00; nếu thiếu, để BLOCKED_THRESHOLD thay vì tùy tiện ra ngưỡng.
- `M09-TC-GMVP-014-N01` (NEGATIVE): Crash/data corruption/leak hay timeout vượt ngưỡng đã duyệt: FAIL.
- `M09-TC-GMVP-014-N02` (NEGATIVE): Hệ thống không có test GPU target: BLOCKED_ENVIRONMENT.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-014/robustness-performance.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- G00 budget; CI perf reports
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
