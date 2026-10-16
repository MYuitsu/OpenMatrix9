# M07 L4 v0.13 — tự đánh giá sau khi tạo (planning QA, không phải runtime QA)

## Kết luận

**Kết quả:** tài liệu L4 **PASS static planning validation**, chưa đủ điều kiện mở coding/release. Mức tin cậy về tính toàn vẹn kế hoạch: **cao**; về đúng API nguồn và chạy thật: **chưa được xác nhận**.

| Tiêu chí | Kết quả đánh giá | Bằng chứng |
|---|---|---|
| Bao phủ scope E12 | **20/20 L3; 4/4 module owner** | filter `execution_stage=E12` từ manifest gốc và file `M07_L4_EXECUTION_MANIFEST.json` |
| Độ mịn để giao Luna | **69 L4**, 3 hoặc 4 tác vụ/L3 | 1 source/contract + 1 integration + 1 UI/BE acceptance + 9 adversarial gates |
| Phụ thuộc có thể sắp xếp | **PASS** — DAG 6 waves, không cycle, mọi link tồn tại | `tools/validate_m07_l4.py` |
| Ownership + ID giữ nguyên | **PASS** — 95 L2, 772 L3, 602 mainline, 170 AFTER; STL vẫn owner M01 nhưng chạy E12 | `validate_l3.py`, `validate_v010_plan.py` |
| Không overclaim | **PASS (static)** — toàn bộ status PLANNED; local source arrays rỗng, chưa ghi số đo/giấy phép giả | review schema + manifest; 0 implemented |
| Geometry & QA exit | **Đã chỉ định tiêu chí**, chưa có actual build/test | `l4.json`, `TASK.md`, `RELEASE_GATE.md` |
| Manual provenance | **Có refs**, các Book 2 pp.498–518 đã đọc; command alias chỉ verified ở 8 L3 | `origin_manual_refs`, `manual_command_verified_in_pdf` |
| Quy tắc Luna context | **PASS design** — 1 task/vòng, điều kiện dừng, output format, waves | `LUNA_START_HERE.md`, `GPT6_LUNA_TASK_PROMPT.md` |
| Đúng API QymCAD thực tế | **BLOCKED** — source v0.12 reviewed remote only, chưa checkout/build local | M00 v0.12 report; mọi L4 `reuse_decision=PENDING...` |
| Test/manufacturing readiness | **BLOCKED** — chưa có OCCT fixture test, independent STL viewer, manual sign-off | `RELEASE_GATE.md` |

## Chấm điểm thiết kế (không phải phần mềm)

- Bao phủ & traceability: **10/10**.
- Phân rã và khả năng giao Luna: **9/10**. Phần UI/service được tách bằng task phase; có lúc một task integration vẫn cần chia thêm sau source audit.
- Ràng buộc an toàn hình học/transaction: **9/10**. Chưa biết topology robustness thực tại pin source.
- Chứng cứ/QA/rollback: **9/10** ở cấp kế hoạch. Chưa test runtime.
- Mức sẵn sàng tự thực thi ngay: **2/10** vì G00/E01–E11 chưa có PASS trong ZIP và không kèm app/source chạy.

**Đánh giá tổng thể L4 planning: 9.2/10 (đánh giá chủ quan, không phải phép đo hiệu năng mô hình).** Không chấm release readiness vì chưa có test runtime.

## Những hạn chế còn lại được phát hiện khi tự review

1. **QymCAD pin/copy source:** chưa có source checkout local, nên `files_expected_to_touch` chỉ là placeholders có chủ đích. Sau khi audit phải chốt files & seam cụ thể trước PR.
2. **Kiểm thử Rust/OCCT:** chưa có numerical thresholds ký tại G00; mọi L4 cố tình không đặt magic tolerances. Phải tạo golden fixtures & measurement conditions.
3. **Tính năng cutter theo manual:** một số L3 diễn giải workflow từ chương, không có lệnh gõ trực tiếp độc lập; chỉ 8 L3 có alias thực đã xác minh; không được đăng ký alias tự chế.
4. **STL không lưu units trong bản thân định dạng:** phải quy ước export coordinate mm và ghi kèm metadata/cảnh báo; so sánh kích thước bằng parser/viewer độc lập. Tệp STL hợp lệ không chứng minh đủ tiêu chuẩn đúc/in mọi máy.
5. **Legacy Matrix boolean có thể phá History:** Book 2 p.517 nói rõ; L4 yêu cầu ghi một trong hai nhánh: tính liên kết được kiểm thử hoặc cảnh báo detach, không được hứa associative CAD không có test.
6. **Force Boolean:** manual Book 2 p.518 gợi ý move/scale; OpenMatrix9 phải không chỉnh hình học âm thầm. Chỉ cho phép user-acknowledged recovery và numeric diff.
7. **Legal risk:** các cutter presets/assets gốc không được coi là tái sử dụng được khi không có license. QymCAD AGPL và openNURBS cần reviewer.
8. **Nguồn code thực:** OpenMatrix9 ZIP đang là đặc tả và kế hoạch, chưa có working Rust/OCCT app. Không thể thực hiện GUI manual acceptance trong bản ZIP này.

## Cải tiến đã đưa vào sau tự review vòng một

- Tách README/start prompt, yêu cầu chỉ một L4/vòng và status format để Luna giữ context gọn.
- Cross-task dependency explicit, tránh coding GUI khi Boolean service chưa PASS.
- Tăng adversarial gates cho 9 capability có nguy cơ sai geometry, undo, STL hoặc history.
- Đồng bộ status E12 thành `PLANNED_NOT_EXECUTED`, không để manifest vẫn báo L4 chưa có.
- Đánh dấu command line alias **đã xác minh** tách khỏi UI label **chưa xác minh**.
- Nêu ngoại lệ `STL` trong M01 target M07 (không chuyển module).

## Hành động kế tiếp trước khi giao coding cho Luna

Chốt G00 reviewer → chạy build E01–E11 và xuất evidence → xác lập OCCT tolerance và fixture → dùng `GPT6_LUNA_TASK_PROMPT.md` chạy L4 theo wave, một L4/PR → gated manual/independent STL → hội đồng ký G-MVP.
