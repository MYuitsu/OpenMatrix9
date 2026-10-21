# Đánh giá viewport navigation — 2026-10-21

Trạng thái: **phân tích log hiện hữu + review source + đề xuất thiết kế; chưa triển khai hoặc nghiệm thu runtime mới**.

Đối tượng: [VIEWPORT-NAVIGATION-FORENSICS.md](VIEWPORT-NAVIGATION-FORENSICS.md), bản SHA256
`951e31bda83c4e91206e27151a44c81dd757147a97e8c75eb4ef331e106b525c`.
Giữ nguyên bản đó để đối chiếu. Review này ghi các chỉnh sửa cần thực hiện trước khi dùng nó làm kế hoạch thi công.

Nguồn số liệu có thể tái lập: [evidence JSON](../../evidence/M01/2026-10-21-navigation-forensics-review.json),
[script phân tích](../../tools/analyze_viewport_navigation_review.py).

## 1. Kết luận

Tài liệu nhận diện đúng nhiều vấn đề trong code: pacing chen vào xử lý từng pane, wheel tạo lại
transition, sampling cộng trước 16 ms và thiếu bản ghi cho settle frames. Giữ direct drag, quaternion,
zoom-to-cursor và đo ở 30 Hz trước là hợp lý. Tuy nhiên **chưa nên triển khai nguyên xi** vì:

1. `dt_ms` của Side View có dấu hiệu là giá trị mặc định thay vì phép đo. “Pan perfect” chưa được chứng minh.
2. Khoảng cách giữa camera records không phải khoảng cách giữa raw input events hoặc toàn bộ rendered frames.
   Không thể khẳng định tất cả khoảng trống dài là idle, hoặc ngoại lệ liên tục chắc chắn do pacer/GPU không liên quan.
3. State riêng cho từng view là đúng; bốn pacer có thể `sleep` trên cùng UI thread là kiến trúc không nên chọn.
4. `egui` đã giữ tương tác khi kéo ra ngoài widget. Mất capture tại mép canvas cần reproduction;
   lifecycle mất focus, release ngoài cửa sổ, tool conflict mới cần được đặc tả rõ.
5. `retarget()` hiện chỉ giữ liên tục vị trí, không giữ vận tốc; phép nội suy zoom còn có lỗi neo trung gian.
6. Cây decompile Rhino có các thân hàm camera để đọc; giới hạn của riêng Matrix UI không phải giới hạn của toàn bộ nguồn hiện có.

Hướng ưu tiên: **một scheduler không chặn cho mỗi cửa sổ, state theo viewport, một chủ sở hữu gesture
cho mỗi luồng pointer, input được tiêu thụ đúng một lần và telemetry xuyên suốt một frame**.
Đây là đề xuất kỹ thuật có căn cứ, chưa phải “tối ưu nhất” đã được benchmark.

## 2. Log nào mới nhất và chứng minh được gì?

Đã kiểm kê 72 file JSONL trong checkout, loại trừ `.git`, `ref`, `QymCAD`, `target`, `node_modules`,
`.venv`, `__pycache__`. Mới nhất theo thời gian sửa file:

| File | Thời gian Asia/Saigon |
| --- | --- |
| `build/stress-27000/nav-pointer-125.jsonl` | **2026-10-21 05:43:55.058161 +07:00** |
| `build/stress-27000/nav-hold-125.jsonl` | 2026-10-21 05:39:09.601957 +07:00 |
| `build/stress-27000/nav-rig-125.jsonl` | 2026-10-21 05:33:49.886369 +07:00 |

Kết luận “mới nhất” chỉ trong phạm vi kiểm kê này, không bao gồm log bên ngoài checkout/AppData hoặc
ổ đĩa khác. Input 156.947 byte; SHA256
`cabd7f9ebc815732a0f961c277ff0656df1f49213a33db4692a63bca5f0a4af9`.
327 dòng JSON hợp lệ: 1 `session_start` và 326 `camera`; không có `nav_frame`.
Session ghi 125 document objects, còn camera records ghi 3.750 scene objects: hai cấp đếm khác nhau.
`max_objects=500000` là cấu hình giới hạn, không phải workload đã chạy.

| Chỉ số | Tính lại |
| --- | --- |
| Orbit / zoom / pan | 201 / 113 / 12 |
| Camera records theo pane | Perspective 231; Side View 95; hai pane còn lại không có camera record |
| `dt_ms` toàn bộ: median / p95 / max | 33,515 / 725,286 / 2.001,339 ms |
| Khoảng cách giữa camera records: median / p95 / max | 34 / 758,6 / 3.536 ms |
| Khoảng cách >100 ms | 59/325 cặp |
| Cặp cách nhau ≤100 ms | 266 cặp |
| `dt_ms` trong nhóm này: median / p95 / p99 / max | 33,512 / 33,710 / 67,364 / 81,698 ms |
| Ngoại lệ `dt_ms` >40 ms trong nhóm này | 6; tất cả Perspective orbit |
| `frame_ms` trong nhóm này: p95 / max | 38,204 / 105,797 ms |
| CPU `prepare_ms`: p95 / max | 0,0194 / 0,0216 ms |
| CPU `draw_ms`: p95 / max | 0,4679 / 1,1543 ms |
| Perspective orbit radius | 74,841702313625 mm, sai khác cực nhỏ |
| Perspective zoom radius quan sát | 74,8044 xuống 8,9866 mm |

