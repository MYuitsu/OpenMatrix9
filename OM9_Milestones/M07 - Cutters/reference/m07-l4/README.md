# M07 / E12 — 69 L4 đã lập kế hoạch (chưa thực thi)

**Bắt đầu cho GPT-6 Luna:** [LUNA_START_HERE.md](LUNA_START_HERE.md).  
**Lấy task theo dependency:** [M07_L4_EXECUTION_MANIFEST.csv](M07_L4_EXECUTION_MANIFEST.csv) / [JSON](M07_L4_EXECUTION_MANIFEST.json).  
**Prompt giao task:** [GPT6_LUNA_TASK_PROMPT.md](GPT6_LUNA_TASK_PROMPT.md).  
**Nghiệm thu:** [RELEASE_GATE.md](RELEASE_GATE.md).  
**Đánh giá thiết kế:** [SELF_REVIEW.md](SELF_REVIEW.md).

## Phạm vi

- `E12` sở hữu 20 L3, chia thành **69 L4** và **6 đợt phụ thuộc** (waves), không bắt Luna xử lý cả 69 một lúc.
- `OM9-L2-01-12`: 4 L4 / 1 L3, STL export codec (L3 gốc thuộc M01, lịch E12/M07).
- `OM9-L2-04-16`: 24 L4 / 7 L3, gem/azure/bezel cutter.
- `OM9-L2-04-17`: 19 L4 / 6 L3, channel, microprong, bright cut.
- `OM9-L2-04-18`: 22 L4 / 6 L3, finger/plane/quad, Boolean, history, watertight check.

Mỗi L4 có `l4.json` (đọc máy) + `TASK.md` (agent thực thi): task ID, parent L3, owner L2, input/output, dependencies, source audit, manual refs, API seam, steps, tests, rollback, fixtures, evidence, manual acceptance.

## Trạng thái đúng

**69 task = PLANNED_NOT_EXECUTED, 0 implemented, 0 tested, 0 approved.**

Manual Matrix 8 Book 2 pp. 498–518 (PDF 272–292) đã đối chiếu để hình thành kế hoạch, không chép tài nguyên vào source. L3 STL dùng phạm vi refs Book 1 gốc; **chưa có trang độc lập chứng thực code export**. Source audit QymCAD v0.12 mới đọc remote tại SHA `746a17f127897e921a57118b6caad08968602bf5`, **chưa clone/build/test local**.

## Không được bắt đầu coding nếu chưa qua gate

`G00 PASS → E01 ... E11 PASS → E12 → G-MVP`.

Thiếu G00, OCCT FFI, legal review, E11, source local, golden fixtures hoặc chủ sở hữu API chưa rõ: trả `BLOCKED`; không tự code stub để chuyển trạng thái.

## Kiểm định planning

```sh
python tools/validate_l3.py
python tools/validate_v010_plan.py
python tools/validate_m00_l4.py
python tools/validate_m07_l4.py
```

Tên script `validate_v010_plan.py` giữ lại để tương thích đường dẫn cũ; logic đã cho phép E12 L4 planned trong v0.13.
