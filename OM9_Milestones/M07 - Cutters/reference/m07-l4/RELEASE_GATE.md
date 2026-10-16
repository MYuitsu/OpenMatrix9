# M07/E12 → G-MVP — Gate kiểm thử sản xuất (chưa chạy)

## Must-pass criteria

| Nhóm | Điều kiện nghiệm thu | Evidence bắt buộc |
|---|---|---|
| Gem/azure/bezel cutters | Cut/seat đúng GemRef với test shape; không làm hỏng gem và kích thước; mode/UI cập nhật | OCCT golden BRep, typed parameter fixture, actual vs expected |
| Channel/microprong/bright cut | End A/B, mirror, row orientation và spacing đúng, không self-intersect | multi-gem fixtures, topology/edge tests, screenshot GUI có tester |
| Boolean | Subtract/union/intersect đúng, không silent no-op; fail nguyên tử | API/integration logs, pre/post hashes, Undo/Redo |
| History | No silent detach; Save/Open không mất feature IDs, units, links; unsupported cases warned | migration/roundtrip tests, DAG diff |
| Manufacturability preflight | BRep valid, không naked edges ngoài intentional open shape, correct orientation | kernel diagnostics, issue IDs, independently reproduced negative fixtures |
| STL | mesh closed manifold, normals/winding nhất quán, size mm, file atomic | source BRep + STL hash, independent parser/viewer, bounds/volume comparison |
| GUI acceptance | 19 FEBE L3 có thao tác thật, failure feedback, preview/cancel | tester, date/build/OS/GPU, screenshots/video, manual PASS/FAIL |
| Legal | AGPL/OCCT/notices/assets và license được phê duyệt | SBOM, notices, reviewer sign-off |

## E2E bắt buộc

`Create Ring (E09) → Place Gems (E10) → Setting (E11) → Cutter (E12) → Boolean → geometry diagnostics → STL binary export → mở bằng viewer độc lập → đo bounding/units/shell validity`.

1. Dùng ít nhất fixture vòng có một đá và hàng nhiều đá; ghi hash/số mặt/units và dimensions expected từ fixture tự tạo.
2. Test thay đổi ring size/gem sau khi cắt: phải recompute qua history hoặc hiện warning cần rebuild; không tự nhận history vẫn intact.
3. Inject Boolean fail/degenerate/tangency; test preview cancellation, Undo/Redo, Save/Open, file write fault; không được có partial document hoặc file STL hỏng.
4. Export không được bỏ qua manifold/naked-edge check. Nếu file multi-shell phù hợp thực tế phải có policy kiểm định đã ký; không mặc định multi-shell luôn OK.
5. Reviewer ký `G-MVP` chỉ khi tất cả critical criteria PASS, không còn blocker license/geometry/persistence, và giới hạn được công bố.

## Cần phân biệt

- `STL exported` ≠ `manufacturing validated`.
- `BRep closed` ≠ `safe casting/printing` (thickness/resolution/process phụ thuộc manufacturing profile có bằng chứng).
- `source read` ≠ `local build PASS`.
- `READY_FOR_MANUAL_TEST` ≠ `DONE`.
- Công cụ `Force Boolean` trong Matrix Book 2 p.518 có thể dịch/scale cutter nhẹ; bản OpenMatrix9 **không tự động sửa hình học** hoặc làm dịch đồ trang sức không thông báo. Nếu triển khai recovery, đó là tính năng opt-in riêng, phải log numerical delta và user confirmation.
