# M09 — Release acceptance L4 cho GPT-6 Luna (planning only)

- **M09 không sở hữu L2/L3.** Canonical `95 L2 / 772 L3` bất biến; release chỉ liên kết L3 có sẵn và kiểm định.
- **Tổng:** `30 atomic release L4`: 18 G-MVP (sau E12) và 12 G-FULL (sau signed G-MVP + E13).
- **Trạng thái:** 30/30 `PLANNED_NOT_EXECUTED`, 0 runtime test PASS, không có signed release gate.
- ZIP này là **specs + planning**, không có Rust application source. Nguồn QymCAD pinned M00 mới được remote reviewed, chưa chứng minh build/OCCT ABI.

## Bắt đầu

1. [`GPT6_LUNA_EXECUTION_PROMPT.md`](GPT6_LUNA_EXECUTION_PROMPT.md) — copy/paste vào GPT-6 Luna có repo app thực.
2. [`M09_EXECUTION_MANIFEST.json`](M09_EXECUTION_MANIFEST.json) / [`M09_EXECUTION_MANIFEST.csv`](M09_EXECUTION_MANIFEST.csv) — danh sách 30 L4, dependency, canonical references, expected evidence.
3. [`M09_EXECUTION_TREE.txt`](M09_EXECUTION_TREE.txt) — thứ tự topo đề xuất; agent vẫn phải kiểm signed gate trước.
4. [`G_MVP_CHECKLIST.md`](G_MVP_CHECKLIST.md) và [`G_FULL_CHECKLIST.md`](G_FULL_CHECKLIST.md) — checklist release chưa đánh dấu.
5. [`M09_EVIDENCE_TEMPLATE.json`](M09_EVIDENCE_TEMPLATE.json) và [`M09_GATE_SIGNOFF_TEMPLATE.json`](M09_GATE_SIGNOFF_TEMPLATE.json) — chỉ biểu mẫu; không là proof.
6. `python tools/validate_m09_l4.py` — kiểm metadata/G-MVP/G-FULL DAG/CSV/parent IDs/no fake L2.
7. `python tools/m09_next_task.py --state evidence_state.json --gate G-MVP` — helper chọn đúng task tiếp theo sau khi được cung cấp bản ghi signed evidence thực; file `M09_RUNTIME_STATE_TEMPLATE.json` không được coi là evidence.
8. [`M09_TEST_CASES.csv`](M09_TEST_CASES.csv) — 177 ca kiểm thử có ID và oracle phụ thuộc fixture đã phê duyệt.

## Quy tắc thiết kế

- Release verification ≠ triển khai chức năng CAD. Không sửa source code trong owner M09; bug phải chuyển về đúng L2 qua canonical trace.
- Mỗi task có input, workflow, negative cases, evidence đích, prerequisites và acceptance riêng. Không khẳng định command/test file có thật khi chưa audit checkout.
- Bắt buộc human sign-off, legal review, OCCT BRep numeric oracle, STL viewer độc lập, import 3DM scope, regression MVP bất biến.
- Giữ trạng thái BLOCKED khi chưa có signed G00–E12 / E13, build SHA, toolchain, fixtures, viewer, reviewer.
- Không waive data corruption, invalid solid, history detach, sai đơn vị, security blocker hay giấy phép chưa rõ.

**Nguồn:** `planning/M09_RELEASE_GATES.md`, `planning/GATES_AND_EVIDENCE.md`, `planning/DEPENDENCY_AND_RELEASE_POLICY.md`, execution manifest E01–E13, manual Rhino 5 / Matrix 8 / Matrix 9 addendum ở mức tham khảo workflow.
