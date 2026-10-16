# OM9-M09-L4-GMVP-008 — Lưu/mở, migration và bảo vệ tệp

**Gate:** `G-MVP` · **Loại:** `PERSISTENCE_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Kiểm khả năng bảo toàn document, units, BRep, feature params/refs và atomic writes khi gặp lỗi.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Ghi tài liệu mẫu gồm layers, units, gem metadata, solids và history; save/open qua clean process rồi so deterministic semantics.
4. Thử tệp cũ theo migration range đã công bố; unknown schema phải lỗi minh bạch, không overwrite.
5. Giả lập mất quyền ghi/đầy ổ/gián đoạn tiến trình khi save; original file phải còn nguyên hoặc có recovery có thể xác minh.
6. Đối chiếu stable IDs, instance references, feature DAG, object transforms trước và sau mở.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-12-001` · `OM9-L2-01-12` · `E02` · `cap.01-12.native-project-container-format`
- `OM9-L3-01-12-002` · `OM9-L2-01-12` · `E02` · `cap.01-12.scene-and-layer-state-encoding`
- `OM9-L3-01-12-003` · `OM9-L2-01-12` · `E02` · `cap.01-12.brep-mesh-and-instance-encoding`
- `OM9-L3-01-04-001` · `OM9-L2-01-04` · `E09` · `cap.01-04.feature-record-and-typed-dependency-edges`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-007`
- `OM9-M09-L4-GMVP-004`

## Acceptance

**Oracle cụ thể:** Save/Open preserves declared scene, units, geometry, history and IDs semantically; failed write preserves last valid file; migration or unsupported formats are explicit.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Mất dữ liệu, silent truncation hoặc ghi đè file lỗi: CRITICAL_FAIL.
- Nếu backward compatibility chưa được hỗ trợ, phải ghi limitation và có phê duyệt release.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-008-01` (ACCEPTANCE): Ghi tài liệu mẫu gồm layers, units, gem metadata, solids và history; save/open qua clean process rồi so deterministic semantics.
- `M09-TC-GMVP-008-02` (ACCEPTANCE): Thử tệp cũ theo migration range đã công bố; unknown schema phải lỗi minh bạch, không overwrite.
- `M09-TC-GMVP-008-03` (ACCEPTANCE): Giả lập mất quyền ghi/đầy ổ/gián đoạn tiến trình khi save; original file phải còn nguyên hoặc có recovery có thể xác minh.
- `M09-TC-GMVP-008-04` (ACCEPTANCE): Đối chiếu stable IDs, instance references, feature DAG, object transforms trước và sau mở.
- `M09-TC-GMVP-008-N01` (NEGATIVE): Mất dữ liệu, silent truncation hoặc ghi đè file lỗi: CRITICAL_FAIL.
- `M09-TC-GMVP-008-N02` (NEGATIVE): Nếu backward compatibility chưa được hỗ trợ, phải ghi limitation và có phê duyệt release.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-008/save-open-migration.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- M00 source audit of QymCAD .qcad compatibility
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
