# W2 — port kernel đường cong (thuộc M03 `OM9-L2-02-04`)

**Vị trí milestone:** M03 – Geometry 3DM, L2 `OM9-L2-02-04` (Febe curves). W2 là kernel mà các L3 của
L2 này sẽ dùng; nó chỉ là **port kernel có test**, không phải nghiệm thu L3.

| Hàm kernel | L3 đích trong M03 | Ghi chú |
| --- | --- | --- |
| `interpolate`, `parameters`, `basis` | `OM9-L3-02-04-003` Interpolated And Controlpoint NURBS Curve | InterpCrv; quy tắc degree lẻ 1..=11 của tầng lệnh |
| `circle_through`, `arc_through` | `OM9-L3-02-04-*` (Circle/Arc trong nhóm lệnh curves) | đường tròn/cung qua 3 điểm |
| `ellipse` | `OM9-L3-02-04-*` (Ellipse) | rational quadratic chính xác |
| `polyline`, `rectangle` | `OM9-L3-02-04-005` Join Explode Close And Split Curves (nền) | polyline/polycurve |
| `isocurve` (phía surface) | `OM9-L3-02-04-010` Duplicate Edge Border Isocurve | xem `OM9-L2-02-05/W3-SURFACE-KERNEL.md` |
| `rebuild` | `OM9-L3-02-04-*` (Rebuild) | lấy mẫu + nội suy lại |
| `insert_knot`, `elevate_degree` | `OM9-L3-02-04-*` (Convert/degree) | chuyển đổi chính xác |
| `fit_points` | `OM9-L3-02-04-*` (FitCrv) | bình phương tối thiểu, hai đầu nội suy |

Trạng thái L3: **không** L3 nào được đánh dấu nghiệm thu. Việc đóng L3 M03 cần runbook/spec của M03
(source audit, oracle, gate) — kernel này là điều kiện cần, không phải bằng chứng nghiệm thu.

## Đợt 1 — dựng đường cong

| Legacy | Hành vi giữ lại | Target |
| --- | --- | --- |
| `rust/src/nurbs_basis.rs`, `rust/src/spline.rs::Basis` | `basis()` Cox–de Boor, partition of unity, xử lý đúng đầu mút đóng của domain | `curve_build::basis` |
| `rust/src/spline.rs::parameters` | tham số chord-length / uniform, chuẩn hoá `[0,1]`; điểm lặp → fallback uniform thay vì chia 0 | `curve_build::{parameters, KnotMode}` |
| `rust/src/spline.rs::{interpolation_degree_valid, interpolate}` | quy tắc degree lẻ 1..=11, giải hệ N·P=Q, knot vector trung bình, curve clamped | `curve_build::interpolate` |
| `rust/src/circle.rs` | đường tròn qua 3 điểm: tâm ngoại tiếp, bán kính, frame trực chuẩn, domain một vòng | `curve_build::circle_through` |
| `rust/src/rectangle.rs` | hình chữ nhật đóng trên frame trực chuẩn (4 đoạn thẳng nối tiếp) | `curve_build::rectangle` |
| `rust/src/polycurve_contract.rs` | polyline = một đoạn thẳng cho mỗi cặp điểm | `curve_build::polyline` |

Mọi hàm trả về `Curve` **đã validate** theo `Limits` (số cực, knot, domain, trọng số), nên không thể
tạo bản ghi mà phần còn lại của tài liệu sẽ từ chối.

## Đợt 2 (bổ sung)

| Legacy | Hành vi giữ lại | Target |
| --- | --- | --- |
| `rust/src/circle_tangent.rs` (phần dựng cung) | cung qua 3 điểm có điểm giữa xác định hướng; `domain=[0,1]` + `angle_domain=[θ_start, θ_start+sweep]`; đảo trục `y` khi cung chạy ngược để ánh xạ tham số vẫn tăng (`rate > 0` theo ràng buộc evaluator) | `curve_build::arc_through` |
| `rust/src/ellipse.rs` | ellipse chính xác dạng rational quadratic 9 cực, trọng số `1,√2/2,…`, knot `0,0,0,.25,.25,.5,.5,.75,.75,1,1,1` trên frame trực chuẩn | `curve_build::ellipse` |
| `rust/src/spline.rs::rebuild` | rebuild = lấy mẫu đường gốc rồi nội suy lại thành bản ghi mới (không sửa bản gốc) | `curve_build::rebuild` |

### Hai quyết định phải chốt khi port

1. **Ngữ nghĩa arc.** `evaluate_curve` coi `domain` là miền **tham số** và `angle_domain` là ánh xạ góc
   tuyệt đối, đồng thời `Curve::Circle::validate` đòi tốc độ góc `> 0`. Vì vậy arc dùng
   `domain = [0,1]`, `angle_domain = [θa, θa + sweep]`, và khi cung tự nhiên chạy **ngược chiều** thì
   trục `y` của frame bị đảo để sweep vẫn dương. Không có "domain âm".
