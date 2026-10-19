# W3 — port kernel bề mặt (thuộc M03 `OM9-L2-02-05`)

**Vị trí milestone:** M03 – Geometry 3DM, L2 `OM9-L2-02-05` (Febe surfaces). W3 là kernel mà các L3 của
L2 này (và snapping của M02 `OM9-L2-02-03`) sẽ dùng; đây là **port kernel có test**, không phải nghiệm
thu L3.

| Hàm kernel | L3 đích trong M03 | Ghi chú |
| --- | --- | --- |
| `evaluate`, `derivatives`, `normal` | `OM9-L3-02-05-*` (mọi lệnh surface cần điểm/pháp tuyến); `OM9-L2-01-11 L3-006` Surface UV Normal Placement Frames | đánh giá rational + pháp tuyến đơn vị; từ chối điểm suy biến |
| `refit` | `OM9-L3-02-05-006` Surface Rebuild Refit And Softedit | lấy mẫu + nội suy 2 chiều |
| `loft` | `OM9-L3-02-05-002` Loft And Curve Network Workflows | loft qua các tiết diện, v nội suy đúng tiết diện |
| `isocurve` | `OM9-L3-02-04-010` Duplicate Edge Border Isocurve | iso-curve chính xác (giữ trọng số) |
| `plane`, `cylinder` | `OM9-L3-02-05-004` Edge Planar And Patch Surface Workflows (nền) | builder analytic chính xác |
| **chưa port — trim/split/untrim** | `OM9-L3-02-05-005` Trim Split Untrim And Shrink Surface | **bị chặn**: mô hình target chưa có trimmed B-Rep (`Brep` → `Unsupported("brep geometry")`), phụ thuộc quyết định M00/OCCT |
| `path_frames`, `sweep1`, `sweep2`, `arc_length_stations` | `OM9-L3-02-05-001` Sweep1 Sweep2 And Profile Sweep Workflows | sweep1 + **sweep2** đã port theo `docs/features/OM9-SURFACE-003.md`; multi-profile/G1-G2/Slash/closed rails chưa |

Trạng thái L3: **không** L3 nào được đánh dấu nghiệm thu — đóng L3 M03 cần runbook/spec của M03.

## Đã port (đợt 1)

| Legacy | Hành vi giữ lại | Target |
| --- | --- | --- |
| `support_snapshot.rs` (đánh giá support cho snap) | điểm bề mặt `S(u,v)` dạng rational, kiểm tra domain, mẫu số thuần nhất | `surface_kernel::evaluate` |
| `derived_support.rs` | đạo hàm riêng `S_u`, `S_v` theo quy tắc thương của dạng rational; pháp tuyến đơn vị; điểm suy biến bị từ chối | `surface_kernel::{derivatives, normal}` |
| `analytic_support.rs` | bề mặt analytic chính xác: mặt phẳng (degree 1) và trụ tròn (rational degree 2 theo u, tuyến tính theo v) trên frame trực chuẩn | `surface_kernel::{plane, cylinder}` |
| `surface_refit.rs` | refit = lấy mẫu lưới rồi nội suy hai chiều liên tiếp (bản ghi mới, không sửa bản gốc) | `surface_kernel::refit` |
| (mới, cần cho snapping/trim) | **iso-curve chính xác**: giữ 1 tham số cố định, cực = tổ hợp trọng số của hàng, trọng số = tổng trọng số hàng | `surface_kernel::isocurve` |
| `surface.rs` (nhánh loft) | **loft qua tiết diện**: mỗi tiết diện được lấy mẫu trên lưới u chung rồi nội suy theo u, sau đó các cực được nội suy qua các tiết diện theo v | `surface_kernel::loft` |
| `surface.rs::transport/transport_with_height` | **frame dẫn đường rotation-minimizing** (phương pháp chiếu: mỗi bước bỏ thành phần tiếp tuyến của trục x rồi chuẩn hoá); frame trực chuẩn và vuông góc tiếp tuyến tại mọi trạm | `surface_kernel::path_frames` |
| `surface.rs` (nhánh sweep1) | **sweep1**: profile được mang cứng theo frame dẫn đường (toạ độ cục bộ trong frame đầu, đặt lại vào frame hiện tại), nên rail thẳng tái tạo đúng profile | `surface_kernel::sweep1` |
| `surface.rs::sweep2` / `sweep2_contact` | **sweep2 theo hợp đồng Matrix 8** (Book 1 pp.174–176, bản ghi `docs/features/OM9-SURFACE-003.md`): 65 trạm **arc-length** mỗi rail; frame dùng **rail separation** cho trục x và **primary rail tangent** cho hướng quét; width scale theo separation; toạ độ pháp tuyến giữ nguyên khi Maintain Height ON và scale theo width khi OFF; toạ độ tangent không đổi; degree ≤ 3; profile phải phủ cả hai đầu rail | `surface_kernel::{sweep2, arc_length_stations, Sweep2Options}` |
| (mới, bắt buộc cho profile polyline) | **lấy mẫu polycurve**: evaluator từ chối tham số cho `Curve::Polycurve`, nên kernel lấy mẫu từng đoạn (chia đều theo đoạn, trùng khớp seam khi `count-1` chia hết số đoạn) và tôn trọng `reversed` | `surface_kernel::sample_curve` |

