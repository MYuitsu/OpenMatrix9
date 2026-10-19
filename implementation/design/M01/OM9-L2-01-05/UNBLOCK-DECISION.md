# OM9-L2-01-05 — quyết định mở khoá 7 L3 kernel (bằng chứng thật)

Ngày: 2026-10-17 · Trạng thái: **đã chứng minh toolchain + đo đúng năng lực kernel; 5/7 L3 mở được, 2/7 cần quyết định**

Tôi đã làm **việc tiên quyết nhất**: fetch đúng pin, build, và **chạy** một probe kernel thật. Dưới đây là kết quả đo,
không phải giả định. Bằng chứng: [`implementation/evidence/M01/OM9-L2-01-05/NATIVE-TOOLCHAIN-PROOF.{json,log}`](../../evidence/M01/OM9-L2-01-05/NATIVE-TOOLCHAIN-PROOF.log).

## 1. Tiên quyết đã xong (bước 1)

| Hạng mục | Kết quả đo |
| --- | --- |
| Pin openNURBS | `eb92af3ba1806b0a34a99aba0d3bda83e3d46083` — **khớp** (`git rev-parse` = pin) |
| Nguồn | `https://github.com/mcneel/opennurbs`, license `LICENSE` (notice `Resources/licenses/openNURBS.txt`) |
| Vị trí checkout | `build/standalone-deps/opennurbs` (build/ đã gitignore, không vào repo) |
| Toolchain | CMake `4.x` (`C:\Program Files\CMake`), MSVC `14.44.35207`, VS 2022 x64 |
| Configure | `cmake -S build/native-brep -B build/native-brep/out -G "Visual Studio 17 2022" -A x64` → exit 0 |
| Build | `cmake --build ... --config Release --parallel 2` → exit 0, `om9-brep-probe.exe` |
| Chạy probe | exit 0, JSON ở dưới |

```json
{"box_made": true, "box_vertices": 8, "box_edges": 12, "box_trims": 24, "box_loops": 6,
 "box_faces": 6, "face_loop_edges": 4, "face_loop_closed": true,
 "nurbs_curve_valid": true, "nurbs_curve_degree": 2, "nurbs_curve_cvs": 9,
 "curve_start": [2,0,0], "curve_mid": [0,2,0], "curve_end": [2,0,0],
 "face_evaluated": true, "face_point": [0.5,0,0.5],
 "translated_point": [1,2,8], "rotated_point": [0,1,0],
 "box_transformed": true, "box_min_z": 5, "box_max_z": 9,
 "curve_surface_intersection_api": false, "brep_mesher_available": false,
 "plane_mesh_vertices": 4, "plane_mesh_faces": 1,
 "sphere_made": true, "sphere_faces": 1, "status": "ok", "kernel": "opennurbs"}
```

## 2. Năng lực kernel đo được (bước 2: cái gì mở được, cái gì không)

| L3 | Năng lực cần | openNURBS pin | Bằng chứng đo |
| --- | --- | --- | --- |
| `L3-005` Face/Wire/Edge/Topology | `ON_Brep` m_V/m_E/m_T/m_L/m_F, đọc chuỗi wire | **CÓ** | `8` vertex, `12` edge, `24` trim, `6` loop, `6` face; loop 4 cạnh và **kín** |
| `L3-003` NURBS curve evaluation | đánh giá curve tại tham số | **CÓ** | đường tròn rational degree 2, 9 CV: `(2,0,0) → (0,2,0) → (2,0,0)` đúng bán kính 2 |
| `L3-004` NURBS surface evaluation | đánh giá surface tại (u,v) | **CÓ** | `face_evaluated: true`, `face_point (0.5,0,0.5)` |
| `L3-006` transform matrix | `ON_Xform` | **CÓ** | dịch `(1,2,3)+(0,0,5)=(1,2,8)`; quay 90° `(1,0,0)→(0,1,0)`; brep z `0..4 → 5..9` |
| `L3-011` fixture shapes | B-Rep nguyên thuỷ | **CÓ** | `ON_BrepBox` valid; `ON_BrepSphere` valid, 1 face |
| `L3-007` Geom Intersection | intersector curve/surface/brep | **KHÔNG** | chỉ có line/line, sphere/sphere, bbox-plane — `curve_surface_intersection_api: false` |
| `L3-008` Shape Tessellation | mesher B-Rep | **KHÔNG** | `ON_Brep::CreateMesh` **khai báo nhưng không implement** trong nguồn công khai (`brep_mesher_available: false`); chỉ `ON_PlaneSurface::CreateMesh` chạy được (4 vertex/1 face) |

