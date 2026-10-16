# OM9-M06-L4-15-004-01 — Metal Piece From Gems: Chốt dữ liệu/API và fixture

**Owner:** `OM9-L2-04-15` · **L3:** `OM9-L3-04-15-004` · **capability:** `cap.04-15.metal-piece-from-gems` · **E11/M06**
**Stack:** Rust + OCCT qua FFI đã được audit; đọc `planning/M06/M06_IMPLEMENTATION_CONTRACT.md`.
**State:** `PLANNED_NOT_EXECUTED`; G00/E10 chưa được chứng minh PASS. Không có source implementation được xác nhận từ ZIP.

## Đầu vào và nguồn

- Canonical spec: `specs/06-ms06-settings-and-prongs/02-febe-prong-channel-metal-and-bead-settings/004-febe-metal-piece-from-gems/l3.json`.
- Matrix 8 Book 2: printed 490/pdf 264 (section_heading); printed [468, 497]/pdf [242, 271] (parent_l2_range); target PDF pages [264, 265, 266, 267, 268].
- Sách 2 trang in 490–494, Metal from Gems.
- Input: ordered gem row open/closed; feature params; optional channel border.
- Controls: open/closed modes; gem offset/end offset; wall/height/profile end widths.

## Phạm vi đúng 1 thay đổi

Đọc L3, manual và mã nguồn thực, mô tả typed request/result, giới hạn ownership, kiểm chứng dependencies và tạo fixtures theo provenance. Không cài đặt hình học hay GUI trong task này.

**Target invariant:** metal rim/channel walls đúng offsets, không lẫn cutter với metal object.

## Thứ tự làm, từng bước

1. Đọc specs/06-ms06-settings-and-prongs/02-febe-prong-channel-metal-and-bead-settings/004-febe-metal-piece-from-gems/l3.json và tài liệu matrix_8_manual_book2.pdf trang PDF [264, 265, 266, 267, 268]; phân biệt manual fact và engineering proposal.
2. Khoanh input/constraints của Metal Piece from gems; đọc các owner-module dependencies ['OM9-L2-04-10', 'OM9-L2-04-14'].
3. Ở source checkout thực, chạy git rev-parse HEAD, đọc AGENTS.md nếu có, tìm API, phân loại REUSE/ADAPT/ADD và ghi code path + signature.
4. Chốt typed request/result/error, feature references, units/tolerance from G00; xác nhận E10 gem contract, không tạo API giả.
5. Soạn tối thiểu 3 fixtures VALID/INVALID/UPDATE: provenance, source SHA, expected numeric properties và test owner.

## Dependencies

- Gate ngoài: G00_PASS + E10_PASS + frozen gem input contract.
- L4 nội bộ: không có; được phép đọc/sửa spec trước gates, KHÔNG code runtime.

## Acceptance / tests

- [ ] Schema/contract linter verifies required fields, no missing source provenance, fixture metadata non-empty
- [ ] Cross-module call graph ownership check: no 90-after dependency, no geometry invoked from UI
- [ ] Attempt negative fixture in contract validator; reject unsupported/invalid params without code stub

## Manual UI

Không nghiệm thu GUI ở task contract/BE này; task FEBE/QA sẽ kiểm tra sau.

**Boundary M06→E12:** `Metal Piece > Top` mô tả profile tương tự cutter; sinh geometry/preview trung gian, không nhân bản cutter/boolean manufacturing đã thuộc E12.

## Lỗi phải xử lý

- row discontinuity; self-crossing offsets; thickness zero; closed-loop seam drift.
- Panic/kernel failure/Cancel phải rollback, không thay đổi model/ID/scene một phần.

## Bằng chứng và Done

- [ ] Contract and fixture manifests reviewed and stored at declared evidence paths
- [ ] Known source paths+ABI signatures locally verified; no unverified REUSE claim
- [ ] Local source/audit evidence provided; coding remains BLOCKED until G00/E10 PASS

**Evidence paths (planned, not created):** `implementation/evidence/M06/OM9-L3-04-15-004/contract-and-source-audit.json`, `implementation/fixtures/M06/OM9-L3-04-15-004/fixture-manifest.json`

**Khi thiếu điều kiện:** ghi `BLOCKED_SOURCE`, `BLOCKED_GATE`, `BLOCKED_REFERENCE` hoặc `BLOCKED_TOOLCHAIN`, lưu thông tin có thể tái hiện; không viết mock/stub để tuyên bố PASS.

L4 metadata chi tiết: [l4.json](l4.json).
