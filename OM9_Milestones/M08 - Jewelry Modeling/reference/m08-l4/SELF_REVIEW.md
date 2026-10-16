# M08/E13 L4 v0.14 — Self-review sau khi tạo (planning QA)

## Kết luận

**PASS: static planning integrity; BLOCKED: implementation readiness.** Tài liệu M08 dùng được làm kế hoạch theo task cho GPT-6 Luna nhưng hiện chưa thể coding hợp lệ do không có G00, E01–E12 và G-MVP reviewer signed evidence, chưa checkout/build nguồn QymCAD/OCCT local. Không có lý do nào để ghi `DONE` cho một L4.

| Tiêu chí | Kết quả | Bằng chứng hoặc thiếu sót |
|---|---|---|
| E13 scope | **PASS: 73/73 L3, 9/9 L2** | E13 canonical manifest; phân phối 22+5+8+6+9+8+5+5+5 |
| L4 granularity | **221 tác vụ** | 73 source/contract, 51 domain implementation, 51 FEBE acceptance, 22 UI adapters, 24 adversarial gates |
| DAG / cross-owner | **PASS: 8 waves**, no cycles | `tools/validate_m08_l4.py`, strict `wave(dep) < wave(task)` |
| Original ownership | **PASS** | 95 L2, 772 L3 unique, giữ 22 M03-owned transform L3 target M08; 24 old milestone/target exceptions không đổi |
| No M00/M07 regression | **PASS (static)** | `validate_m00_l4.py`, `validate_m07_l4.py`; không sửa byte tài liệu riêng trong `planning/M07/` |
| L3/Registry/CSV/JSON trace | **PASS** | `validate_l3.py`, `validate_v010_plan.py`, `validate_m08_l4.py`; 290 L4 planned M07+M08, 0 implemented |
| Domain test specificity | **PASS design, chưa chạy** | Mỗi L4 có typed inputs, feature-specific positive oracle, negative case, stable IDs/history/undo, manual test |
| Menu / duplicated geometry | **PASS design, chưa chạy** | 22 bindings chỉ dispatch đến `04-05`/`04-07`/`04-08`, không tạo trùng BRep functions |
| Manuals | **Partial** | Book2 Transform/Smart Flow + Addendum Smart Pattern/Rope/Library đối chiếu bằng text; toàn bộ từng `origin_manual_refs` inherited từ canonical L3, không cam kết đã verify độc lập từng alias/page |
| Runtime source API | **BLOCKED** | QymCAD SHA candidate chỉ remote review v0.12, không có local pinned checkout, ABI/OCCT build, geometry fixtures |
| Legal/license | **BLOCKED** | Cần license review QymCAD/OCCT/third-party; không được tái sử dụng Gemvision assets/styles/presets chỉ vì chúng được nhắc trong manual |
| Human FEBE/G-FULL | **NOT RUN** | Thiếu executable app, manual tester, screenshot, reviewer signed G-MVP & G-FULL |

## Số liệu kiểm thử tĩnh đã chạy

1. `python tools/validate_l3.py` — PASS: 95 L2/772 L3.
2. `python tools/validate_v010_plan.py` — PASS: canonical owners, 602 mainline/170 After, 24 owner vs target exceptions.
3. `python tools/validate_m00_l4.py` — PASS: 73 planning L4 G00, không nhận là đã thực thi.
4. `python tools/validate_m07_l4.py` — PASS: 69 planned M07 L4, phụ thuộc 6 waves; byte-level M07 task docs giữ nguyên.
5. `python tools/validate_m08_l4.py` — PASS: 221 planned L4, 73 L3, all paths, DAG, statuses, CSV/JSON, 8 waves.

**Wave size (số L4):** `1:73, 2:17, 3:33, 4:41, 5:30, 6:18, 7:6, 8:3`. Wave 1 là source/contract; không có nghĩa 73 task được thực hiện đồng thời. `PLANNED_NOT_EXECUTED=221/221`, tested/approved=0.

## Đánh giá thiết kế (chủ quan; không phải test model/runtime)