P95 trong bảng dùng nội suy tuyến tính tại `(n−1)×0,95`. Tài liệu cũ phù hợp cách lấy hạng
`floor(n×0,95)−1`: lần lượt 724,190 ms, 749 ms và 33,700 ms. Chênh lệch nhỏ này do định nghĩa percentile;
không nên xem là regression. Cần ghi rõ quy ước và số mẫu trong các lần đo sau.

### Những cách diễn giải phải sửa

- **95/95 Side View records có `dt_ms=33.333333` và `pace_late=false`**; gồm 83 zoom và 12 pan.
  Nhánh fallback ở `ui.rs:1964` tạo chính bộ giá trị đó khi pane không gọi `FramePacer::begin`.
  Đây là dấu hiệu mạnh về telemetry mặc định, không phải chứng cứ cadence hoàn hảo.
  Side zoom có median gap 17 ms; pan có cả gap 585 ms dù `dt_ms` vẫn 33,333333.
  Chưa có executable hash để chứng minh log được tạo từ chính source đang đọc, nên giữ kết luận
  “phù hợp nhánh fallback”, không khẳng định nhận dạng binary.
- **36/59** khoảng cách camera-record >100 ms có `|dt−gap|≤2 ms`. Tương quan này chỉ chứng minh
  hai timestamp thường theo cùng nhịp gọi hàm. Không có raw-event stream/pressed state để phân biệt
  idle, input starvation, missing logs và event-loop blocking.
- Sáu ngoại lệ >40 ms ở dòng log 138, 219, 265, 266, 268, 289 lần lượt là
  78,2645; 72,1621; 40,1519; 64,7811; 54,6609; 81,6979 ms. Đây là danh sách cần trace lại,
  không phải sáu nguyên nhân đã xác định.
- “Continuous” trong tài liệu hiện chỉ là bộ lọc `camera gap≤100 ms`. Nhóm này không chứa
  Perspective zoom và chứa nhiều `dt` mặc định. Không dùng p95 33,71 để nghiệm thu toàn bộ navigation.
- Radius 100 mm thuộc Side View orthographic; zoom ở đó thay span, không thay radius.
  Vì vậy **100→9 mm không phải một đường zoom Perspective liên tục**. Chưa có bằng chứng 0,2 mm,
  không clipping, chất lượng chi tiết hay cursor ổn định ở mọi frame.
- `culled=0` và `draw_calls=visible` là số counters được ghi; chưa chứng minh không culling hoặc
  đã loại toàn bộ đối tượng ngoài frustum. Cần xác định ý nghĩa bộ đếm và frame nguồn.

Đối chiếu hai capture trước cũng cho thấy vấn đề đo này chưa mới: `nav-rig-125` có 583 camera events,
trong đó cả 93 Side pan, 356 Side zoom và 54 Through Finger zoom đều có `dt` đúng target tuyệt đối.
`nav-hold-125` có 114 events và chỉ Perspective. Các log khác thao tác và không có build identity;
không dùng chênh lệch thống kê giữa chúng như một controlled A/B chứng minh bản sửa nhanh hơn.

## 3. Những vấn đề xác nhận từ source và mức ưu tiên

Source được fingerprint trong evidence; checkout có nhiều thay đổi chưa commit. Các nhận định dưới
đây áp dụng cho source được đọc, không tự động xác nhận nguyên nhân của binary đã tạo log.

