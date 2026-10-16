# OpenMatrix9 — v0.11 M00 L4 source-audit extension over v0.10 execution plan

> Đây là **bản tái lập kế hoạch từ v0.9**, không chứa mã nguồn phần mềm, không có L4 đã triển khai, không xác nhận QymCAD. Tên v0.10 nói về **planning/spec edition**, không phải phiên bản app.

**M00 L4 (mới):** [M00/README.md](M00/README.md) — 7 feasibility gates, 19 planning L3, 73 actionable L4 (xem manifest để lấy số chính xác). Không đổi 95 L2/772 L3 mainline.

## Quyết định nền tảng

1. Giữ **95 L2 và 772 L3** cùng canonical `module_id/l3_id/capability_id` và đường dẫn sở hữu. Không duplicate chức năng, không chuyển file L3 gây đứt ref.
2. Giữ `milestone_id` = milestone **sở hữu**, `target_milestone` = milestone **đã dự kiến triển khai** từ v0.9; dùng `execution_stage` mới để Codex biết **chính xác đợt làm**.
3. Bổ sung cổng **G00/M00**: source audit, licensing, GUI/renderer chọn thực nghiệm, OCCT–Rust FFI, openNURBS, BRep/trimming, topology history proof. Có blockers chưa PASS thì KHÔNG tạo coding task hàng loạt.
4. M03 chia thành E04–E08 (vertical slice, Curve, Surface, Solid/Transform, 3DM interop).
5. Tách **G-MVP** sau E12 và **G-FULL** sau E13. M08 không chặn MVP, nhưng phải PASS trước bản full.
6. **AFTER 170 L3** vẫn được bảo lưu, tuyệt đối không phụ thuộc ngược từ mainline. Clayoo, T-Splines, Emboss, Render/CAM chưa thuộc hai release.

## Bảng đợt triển khai (chính xác theo 772 L3)

| Stage | Milestone owner | L3 | Phạm vi | Điều kiện vào |
|---|---|---:|---|---|
| [E01](E01.md) | M01 | 42 | Kernel & domain contracts | G00 PASS |
| [E02](E02.md) | M01 | 74 | App shell, four viewports, native persistence | E01 PASS |
| [E03](E03.md) | M02 | 59 | Viewport interaction | E02 PASS |
| [E04](E04.md) | M03A | 15 | CAD vertical slice | E03 PASS |
| [E05](E05.md) | M03B | 66 | Curve engine and command parity | E04 PASS |
| [E06](E06.md) | M03C | 56 | Surface and trimmed NURBS | E05 PASS |
| [E07](E07.md) | M03D | 71 | Solid, boolean, transforms and health | E06 PASS |
| [E08](E08.md) | M03E | 12 | 3DM import and interoperability | E07 PASS |
| [E09](E09.md) | M04 | 52 | Ring builder and associative history | E08 PASS |
| [E10](E10.md) | M05 | 43 | Gems and pavé layout | E09 PASS |
| [E11](E11.md) | M06 | 19 | Head, bezel, prong and metal settings | E10 PASS |
| [E12](E12.md) | M07 | 20 | Cutters, STL and production validation | E11 PASS |
| [E13](E13.md) | M08 | 73 | Advanced jewelry modeling | G-MVP PASS |
| AFTER | 90-after | 170 | Không thuộc mainline | Chỉ khởi động theo quyết định sản phẩm |

**M00/G00:** audit trước E01. **G-MVP:** sau E12. **G-FULL:** sau E13. M09 sở hữu gates, không thêm L2 implementation.

## Hồ sơ trong bản ZIP

- `planning/G00-M00_FEASIBILITY.md`: checklist kỹ thuật/giấy phép cần proof.
- `planning/E01.md` ... `E13.md`: việc theo đợt và exit gates.
- `planning/EXECUTION_MANIFEST.csv|json`: 772 hàng, lọc theo `execution_stage`.
- `planning/M03_SPLIT.md`: ranh giới giao lệnh giữa stages.
- `planning/DEPENDENCY_AND_RELEASE_POLICY.md`: ownership, dependencies, release gates.
- `planning/GATES_AND_EVIDENCE.md`: điều kiện pass, bug triage, artifacts.
- `planning/CODEX_TASK_PROTOCOL.md` và `planning/L4_TEMPLATE.md`: giao Codex đúng cách.
- `planning/RISK_REGISTER.csv`: blocker/fallback để ra quyết định.
- `tools/validate_v010_plan.py`: validation chạy local, dùng cùng `tools/validate_l3.py` gốc.

## Lưu ý

- `24` L3 có `milestone_id != target_milestone` được **giữ có chủ đích** để một chủ sở hữu duy nhất quản lý đa đợt (1 từ M01→M07, 1 từ M03→M04, 22 từ M03→M08).
- Không tự suy ra ngày công; `implementation_estimate` để trạng thái `UNESTIMATED_PENDING_SOURCE_AUDIT`.
- **772 L3 không tương đương 772 tính năng hoàn chỉnh**: có binding giao diện, adapters, framework infrastructure.
- MVP không hỗ trợ export 3DM trừ khi có L3 riêng được thêm và kiểm chứng; import 3DM cũng bị chặn bởi feasibility.

**Lưu ý critical path:** 13 đợt không tương đương 13 lần prompt; E05–E07 là các đợt lớn cần chia L4 tiếp, triển khai và test nhiều vòng. Không để AI nhận cùng lúc 602 L3.

## M06/E11 L4 phân rã (bổ sung không thay đổi canon)

Bộ [M06 L4 cho GPT-6 Luna](M06/README.md) bổ sung 76 planned work orders cho 19 L3/2 L2, manifest riêng và kiểm tra DAG. `specs/**/l3.json` và `planning/EXECUTION_MANIFEST.*` được giữ nguyên; `E10` và `G00` vẫn là blockers trước khi code.

## M09 release acceptance L4 (phần bổ sung kế hoạch)

[M09/G-MVP/G-FULL release L4](M09/README.md) gồm 30 work orders kiểm định (18 MVP / 12 FULL), không thêm L2/L3; tất cả chưa thực thi, cần app thật và human signoff.
