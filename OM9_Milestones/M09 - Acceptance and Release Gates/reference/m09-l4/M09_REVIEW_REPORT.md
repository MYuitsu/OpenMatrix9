# Đánh giá lại M09 L4 — OpenMatrix9 v0.12 + release gates

**Ngày biên tập:** 2026-10-16  
**Phạm vi đánh giá:** tính đúng đắn và khả năng giao việc của kế hoạch M09 cho GPT-6 Luna. **Không phải** kiểm thử chương trình Rust/OCCT hoặc quyết định phát hành.

## 1. Ranh giới từ bản nguồn

- `planning/M09_RELEASE_GATES.md` định nghĩa **G-MVP** (sau E12) và **G-FULL** (sau G-MVP signed + E13); M09 **không sở hữu L2 hoặc L3**.
- Bản v0.12 có `95 canonical L2`, `772 canonical L3`; `602` mainline qua E01–E13, `170 AFTER` ngoài phạm vi release.
- G-MVP bao phủ nhiệm vụ release trên E01–E12 (**529 L3**), G-FULL trên E13 (**73 L3**), nhưng các release L4 **không sao chép lại** 602 L3 thành ownership mới.
- ZIP là specs + planning. Source QymCAD pinned trong báo cáo M00 là **remote-reviewed**, chưa có workspace app chạy, chưa có signed G00/E12/E13. Không có phép đo BRep, STL hay benchmark nào được thực hiện ở đây.

## 2. Thành phẩm L4

| Gói | Tổng | Loại công việc | Quy tắc vào |
|---|---:|---|---|
| `G-MVP` | **18 L4** | Gating, coverage, legal/SBOM, build, UI, geometry, undo/history, save/import, E2E, cutters, STL, robustness, manual, packaging, sign-off | G00 và E01–E12 signed PASS |
| `G-FULL` | **12 L4** | MVP baseline, E13 coverage, SmartFlow UV/rigid, deformations, orient/ornament, Pattern/Rope, user library, advanced history, regression, packaging, sign-off | G-MVP human-signed PASS và E13 PASS |
| **Tổng** | **30 L4** | **177 test cases có ID** | Cần source/build/artifact/evidence thực |

- Mỗi L4 có `TASK.md`, `l4.json`, trace canonical, DAG dependencies, negative scenarios, G00-sourced oracle, evidence target, reviewer policy.
- 177 ca `M09-TC-*` gồm kiểm tra thành công và failure/blocked; mọi status = `NOT_EXECUTED`.
- 59 **L3 duy nhất** được trích dẫn trực tiếp trong task documents; ma trận FULL/MVP khi thực thi **phải** đối chiếu toàn bộ 529+73 L3, chưa tuyên bố có test riêng cho mọi L3.
- `M09_EVIDENCE_TEMPLATE.json`, `M09_GATE_SIGNOFF_TEMPLATE.json` và `M09_RUNTIME_STATE_TEMPLATE.json` đều chỉ là biểu mẫu, không phải bằng chứng.
- `m09_next_task.py` chọn work order có thể bắt đầu từ **signed evidence do con người/CI cung cấp**; nó không xác thực chữ ký hay tự cấp PASS.

## 3. Kết quả kiểm tra tự động (đã thực hiện trên gói kế hoạch)

| Kiểm tra | Kết quả | Mức chứng minh |
|---|---|---|
| `python tools/validate_l3.py` | **PASS** | 95 L2, 772 L3 không thiếu/nhân bản |
| `python tools/validate_v010_plan.py` | **PASS** | 602 mainline/170 AFTER, 13 stages, 0 L4 canonical implemented |
| `python tools/validate_m00_l4.py` | **PASS** | 73 M00 planning L4 giữ nguyên |
| `python tools/validate_m06_l4.py` | **PASS** | 76 M06 planning L4 giữ nguyên |
| `python tools/validate_m09_l4.py` | **PASS** | 30 M09 L4; DAG không cycle; trace L3 chuẩn; CSV/JSON đồng nhất; 177 test IDs |
| `python tools/m09_next_task.py --state planning/M09/M09_RUNTIME_STATE_TEMPLATE.json --gate G-MVP` | **BLOCKED_STATE (dự kiến)** | Helper chủ động từ chối template thiếu bằng chứng thật |
| So sánh gói M06 ZIP gốc | **PASS** | 2.288 file gốc không đổi byte; chỉ 3 README/release-gates docs được nối thêm tham chiếu M09; không xóa file |
| Rust `cargo build/test`, OCCT checks, STL viewer, GPU/manual tests | **NOT RUN** | Không có executable app source, toolchain, fixture/gates signed trong ZIP |

`PASS` ở đây là **plan metadata consistency**, **không đồng nghĩa** G-MVP hoặc G-FULL PASS.

## 4. Rà soát nghiệp vụ và điều chỉnh

