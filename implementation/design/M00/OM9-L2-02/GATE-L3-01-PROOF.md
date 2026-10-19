# M00-L2-02 · L3-01 — Toolchain and C ABI bridge proof

Ngày: 2026-10-17 · Người thực hiện: agent · Môi trường: Windows, MSVC 14.44.35207, CMake 4.x (VS 2022 x64)

Bản ghi này theo **đúng 4 L4 của gate** (`OM9-M00-L3-02-01`), không gộp:

| L4 | Việc | Trạng thái |
| --- | --- | --- |
| `OM9-M00-L4-02-01-01` | Audit `build.rs` OCCT và ABI C++ của QymCAD (revision pin) | ✅ xong (bản ghi này §1) |
| `OM9-M00-L4-02-01-02` | Thử compile `qymcad-kernel` tại revision pin với OCCT trên máy | ✅ chạy thật — **KHÔNG compile được với 8.0.1** (§2) |
| `OM9-M00-L4-02-01-03` | Xác nhận C ABI ownership và ngoại lệ | ✅ probe OCCT 8.0.1 (§3) |
| `OM9-M00-L4-02-01-04` | Proof gọi OCCT từ ứng dụng Rust tối giản | ✅ `native/occt-rust-proof` (4 test + binary JSON) |

## 1. OCCT được chọn (thay cho 7.8.1 của FreeCAD)

Ông chủ chỉ đạo: **lấy bản OCCT mới nhất**, không dùng 7.8.1 mà FreeCAD đóng gói.

| Hạng mục | Giá trị |
| --- | --- |
| Phiên bản | **8.0.1** (`OCC_VERSION_COMPLETE "8.0.1"`), release 2026-07-30 |
| Tag / commit | `V8_0_1` (tức `V8.0.1`) · `b8f597c677811d1f9f4d8a97f5ae2825c0353a42` |
| Nguồn | GitHub release `Open-Cascade-SAS/OCCT` |
| Gói dùng | `opencascade-release-no-pch.zip` — 54 234 931 B, sha256 `307f694f1d4a280c7f58ee2ddb69a7f7e2b78d82749339efd40ce8b8b116d76c` (**khớp digest của asset**) |
| 3rdparty bắt buộc | `3rdparty-vc14-64.zip` — 188 827 053 B, sha256 `12d04f688c583b8e896758f65cd7012458fee921cedb792ec6caaf469b372598` (**khớp**) — cần `tbb12.dll` + `jemalloc.dll` |
| Nội dung gói | 7 478 header (`inc/`), **74 `.lib`**, **74 `.dll`** (`win64/vc14/{lib,bin}`) |
| License | `LICENSE_LGPL_21.txt` + `OCCT_LGPL_EXCEPTION.txt` (có trong gói) |
| Baseline ngôn ngữ | C++17; `Standard_Failure` kế thừa `std::exception` (8.0 thay đổi so với 7.x) |

Vị trí giải nén: `build/standalone-deps/occt-8.0.1/…` (gitignored, không vào repo).

## 2. Audit `build.rs` của QymCAD (L4-01)

Đọc `QymCAD/crates/qymcad-kernel/build.rs` (revision pin của QymCAD: `746a17f1…`):

- Biên dịch bridge C++ bằng `cc`, `-std=c++17`, `_USE_MATH_DEFINES`, include `OCCT_INCLUDE_DIR`, link `OCCT_LIB_DIR`.
- MSVC: `/EHa`; nền khác: `OCC_CONVERT_SIGNALS`.
- MSVC: `NOMINMAX`, `WIN32_LEAN_AND_MEAN`.
- Link danh sách toolkit **OCCT ≥ 7.8 gộp module**: `TKXCAF, TKLCAF, TKCAF, TKCDF, TKDESTEP, TKDEIGES, TKXSBase, TKDE, TKMesh, TKShHealing, TKFillet, TKOffset, TKBool, TKPrim, TKBO, TKFeat, TKGeomAlgo, TKTopAlgo, TKBRep, TKGeomBase, TKG3d, TKG2d, TKMath, TKernel`.
- C++ runtime chọn theo toolchain (`cxx_runtime.rs`).

**Kết luận audit:** công thức này dùng được cho OCCT 8.0.1 vì tên toolkit 8.0 vẫn giữ nguyên `TK*`; rủi ro thật nằm ở **API** 8.0 đã đổi (ví dụ `Standard_Failure` bỏ `Raise()/DynamicType()`, thêm `ExceptionType()`; `BRepPrimAPI_MakeCylinder` bỏ overload `gp_Pnt`; `Geom_*` có `EvalD0..`). Mục §2 (compile thử) chính là để đo rủi ro đó.

### Kết quả đo L4-02 (compile thử reference bridge)

Chạy `cargo build -p qymcad-kernel -j 2` với `OCCT_INCLUDE_DIR`/`OCCT_LIB_DIR` trỏ vào gói 8.0.1:

```
occt_common.hpp(169): fatal error C1083: Cannot open include file:
  'TopTools_ListIteratorOfListOfShape.hxx': No such file or directory
error occurred in cc-rs: ... cl.exe ... occt_bridge.cpp  (exit 2)
cargo exit=101
```

