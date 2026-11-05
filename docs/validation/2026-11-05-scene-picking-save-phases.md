# Scene/picking owners và cấp RAM theo giai đoạn cho Save — 2026-11-05

Phạm vi bàn giao: metadata scene chính, cache picking có owner, kiểm chứng cuối code đang sửa, và đường Save không còn chiếm toàn ngân sách trống. Đây là một phần R1/R2a; chưa hoàn tất plan viewport/export. Giữ nguyên phiên OM9 đang mở; không launch, đóng hoặc sửa bản vẽ người dùng.

## Thay đổi và điều được chứng minh

Scene giữ quota metadata riêng và allowance arena spheres/BVH. Metadata được ghi allocated theo capacity thực; arena chưa tạo giữ pending, kiểm lại physical/commit trước cold allocation. Startup, full/incremental candidate, history, New/Open và snapshot chuyển allowance sang scene owner, không nhả rồi xin lại. Buffer geometry dùng lease đã có, không cộng lại trong held metadata.

Arena chứa spheres, BVH nodes, order và workspace sắp xếp ổn định. Spheres có thể dùng trước khi dựng BVH; storage index đã dự trù, xây index theo nhu cầu. Preview làm mất hiệu lực nội dung nhưng giữ allocation/charge để tái dùng. Test so phép sắp xếp với stable sort của thư viện, kể cả tie, và kiểm capacity bound. Eviction thực sự thả data rồi quota; muốn dựng lại phải re-admit. Cold allocation bị từ chối do commit vẫn giữ document và có thể thử lại khi đủ bộ nhớ. Nếu bounds không khả dụng, optional render culling vẽ toàn bộ thay vì bỏ object.

**Giới hạn còn lại:** quota arena đang được giữ trước có thể làm nhiều snapshot giữ nhiều pending không cần thiết. API eviction mới chỉ có caller test, chưa nối chính sách áp lực trong app. Query scratch, auxiliary/high-water, upload/retired/native owners và diagnostics vẫn cần audit; Scene raw không gắn tài khoản chỉ có local Limits. Không khẳng định toàn bộ R1 hoàn tất.

Save trước đây gọi `reserve_remaining_external_memory_with_held`, khiến peer bị từ chối dù Save nhỏ. Nay `Archive::save_with_admission` báo capacity buffer còn sống và scratch dự kiến theo giai đoạn; Controller dùng cùng registry để reserve/grow/materialize/shrink. Entry `Archive::save` cũ vẫn dùng cùng implementation và local Limits, không tự có tài khoản phần cứng.

Các ranh giới có admission: validation scratch, JSON realloc với old/new overlap, JSON + container output, decoder kiểm chứng, staging reread và decoder của reread. JSON và decoder/reread được thả trước khi thu nhỏ charge. Decoder callback đã có được tái dùng, trừ input đang được buffer owner tính; không cộng source document vào lease lần nữa. Guard ngoài Archive tồn tại đến khi các buffer đã drop, kể cả lỗi. Không có callback có thể từ chối sau khi file đã được publish.

Callback preflight từng JSON token dùng cửa sổ tăng trưởng tối thiểu 64 KiB để tránh kiểm kê coordinator và probe OS từng scalar; cửa sổ được trả khi phase kết thúc. Nếu xin dư cửa sổ bị Budget, thử đúng số cần. Đây là lượng giữ trước tối đa thêm cho mỗi lần tăng nhỏ, không là hệ số dự báo model/native. Known capacities chuyển sang allocated; decoder scratch vẫn là estimate pending trong phase, chưa có exact allocation hooks cho mọi container/parser.

**Giới hạn Save:** kiểm kê codec là ước tính, chưa chứng nhận allocator hoặc process peak; scratch decoder đang tồn tại có thể còn bị coi pending trong phase. Path clones và allocation bên trong VerifiedFile/OS publication chưa được kiểm kê đầy đủ. Đường Save vẫn đồng bộ, chưa có số đo release mới cho model lớn. Hai đường captured/detached STL vẫn giữ toàn headroom; worker STL/UI, OCCT peak, v5, GPU batching, STL/3DM accuracy và checkpoint vẫn mở.

## Kiểm chứng

Môi trường: Windows x64; workspace root; Cargo default features; `CARGO_TARGET_DIR=E:\om9-surface-snap-validation`, `-j 2`, dev debug=0, incremental=0. PATH có OCCT 8.0.1 SDK, TBB và jemalloc runtime của checkout. Không đổi backend hoặc profile dung sai.

Batch `build/standalone-validation/ui-artifacts/commit-metadata-20261105-025516`, chạy bằng `build/standalone-validation/run-r1-commit-metadata-20261105.ps1` trước sửa Save:

- Native memory 5, core budget 4, retained/account 101, presentation 10, hover 14, scene memory 7, replacement 7, STL lifecycle 44: tất cả pass.
- Đóng thất bại oracle ownership trong focused log cũ `commit-metadata-20261105-024241/replacement.log`; không dùng suite khác để coi là đã sửa.

Batch integration có handle đã xác nhận kết thúc exit 0: `build/standalone-validation/ui-artifacts/commit-integration-20261105-024737`. Trước sửa Save, overlay 14 pass/1 ignored; B-rep picking 19, Gumball interaction 7, Gumball viewport 7, GPU publication 28, document 12, file 20, ordinary RAM 9, real resource 270 MiB 1 và STL export 60 pass. Lượt ordinary RAM ignored 6 large cases; fixture resource được chạy riêng, 5 large khác vẫn chưa chạy. Test responsiveness opt-in chưa chạy. Không dùng broad integration trước Save để xác nhận Save mới.