1. **Không tạo parent L3 giả:** M09 L4 dùng `parent_gate_id` (`G-MVP`/`G-FULL`), `owner_milestone=M09` và trace read-only tới owner có thật.
2. **Giữ gate thứ tự:** G-MVP nhận E12; G-FULL nhận G-MVP *đã ký* và E13; không chuyển advanced thành điều kiện MVP.
3. **Độ đúng của geometry:** BRep closed kiểm OCCT IsValid, orientation và topology; curve/open surface có oracle đúng loại, không bắt chúng thành solid.
4. **3DM scope:** kiểm *import 3DM*, units/layers/trim/blocks và thông báo dữ liệu không hỗ trợ; **không** tự thêm export 3DM vào release.
5. **STL không phải bằng chứng in được sản phẩm:** yêu cầu viewer độc lập, mm/winding/manifold, nguồn BRep và fixture; không tuyên bố đã sản xuất vật lý.
6. **History là dữ liệu:** các bài test gồm recompute ring/gem/setting/cutter, conflict explicit, atomics save/open/Undo/Redo/Cancel.
7. **Giấy phép đúng chủ:** AGPL QymCAD, OCCT, openNURBS, SBOM và asset/trademark chỉ do reviewer được phân quyền xác nhận.
8. **FULL tránh hồi quy MVP:** dữ liệu MVP đã khóa phải tái chạy trên FULL binary; không dựa vào PASS của binary cũ.
9. **GPT-6 Luna:** một L4 mỗi lần; nếu thiếu prerequisite, toolchain, golden/reference, tester hoặc reviewer, trả BLOCKED thay vì fake completion.

## 5. Chấm điểm kế hoạch (ước lượng biên tập, không benchmark)

| Tiêu chí | Điểm /10 | Nhận định |
|---|---:|---|
| Ranh giới ownership + scope gate | 9.7 | Không chiếm L2/L3; AFTER ngoài release, M06/M00 không sửa |
| Phụ thuộc, manifest, công cụ điều phối | 9.2 | Không cycle, file-mapping/tên ID/tiền điều kiện đầy đủ; chữ ký còn phải kiểm ngoài công cụ |
| Kịch bản QA / oracle/negative tests | 8.8 | 177 ca có ID; ngưỡng numeric/fixtures mới là contract, chưa có giá trị G00 |
| Độ phù hợp GPT-6 Luna | 9.0 | Task nhỏ theo gate, prompt stop/continue, manifest JSON/CSV và template evidence |
| Mức có thể chạy release ngay | 1.0 | Thiếu source app, signed gates, fixture/threshold và reviewer |

**Điểm chất lượng bộ kế hoạch M09: 9,0/10** (đánh giá biên tập). **Tỷ lệ task release đã được kiểm thử: 0/30.** Hai gate hiện tại: `NOT_TESTED`, **không có quyền công bố GO**.

## 6. Rủi ro còn lại và đề xuất xử lý

| Rủi ro | Độ nghiêm trọng | Cách xử lý |
|---|---|---|
| Chưa có executable Rust/OCCT và ABI/toolchain thật | BLOCKER | Chạy G00 local source checkout, pin/hash OCCT và clean builds |
| G00/E01–E12/G-MVP/E13 chưa có signed PASS | BLOCKER | Thực hiện giai đoạn trước; M09 chỉ verify release không ký thay |
| Chưa có golden geometry, tolerance, performance budget hay physical target | BLOCKER | Reviewer phê duyệt fixture and budgets ở G00; không tự chọn ngưỡng |
| SmartFlow và UI/interop có thể cần thêm test matrix theo source thực | HIGH | Khi triển khai, chia thêm fixture/case nhỏ và cập nhật manifest có review, không silently widen scope |
| Test-case oracle cùng một task dùng contract ở mức task thay vì con số cụ thể | MEDIUM | Khi G00 frozen, ghi expected metrics riêng theo fixture từng case; thiếu thì BLOCKED_REFERENCE |
| Chữ ký gate không được helper tự xác thực về mật mã | MEDIUM | Human release authority xác nhận chữ ký, raw evidence và artifact hashes trước GO |
| Mâu thuẫn OCCT 7.8.1/7.9.3 trong nguồn QymCAD đã khảo sát | HIGH | Kiểm toolchain phiên bản thực tại clean checkout, không chọn theo suy đoán |

## 7. Kết luận

- **READY_FOR_AGENT_HANDOFF:** 30 planned M09 L4, `GPT6_LUNA_EXECUTION_PROMPT.md`, `M09_EXECUTION_MANIFEST.json`, `M09_TEST_CASES.csv`, validator.
- **NOT_READY_FOR_RELEASE:** cả hai gate, vì hiện không có chạy code thật và chữ ký nghiệm thu.
- **Điểm bắt đầu:** `planning/M09/README.md` → `planning/M09/GPT6_LUNA_EXECUTION_PROMPT.md` → manifest → một TASK.md đủ prerequisites.
