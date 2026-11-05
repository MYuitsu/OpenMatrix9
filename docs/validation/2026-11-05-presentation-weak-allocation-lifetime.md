# RAM snapshot viewport: allocation sống qua Weak — 2026-11-05

Phạm vi R1: presentation snapshot/cache trong `gpu_viewport.rs`. Không đổi geometry, mesher/export profile, native adapter, input/Gumball hay phiên OM9 người dùng. [Plan](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md); [RAM diagnostics](2026-11-05-on-demand-memory-diagnostics.md).

## Audit và lỗi tái hiện

Giả thuyết lease chain của incremental scene giữ khoản buffer đã mất chưa được chứng minh. Source `Scene::update_owned` chỉ chia sẻ khi tất cả geometry/placement/identity cũ giữ nguyên; addition là bổ sung, còn thay/xóa geometry đi qua candidate đầy đủ với owner riêng. Không thay lease chain khi chưa có lỗi tái hiện; các regression New/Open/Undo/Redo/snapshot tiếp tục kiểm vòng đời này.

Một thiếu sót riêng đã tái hiện ở presentation cache: nó giữ `Weak<TrackedPresentationSnapshot>`, nên sau last strong drop, allocation chứa struct vẫn sống dù các field/data đã drop. Reservation nằm trong struct cũ bị trả cùng fields; Weak cache vẫn giữ allocation không còn charge.

Run đỏ chuẩn `build/standalone-validation/ui-artifacts/presentation-allocation-20261105-054749`: 10 passed, 1 failed, exit 101. Fixture đã đăng ký trước buffer lease rỗng của main scene để không lẫn khoản đó với snapshot metadata. Kỳ vọng cache Weak còn sống giữ phần allocation; thực tế CPU charge về baseline quá sớm. Run hiệu chuẩn trước đó trộn lease rỗng vào baseline, không dùng chênh lệch của nó làm số đo lỗi.

## Sửa owner bằng std Rust

Scene của presentation snapshot chuyển vào `Box<Scene>`, vẫn dùng chính metadata/arena reservation hiện có. Vì vậy last strong drop trả các Vec/Scene/data riêng mà không phải giữ toàn Scene struct trong allocation của Weak presentation.

Allowance cho presentation allocation nằm trong `Arc<ExternalMemoryReservation>` dùng chung bởi snapshot và cache entry. Khoản này tính đủ fixed `TrackedPresentationSnapshot`, reservation Arc body và hai Arc control/alignment allowances. Không tính Scene body hai lần: Scene có Box allocation riêng đã nằm trong Scene metadata estimate. Cloning allocation owner không tạo reservation thứ hai. Mark allocated chỉ sau khi các Arc/Box thực sự được dựng; geometry buffers tiếp tục dùng lease cũ, không sao chép toàn scene.

Cache tuple giữ Weak trước allocation owner: khi entry retired, Weak được drop trước guard. Nếu strong snapshot vẫn còn, nó giữ guard; nếu strong đã hết thì cache giữ guard cho allocation còn sống vì Weak. Entry Weak hết strong được prune trước admission successor, tránh giữ một allocation cũ không dùng suốt lúc xin quota mới. Không dùng destructor lấy cùng cache lock hoặc deferred queue, tránh một vòng khóa mới trên đường drop.

Đổi này tăng bookkeeping hợp lệ trong live snapshot và giữ đúng một khoản nhỏ sau last strong drop nếu cache vẫn chứa Weak. Đó là allocation còn thật, không gọi là leak hoặc peak lịch sử. Dọn entry trả khoản đó. Không giữ source document/buffers/arena chỉ để giữ allowance header. Cache container/SharedResourcePool owners ngoài phạm vi này vẫn cần audit; đây không phải bằng chứng toàn RAM/RSS đã được hạch toán.

## Oracle và kiểm thử

Windows x64, default Cargo features, target riêng `E:\om9-surface-snap-validation`, `-j 2`, dev debug=0/incremental=0; OCCT 8.0.1 SDK/TBB/jemalloc như trước. Không tạo cửa sổ desktop mới.

Ba regression mới dùng ba document rỗng có template tolerance:

- Last strong snapshot chết: scene metadata/arena riêng được trả, fixed presentation allocation còn đúng charge khi Weak cache còn; prune entry trả charge về baseline.
- Cache bị drop trước strong reader cuối: allowance còn đến khi reader cuối thả; source document đúng.
- Successor dùng lại cache đã expired: không giữ thêm charge của predecessor; không tự tạo thêm một geometry reservation.

Các oracles New, Undo/Redo, Snapshot/Master và metadata lifetime được cập nhật bằng equality: sau last strong drop, còn `presentation_snapshot_overhead()` vì Weak thực sự còn; sau explicit prune về đúng current-scene baseline. Không đổi sang bất đẳng thức rộng hoặc xóa assertions. Existing sharing/low-budget refusal/retained picking, cancel và actual capacity checks giữ nguyên. Một compile lượt đầu còn tên field cũ trong test, đã sửa theo owner mới trước lượt xanh; không tính lượt đó là pass.

Batch cuối `build/standalone-validation/ui-artifacts/presentation-allocation-20261105-055144` kết thúc exit 0: **176 distinct passed, 1 ignored, 0 failed**; GPU log không có SKIP.

| Nhóm | Passed / ignored | Elapsed gồm build (giây) |
|---|---:|---:|
| Presentation snapshot (3 mới nằm trong 13) | 13 / 0 | 26,77 |
| Hover | 14 / 0 | 9,77 |
| Incremental scene memory | 7 / 0 | 1,56 |
| Replacement budgets | 8 / 0 | 4,66 |
| STL phase/session | 58 / 0 | 1,68 |
| GPU offscreen publication | 29 / 0 | 33,91 |
| B-rep picking/Gumball interaction/viewport/overlay | 47 / 1 | 19,93 |

Một overlay responsiveness measurement vẫn ignored theo cấu hình có sẵn. Không chạy lại toàn core/native STL accuracy matrix vì production diff chỉ ở presentation ownership. Pixel readback, process RSS và desktop latency chưa đo; GPU publication/counter tests không thay các gate đó.

Source scope: raw file bytes SHA-256 của `crates/om9-app/src/gpu_viewport.rs` là `71addf54291e6601ebf92d6c91f726442cfe217256ec7d995ccf676c84016a42`; không phải Git commit hoặc fingerprint toàn workspace. Plan/alias, README, live ledger và runtime owner M01 L2-01-02/03/06 đã đồng bộ phạm vi. Audit/JSON/links và prune chốt bên dưới. Không suy tốc độ desktop hay độ chính xác STL từ lifetime test metadata.

`tools/public_source_audit.py` exit 0: 15.187 source files, 594 icon bindings, 593 SVG, errors rỗng. JSON, fingerprint, terminal receipts và 55 local Markdown links đã kiểm tra. `git diff --check` không lỗi whitespace trong scope, chỉ cảnh báo CRLF/LF sẵn có. Prune qua `tools/prune_build_artifacts.py --apply` sau xác minh đích trong checkout, giữ 10 mới nhất mỗi root cùng các run được hồ sơ dẫn; không xóa target ngoài checkout. Không chạy lại Rust sau thay đổi chỉ ở hồ sơ.

## Phần còn mở

- Other auxiliary/peer/retired/query/container owners và actual failure/codec provenance R1 chưa đóng. `scene_high_water`/captured auxiliary, lease family và cache containers còn phải đối chiếu theo caller thật; không nhận giả thuyết ban đầu đã thành lỗi buffer được sửa.
- Automatic pressure reclamation của các cache khác, native envelope/peak R2 và worker STL UI vẫn mở.
- Không đổi display/source/v5, GPU admission/fence/batching, STL/3DM geometry/topology accuracy hoặc checkpoint. Không chạy release/model lớn/desktop/WPR mới hay batch đo openNURBS/3DM mới; packages exchange không đóng, tổng batch tương lai chưa xác định.

Tiếp theo: kiểm kê auxiliary/cache/peer owners còn thiếu trên caller production và native envelope, rồi UI STL worker. Toàn R1/P1–P5 vẫn mở.
