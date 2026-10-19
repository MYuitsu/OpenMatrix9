# OM9-M06-L4-14-001-01 — Head Builder Basket: Chốt dữ liệu/API và fixture

**Owner:** `OM9-L2-04-14` · **L3:** `OM9-L3-04-14-001` · **capability:** `cap.04-14.head-builder-basket` · **E11/M06**
**Stack:** Rust + OCCT qua FFI đã được audit; đọc `planning/M06/M06_IMPLEMENTATION_CONTRACT.md`.
**State:** `PLANNED_NOT_EXECUTED`; G00/E10 chưa được chứng minh PASS. Không có source implementation được xác nhận từ ZIP.

## Đầu vào và nguồn

- Canonical spec: `specs/06-ms06-settings-and-prongs/01-febe-head-and-bezel-brep-settings/001-febe-head-builder-basket/l3.json`.
- Matrix 8 Book 2: printed 469/pdf 243 (section_heading); printed [468, 497]/pdf [242, 271] (parent_l2_range); target PDF pages [243, 244, 245].
- Sách 2 trang in 469–471, Head Builder và Level 1.
- Input: một gem có girdle, pavilion, normal hợp lệ; số chân; profile; số/lớp rail.
- Controls: head mode, prong count, profile, head angle, rail drop, height above girdle.

## Phạm vi đúng 1 thay đổi

Đọc L3, manual và mã nguồn thực, mô tả typed request/result, giới hạn ownership, kiểm chứng dependencies và tạo fixtures theo provenance. Không cài đặt hình học hay GUI trong task này.

**Target invariant:** dựng chân và gallery rails theo hệ tọa độ viên đá, không chọc xuyên pavilion; solid hợp lệ.

## Thứ tự làm, từng bước

1. Đọc specs/06-ms06-settings-and-prongs/01-febe-head-and-bezel-brep-settings/001-febe-head-builder-basket/l3.json và tài liệu matrix_8_manual_book2.pdf trang PDF [243, 244, 245]; phân biệt manual fact và engineering proposal.
2. Khoanh input/constraints của Basket Head; đọc các owner-module dependencies ['OM9-L2-04-09', 'OM9-L2-01-09', 'OM9-L2-03-11'].
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

## Lỗi phải xử lý

- gem thiếu girdle; profile mở; chân cắt nhau; góc gây self-intersection.
- Panic/kernel failure/Cancel phải rollback, không thay đổi model/ID/scene một phần.

## Bằng chứng và Done

- [ ] Contract and fixture manifests reviewed and stored at declared evidence paths
- [ ] Known source paths+ABI signatures locally verified; no unverified REUSE claim
- [ ] Local source/audit evidence provided; coding remains BLOCKED until G00/E10 PASS

**Evidence paths (planned, not created):** `implementation/evidence/M06/OM9-L3-04-14-001/contract-and-source-audit.json`, `implementation/fixtures/M06/OM9-L3-04-14-001/fixture-manifest.json`

**Khi thiếu điều kiện:** ghi `BLOCKED_SOURCE`, `BLOCKED_GATE`, `BLOCKED_REFERENCE` hoặc `BLOCKED_TOOLCHAIN`, lưu thông tin có thể tái hiện; không viết mock/stub để tuyên bố PASS.

L4 metadata chi tiết: [l4.json](l4.json).
