# Viewport navigation — implementation checkpoint 2026-10-21

Trạng thái: **runtime đã thay đổi; nghiệm thu hiệu năng và UI đầy đủ còn mở**. Không dùng số đo
app/submit để tự chứng nhận presented frames. Hồ sơ forensic trước đó là lịch sử, giữ nguyên.

Kế hoạch được người dùng chấp thuận: [implementation plan](../../design/M01/VIEWPORT-NAVIGATION-IMPLEMENTATION-PLAN.md).
Owner navigation: M02 `OM9-L2-03-02`; render/resources/invalidation: M01 `OM9-L2-01-06`.

## Code và hành vi

| Phần | Thay đổi thực tế |
| --- | --- |
| `om9-app/navigation.rs` | State từng pane, một gesture owner, direct orbit/pan, không inertia; wheel raw → log target → nghiệm critically damped, chốt 180 ms/speed; hủy tail khi drag; rig đồng bộ toàn bộ camera workspace, kể cả restore pose có cùng revision |
| `om9-app/frame_scheduler.rs`, `ui.rs` | Một clock cho cửa sổ; deadline không sleep; input một lần qua nhiều layout passes; giữ capture qua pane, kết thúc khi focus/Escape/modal/pane biến mất; bỏ state của pane ẩn để không tick vô hạn |
| `camera_rig.rs` | Clamp góc trước khi qua pole; clamp zoom trước khi bù anchor để dùng cùng scale thực; quaternion/f64 được giữ |
| `gpu_viewport.rs` | Q0–Q3 resolution/MSAA; camera/scene/selection/size/generation guards; worker sở hữu snapshot; resource upload chung 8 MiB/2 ms mỗi lượt; cache theo document identity; texture chốt chỉ publish sau GPU queue completion |
| `om9-render/scene.rs` | Mesh unique + occurrences; GPU draw instanced; snapshot camera và instance buffer riêng mỗi pass; encoder API dùng chung cho cửa sổ, wrapper headless được giữ |
| `om9-render/overlay.rs` | Upload tăng dần; camera/model uniforms bất biến theo pass; chia sẻ resident buffers khi đổi MSAA, không dựng lại tất cả curves |
| `om9-render/timing.rs` | Timestamp queries tùy capability, readback không chờ CPU; tag frame gốc; không gọi timestamp là presentation |
| `interaction_log.rs` | Queue giới hạn 4096, BufWriter 64 KiB, flush định kỳ, session/executable/source fingerprints, dropped records; raw navigation sequence, deadline, phase, requested/displayed camera, encoded/published/GPU frame IDs |
| Hover | Dừng hover khi camera chuyển động; truy vấn hover lớn chạy trên owned snapshot, không chặn UI sau release; kết quả được kiểm tra document/camera/scene/viewport |
| Picking bounds | Sửa enclosing sphere bị thiếu góc hộp và bỏ qua scale/shear. GPU culling dùng cùng quy ước row-major với dynamic model; static API cũ vẫn column-major |

Ảnh cuối dùng full physical target, MSAA 4×, nguồn mesh LOD0, các curves hiện có và grid/CPlane đúng
pane. Đã bỏ focus-distance 3 mm khỏi đường chốt. Không đổi geometry chuẩn, file schema, Undo hoặc
native camera runtime. Khi Q0 hiện hữu đã đầy đủ và camera/source không đổi, chờ queue tail rồi tái
chứng nhận texture cache; không dựng lại geometry chỉ vì chuyển phase.

**Lựa chọn bảo thủ:** các profile preview hiện giữ mesh LOD0 cho mọi đối tượng; sai số decimation
bằng 0, nhỏ hơn các trần 1/2/4 px. Resolution/MSAA thích ứng hoạt động. Không lấy QEM quadric cost
làm chứng nhận Hausdorff hoặc screen-space error. Tích hợp decimation có chứng nhận sai số vẫn là
cơ hội tối ưu tiếp theo, chưa được tuyên bố đã có.

## Kiểm tra tự động đã chạy

- App library + navigation/input/logger/selection/Gumball/layer persistence: 180 passed trong 10 suites
  ở checkpoint trước; lifecycle + precision + deadline mới nhất: 12 passed, gồm DPI/layout owner biến mất,
  nhiều egui passes, focus/Escape, interruption, pole và anchor ở tọa độ 1e9/deep zoom 0,2 mm.
- Scene camera/grid/picking/selection + snapshot: 30 passed trong 6 suites; ca mesh vertex trước đó
  thất bại đã qua sau khi sửa conservative sphere.