| Ưu tiên | Vị trí | Phát hiện và xử lý đề xuất |
| --- | --- | --- |
| P0 | `ui.rs:1957–1968`, `frame_pace.rs:51–64` | Pacer bị gọi trong mỗi pane. Điều kiện mới gồm gesture, transition và active+any button, nên một frame có thể gọi `begin()` nhiều lần. Một view đang settle cộng một view đang drag, hoặc active khác gesture owner, có thể gây nhiều lần chờ nối tiếp. Đưa scheduling ra cấp cửa sổ. |
| P0 | `ui.rs:1956`, `1986`, `2064`, `2111` | `frame_now` lấy trước khi `begin()` ngủ, rồi dùng để sample transition. Pose có thể chậm tới một khoảng chờ. Đo input trước, lấy một `sample_now` chung tại thời điểm cập nhật camera sau khi event loop đánh thức; không ngủ giữa hai bước này. |
| P0 | `ui.rs:2108–2118` | Wheel dựng transition 140 ms mới mỗi lần xử lý pending; +16 ms là sample tương lai. Settle sau đó lại sample hiện tại; frame đến sớm có nguy cơ quay về pose cũ hơn. Dùng timeline thật và xác nhận không backtrack. |
| P0 | `ui.rs:2134–2162`, `2178`, `2201`; `gpu_viewport.rs:720–827` | Log camera được ghi trước draw của pane; prepare/draw là dữ liệu render trước đó. Các timer đo CPU và không bao gồm toàn bộ UI, lock wait, GPU execution, acquire/present. Gắn frame/revision và log sau các giai đoạn thực, không suy “GPU <1 ms”. |
| P1 | `ui.rs:1934`; egui `input_state/wheel_state.rs:137–171` | `smooth_scroll_delta` đã có smoothing; sau đó thêm pending drain rồi transition. Ba tầng có thể tăng đuôi trễ. Đo raw wheel → normalized target và chỉ giữ một tầng làm mượt có kiểm soát. Không thay tỉ lệ wheel trước khi có replay tương đương. |
| P1 | `ui.rs:2054–2060`, `2075–2106` | Pending chỉ giữ scalar; cursor lấy từ hover hiện tại. Cursor rời pane/chuyển pane có thể đổi hoặc mất anchor trong khi wheel cũ còn pending. Giữ owner+anchor theo burst, cộng gộp chỉ khi anchor tương thích. |
| P1 | `camera_transition.rs:88–95`; `camera_rig.rs:252–271` | Pivot lerp kết hợp distance/span log-lerp không giữ anchor giữa transition. Nội suy cùng một hệ số scale cho cả khoảng cách và pivot. Ở giới hạn zoom dùng tỉ lệ thực sau clamp để pivot không tiếp tục chạy. |
| P1 | `camera_transition.rs:105–113` | `retarget` sample lại pose và reset `started`; chỉ đảm bảo C0. Không có vận tốc đầu vào để giữ C1. Chọn rõ yêu cầu smoothness trước khi xem retarget là đã giải quyết hết giật. |
| P1 | `ui.rs:2235–2237`, `1268–1284` | Hover picking vẫn được gọi cho active view; camera revision thay đổi làm cache stale. Không thấy runtime call `set_moving(...)` trong app source dù setter tồn tại. Đo riêng hover/selection/overlay và nối navigation state vào render/pick policy khi triển khai. |
| P1 | `gpu_viewport.rs:665`; `ui.rs:2189–2195` | Trạng thái moving chưa nối vào call path; close mode dùng camera active, có thể khác camera gesture. Nguy cơ chọn chất lượng theo sai pane. Shared geometry policy phải tổng hợp nhu cầu các pane, không rebuild theo thứ tự pane. |
| P2 | `camera_rig.rs:219`, `camera.rs:394–399` | Clip fitting dựa khoảng cách pivot; chưa dùng toàn bộ bounds scene trong frustum. Deep zoom, vật thể nằm xa pivot và large coordinates cần test trước khi kết luận không clipping. |
| P2 | `frame_pace.rs:31–45`; `ui.rs:2017–2021` | Parse env chưa loại NaN/Infinity rõ ràng. Cần finite validation/fallback cho Hz/speed; test cấu hình sai khi sửa pacer. |

Các vấn đề bổ sung cần thiết kế nhưng chưa có reproduction: burst pointer rất lớn qua cực camera,
DPI đổi trong gesture, title/maximize làm widget biến mất, dialog/modal giành pointer, wheel đảo chiều
và drag xen với zoom pending. Không tự nhận đây đều là bug đã tái hiện.

### Capture: sửa khẳng định quá sớm

Dependency được dùng là **egui/eframe 0.36.2**, winit 0.30.13, wgpu 30.0.1.
Trong source egui 0.36.2, `Response::dragged_by` kiểm tra dragged state và button down;
`drag_delta` lấy pointer delta, không yêu cầu hovered (`response.rs:439–464`).
`is_pointer_button_down_on` được mô tả là vẫn đúng khi dragging ngoài widget (`585–596`).
OM9 dùng widget ID theo `ViewId` và `Sense::click_and_drag()` (`ui.rs:2172–2177`).

Vì vậy, rời canvas **không mặc nhiên làm mất drag**. Trước khi thay cơ chế capture, phải test current
path ở mép pane, qua pane khác và ngoài cửa sổ. Ngưỡng phân biệt click/drag cũng có thể làm chậm
khởi đầu; không thay `Sense` tùy tiện vì canvas còn xử lý selection/click.