| Hạng mục | Điểm |
|---|---:|
| Bao phủ L3 và minh bạch owner/ID | 10/10 |
| Phân rã, khả năng giao từng lượt cho Luna | 9/10 |
| Độ cụ thể input/oracle/negative case | 9/10 |
| Phụ thuộc và release gates | 9.5/10 |
| Chứng cứ, fault injection và regression MVP | 9/10 |
| Khả năng tự coding ngay trên ZIP planning | **1/10** |

**Điểm tổng quan chất lượng kế hoạch: 9.2/10** (đánh giá chủ quan, không phải kết quả so sánh GPT-6 Luna trên các lượt chạy thực). **Không** đưa điểm này thành chỉ số chất lượng phần mềm hoặc release readiness.

## 8 phát hiện và giới hạn phải bảo lưu

1. **Không có app source:** ZIP v0.14 không chứa local QymCAD working tree hay bản build OpenMatrix9; mỗi `files_expected_to_touch` tránh đặt tên crate giả. API service thực tế phải audit, build và ký contract trước code.
2. **Manual cross-version:** Matrix 8 Book2 mô tả T-Splines và các tool khác không tự động đồng nghĩa Matrix9 Addendum; các tác vụ M08 được lấy từ E13 L3 canonical. Ứng dụng chưa xác thực rằng tên alias gốc gọi được trong QymCAD.
3. **History/UV:** Mapping Smart Flow khó vì trimmed UV, normal flip, seam, persistent topology naming và dịch chuyển rigid gems. Các L4 chỉ mô tả test oracle; chưa chứng minh OCCT qua thực nghiệm.
4. **Hình học đòi numeric oracle:** Geometry correctness phải dùng OCCT validation và signed tolerances. Không được lấy ảnh viewport làm xác nhận; không đặt magic tolerances hoặc runtime FPS giả.
5. **L3 có scope nhiều toán tử:** Deformation `Twist And Maelstrom`, `Stretch And Taper` và các Builder tổng hợp có thể phải tiếp tục tách L4 sau khi source audit. Hiện đã chia contract→service→UI→adversarial tương đối chặt nhưng không bảo đảm tối ưu theo files touched thực.
6. **Rope & manufacturing:** Cap=None là mặt mở theo manual, không có nghĩa xuất STL watertight. Model validity/3D-print safety không được tự suy ra từ Boolean hoặc render.
7. **User library security/rights:** Path traversal, symlink, corrupt assets, atomic save, ID remapping, incompatible .qcad/.3dm cần kiểm thử; asset thương mại không được kèm/copy.
8. **MVP preservation:** E13 chỉ khởi động sau G-MVP signed; regression E13 phải giữ ring/gem/cutter/STL/3DM baseline. `G-FULL` cần reviewer riêng; không gộp ký hai gates.

## Những cải tiến sau review so với M07

- Scale planning M07 20→M08 73 L3, nhưng tránh lãng phí bằng 2-phase adapter và 3/4-phase domain tasks thay vì 4 task cho mọi L3.
- Cross-owner dependency tường minh: transform adapter chờ owner Smart Flow, Orient/Array, Deformation. DAG topologically layered thay vì dựa vào thứ tự tên folder.
- Với từng L3 trong 51 L3 domain: thêm đặc tả riêng typed inputs, observable oracle, error pathology, manual + Undo/Save, không dùng mỗi generic smoke test.
- 24 task phase 4 tập trung ở UV/history/rigid/live mirror/library IO/attractor/rope failure, kèm baseline G-MVP regression.
- Schema/planning thống nhất M00/M07/M08 và không làm mất tương thích validation cũ; M07 task content giữ nguyên.

## Điều kiện xuất phát cho Luna

Dùng `planning/M08/LUNA_START_HERE.md` và `planning/M08/GPT6_LUNA_TASK_PROMPT.md`. Trước yêu cầu triển khai, cần **bằng chứng ký G-MVP**, local source SHA và toolchain thực, source ownership/license, gold fixtures and approved tolerance. Nếu chưa đủ: Luna trả `BLOCKED_PRECONDITION`, không giả tạo code/test/evidence.
