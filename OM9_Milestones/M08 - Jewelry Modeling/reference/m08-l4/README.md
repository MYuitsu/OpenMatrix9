# M08 / E13 — 221 L4 dành cho GPT-6 Luna (kế hoạch, chưa triển khai)

**Đọc trước:** [LUNA_START_HERE.md](LUNA_START_HERE.md) → [GPT6_LUNA_TASK_PROMPT.md](GPT6_LUNA_TASK_PROMPT.md).  
**Chọn task bằng ID/dependency:** [M08_L4_EXECUTION_MANIFEST.json](M08_L4_EXECUTION_MANIFEST.json) hoặc [CSV](M08_L4_EXECUTION_MANIFEST.csv).  
**Exit gate:** [RELEASE_GATE.md](RELEASE_GATE.md); [SELF_REVIEW.md](SELF_REVIEW.md).

## Mục tiêu M08

E13: Advanced jewelry modeling, **73 L3 / 9 L2**. Đặc tả đã được chia thành **221 L4 / 8 waves**. 22 L3 thuộc module M03 `OM9-L2-04-01` thực thi tại E13: giữ nguyên milestone owner gốc, 2 L4/binding vì đây là GUI adapter, không có quyền viết thuật toán CAD mới. Các L3 còn lại thuộc M08, thường 3–4 L4: source+contract, owner implementation, transaction/UI, và adversarial gate cho 24 khả năng rủi ro cao.

| Owner module | Phạm vi | L3 | L4 |
|---|---|---:|---:|
| `OM9-L2-04-01` | Menu Transform binding sang owner service | 22 | 44 |
| `OM9-L2-04-04` | Smart Target / Smart Blend / MSR | 5 | 16 |
| `OM9-L2-04-05` | Smart Flow / UV / Rigid / History | 8 | 32 |
| `OM9-L2-04-06` | Knots / Jump Ring / Nautilus / Builder | 6 | 19 |
| `OM9-L2-04-07` | Orient / Array / Flow / Splop | 9 | 30 |
| `OM9-L2-04-08` | Deformations / Cage / Live symmetry | 8 | 27 |
| `OM9-L2-05-01` | Smart Pattern | 5 | 18 |
| `OM9-L2-05-02` | Rope | 5 | 18 |
| `OM9-L2-05-03` | User Library | 5 | 17 |
| **Tổng** | | **73** | **221** |

`l4.json` và `TASK.md` có đặc tả riêng theo tính năng, yêu cầu typed input, numerical oracle, negative cases, contract ownership, preconditions, full dependencies, tests/manual actions, rollback và evidence. Các phép hình học dùng Rust domain + OCCT qua FFI; GUI không thay kernel; không tự sản xuất preset thương mại của Gemvision.

## Tài liệu đã đối chiếu

- Matrix 8 Book 2 pp. 235–266: Transform, Smart Flow, Cage/Flow/Array/Orient; pp. 340–357: Ornament Builders; pp. 372–375: Smart Target, theo `origin_manual_refs` trong từng L3.
- Matrix 9 Addendum pp. 80–82: Smart Pattern / Rope (PDF trang 86–88); pp. 85–86: Styles / User Libraries (PDF trang 91–92). Những thông số được ghi trong từng L4 là input/acceptance dự kiến để Luna xác nhận bằng runtime, không phải xác nhận feature đã có trong QymCAD.
- Căn cứ pháp lý/nguồn: `planning/M00/SOURCE_AUDIT_V012.md` đọc từ xa QymCAD commit `c116f8f224ebffe0ada049c91c0d15df3634f00d`; **không phải source checkout/build local**.

## Điều kiện dừng bắt buộc

`G00 PASS → E01…E12 PASS → G-MVP SIGNED → E13 (M08) → G-FULL SIGNED`.

Không có bằng chứng G-MVP, local source, OCCT numerical fixtures/approved tolerances hay UI test: `BLOCKED`, không code giả để qua gate. Thực thi **một** L4/vòng theo `execution_wave`, chỉ khi mọi dependency task có PASS thật; `READY_FOR_MANUAL_TEST` chưa phải PASS.

## Chạy validators (chỉ xác nhận cấu trúc kế hoạch)

```sh
python tools/validate_l3.py
python tools/validate_v010_plan.py
python tools/validate_m00_l4.py
python tools/validate_m07_l4.py
python tools/validate_m08_l4.py
```

**Trạng thái:** 221 `PLANNED_NOT_EXECUTED`, 0 implementation, 0 runtime tested, 0 reviewer signed. ZIP v0.14 là kế hoạch; không chứa OpenMatrix9 source/build và không đính kèm tài liệu thương mại.
