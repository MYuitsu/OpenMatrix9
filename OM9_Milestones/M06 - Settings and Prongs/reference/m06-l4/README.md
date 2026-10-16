# M06/E11 — Settings, Prongs & Bezel L4 task pack for GPT-6 Luna

**Trạng thái:** kế hoạch L4, chưa triển khai/chưa chạy test. **Coverage:** 2 canonical L2, 19 canonical L3, 76 atomic L4 (mỗi L3: contract → backend → FEBE → QA). Tên GPT-6 Luna là cấu hình tác nhân được người dùng sử dụng; nhiệm vụ vẫn phải kiểm chứng bằng công cụ, không tự gắn PASS.

## Cách dùng (không bỏ qua G00/E10)

1. Đọc [`GPT6_LUNA_EXECUTION_PROMPT.md`](GPT6_LUNA_EXECUTION_PROMPT.md), `planning/README.md`, `planning/CODEX_TASK_PROTOCOL.md`, `planning/E11.md`, `planning/GATES_AND_EVIDENCE.md` và `planning/DEPENDENCY_AND_RELEASE_POLICY.md`.
2. Chạy `python tools/validate_l3.py && python tools/validate_v010_plan.py && python tools/validate_m00_l4.py && python tools/validate_m06_l4.py` tại repo root.
3. Kiểm gate G00 và E01–E10; nếu chưa PASS, chỉ đọc source/chuẩn bị spec/fixture. **Không lập trình stage E11 khi gate chưa đạt**.
4. Dựa vào [`M06_EXECUTION_MANIFEST.json`](M06_EXECUTION_MANIFEST.json) đọc `depends_on_task_ids`; topo thứ tự đề xuất có trong [`M06_EXECUTION_TREE.txt`](M06_EXECUTION_TREE.txt).
5. Với một L4 đã đủ phụ thuộc, mở `TASK.md` và `l4.json`; trước khi code, audit source checkout + toolchain Rust/OCCT + implementation contract E10. Chỉ chọn REUSE/ADAPT/ADD từ bằng chứng có thật.
6. Thực hiện BE tests rồi FEBE UI, đo BRep IsValid/edges/tolerance, Undo/Redo/Cancel và save/open. Chỉ đánh dấu DONE khi evidence + người kiểm thử (FEBE) xác nhận.

## Các module

- [`OM9-L2-04-14`: Head, Bezel, geometry](01-head-and-bezel/README.md) — 9 L3 / 36 L4
- [`OM9-L2-04-15`: Prong, Metal, Bead](02-prong-metal-and-bead/README.md) — 10 L3 / 40 L4

## Các tệp vận hành

- [`M06_EXECUTION_MANIFEST.csv`](M06_EXECUTION_MANIFEST.csv) / [JSON](M06_EXECUTION_MANIFEST.json): 76 work orders gắn chính xác parent/owner/dependencies.
- [`M06_EXECUTION_TREE.txt`](M06_EXECUTION_TREE.txt): topological order được kiểm tra.
- [`M06_MANUAL_TRACE.csv`](M06_MANUAL_TRACE.csv): trace manual refs; phân biệt section_heading và parent_l2_range.
- [`M06_EVIDENCE_TEMPLATE.json`](M06_EVIDENCE_TEMPLATE.json): log PASS/FAIL/BLOCKED ghi sau khi chạy thực.
- [`M06_IMPLEMENTATION_CONTRACT.md`](M06_IMPLEMENTATION_CONTRACT.md): typed Rust/OCCT seam đề xuất, phân biệt BRep, curve, instance và persistence.
- [`M06_GATE_CHECKLIST.md`](M06_GATE_CHECKLIST.md): checklist nghiệm thu E11 chưa được ký.
- [`M06_REVIEW_REPORT.md`](M06_REVIEW_REPORT.md): kết quả đánh giá thiết kế sau khi tạo, không phải nghiệm thu code.

**Cảnh báo:** ZIP gốc chỉ chứa kế hoạch và đặc tả, không chứa workspace ứng dụng; các đường dẫn `implementation/*` chỉ là **deliverable dự kiến**, chưa tồn tại. Các ngưỡng sai số và benchmark phải lấy từ G00, không bịa số. Các phần After (Clayoo/T-Splines/Emboss) không có trong M06.