Vẫn nên có gesture owner ở tầng OM9 để thống nhất navigation với Gumball, command preview và
selection, đồng thời xử lý focus loss/release/cancel. Một `Option<DragCapture>` trong mỗi view chưa
tự đảm bảo chỉ một view sở hữu pointer. `workspace.active` và gesture owner là hai khái niệm riêng;
không dùng active như một khóa pacing. Middle-button mapping vẫn là quyết định đã hoãn của người dùng.

### Zoom: một phép kiểm độc lập

Giữ camera orientation, anchor `q` trên mặt phẳng pivot ban đầu và factor đích `f`.
Với tiến độ đã easing `u`, code hiện dùng:

```text
p(u) = q + (p0-q) × (1-u+u×f)
d(u) = d0 × f^u
```

Tỉ lệ khoảng cách từ điểm neo tới tâm màn hình trở thành `(1-u+u×f)/f^u`, chỉ bằng 1 ở hai đầu.
Ví dụ `f=0,81`, `u=0,5`, điểm cách tâm 200 px sẽ trôi **1,111 px** ở giữa. Đây là phép kiểm toán học
cho đường nội suy, không phải số đo screenshot của log. Endpoint test hiện có không bắt được lỗi này.

Đề xuất dùng `s(u)=exp(u×ln(f))`, rồi `d=d0×s`, `p=q+(p0-q)×s`.
Orthographic dùng span tương ứng. Khi anchor là một điểm bề mặt có depth khác pivot plane,
phải ràng buộc ray/projection đúng depth đó; không áp công thức pivot-plane như thể đã chính xác
cho mọi điểm 3D. Test reprojection toàn bộ intermediate samples và dùng actual factor sau clamp.

## 4. Decompile thực sự cung cấp được gì?

Chỉ đọc các file/hàm liên quan, không chép source thương mại vào deliverable. Hash nguồn lưu trong
evidence JSON; bên dưới là behavioral summaries. Không có kiểm chứng bằng native disassembly/binary
trong lượt này, do đó không cấp nhãn exact-VA hoặc phục dựng đầy đủ.

| Nguồn riêng cục bộ | Phạm vi đã đọc | Điều chứng minh / giới hạn |
| --- | --- | --- |
| Matrix `Commands/MouseCallBackHandler.cs` | `OnMouseDown/Up`, RVA ghi trong export `0x23680/0x2369C`, managed token `0x060007A0/A1` | Forward thông báo left mouse và gọi base Rhino callback. Không phải vòng orbit/pan/zoom; không dùng làm chứng cứ đổi MMB mapping. |
| Matrix `Commands/SetCameraTargetCommand.cs` | `RunCommand`, managed token `0x06000F0A`, RVA ghi `0xBC700` | Active viewport; chọn object; xử lý instance transform; constrain point theo BRep/surface; tùy chọn bounding-box center; cancel/exit/failure và SetTarget. Có tham chiếu nghiệp vụ pivot cụ thể, không chỉ display UI. |
| Matrix `Commands/Test/TestVRayDisplay.cs` | `VRayDisplay.ExecConduit`, managed token `0x06000F85`, RVA ghi `0xBDC90` | Phân biệt dynamic display, feedback và command khi vẽ preview. Là test V-Ray, không phải cadence navigation production. Không port các Sleep/worker của đoạn test này vào viewport OM9. |
| RhinoCommon `Display/RhinoViewport.cs:694–737` | mouse wrapper methods | Truyền previous/current screen points và mode xuống native `CRhinoViewport_MouseAdjust`. |
| `rhcommon_c.dll.c:96411–96450` | Definition `CRhinoViewport_MouseAdjust`, export offset/RVA ghi `0x70380` | Null guard + dispatch modes 0…5 đến rotate/dolly/magnify/tilt. Là wrapper, không phải toàn bộ event loop. |
| `Rhino.exe.c:749258–749595` | Definition `CRhinoViewport::MouseRotateView`, export offset/RVA ghi `0x4476F0` | Camera location/direction/up lock; deltas previous/current; pivot candidate; transforms; update target/location và invalidation. Có thân camera math trong bộ nguồn hiện có. Globals/helper chưa giải nghĩa, không suy toàn bộ original semantics từ tên hàm. |
| `Rhino.exe.c:750818–750897` | Definition `CRhinoViewport::MouseDollyZoom`, export offset/RVA ghi `0x44A370` | Hệ số multiplicative từ delta; phối hợp lens/magnification và cập nhật view. **DollyZoom ở đây không đồng nghĩa wheel zoom-to-cursor**; không sao chép công thức hay hằng số chưa giải nghĩa. |

Các offset ghi trên là RVA/export offsets từ decompile; không tự cộng base của Matrix90 vào binary
Rhino x64 hoặc coi là absolute VA. Mapping confidence: symbol/body đã tìm đúng definition theo tên;
absolute entry address và assembly correspondence chưa kiểm. Behavioral confidence cao cho wrapper
dispatch, trung bình cho native math do unresolved globals và dấu hiệu decompiler type recovery.