- Render retained camera/timestamps/fallback + AA + render loop: 15 passed trong 3 suites.
- Scene bench + retained camera: 17 passed trước khi thêm ca timestamp fallback; 4 benchmark đo riêng
  được ignore theo cấu hình. Không cộng các lượt chạy lặp thành số test độc lập.
- Build/check dùng `--target-dir L:/om9-target -j 2`; release `opt-level=3`, thin LTO.
- `tools/public_source_audit.py`: 8724 source files, 511 bindings, 510 SVG, 0 errors ở checkpoint.

Các số cuối và command receipts được cập nhật trong JSON đi kèm sau batch hoàn tất.

## Benchmark và giới hạn chứng cứ

Harness: [benchmark_viewport_navigation.py](../../../implementation/tools/benchmark_viewport_navigation.py).
Mỗi batch đóng băng executable riêng; mỗi child warm geometry rồi mới replay orbit/pan/wheel.
Không loại active outliers. `generation=0` là cold initialization, không trộn vào warm refinement.
CPU encode, GPU timestamps, app interval và refinement quan sát GPU completion là các chỉ số riêng.

Các checkpoint trước batch cuối được giữ riêng:

- `build/standalone-validation/ui-artifacts/navigation-20261021-081510/summary.json`: debug baseline;
  cho thấy rebuild MSAA/overlay và chi phí CPU debug, không dùng làm kết quả release cuối.
- `build/standalone-validation/ui-artifacts/navigation-20261021-082343/summary.json`: release pilot;
  số refinement cũ còn gồm cold frames và thời điểm submit, không phải gate warm GPU completion.
- `build/standalone-validation/ui-artifacts/navigation-20261021-084114/summary.json`: queue-completion
  pilot, warm refinement được tách cold và giữ tất cả active outliers; chưa đủ ma trận/nghiệm thu.

PresentMon 2.6.0 được tải từ GitHub chính thức, SHA256
`b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af`; thử đối chiếu 2.3.1,
SHA256 `364e5d98d4d134bd54dd25c22ed2ca2f4883f8bc3ed6502bee0c151e3436d30c`.
Chỉ nhắm PID child của harness và ETW session tên riêng; không đổi quyền tài khoản, không tắt session
của người dùng. Pilot date-time có CSV nhưng chưa có phép nối clock đáng tin; các lần QPC capture
chưa thu được CSV ổn định. **Presentation acceptance vẫn chưa chứng minh.** Không suy ra quyền
elevation hay loại phiên Console/RDP từ việc có hardware GPU.

DPI 125/150; batch trước sửa giữ làm regression evidence, không gọi là PASS.

DPI trong harness là egui scale simulation 100/125/150%; không thay thế nghiệm thu DPI màn hình,
OS focus/capture ngoài cửa sổ hoặc RDP. Vẫn cần kiểm tra manual ở các cấu hình thật.

## Phần còn mở

- Đạt toàn bộ p95/p99 presented-frame và receive→submit targets, A/B queue 1/2 đầy đủ, classification
  local/RDP và capture presentation hợp lệ; các outlier đã thấy phải được giữ và giải thích.
- GPU memory-pressure/device/surface recovery trong cửa sổ thật; GPU timestamp fallback đã có test
  nhưng không thay thế fault injection native. Rust allocation abort không được gọi là recoverable.
- Cảnh cực lớn: upload có budget, nhưng một draw/encode hoàn chỉnh vẫn có thể vượt ngân sách;
  chưa chứng minh giới hạn thời gian GPU của mỗi lượt refinement hoặc render tất cả tập dữ liệu nặng.
- So sánh ảnh cuối với ảnh độc lập hiện dùng synthetic LOD0/4×, bốn cameras và occurrences;
  chưa phải chứng nhận mọi CAD boundary/isocurve/display mode của toàn bộ milestone.
- F64 camera/pick math có fixture tọa độ 1e9; raster GPU còn f32, chưa có origin rebasing tổng quát.
- Chưa nâng bất kỳ manual UI/performance gate nào thành PASS; chưa sửa reusable skill/business rules.

## Dự thảo skill — chưa áp dụng

Trước: wheel qua nhiều tầng smoothing/drain/transition, pacing sleep theo pane và không phân biệt
khung xem trước với ảnh chốt có chứng cứ. Sau: direct drag, wheel analytic một tầng, scheduler theo
cửa sổ và khung chốt được kiểm tra identity + GPU completion.