**Tự sửa lỗi của mình:** ở lượt trước tôi nói openNURBS có `ON_Intersect` cho curve/surface và mesher B-Rep — **sai** với
pin này. Probe đã đo và ghi lại đúng; nếu không chạy thật thì đó sẽ là khai man trong kế hoạch.

## 3. Kết luận: 5/7 mở được không cần OCCT, 2/7 phải chọn

| Nhóm | L3 | Cách mở | Cần quyết định? |
| --- | --- | --- | --- |
| **Mở ngay bằng openNURBS** | `003`, `004`, `005`, `006`, `011` | worker `native/brep_kernel.cpp` + `impl KernelBackend for OpenNurbsBackend` + bằng chứng 5 bộ | **Có**: chấp nhận backend openNURBS cho các L3 mang tên "OCCT" (hoặc đổi tên L3 thành "kernel bridge (openNURBS backend; OCCT pluggable)") |
| **Cần kernel thật để có chất lượng CAD** | `007` intersection, `008` tessellation | **hoặc** OCCT (gate M00-L2-02 + license M00-L2-06) **hoặc** kernel Rust: giao tuyến bằng marching + Newton trên evaluator sẵn có, tessellation bằng lấy mẫu surface thích nghi | **Có**: chọn OCCT hay Rust cho 2 L3 này |

Nếu chọn đường **Rust cho 007/008**: mở được cả 11/11 mà **không** cần OCCT, nhưng bằng chứng phải ghi rõ đây là kernel
Rust tự viết (độ chính xác theo tolerance khai báo, không phải chất lượng OCCT), và khi OCCT được duyệt thì thay backend
sau — `trait KernelBackend` giữ nguyên.

Nếu chọn **OCCT**: bắt buộc chạy trước M00-L2-02 (3 L3: toolchain/C-ABI proof, lifetime/threading/error safety, numeric
invariants) và M00-L2-06 license; sau đó `impl KernelBackend for OcctBackend`.

## 4. Việc đã làm trong lượt này (không đóng L3 nào)

- `native/brep_probe.cpp`: probe kernel thật (topology, NURBS, transform, fixture, mesher/intersection capability).
- `build/native-brep/CMakeLists.txt`: project probe tách khỏi build exchange (không cần pin JSON), **cùng pin** openNURBS.
- `build/standalone-deps/opennurbs`: checkout đúng pin (gitignored).
- Bằng chứng: `NATIVE-TOOLCHAIN-PROOF.{log,json}`.
- Dashboard: **giữ ✅4 🟡0 ⬜7** — đúng sự thật, vì 5 L3 kia chưa có adapter + bằng chứng, và 2 L3 còn lại chưa có kernel.

## 5. Bước tiếp (chờ bạn chốt 2 câu hỏi ở §3)

1. Worker ABI + `OpenNurbsBackend` → đóng `003/004/005/006/011` (≈2–3 session, có test + bằng chứng).
2. `007`/`008`: OCCT gate hoặc kernel Rust (≈1–2 session cho Rust).
3. Khi có đủ: dashboard L2-01-05 = **✅11/11**, hoặc **✅9/11** nếu chọn chờ OCCT cho 007/008.