Đồng thời cl.exe cảnh báo hàng loạt header đã deprecated từ 8.0.0 (`TDF_LabelSequence.hxx`,
`TColgp_Array2OfPnt.hxx`, `TopTools_ListOfShape.hxx`, `TopTools_MapOfShape.hxx`,
`BRepCheck_ListOfStatus.hxx`, …).

**Kết luận (phát hiện của gate):** bridge của QymCAD pin (7.x API) **không dùng được trực tiếp** với
OCCT 8.0.1: header iterator `TopTools_ListIteratorOfListOfShape.hxx` đã bị bỏ, và nhiều container
`TCol*`/`TopTools_*` đã chuyển sang `NCollection_*`. Vì vậy:

1. OCCT 8.0.1 **dùng được** (probe §3 chứng minh: boolean, mesh, intersection, ngoại lệ, volume chính xác),
2. nhưng **bridge cho OM9 phải viết theo API 8.0** (dùng `NCollection_List<TopoDS_Shape>` và
   iterator của nó), không port nguyên `occt_common.hpp` của QymCAD.
3. Đây là lý do gate tồn tại: nếu chỉ "tin" QymCAD là chạy được, ta sẽ phát hiện muộn ở M01.

Bằng chứng: `implementation/evidence/M00/OM9-L4-02-01-02/qymcad-kernel-build-against-occt-8.0.1.log`.
## 3. Probe C ABI ownership + ngoại lệ (L4-03) — **đo trên OCCT 8.0.1**

Probe: [`native/occt_probe.cpp`](../../../native/occt_probe.cpp) · project: `build/native-occt/CMakeLists.txt` · bằng chứng: `implementation/evidence/M00/OM9-L4-02-01-03/occt-probe.{json,log}`.

Kết quả chạy thật (`exit 0`):

```json
{"status": "ok", "kernel": "occt", "version": "8.0.1", "version_major": 8, "version_minor": 0,
 "version_maintenance": 1, "box_made": true, "box_faces": 6, "box_edges": 24, "box_vertices": 48,
 "box_wires": 6, "box_valid": true, "moved_min_z": 4.9999998999999997,
 "moved_max_z": 9.0000000999999994, "mesh_done": true, "mesh_nodes": 24, "mesh_triangles": 12,
 "line_face_hits": 2, "first_hit": [5, 5, 5], "first_hit_parameter": 15,
 "boolean_done": true, "boolean_valid": true, "boolean_faces": 7,
 "box_volume": 399.99999999999989, "cut_volume": 349.73451754256325,
 "invalid_input_caught": true, "invalid_input_type": "Standard_ConstructionError"}
```

Đọc kết quả:

| Đo | Ý nghĩa |
| --- | --- |
| `version 8.0.1` | Đúng bản đã tải, không phải 7.8.1 của FreeCAD |
| topo 6F/24E/48V/6W, valid | Đọc topology B-Rep hoạt động |
| `moved_*_z` 5→9 | `BRepBuilderAPI_Transform` + `Bnd_Box` hoạt động |
| `mesh_nodes 24 / triangles 12` | `BRepMesh_IncrementalMesh` hoạt động trên 6 mặt phẳng |
| `line_face_hits 2`, hit `(5,5,5)` | `IntCurvesFace_ShapeIntersector` hoạt động — thứ openNURBS **không có** |
| `boolean_valid true`, 7 faces | `BRepAlgoAPI_Cut` hoạt động — thứ openNURBS **không có** |
| `box_volume 400.0` | 10×10×4 = 400 **chính xác** (sai số 1e-13) |
| `cut_volume 349.73451…` | 400 − π·2²·4 = **349.7345175425633** → khớp 16 chữ số |
| `invalid_input_caught true`, `Standard_ConstructionError` | Ngoại lệ OCCT trả về như C++ exception, không kill process |

## 4. Proof từ Rust (L4-04) — harness, không phải crate sản phẩm

M00 là **gate**, không chứa code sản phẩm. Bằng chứng L4-04 là một harness Rust tối giản, standalone, nằm
dưới `native/occt-rust-proof/` (không phải member của workspace):

```
{"occt_version": "8.0.1",
 "box_volume": 399.99999999999988631,          // 1.1e-13 từ 400
 "drilled_volume": 349.73451754256325330,      // analytic 349.73451754256331014
 "invalid_input": "kernel failure: Standard_ConstructionError"}
```

`cargo test` = 4 passed. Crate sản phẩm `crates/om9-kernel` **được tạo ở M01-L2-01-05** với cùng hợp đồng
C ABI; M00 không sở hữu code đó.

## 5. Việc còn lại của gate

- `L4-02`: kết quả compile `qymcad-kernel` (chạy nền, ghi log) — nếu lỗi thì đây là **phát hiện**: bridge QymCAD cần chỉnh cho OCCT 8.0.
- `L4-04`: ✅ đã xong bằng harness `native/occt-rust-proof` (xem §4).
- Sau L3-01: `L3-02` lifetime/threading/error safety (10k shape create/destroy, một call/lúc) và `L3-03` numeric invariants (tolerance, `BRepCheck`, so sánh thể tích/diện tích giải tích).