Ghi chú thiết kế: `curve_build::interpolate` (dạng lệnh) chỉ nhận degree **lẻ** 1..=11 theo hợp đồng
InterpCrv, nên kernel refit dùng `curve_build::interpolate_any` (degree bất kỳ trong 1..=11) để tái tạo
được nguồn degree chẵn như trụ (degree 2). Hành vi lệnh không đổi: test cũ vẫn khẳng định degree 2 bị
`Invalid("interpolation degree")`.

## Test (21)

`crates/om9-core/tests/surface_kernel.rs`:

- mặt phẳng: 4 điểm mốc đúng frame; `S_u`/`S_v` đúng trục; pháp tuyến `[1,0,0]` tại mọi mốc;
- trụ: 65 điểm (13×5) đều `|xy| = 5` (≤1e-9) và cao độ `8v`; pháp tuyến hướng tâm, dài 1, không thành
  phần theo trục;
- tham số ngoài domain / NaN bị từ chối (`Invalid("surface parameter")`);
- bề mặt suy biến (`S_v = 0`) bị từ chối pháp tuyến (`Invalid("degenerate surface normal")`);
- iso-curve của mặt phẳng là đoạn thẳng chính xác (đầu/cuối đúng);
- iso-curve của trụ theo u là **đường tròn rational chính xác** (degree 2, 9 cực, trọng số `1, √2/2, …`,
  mọi điểm cách trục đúng 5); theo v là đường sinh dài 8;
- refit mặt phẳng tái tạo chính xác (≤1e-9); refit trụ 16×3 lệch < 0,05 mm so với nguồn;
- refit từ chối lưới không chứa nổi degree;
- builder từ chối frame không trực chuẩn / kích thước ≤ 0 / frame trụ sai;
- **kiểm chéo hai kernel**: iso-curve u của trụ trùng với `curve_build::circle_through` (đường tròn qua
  3 điểm) tại 9 mốc tham số — hai đường code độc lập phải cho cùng hình học;
- **loft** 3 đường tròn bán kính 5 ở z = 0/5/10: bán kính sai ≤ 0,05 tại mọi (u,v), cao độ đúng theo
  tiết diện (≤1e-9), pháp tuyến không thành phần thẳng đứng (vì kết quả là trụ); loft từ chối < 2 tiết
  diện, 2 tiết diện với degree v = 2, và quá ít mẫu u;
- **path_frames** trên cung 1/4 vòng: 9 frame đều trực chuẩn, vuông góc tiếp tuyến, mọi trạm nằm trên
  đường tròn rail bán kính 5;
