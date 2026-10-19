# 3DM import / export / copy — hiện trạng và lát cắt hoàn thiện

Ngày: 2026-10-17 · Người đề xuất: coding agent · Người quyết: project owner
Trạng thái: **ĐỀ XUẤT — chưa chốt**. Tài liệu chỉ ghi hiện trạng đo được và lát cắt đề nghị; không đổi cây kế hoạch.

## 0. Bối cảnh

Bạn yêu cầu: gộp code ra root (đã xong, xem `docs/standalone-foundation.md` + lịch sử ledger), và
"import export copy 3dm thì điều chỉnh để hoàn thiện luôn". Tài liệu này trả lời đúng câu thứ hai:
**cái gì đã có, cái gì còn thiếu, hoàn thiện theo lát cắt nào**.

## 1. Cái đã có thật (đo trên cây sau khi gộp)

| Lớp | Vị trí | Bằng chứng chạy được |
| --- | --- | --- |
| Rust exchange | `crates/om9-exchange/src/` — 18 module, ~300 KB: `retained_cold`, `retained_packet`, `retained_stage`, `retained_write`, `source_preparation`, `staged`, `current_overlay`, `wire_output`, `output_identity`, `import_capture`, `import_dto`, `import_graph`, `imported`, `inspection`, `dependency_cache`, `merge_inputs` | 19 test binary (`cargo test -p om9-exchange`), gồm `retained_native_import`, `retained_native_write`, `retained_native_inputs`, `staged_export`, `current_overlay`, `wire_output_review` |
| Cầu openNURBS | `native/native_exchange.cpp` (107 KB), `native/retained_merge.hpp` (94 KB), `native/CMakeLists.txt` pin openNURBS `eb92af3…` | build native + 4 fixture executable (`*_fixture.cpp`) |
| Validator | `tools/validate_native_exchange_conversion.py`, `validate_standalone_exchange.py`, `validate_native_retained_{source,current}*.py`, `verify_standalone_foundation_evidence.py` | chạy được, xem `docs/validation/*` |
| Mô hình retained | `crates/om9-core/src/retained.rs` (SourceRecord, ImportNamespace, EntityBaseline, ImportBundle) + `provenance.rs` | `om9-core` test `retained_imports`, `retained_peak_review`, `provenance` |

Nghĩa là: **đường 3DM đã có thật ở dạng "retained + byte-preserving"** — đọc file nguồn, giữ nguyên bản
ghi native, xuất lại kèm báo cáo mất mát. Đây không phải "import hình học đầy đủ".

## 2. Cái đang thiếu — đo bằng chính mã

### 2.1 Các biên `Unsupported(...)` trong `om9-exchange` (18 chỗ)

Đây là danh sách *chính xác* những gì chưa hoàn thiện, nhóm theo nghĩa:

- **Hình học native chưa map được**: `zero native normal` (inspection), `nonuniform native circle
  placement` (inspection), `native component type mapping` (output_identity),
  `selected original-wire component codec` (source_preparation), `new wire native family` (wire_output),
  `native component type mapping`.
- **Rational/NURBS biên**: `cold nonpositive rational hull` ×2 (retained_cold).
- **Layer/selectable**: `independent native selectable`, `independent native layer selectable`
  (retained_cold).
- **Policy/unit context**: `legacy import policy is unknown` ×2 (current_overlay, output_identity),
  `output definition unit context missing` (output_identity), 2 biên khác trong `wire_output`.

Mỗi dòng trên là một quyết định kỹ thuật đã ghi trong code, không phải lỗi: chúng từ chối thay vì đoán.

### 2.2 Lỗ hổng lớn: **copy/clipboard**

- v1 có `ThreeDmClipboard.py` (13 KB) trong bản FreeCAD cũ ở root, nhưng **kiến trúc mới không có module
  clipboard nào**: grep `clipboard` trong `crates/**` không thấy crate/module nào sở hữu nó.
- Cây kế hoạch **không có L2/L3 nào đặt tên clipboard cho 3DM standalone**. L3 gần nhất là
  `OM9-L3-03-08-001 Top11 Duplicate Action` (M03, duplicate chứ không phải clipboard native) và
  `OM9-L3-01-11-008 Array Bake Vs Instance Policy`.
- Hệ quả: "copy 3DM" hiện **không có owner** trong spec lẫn code. Đây là gap cần bạn quyết: thêm L3 mới
  (ví dụ dưới `OM9-L2-01-13` import-rhino-3dm) hay mở rộng M00-L2-04.

### 2.3 Chủ sở hữu spec của import/export

- `OM9-M00-L2-04` — 3DM/openNURBS interoperability (feasibility, 3 L3, đang ⬜).
- `OM9-L2-01-13` — `14-febe-import-rhino-3dm` (feature import phía UI).
- `OM9-L2-01-12` — native project codec + STL export (10 L3, đang ⬜).
- Export hình học đầy đủ phụ thuộc `OM9-L2-01-09..11` (OCCT) → **bị chặn bởi M00-OCCT**.

## 3. Đề nghị lát cắt (theo thứ tự, mỗi lát test được)

| # | Lát cắt | Nội dung | Phụ thuộc | Giá trị |
| --- | --- | --- | --- | --- |
| **S1** | `import-diagnostics` | Đóng các biên Unsupported **có fixture thật** trước: `nonuniform native circle placement`, `zero native normal`, `native component type mapping`; mỗi cái thành một báo cáo có mã lỗi ổn định + test | không | Làm import "không im lặng" trên file thật |
| **S2** | `3dm-clipboard` | Copy/paste native records giữa hai `.om9` (và từ 3DM đã retained sang document), dùng `ImportBundle` + `SourceRecord` sẵn có; một transaction, một Undo | cần L3 owner (xem §2.2) | Đây chính là "copy 3dm" bạn hỏi |
| **S3** | `export-completeness` | Viết 3DM đầy đủ (không chỉ retained write): map `Geometry` → native cho Point/Curve/Surface/Mesh, còn `Brep` chờ OCCT | `OM9-L2-01-12`, và S? cho OCCT | Xuất được hình học mới tạo trong OM9 |
| **S4** | `occt-geometry` | `OM9-L2-01-05` + `OM9-L2-01-09..11` để có BRep thật | **M00-OCCT chưa duyệt** | Mở khoá 148 L3 |

Khuyến nghị: **S1 + S2 ngay** (không cần OCCT, không cần SDK mới, dùng hạ tầng đã có), **S3** sau khi
`OM9-L2-01-12` được duyệt làm, **S4** khi bạn chốt M00-OCCT.

## 4. Licence / provenance

- openNURBS đã được pin (`eb92af3…`) và đã có notice trong `THIRD_PARTY_NOTICES.md` + CMake kiểm tra
  hash; làm S1–S3 **không thêm phụ thuộc licence mới**.
- Bản v1 (`ThreeDm*.py`, `Gui/ThreeDm*.cpp`) là **bản gốc lịch sử**: giữ nguyên, không xoá, không copy
  code vào crate mới; chỉ đọc để đối chiếu hành vi (theo AGENTS.md).

## 5. Việc cần bạn chốt

1. **Phạm vi 3DM**: M00 ghi 3DM *chưa* duyệt (chỉ QymCAD reuse). Yêu cầu hôm nay được hiểu là duyệt 3DM —
   xác nhận để tôi ghi vào ledger/gate.
2. **Owner cho clipboard**: thêm L3 mới dưới `OM9-L2-01-13` (đề nghị) hay mở rộng `OM9-M00-L2-04`?
3. Làm **S1+S2** trước hay chỉ **S2**?
