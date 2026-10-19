# M00-L4-02-01-01 — Audit `build.rs` OCCT và ABI C++ (QymCAD, revision pin)

Nguồn đọc: `QymCAD/crates/qymcad-kernel/build.rs`, `src/occt_common.hpp`, `src/cxx_runtime.rs`.
Revision pin của QymCAD: `746a17f127897e921a57118b6caad08968602bf5`.

## Cách QymCAD nối OCCT

- `cc::Build` biên dịch `occt_bridge.cpp`, `occt_helical.cpp`, `occt_faces.cpp`, `occt_io.cpp`
  với `-std=c++17`, `_USE_MATH_DEFINES`, include `OCCT_INCLUDE_DIR`, link `OCCT_LIB_DIR`.
- MSVC: `/EHa`; nền khác: `OCC_CONVERT_SIGNALS`.
- MSVC: `NOMINMAX`, `WIN32_LEAN_AND_MEAN`.
- Toolkit link: `TKXCAF, TKLCAF, TKCAF, TKCDF, TKDESTEP, TKDEIGES, TKXSBase, TKDE, TKMesh,
  TKShHealing, TKFillet, TKOffset, TKBool, TKPrim, TKBO, TKFeat, TKGeomAlgo, TKTopAlgo, TKBRep,
  TKGeomBase, TKG3d, TKG2d, TKMath, TKernel`.
- C++ runtime: chọn theo toolchain trong `cxx_runtime.rs`.

## Kết luận audit

Công thức (env-var include/lib, `/EHa`, `NOMINMAX`, danh sách TK*) **dùng được cho OCCT 8.0.1**:
tên toolkit của 8.0 vẫn giữ nguyên `TK*`, và 8.0 giữ baseline C++17.

Rủi ro không nằm ở danh sách toolkit mà ở **API**: OCCT 8.0 đổi nhiều thứ so với 7.x
(`Standard_Failure` bỏ `Raise()/DynamicType()`, thêm `ExceptionType()`; `BRepPrimAPI_MakeCylinder`
bỏ overload `gp_Pnt`; nhiều header `TCol*`/`TopTools_*Collection*` bị đánh dấu deprecated và một số
header iterator bị bỏ). Đo thực tế ở `OM9-L4-02-01-02`.
