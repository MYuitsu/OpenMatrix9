# OM9-L2-01-05 — Be Occt Brep Kernel Bridge (Rust half)

Ngày: 2026-10-17 · Trạng thái: **4/11 L3 nghiệm thu trên bằng chứng tự động; 7 L3 chờ quyết định M00/OCCT**

L2 này sở hữu **adapter**, không sở hữu OCCT: "Adapter OCCT, BRep, NURBS evaluation, tolerances,
memory ownership/FFI safety". Nửa adapter chạy được bằng Rust thuần và đã đóng; nửa kernel cần OCCT
thật thì **chưa thể** đóng vì workspace không có OCCT và quyết định M00/OCCT chưa được duyệt.

## Đã hoàn thành (4 L3, cổng AUTO_ONLY)

| L3 | Nội dung | Bằng chứng |
| --- | --- | --- |
| `OM9-L3-01-05-001` | Safe handle lifecycle: `KernelHandle { slot, generation }` + `HandleStore<T>` (Vec slot + free list + live count). Handle cũ **không bao giờ** alias object mới; xoá hai lần là `StaleHandle`; slot được tái dùng với generation mới | design/evidence/fixtures L3-001 |
| `OM9-L3-01-05-002` | B-Rep records + validation: vertex/edge/wire/face/shell, chuỗi wire, wire kín, cạnh tự nối, face wires, shell kín không trùng face, budget | design/evidence/fixtures L3-002 |
| `OM9-L3-01-05-009` | Error contract: 9 `KernelError` → đúng một `Error`, giữ nguyên văn thông điệp người dùng | design/evidence/fixtures L3-009 |
| `OM9-L3-01-05-010` | Thread/transaction policy: một call tại một thời điểm, một transaction mỗi document, thread affinity | design/evidence/fixtures L3-010 |

## Độ chính xác (CAD chính xác)

- So sánh dung sai là **quy tắc exact** `|a − b| ≤ tolerance`, không fuzzy; biên đúng 1 ULP được test
  (dung sai thân đúng `MIN_TOLERANCE` được nhận, 1 ULP dưới bị từ chối).
- **Không dùng ngưỡng khoảng cách để hợp nhất điểm**: một cặp đỉnh phân biệt cách nhau ≤ dung sai thân bị
  **từ chối** (`Invalid("geometry below kernel tolerance")`) thay vì để kernel collapse cạnh thành điểm.
- Identity của đỉnh/cạnh là **chỉ số topology**, không phải so toạ độ gần đúng — nên hai đỉnh trùng toạ độ
  vẫn là hai đỉnh nếu topology nói vậy.
- Mọi toạ độ đi qua `finite()`; biến thể ma trận dùng `Transform` chung của `om9-core` (validate det ≠ 0).

## Hiệu năng (đo, release)

| Thao tác | Kết quả |
| --- | --- |
| `HandleStore::insert` | 100 000 handle trong **520 µs** — 5,2 ns/handle |
| `HandleStore::get` | 100 000 tra cứu trong **68 µs** — 0,68 ns/tra cứu |
| `HandleStore::len` | O(1) nhờ bộ đếm `live` (trước là quét slot) |
| `BrepBody::validate` (tam giác) | 20 000 body trong **781 µs** — 39 ns/body |

Bulk build không cấp phát per-handle (`with_capacity`), free list tái dùng slot, và validation chạy
trước mỗi kernel call nên được giữ ở mức 42 ns cho topology nhỏ.

## Bị chặn (7 L3) — cần quyết định, không phải thiếu code

| L3 | Lý do chặn |
| --- | --- |
| `OM9-L3-01-05-003` NURBS Curve Evaluation Adapter | cần OCCT để đối chiếu parameterization; Rust đã có evaluator riêng |
| `OM9-L3-01-05-004` NURBS Surface Evaluation Adapter | như trên (HIGH risk) |
| `OM9-L3-01-05-005` Face Wire Edge Topology Access | cần `TopoDS` thật (HIGH) |
| `OM9-L3-01-05-006` OCCT Transform Matrix Service | cần `gp_Trsf`; adapter đã dùng `Transform` chung |
| `OM9-L3-01-05-007` Geom Intersection Query Adapter | cần `IntCurvesFace_ShapeIntersector` |
| `OM9-L3-01-05-008` Shape Tessellation Interface | cần `BRepMesh` (HIGH) |
| `OM9-L3-01-05-011` Reference Shape Fixture Evaluation | cần fixture shape do OCCT tạo |

**Quyết định cần:** duyệt M00/OCCT (SDK, phiên bản, license, cách link) — đúng gate đã ghi ở §5 của
`implementation/design/M01/OM9-L2-01-08/api.md` và trong AGENTS.md ("M00 must be signed off before M01").
Trước khi có quyết định đó, `UnlinkedBackend` là backend duy nhất và **mọi** call kernel trả
`Unsupported("occt kernel")`; không có hình học nào bị bịa ra.

## Hợp đồng backend

```rust
pub trait KernelBackend {
    fn build(&mut self, body: &BrepBody) -> Result<KernelHandle, KernelError>;
    fn destroy(&mut self, handle: KernelHandle) -> Result<(), KernelError>;
    fn transform(&mut self, handle: KernelHandle, transform: Transform) -> Result<KernelHandle, KernelError>;
    fn tessellate(&mut self, handle: KernelHandle, chord_tolerance: f64)
        -> Result<(Vec<Point3>, Vec<[u32; 3]>), KernelError>;
}
```

Khi OCCT được duyệt, chỉ cần thêm một `impl KernelBackend` gọi OCCT: adapter, validation, handle
lifetime, error contract và policy **không phải sửa**. Test dùng `FixtureBackend` trong test file để
chứng minh contract này (build → transform → tessellate → destroy → stale).

## Test (11)

`crates/om9-core/tests/brep_kernel.rs`: tam giác hợp lệ; 10 ca topology hỏng; biên dung sai exact (kể cả
1 ULP); budget; handle stale/tái dùng slot/1000 object không rò; 9 ánh xạ lỗi; policy serial + thread
affinity; unlinked backend từ chối cả 4 call với cùng contract error; adapter contract qua fixture
backend; đo hiệu năng.

## Ghi chú phạm vi

Không L3 nào trong 7 L3 bị chặn được đánh dấu ✅. Dashboard hiển thị **✅4 🟡0 ⬜7** cho L2 này — con số
đúng: adapter đã xong, kernel thì chưa có.