Đề xuất bổ sung vào skill workspace/navigation sau khi người dùng đồng ý:

> Rotate/pan phản hồi trực tiếp, thả dừng; wheel neo con trỏ dùng một response critically damped
> và chốt target trong 180 ms/speed. Một scheduler/window và một pointer owner; camera workspace
> là nguồn lưu. Preview được giảm resolution/MSAA; khung chốt chỉ được xác nhận sau GPU completion
> khi document/scene/camera/selection/size/generation khớp, full resolution/4×/LOD0. Không suy app
> timing thành presentation acceptance và không nâng manual gate từ test tự động.

Theo `AGENTS.md` và `skills/openmatrix9-workflow/references/business-rule-updates.md`, văn bản này
chỉ là đề xuất; bản skill nguồn và bản cài đặt chưa được sửa.

### Initial matrix (before DPI guard)

`build/standalone-validation/ui-artifacts/navigation-final-20261021/summary.json` contains 108 fixed-binary runs: 72 exited normally and 36 panicked on oversized 1V targets at simulated 125/150 DPI. Its folder name predates the discovered failure; this is a failed checkpoint, not final acceptance. All raw records and stderr remain available. `navigation-dpi-guard-20261021` removed the allocation panic in a debug reproduction but timed out during teardown after a complete screenshot/log; it is not a clean process-exit PASS. A later corrected release batch is required.

### Scheduler checkpoint

Native traces revealed a strict-deadline boundary issue: an event-loop delivery a fraction early could be rejected and force a cached frame for one more vsync. The scheduler now permits <=1 ms CPU dispatch lead while keeping absolute cadence and sampling the camera at real now. A deterministic 120-slot early-delivery regression test passes. Quality feedback is paused while refining so asynchronous completion cannot masquerade as 60 consecutive cheap preview frames.

### Timing definitions

Benchmark schema v2 reports `geometry_submit_interval_ms` from monotonic logger timestamp differences (1 ms resolution, interval quantization up to ±1 ms) between actual encoded frames within each continuous moving phase. Scheduler dt alone is insufficient because it also advances while resources are pending. Both metrics and all raw records are retained. `active_frames` counts OM9 geometry submissions, while `active_app_records` includes moving frames that reused a cached texture; neither is claimed to be a physical presentation count. Receive-to-submit samples describe the latest received input batch incorporated in a submission, not OS device-to-photon latency.

### GPU bridge regression

`om9-app/tests/gpu_navigation_publication.rs`: 2 hardware-backed tests pass with timestamp queries disabled. They render through the actual egui/wgpu bridge, reject an obsolete generation, force a new encode for a restored pose with an unchanged revision, invalidate equal-epoch/equal-camera caches when document identity changes, reject an oversized texture request before allocation and recover on resize without changing the document. These tests complement the independent four-camera pixel comparison in om9-render.

### Corrected DPI matrix and input-cadence checkpoint

`build/standalone-validation/ui-artifacts/navigation-final-r2-20261021/summary.json`: 108/108 clean
exits, screenshots and closed logs; no dropped records or receipt-integrity errors. See
[R2 checks](2026-10-21-navigation-r2-checks.json). Executable SHA256
`91a6de9c07728cdf35f8a46be2d4adfcc4caa92a5f6b0f67b1803a7e3bfe6ce9`;
compiled four-source byte fingerprint `73cbacf4addce8014bd68c48617f044d46ac14ac547c75276199bafbaf426828`
(the LF-normalized text fingerprint is `c6e9bfbab1fca31e63170796e52280ac99da47d1df56287569b2f89079eb3be7`).
The different hashes reflect CRLF bytes versus normalized text, not two binaries in one batch.

All 108 runs exceeded at least one literal geometry-submit interval threshold (216 p95/p99
observations). Inspection of raw input sequences found the diagnostic replay itself resetting a
relative timer at each callback: fractionally early deliveries suppressed an input sample, then
delayed it until another refresh. Cached frames without new camera input are expected; these gaps
cannot all be attributed to renderer throughput. The records are retained without deleting outliers.

The opt-in replay now uses absolute deadlines with a <=1 ms CPU dispatch lead and skips obsolete
slots instead of generating catch-up input. It still samples actual time and uses the normal raw
event router. A deterministic regression through the replay checks 120 slots at each of 30/60 Hz,
early callbacks, duplicate callbacks and a long late delivery. The production navigation response
and sensitivity are unchanged by this diagnostic correction. A new fixed-binary matrix is required
before interpreting cadence numbers.
