# M01 — chọn module backend kế tiếp (sau OM9-L2-01-03)

Ngày: 2026-10-17 · Người đề xuất: coding agent · Người quyết: project owner
Trạng thái: **ĐỀ XUẤT — chưa chốt**. Tài liệu này chỉ ghi phân tích và khuyến nghị; không đổi cây kế hoạch.

## 0. Đầu vào đã đo (không suy đoán)

| Sự kiện | Bằng chứng |
| --- | --- |
| `OM9-L2-01-03` đã xong 13/13 L3 | `implementation/design/M01/OM9-L2-01-03/api.md` §7; verification `26/26 groups ok, 0 failed, 0 evidence gaps` (`implementation/evidence/M01/OM9-L2-01-03/AUTOMATED_VERIFICATION.log`) |
| `OM9-L2-01-08` chưa có gì, nhưng **đã có thiết kế sẵn** | `implementation/design/M01/OM9-L2-01-08/{api.md,crosswalk.json,oracles.json}`; audit: 0 DONE / 6 PARTIAL / 1 MISSING; kế hoạch chi tiết 8.1–8.8 trong `implementation/design/M01/IMPLEMENTATION-CHECKLIST.md` §1 |
| `OM9-L2-01-05` chưa có thiết kế, chưa có SDK | không có thư mục `implementation/design/M01/OM9-L2-01-05/`; không có OCCT header/lib/SDK trên máy (`Standard_Handle.hxx` không tồn tại), không có `CASROOT`/`OCCT*` trong môi trường |
| Native build hiện tại **cố ý loại OCCT** | `standalone/native/CMakeLists.txt`: "No legacy OM9ThreeDm, FreeCAD, Qt, OCCT, OCAF or Qym application target"; chỉ pin openNURBS (`eb92af3…`) |
| Số L3 khai báo phụ thuộc `OM9-L2-01-08` | **34** (27 M01, 7 M02) |
| Số L3 khai báo phụ thuộc `OM9-L2-01-05` | **148** (96 M03, 30 M01, 14 M04, 8 M08) |
| Cổng duyệt | M00 mới duyệt QymCAD reuse; **OCCT chưa duyệt** (`implementation/design/M01/OM9-L2-01-08/api.md` cũng ghi cùng phạm vi) |
| M00 hiện trạng | 0/19 L3; riêng `M00-L2-02 OCCT` (3 L3) chưa bắt đầu |

## 1. Ba lựa chọn

- **A. `OM9-L2-01-08` (units, tolerances, typed parameters) trước.**
- **B. `OM9-L2-01-05` (OCCT B-Rep kernel bridge) trước.**
- **C. Làm A, và song song mở cổng M00-OCCT + spike mỏng 2 L3 của B** (`-001` FFI handle lifecycle, `-002` B-Rep shape type/validation).

`OM9-L2-01-03` không còn là lựa chọn (đã hoàn tất).

## 2. So sánh theo tiêu chí

| Tiêu chí | A: L2-01-08 | B: L2-01-05 | C: A + spike B |
| --- | --- | --- | --- |
| Cổng pháp lý/duyệt | **Không có** — bắt đầu ngay | **Chặn**: M00-OCCT chưa duyệt | A ngay; B chỉ sau khi duyệt |
| Mức sẵn sàng | Thiết kế + oracle + kế hoạch 8 bước đã có | Không có design; phải audit source OCCT trước | A sẵn sàng; spike cần audit trước |
| Hạ tầng | Không cần gì thêm | **Phải có OCCT SDK** (pin + licence) và một target C-ABI mới, trong khi CMake hiện cấm OCCT | như A + chuẩn bị SDK |
| Rủi ro kỹ thuật | NORMAL × 7 | **HIGH × 11** (FFI lifetime/threading, ABI, tessellation) | Trung bình, nhưng chia nhỏ được |
| Số L3 được mở khoá | 34 | 148 | 34 ngay, 148 sau |
| Giá trị chặn đường | Là dependency đã khai báo của L2-01-03 vừa xong; cần cho numeric input M01 và snapping/units M02 | Cần cho toàn bộ M03 geometry, tessellation của L2-01-06 `-003`, M04, M08 | Cả hai, tuần tự hoá đúng thứ tự |
| Ràng buộc kiến trúc | Bảng biên API M01: `LengthMm/AngleDeg/TolerancePolicy/WorldPoint/LocalPoint` thuộc L2-01-08 và **cấm `f64` vô nhãn unit giữa module** | Nếu làm trước, adapter OCCT sẽ phải nhận `f64` vô nhãn rồi sửa lại | Tránh được việc sửa lại đó |
| Cổng nghiệm thu | AUTO_ONLY (test được ngay) | Nhiều L3 manual/HIGH, cần môi trường | như A |
| Khả năng hoàn tác | Cao (thuần Rust, additive) | Thấp (một khi ABI đã đóng băng thì khó đổi) | Cao cho phần A |
| Thời gian tới mốc có ích | Ngắn (7 L3, NORMAL) | Dài (11 L3 + SDK + ABI) | Ngắn cho A |