Câu “Matrix UI không có toàn bộ camera loop” có thể đúng trong phạm vi UI tree, nhưng cần bổ sung
Rhino native sources thay vì dừng tìm kiếm ở đó. Lần đọc tiếp theo nếu cần fidelity: chọn
`MouseLateralDolly`/`Magnify` và caller event handler theo đúng câu hỏi còn thiếu; không đọc cả export.

## 5. Bài học từ các dự án mở

Đã đọc nguồn upstream trực tiếp ngày 2026-10-21. Các link branch `main/master` có thể thay đổi;
đây là đối chiếu thiết kế, không phải benchmark so sánh các ứng dụng trên cùng máy.
Không copy code upstream trong lượt này.

| Nguồn chính | Phần có ích cho OM9 | Điều không nên suy ra |
| --- | --- | --- |
| [Blender navigation](https://raw.githubusercontent.com/blender/blender/main/source/blender/editors/space_view3d/view3d_navigate.cc) và [smooth view](https://raw.githubusercontent.com/blender/blender/main/source/blender/editors/space_view3d/view3d_navigate_smoothview.cc) | Tách modal input/direct manipulation khỏi timed transitions; transition lưu theo region; elapsed time, endpoint exact, hủy timer khi xong. Smooth view có timer 100 Hz và smoothstep. | Không phải cứ có timer nhanh là sai, cũng không phải cần 100/120 Hz cho OM9. Phải tách logical animation timeline khỏi số lần thực sự submit/present. |
| [FreeCAD NavigationStyle](https://raw.githubusercontent.com/FreeCAD/FreeCAD/main/src/Gui/Navigation/NavigationStyle.cpp) | State machine rõ, reset button state về idle, sở hữu interaction và animation; animation có thể bật/tắt. | Không khẳng định mọi FreeCAD navigation đều không inertia. Không lấy mouse mapping của FreeCAD làm default Matrix9. |
| [KiCad 3D canvas](https://gitlab.com/kicad/code/kicad/-/raw/master/3d-viewer/3d_canvas/eda_3d_canvas.cpp) | Lấy elapsed time khi paint, nội suy camera và request refresh; redraw được post/timer một lần, không ngủ trong handler; pan gesture trực tiếp. | Không phải mọi wheel event đều direct; current source có animated camera path. Không áp các hằng số timer/speed của KiCad nếu chưa đo OM9. |
| [OCCT AIS_ViewController](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/Visualization/TKV3d/AIS/AIS_ViewController.cxx) | Phân tách input buffers và flush/update; coalesce zoom cùng cursor point, cộng pan deltas; explicit start/stop/abort. Đây là tham chiếu CAD sát với ownership/input của OM9. | Không đưa AIS viewer hay native interaction owner vào OM9. Ý tưởng buffer/coalesce được viết trong Rust; native vẫn chỉ giữ capability hình học cần thiết. |
| QymCAD `crates/qymcad-ui-state/src/navigation.rs` | Đã đọc enums và helper zoom: tách cursor/center, pivot cố định trong orbit gesture, bù anchor sau clamp bằng scale thực. | Đây là code reference, không phải Matrix UI/default, không thêm dependency build theo path. Không tự áp các preference upstream. |

Về nền tảng hiện dùng: [winit ControlFlow](https://docs.rs/winit/0.30.13/winit/event_loop/enum.ControlFlow.html)
có `WaitUntil` thức dậy khi có input hoặc đến deadline; khi đi qua eframe, ưu tiên cơ chế
[egui request_repaint_after](https://docs.rs/egui/0.36.2/egui/struct.Context.html#method.request_repaint_after).
Không tự giành event loop của framework nếu hook sẵn có đủ dùng.
[wgpu timestamp writes](https://docs.rs/wgpu/30.0.1/wgpu/struct.RenderPassTimestampWrites.html)
cho phép đo GPU pass khi adapter hỗ trợ; GPU query không thay thế đo presentation.
[PresentMode](https://docs.rs/wgpu/30.0.1/wgpu/enum.PresentMode.html) có queue/blocking riêng.
Phải ghi mode thực tế và thời gian acquire/present trước khi quy lỗi cho clock.

## 6. Thiết kế đề xuất để phát triển lâu dài

### 6.1 Ownership và scheduling

```text
Input nhận vào → chuẩn hóa + timestamp + owner arbitration
             → NavState[ViewId] (target, anchor, phase, pending input)
             → cập nhật tất cả view cần đổi tại cùng sample_now
             → publish Camera/revision một lần mỗi view
             → invalidate đúng resources + render các view cần đổi
             → một lần yêu cầu repaint cho deadline sớm nhất của cửa sổ
```

- Giữ một đồng hồ monotonic chung làm nguồn thời gian, timestamps riêng từng view để tính elapsed.
  **Một scheduler/window**, không phải một mutable pacer bị gọi nhiều lần trong vòng bốn pane.
- `NavState` giữ transition/anchor/pending và phase (`Idle`, `Dragging`, `Settling`).
  `GestureSession` ở cấp input router giữ view, pointer/button, mode, press pose và ownership.
  Nhiều transition ở nhiều pane có thể chạy song song; chỉ một owner cho một luồng pointer.
- Camera trong workspace vẫn là nguồn được persist. Rig là derived/ephemeral; phải tái đồng bộ khi
  Home/preset/Open/restore/Undo view thay camera. Đừng tạo hai pose canonical độc lập trong NavState và workspace.
- Bỏ `thread::sleep` trong hàm xử lý UI/pane. Input đến sớm được tiếp nhận/coalesce; render nặng chỉ
  khi cần hoặc tới deadline. `request_repaint_after` có thể bị input đánh thức sớm: không xem nó như
  một hard FPS cap và không đơn giản xóa sleep rồi render mọi event.
- Lập deadline theo cadence; nếu trễ, sample thời gian thật, bỏ slots render đã quá hạn, không chạy
  nhiều frame bù. Kết thúc đúng target. Reset cadence sau idle/suspend, không đưa idle vào histogram active-frame.
- Egui có thể chạy nhiều layout passes. Tiêu thụ input/cập nhật navigation theo một frame token,
  không theo từng lần layout; geometry/render invalidation theo camera revision thực đã đổi.
- Khi giữ nút nhưng đứng yên, có thể giữ gesture và lightweight heartbeat mà không redraw geometry
  liên tục. Đứng yên không phải một frame “bị mất”. Chỉ cần clock bookkeeping không kế thừa idle gap sai nghĩa.

### 6.2 Input và chuyển động

- Direct drag không thêm quán tính mặc định. “1:1” ở đây là phản hồi trực tiếp theo delta, không phải
  một pixel luôn bằng một đơn vị thế giới: pan dùng camera basis/world-per-pixel; orbit dùng sensitivity đã xác lập.
- Chặn gesture mới khi tool/modal khác thực sự sở hữu pointer. Release, focus loss, hidden window,
  Escape, view bị đóng/maximize phải có kết thúc xác định; release ngoài pane không đổi owner.
- Bắt đầu drag từ pose **đã hiển thị**. Nếu advance transition lên thời điểm chưa render rồi mới bắt
  đầu drag, có thể tạo bước nhảy. Ghi rõ policy interruption và test trước khi đổi hành vi.
- Chuẩn hóa wheel Line/Point/Page/trackpad theo đường input của framework, giữ modifiers/device phase.
  Mỗi event chỉ tiêu thụ một lần; event còn tail thuộc pane/anchor ban đầu, không chuyển theo hover sau đó.
- Cộng target trong log-distance/log-span; giữ tổng delta có dấu. Hai sự kiện đảo chiều không phải đợi
  backlog zoom-in chạy hết mới zoom-out. Chỉ coalesce các delta có anchor/owner tương thích.
- Tránh định nghĩa độ trễ theo “2 steps mỗi frame”: log có 252 points = 6,3 steps theo quy ước 40 points,
  cần ít nhất 4 lần drain, chiếm bốn slots 30 Hz (khoảng 100–133 ms tùy thời điểm slot đầu), rồi còn settle.
  Đây là ví dụ với một delta đi vào pending, không phải đo độ trễ thực của mọi wheel event trong log.
  Ở 60 Hz backlog giảm khác hẳn.
  Có thể cập nhật target ngay, giới hạn đáp ứng theo thời gian và khoảng cách trong log-space; đo cảm giác trước khi chọn.
- Bản sửa đầu có thể dùng retarget C0 với settle deadline hữu hạn. Nếu đo vẫn thấy đổi vận tốc gắt,
  thử Hermite trong log-space với vận tốc đầu liên tục và ràng buộc không overshoot; không thêm spring
  cho mọi drag. Đảo chiều/target sát nhau phải được test. Các mốc 80/140 ms là tham số A/B, không phải oracle.
- Giữ anchor đúng trong mọi intermediate sample, sau clamp và khi viewport/DPI đổi. Surface pivot là
  nâng cấp tương lai cần pick snapshot hợp lệ theo revision; chưa thay pivot-plane behavior đã có.

### 6.3 Renderer và khả năng mở rộng

Trước tiên sửa scheduling và đo cả UI/pick: log hiện chưa chứng minh GPU hay triangle count là thủ
phạm của scene 125. Camera change chỉ nên cập nhật uniforms/visibility; không đổi tessellation chuẩn,
document, selection identity, Undo hoặc export.

Sau khi có GPU timings: nếu GPU nặng, tận dụng dirty views, retained geometry/instances và LOD hiện có;
chọn pixel error/dynamic resolution trong giới hạn accuracy đã duyệt, có hysteresis để không nhấp nháy.
Selected/small jewelry features vẫn giữ policy chất lượng hiện hành. Các view khác camera giữ lại
texture nếu scene/selection/overlay không đổi. Đo tổng frame bốn view, không cộng hoặc nhân một timer
pane thành GPU cost của cửa sổ.

Worker chỉ xử lý dữ liệu owned/immutable; camera math rất nhẹ nên ưu tiên UI Rust. Chỉ chuyển culling/
LOD prep nặng sang worker khi trace chứng minh cần; kết quả phải có scene+camera revision, bỏ kết quả
cũ để không publish trạng thái lạc hậu. Chưa có lý do để chuyển navigation sang C++/Rhino runtime.

Khi local 60 Hz khả thi, scheduler có thể hỗ trợ refresh rate cao hơn mà không nhân số view timers.
RDP phải đo riêng: input delivery, encode/network/compositor có thể làm trễ dù app ổn định.
Predictive camera/timewarp, AI smoothing hay thread 120 Hz riêng chưa có nhu cầu được chứng minh.
Loại chúng khỏi đợt đầu vì chi phí/độ phức tạp, không khẳng định chúng luôn sai trong mọi tương lai.

## 7. Telemetry và acceptance cần thay

Không dùng `gap_ms` cho nhiều nghĩa. Schema tối thiểu:

| Nhóm | Trường cần có |
| --- | --- |
| Session | schema/build/executable hash, source fingerprint + dirty state, adapter/backend, vsync/present mode, monitor Hz, size/DPI, local/RDP, workload fingerprint, quality/environment settings |
| Input | `input_seq`, receive timestamp, unit/phase, pointer/button/modifiers, owner view, active view, anchor, captured/cancel reason |
| Frame | `frame_id`, `sample_now`, phase, camera revision before/after, dirty views, pacer call count, deadline/wake lateness, idle/active classification |
| Latency | receive→dispatch, dispatch→camera publish, publish→submit, acquire/present durations; chỉ gọi OS/hardware input age nếu thực sự có timestamp đó |
| Motion | target/display distance hoặc ortho span, orientation/up, anchor reprojection error, pending total, retarget count/generation, progress trong từng generation |
| Work | full UI CPU, pick/overlay, scene lock/prepare, encode/submit, supported GPU queries, async completion; gắn frame/revision cho stats chậm một frame |
| Logging | every active/settle frame kể cả zero delta, dropped-log count; buffer giới hạn để disk write không chặn UI |

Logger hiện `writeln!` trực tiếp trên UI path; tốc độ đĩa là biến đo cần kiểm soát. Lượt này chưa đo
chi phí logger và không quy lỗi các outlier cho nó. Nếu thêm buffered/background diagnostics, flush
ở shutdown và ghi số record bị bỏ; không đánh đổi tính đầy đủ của bằng chứng một cách im lặng.

Sửa test “progress monotonic khi retarget”: `retarget` reset `started`, nên normalized progress của
generation mới sẽ về 0 hợp lệ. Test continuity của pose/scale, tổng target, bounded velocity và
termination; chỉ kiểm monotonic progress **trong một generation**, hoặc dùng một timeline toàn gesture.

### Ma trận kiểm chứng đề xuất

Đây là queue đề xuất, chưa phải các batch đã chạy:

| Batch | Trọng tâm và oracle |
| --- | --- |
| N0 — telemetry | Không dt giả; frame/input/present phân biệt; log đủ settle và loss counters; replay phân loại idle độc lập với measured gaps. |
| N1 — ownership | Active≠drag pane; settle A + drag B; không double-consume hoặc nhiều sleep; camera khác không vô tình thay đổi. |
| N2 — lifecycle | Press/move/hold/release; qua pane/window boundary; focus loss, modal, Escape, maximize, view restore, Gumball/command conflict; idle thật không render liên tục. |
| N3 — wheel | Small wheel, burst ±252 points và lớn hơn, fractional trackpad, reversal, pointer rời pane, đổi DPI; tổng log-scale đúng và chốt target hữu hạn. |
| N4 — camera correctness | Reprojection ở mọi sample; f64/large coordinates; pole crossing; clamp không kéo pivot; ortho span; near/far 100→0,2 mm trên fixture có kích thước đã biết; camera/picking nhất quán. |
| N5 — controlled performance | 125 + 729, 1V/4V, 100/125/150% DPI, held-still vs continuous motion, logs on/off; tách cold/warm; native local và RDP riêng. |
| N6 — scale/60 Hz | Chỉ sau N0–N5; workload 27k đã có, selected/LOD0 và workload nhiều instances; GPU/VRAM/latency/accuracy cùng báo cáo. |

Mỗi run giữ executable hash, input replay, môi trường và số active intervals. Đề xuất tối thiểu
ba lần repeat cùng workload sau warm-up để phân biệt nhiễu; báo số mẫu và cả outliers, không bỏ
frame >100 ms chỉ để làm đẹp p95. Threshold baseline 30 Hz `p95≤35 ms`, `max≤40 ms` chỉ áp cho active
application frame intervals trong môi trường đo có kiểm soát; trace mọi vi phạm. Không dùng timer
pacer hoặc `stable_dt` thay presented-frame interval. Input latency, anchor error, settle deadline
và full-quality restoration phải có oracle riêng trước khi triển khai; 30 Hz đều vẫn có thể bị trễ.

Các suite hiện có (`interaction_log`, `navigation_input`, camera rig/transition, picking) chỉ là điểm
khởi đầu; test wheel hiện xác nhận radius giảm và target đổi, chưa chứng minh anchor đứng yên hoặc
thời gian settle. Không xóa `camera_motion`/tests chỉ vì tài liệu ghi obsolete: kiểm consumers/export
và regression coverage trước khi dọn. Lượt này không chạy lại Rust/UI vì chỉ phân tích và ghi tài liệu.

## 8. Chỉnh kế hoạch gốc và bước tiếp theo

| Phần trong tài liệu gốc | Đánh giá / sửa cần làm |
| --- | --- |
| §1 Log evidence | Giữ counts, thêm hash/timezone/percentile definition, split theo pane; đổi “input gaps” thành “camera-record gaps”; bỏ suy “GPU không phải thủ phạm”. |
| §2 Root causes | Tách confirmed source issues và hypotheses. Capture mất tại mép pane chưa được chứng minh; thêm stale sample time, stats trễ frame, smoothing chồng và hover path. |
| §3–4 Chosen design | Giữ state theo view; thay per-view blocking pacer bằng một nonblocking scheduler/window. Camera canonical chỉ một nguồn. |
| §5 Input | Capture dùng ownership thực, tôn trọng tool/modal/CMD focus; wheel anchor/units/conservation; retarget C0≠C1; interpolation cùng scale. |
| §6 Frame loop | Không giữ sleep trong hướng thiết kế chính. Schedule một lần sau tổng hợp dirty/animation; không redraw chỉ vì nút còn giữ. A/B hiện trạng sleep có thể dùng làm baseline. |
| §7 Instrumentation | Add build/frame/input IDs, phase, true timing source, GPU/present support, log completeness và overhead. Env flags trong tài liệu chưa tồn tại trong runtime; ghi là proposed. |
| §8 Acceptance | Đo từng action/view; không loại gaps bằng ngưỡng rồi gọi continuous. Thay progress monotonic qua retarget bằng trajectory/target/deadline oracle. 0,2 mm còn pending. |
| §9 Phases | Đưa telemetry + targeted reproduction lên trước refactor; scheduler/owner thành một slice; wheel/anchor thành slice tiếp theo; benchmark và quality gate sau. |
| §10 Scope | Giữ Rust-first, geometry/accuracy invariants. Bổ sung exact navigation owner M02 thay vì chỉ ghi M01; không đổi gesture mapping từ nguồn ngoài. |

Traceability: M01 `OM9-L2-01-06` sở hữu render loop/invalidation;
`OM9-L2-03-01` sở hữu layout/active view. Navigation thuộc M02 `OM9-L2-03-02`, đặc biệt
L3-001/002/003/004 (Perspective orbit, ortho pan, wheel cursor zoom, Shift-right pan).
Không tạo owner navigation thứ hai hoặc nâng M01/M02-UI gate từ review này.

**Thứ tự thực hiện nên chọn:** N0 telemetry và reproduction N1/N2 → scheduler không chặn + ownership
→ wheel target/anchor/retarget → N4 correctness → N5 30 Hz → N6 60 Hz/scale nếu trace cho phép.
Trước khi sửa source, dùng review này để chỉnh kế hoạch gốc; sau đó triển khai từng slice với replay
định trước. Lượt review này giữ nguyên business rules/skills và mã runtime.

## 9. Kiểm chứng của lượt review

- Script phân tích chạy trực tiếp trên log; SHA256 và số dòng đã kiểm lại; JSON hợp lệ, đủ action totals.
- Phép tính độc lập xác nhận cách percentile của tài liệu và sai lệch anchor giữa transition.
- Source/dependency/native definitions được đọc có giới hạn; hashes lưu trong evidence.
- Public-source audit lần đầu: PASS, 8.713 source files, 511 icon bindings, 510 SVG, `errors=[]`.
  Kết quả cuối và kiểm tra liên kết/JSON được lưu riêng trong review checks.
- Không rebuild executable, không chạy fixture/UI/GPU benchmark mới, không ký manual acceptance.
  Chưa có controlled A/B hay log 729 mới để chứng minh mức cải thiện. Không tuyên bố completion từ phân tích.
