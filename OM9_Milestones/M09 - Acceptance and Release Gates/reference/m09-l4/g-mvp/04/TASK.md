# OM9-M09-L4-GMVP-004 — Build sạch và nhất quán trên môi trường đích

**Gate:** `G-MVP` · **Loại:** `BUILD_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Chứng minh artifact native được dựng từ đúng source SHA, ABI OCCT và toolchain khóa phiên bản.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Dùng checkout sạch; xác nhận Rust lockfile, OCCT headers/libs và phiên bản thực (M00 đã ghi mâu thuẫn 7.8.1 so với 7.9.3).
4. Build release từ môi trường tái tạo; ghi lệnh, exit code, hash mỗi binary/dll và warning.
5. Chạy test startup/smoke ứng dụng đúng OS/arch/GPU được tuyên bố hỗ trợ; kiểm dynamic library loading.
6. So sánh hai build tương đương theo manifest input; nếu binary không bit-reproducible phải giải thích và so artifact metadata.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-05-001` · `OM9-L2-01-05` · `E01` · `cap.01-05.occt-ffi-safe-handle-lifecycle`
- `OM9-L3-01-06-001` · `OM9-L2-01-06` · `E02` · `cap.01-06.gpu-device-and-render-loop`
- `OM9-L3-01-01-001` · `OM9-L2-01-01` · `E02` · `cap.01-01.application-startup-and-main-window`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-001`

## Acceptance

**Oracle cụ thể:** Release binary starts on each declared target with correct Rust and OCCT ABI; hash and clean-build inputs recorded; no missing runtime library.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Sai OCCT ABI hoặc thiếu DLL: FAIL.
- Không có target runner/hardware: BLOCKED_ENVIRONMENT.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-004-01` (ACCEPTANCE): Dùng checkout sạch; xác nhận Rust lockfile, OCCT headers/libs và phiên bản thực (M00 đã ghi mâu thuẫn 7.8.1 so với 7.9.3).
- `M09-TC-GMVP-004-02` (ACCEPTANCE): Build release từ môi trường tái tạo; ghi lệnh, exit code, hash mỗi binary/dll và warning.
- `M09-TC-GMVP-004-03` (ACCEPTANCE): Chạy test startup/smoke ứng dụng đúng OS/arch/GPU được tuyên bố hỗ trợ; kiểm dynamic library loading.
- `M09-TC-GMVP-004-04` (ACCEPTANCE): So sánh hai build tương đương theo manifest input; nếu binary không bit-reproducible phải giải thích và so artifact metadata.
- `M09-TC-GMVP-004-N01` (NEGATIVE): Sai OCCT ABI hoặc thiếu DLL: FAIL.
- `M09-TC-GMVP-004-N02` (NEGATIVE): Không có target runner/hardware: BLOCKED_ENVIRONMENT.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-004/clean-build-matrix.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- M00 SOURCE_AUDIT_V012; actual cargo and packaging configuration
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