## 3. Khuyến nghị: **A (L2-01-08) trước**, rồi mới tới B; mở cổng M00-OCCT ngay từ bây giờ

Lý do chính, theo thứ tự sức nặng:

1. **B không thể bắt đầu hợp lệ.** OCCT chưa được duyệt ở M00 và **không có SDK** trên máy; `standalone/native/CMakeLists.txt` còn ghi rõ target hiện tại không có OCCT. Làm B trước là vừa vượt cổng, vừa phải tự dựng hạ tầng.
2. **A mở khoá 34 L3 gồm chính các dependency còn treo của module vừa xong**, và là nền cho numeric input M01 + snapping/units M02 — tức là "nợ gần" trả trước.
3. **A rẻ và chắc**: đã có api/oracle/kế hoạch 8 bước, 7 L3 NORMAL, gate AUTO; phần khó nhất của A (float round-trip, mục 8.1) **đã xong** như tác dụng phụ của L2-01-03.
4. **Làm B trước sẽ tạo nợ API**: mọi adapter OCCT sẽ nhận `f64` vô nhãn, vi phạm bảng biên API của M01, rồi phải sửa lại sau khi A landed.
5. **148 phụ thuộc của B là lý do để đóng băng _interface_ sớm (M00), không phải để _implement_ trước.** Đóng băng interface là việc giấy tờ, làm được ngay mà không cần SDK.

**Phản biện công bằng (khi nào chọn B trước):** nếu mục tiêu là "fail fast" cho rủi ro lớn nhất, thì nên làm B — nhưng khi đó bước đầu **không phải 11 L3**, mà là: chốt M00-OCCT → lấy/pin OCCT SDK → spike `L2-01-05-001` + `-002`. Đó chính là phương án C, và C chỉ khác A ở chỗ chạy song song phần chuẩn bị.

## 4. Việc cụ thể cho từng lựa chọn

**Nếu chọn A (khuyến nghị) — `OM9-L2-01-08`, 7 L3:**
1. `L3-001/002`: `LengthMm`, `AngleDeg`, `AngleRad`, `UnitError`, `Unit::parse/symbol` (+ đã có float round-trip).
2. `L3-003`: `Tolerance` + `Document.tolerance` + `Mutation::Tolerance` (đây là chỗ mở lại ADR-03-6 và khoá `(Document,"tolerance")` trong archive).
3. `L3-004/005`: `Value::{length,angle,count,ratio}`, `OptionDomain::{Number.step, validate}`.
4. `L3-006`: serialization an toàn đơn vị.
5. `L3-007`: `WorldPoint/LocalPoint/LocalFrame` + `coords::parse_local/resolve` (thay chỗ parse trong `session.rs`).
6. Chốt: `cargo test` xanh + fixture/oracle + evidence như L2-01-03 + một lệnh verify riêng.

**Nếu chọn B — `OM9-L2-01-05`, 11 L3:** bắt buộc tiền đề (chưa có cái nào): quyết định M00-OCCT; OCCT SDK pin + licence review; target C-ABI mới trong `standalone/native` (đang bị cấm bởi CMake); design folder + crosswalk/oracle cho 11 L3. Sau đó mới tới `-001` handle lifecycle → `-002` B-Rep validate → …

**Nếu chọn C:** chạy A như trên; song song xin quyết định M00-OCCT và chuẩn bị SDK/pin; chỉ mở `-001`/`-002` khi hai điều kiện đó xong.

## 5. Việc không phụ thuộc cả A và B (có thể chạy bất cứ lúc nào)

`L2-01-12` (project codec/STL), `L2-03-01` (viewport layout/CPlane), `L2-03-07` (menu/icon history),
`L2-03-10` (commandline feedback), `L2-03-14` (layers/objectinfo/superselect), `L2-03-15`
(file/project/jobbags) — tất cả đang ⬜ và không cần OCCT.