2. **Ellipse không phải `periodic`.** Evaluator trả `Unsupported("periodic derivative continuity not
   yet certified")` cho NURBS periodic, nên ellipse được ghi là **closed + clamped** (cực đầu = cực
   cuối) với `periodic: false, seam: None`: hình học vẫn đóng, đạo hàm vẫn tính được.

## Đợt 3 (bổ sung): bộ chuyển đổi

| Legacy | Hành vi giữ lại | Target |
| --- | --- | --- |
| `rust/src/spline.rs::host_knots` (ngầm: knot insertion khi dựng dữ liệu host) | chèn knot theo Boehm trong **không gian thuần nhất** để đường rational giữ trọng số; hình học không đổi | `curve_build::insert_knot` |
| `curve_conversion.rs` (ý tưởng "conversion query" — chỉ là tầng query ở legacy) | nâng bậc +1 chính xác: tách thành đoạn Bézier (chèn knot tới multiplicity = degree), nâng từng đoạn, nối lại ở breakpoint C0 | `curve_build::elevate_degree` |
| `fit_point_input.rs` | fit bình phương tối thiểu, hai đầu **nội suy chính xác** (Piegl–Tiller A9.4), phần giữa xấp xỉ | `curve_build::fit_points` |

### Bài học đo được (ghi lại vì đã sửa)

- **Boehm phải chạy trên chuỗi cực MỚI.** Bản đầu tôi lặp trên cực cũ rồi nối thêm một cực, làm dãy
  cực lệch một bậc sau span → đường bị đổi hình. Test "hình học không đổi" bắt được ngay.
- **Công thức knot trung bình A9.3 không dùng cho xấp xỉ.** Với 12 cực điều khiển và 41 mẫu, công thức
  dồn toàn bộ knot trong `[0,07; 0,24]` rồi để một span khổng lồ ở cuối: sai số đo được **0,085 mm**
  trên nửa đường tròn. Đổi sang knot trong đều → sai số **< 0,05 mm** (test).
- **Fit không dùng cho đường kín.** Đường tròn kín có mẫu đầu = mẫu cuối nên fit buộc phải "đóng" bằng
  hai cực trùng nhau, sai số tăng vọt; test dùng **cung nửa vòng** thay vì đường tròn kín.

## Test (21)

`crates/om9-core/tests/curve_build.rs` — đợt 1 (9), đợt 2 (7), đợt 3 (5):

- chèn knot 1/2/3 lần: số cực và số knot tăng đúng, **hình học không đổi** (so 21 mốc tham số, ≤ 1e-9),
  tham số ngoài domain bị từ chối, `times = 0` là identity;
- nâng bậc: degree +1, số cực tăng, **hình học không đổi**, record vẫn validate được;
- fit với số cực = số mẫu: đi qua đúng từng mẫu;
- fit với 12 cực trên cung nửa vòng: hai đầu chính xác, sai số giữa < 0,05 mm;
- fit từ chối số cực không hợp lệ (< degree+1 hoặc > số mẫu) và degree 0.

## Đã sửa trong lúc port (lỗi thật, có test bắt)

- Basis tại `t=1` trả 0 hết → ma trận nội suy suy biến; nay đầu mút đóng thuộc span không rỗng cuối.
- Chord-length với điểm lặp sinh tham số không tăng; nay fallback uniform.
- Ellipse đánh `periodic: true` làm evaluator từ chối đạo hàm; nay closed-clamped.
- Frame đường tròn tính `y` hai lần (clippy `-D warnings` bắt).

## Còn lại (tầng command M03)

- Arc **tiếp tuyến** (`circle_tangent::solve`: tiếp xúc 2 đường/điểm-đường) và ellipse mode-driven
  (`Mode`, `Plan::pick/step`, `Plan::reference/undo`) — đây là máy trạng thái lệnh (mouse picks), không
  phải toán kernel, nên thuộc M03 command tier.
- Option/validate của InterpCrv và FitCrv (`curve_input_log.rs`, `fit_point_input.rs` phần input) gắn
  với descriptor lệnh M03.
- Các file `*_ffi`/`*_tests` của nhóm: `LEGACY_HOST`/`TEST`, không port ranh giới.

## Ghi chú phạm vi

W2 phục vụ các L3 của **M03** (Geometry 3DM), không phải L2 của M01 đang mở. Vì vậy các đợt này **không**
đánh dấu nghiệm thu L3 nào: nó là port kernel có test, còn việc đóng L3 M03 cần chạy đúng runbook/spec
của M03 (source audit, oracle, gate). Không nâng trạng thái kế hoạch.