- **sweep1** rail thẳng: trụ bán kính 2 sai ≤ 1e-6, cao độ đúng `10v` (≤1e-9), pháp tuyến không thành
  phần thẳng đứng;
- **sweep1** profile hình vuông (polycurve) với degree u = 1: mọi điểm nằm **trên biên** hình vuông và
  **cả 4 góc** đều xuất hiện đúng ở tiết diện giữa (mang cứng, không cắt xén) — đây là test bắt lỗi
  "evaluator từ chối polycurve";
- **sweep1** từ chối: quá ít mẫu profile/rail, rail suy biến (không có frame);
- **sweep2 đúng oracle đã ghi trong hồ sơ**: rail width 5 → 9 mm, cung cao 2 mm — Maintain Height **ON** giữ
  đúng 2 mm ở cả hai đầu, **OFF** cho **3,6 mm** ở đầu cuối (= 2 × 9/5), đo theo **khoảng cách vuông góc
  với dây cung** giữa hai rail;
- **sweep2 giữ contact hai rail**: profile điểm đầu/cuối nằm đúng trên rail A/rail B tại 5 trạm, width
  tăng tuyến tính theo separation;
- **arc_length_stations** trên nửa đường tròn: 9 trạm cách đều theo cung (sai ≤ 0,1 %);
- **sweep2 từ chối**: degree > 3 (giới hạn hồ sơ), rail kín (`Unsupported("sweep2 closed rails")`), profile
  không phủ hai rail (`Invalid("sweep2 profile span")`), và hai rail trùng nhau.

## Còn lại của W3

- **Trim/split/untrim** (`surface.rs::Kind/Phase/slash_parameter`, L3 `OM9-L3-02-05-005`): **bị chặn ở
  mô hình**, không phải ở toán. Target chưa có trimmed B-Rep (`Brep` → `Unsupported("brep geometry")`),
  nên một mặt bị cắt không có chỗ để biểu diễn. Cần quyết định M00/OCCT trước khi port; ghi lại để
  không ai tưởng là còn thiếu hàm.
- **Sweep2 phần còn lại** (L3 `OM9-L3-02-05-001`): multi-profile dùng auxiliary spine, Maintain Height
  cho multi-profile, closed rails (cần closed loft), G1/G2 constraints, Add Slash, và tổ hợp fitting. Hợp
  đồng đã ghi rõ các mục này là *unsupported* trong bản ghi 2026-10-09 — port khi có phạm vi mới.
- **Nối native sau (không chặn code)**: kernel thuần Rust ở trên là phần chạy được ngay; OCCT/openNURBS
  sẽ được nối vào sau cho phần *cần* chúng — trimmed B-Rep, boolean, fillet/shell, tessellation chính xác.
  Ranh giới nối đã có sẵn dạng dữ liệu: mọi hàm trả record đã validate + `Error` có nhãn, nên lớp native
  chỉ cần đọc/ghi `NurbsSurface`/`Curve` và map lỗi, không phải sửa kernel.
- **Curve network** (`L3-02-05-002` phần network): cần ghép nhiều tiết diện theo lưới, hiện chỉ có loft
  một chiều.
- **Discrete snapshot** (`discrete_snapshot.rs`, `prepared_discrete.rs`): lưới hiển thị hoá bề mặt cho
  tessellation — gắn với render/scene, cần chốt ngân sách lưới trước.
- `native_surface_capability.rs`: capability scoped native (openNURBS) — thuộc tầng native, không port
  ranh giới. `*_ffi`: `LEGACY_HOST`, không port.

## Ghi chú phạm vi

W3 phục vụ các L3 của **M03** (Geometry 3DM) và snapping của **M02**; đợt này **không** đánh dấu nghiệm
thu L3 nào (cần runbook/spec M03/M02). Đây là port kernel có test, không nâng trạng thái kế hoạch.