Batch cuối sau sửa Save và cửa sổ admission: `build/standalone-validation/ui-artifacts/save-phases-20261105-030142`, script `build/standalone-validation/run-save-phases-20261105.ps1`:

| Nhóm | Kết quả | Build + execution elapsed của nhóm |
|---|---|---|
| Scene unit/instance/managed/shared/surface | 23 + 7 + 2 + 5 + 5 pass | 2,45 s |
| Archive toàn crate | 4 unit + 68 integration pass; 0 doc tests | 1,99 s |
| App replacement/Save | 8 pass | 30,60 s |
| Document/file/ordinary RAM | 12 + 20 + 9 pass; 6 large ignored | 29,52 s |

Đây là thời gian test/build, không phải benchmark thao tác CAD. Batch trung gian `save-phases-20261105-025917` cũng pass nhưng trước cửa sổ admission và các assertion cuối; không thay batch cuối.

Oracle mới có ý nghĩa:

1. Save tài liệu có resource 1 MiB dưới limit 32 MiB vẫn thành công khi peer giữ 16 MiB xuyên suốt Save. Lối cũ giữ toàn headroom sẽ từ chối peer ngay ở guard. Lease Save được trả trên cả thành công và hủy, peer lease không bị thu hồi.
2. Peer chiếm phần còn lại sau admission ban đầu làm phase growth bị từ chối; file đích giữ nguyên bytes, document snapshot và Redo được giữ, Redo vẫn thực hiện được sau đó.
3. Archive callback quan sát old/new overlap và charge giảm; từ chối sau khi staging tồn tại giữ đích, dọn staging, trả đúng lỗi Budget.
4. Source và dung sai không đổi: archive mới bằng bytes encoder hiện có, mở lại đúng document. Giữ nguyên kiểm tra finite/schema/structural/atomic publication hiện có.

Không chạy desktop/release/WPR hoặc các large CAD Save fixtures lần này. Không suy FPS, input-to-display, toàn miền sai số STL hoặc native peak từ headless. Tổng ma trận tương lai chưa liệt kê hết, số batch còn lại chưa xác định.

## Tiến độ và bước tiếp

Đã đồng bộ plan/alias, ledger, README và runtime entries M01 L2-01-02/03/06 theo phạm vi này. R1 vẫn mở cho pressure eviction/re-admission và owner audit; R2a tiếp tục captured/detached STL, giữ soup/topology/native/staging overlap và không đổi chất lượng xuất. R2b đo native/release riêng; toàn P1–P5 chưa nghiệm thu.

Kiểm tra cuối: source fingerprint còn khớp 22 file; đối chiếu terminal summaries khớp số test bảng trên; GPU integration log không có `SKIP: no GPU adapter`. Liên kết local, ledger JSON/ID duy nhất và ba milestone JSON hợp lệ. Public-source audit: 15.177 source files, 594 icon bindings, 593 SVG, không có lỗi. Chạy `tools/prune_build_artifacts.py --apply` sau khi kiểm resolved targets nằm trong thư mục validation: bỏ 2 run không được dẫn, giữ 10 mới nhất cùng các run được tài liệu bảo vệ; không xóa target ngoài checkout. Tool kiểm fingerprint ban đầu dùng sai đường dẫn `src/scene/picking.rs`, đã sửa thành `src/picking.rs` trước khi ghi ledger/hash; kiểm cuối pass. Không commit/publish hoặc sửa skill nghiệp vụ.


Source fingerprint SHA-256: `96a3e41eb44555d0e33f7383c0333596ad1a3383063eca1df070dd75a3b76ab0`. Thuật toán: lần lượt nối UTF-8 đường dẫn repo, byte NUL, rồi bytes file gốc theo đúng thứ tự dưới đây; không phải Git commit hoặc hash toàn checkout đang có thay đổi khác.

- `crates/om9-core/src/transaction.rs`
- `crates/om9-scene/src/scene.rs`
- `crates/om9-scene/src/scene/picking_cache.rs`
- `crates/om9-scene/src/spatial.rs`
- `crates/om9-scene/src/picking.rs`
- `crates/om9-scene/src/osnap.rs`
- `crates/om9-scene/tests/managed_picking_cache.rs`
- `crates/om9-scene/tests/instance_admission.rs`
- `crates/om9-scene/tests/shared_instances.rs`
- `crates/om9-scene/tests/surface_broad_phase.rs`
- `crates/om9-app/src/session.rs`
- `crates/om9-app/src/session/files.rs`
- `crates/om9-app/src/session/stl.rs`
- `crates/om9-app/src/gpu_viewport.rs`
- `crates/om9-app/src/ui.rs`
- `crates/om9-app/tests/brep_picking_precision.rs`
- `crates/om9-app/tests/preview_picking_bench.rs`
- `crates/om9-app/tests/selection_overlay.rs`
- `crates/om9-archive/src/lib.rs`
- `crates/om9-archive/src/container.rs`
- `crates/om9-archive/src/atomic_file.rs`
- `crates/om9-archive/tests/atomic_save.rs`
