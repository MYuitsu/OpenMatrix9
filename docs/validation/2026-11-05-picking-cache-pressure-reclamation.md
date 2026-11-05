# Picking cache: thu hồi theo áp lực và xin lại ngân sách — 2026-11-05

Phạm vi: R1 cho metadata/arena picking của scene chính, viewport và hover snapshots đã bind tài khoản. Không đóng toàn R1, native/RSS, GPU budget hoặc plan P1–P5. Không mở/đóng OM9 hay thay bản vẽ người dùng.

## Thay đổi và vòng đời

Core giữ weak index các owner cache tùy chọn trên tài khoản RAM hiện có. Capacity của index được hạch toán trong metadata registry, old/new index cùng tồn tại phải được admit trước khi grow. Không tạo sổ ngân sách thứ ba. Callback chỉ chạy sau khi nhả khóa tài khoản; cache dùng try-lock, bỏ qua owner đang bận và reader còn pin dữ liệu.

Admission external và retained preflight thử tác vụ trước. Chỉ Budget thuộc bytes/managed RAM/physical/commit mới kích hoạt thu hồi; lỗi structural, invalid, canceled hoặc unsupported không tự xóa cache. Mỗi lần thử bỏ một cache theo thứ tự đăng ký rồi thử lại, dừng ngay khi đủ. Số lần bị giới hạn theo các owner tại đầu yêu cầu, không lặp vô hạn hoặc flush toàn bộ cache. New registrations trong lượt thu hồi được hoãn sang yêu cầu sau.

Cold quota chưa sử dụng cũng được trả. Lazy read xin lại arena qua admission peer dùng chung registries/policy trước allocation. Khi quota đã có nhưng fresh commit/physical recheck thất bại, read cũng có thể thu hồi quota của cache khác; không tự thu hồi owner đang dựng vì state bị khóa. Admission peer chỉ đăng ký thế hệ document đang giữ khi join lần đầu của coordinator, không quét lịch sử mỗi camera/frame. Metadata của state/peer, Arc arena và index được thêm vào các estimate hiện có.

`ObjectSpheres` và `PickingIndex` là handle đọc giữ Arc dữ liệu cùng lease arena. Cache không còn trả reference trần có thể bị eviction làm mất hiệu lực. Preview bình thường giữ storage khi không có reader; nếu reader cũ còn sống, cache mới admit overlap và giữ cả hai phiên bản đúng. Explicit eviction/Scene drop không nhả arena còn reader. Picking giữ handles qua query; GPU optional culling dùng handle trong lượt tính bounds. Không thay canonical geometry, selection, topology IDs hay thuật toán tessellation.

Weak index được prune khi owner scene chết, trước khi metadata lease của state trả về; tránh Weak kéo dài Arc allocation đã hết charge. Cache data và budget owner drop đúng thứ tự. GPU buffers/targets không được thu hồi bởi cơ chế CPU này; chúng vẫn cần fence/lifetime theo P2.

## Kiểm chứng và oracle

Windows x64, Cargo default features, `CARGO_TARGET_DIR=E:\om9-surface-snap-validation`, `-j 2`, dev debug=0/incremental=0, OCCT 8.0.1 SDK/TBB/jemalloc runtime từ checkout. Headless/backend/offscreen, không có desktop/release/WPR mới.

10 managed-cache tests dùng 11 document fixtures, mỗi fixture 200 points, cùng các snapshot/reader/requests bổ sung; không gọi 10 tests là 10 batch:

1. Metadata/arena charge đúng; preview không reader tái dùng allocation, eviction rồi read tự re-admit.
2. Reader sống qua eviction và Scene drop giữ đúng arena lease; last reader drop trả arena. Retained usage xác nhận weak registry slot cũng bị dọn.
3. Preview khi còn reader giữ đúng bounds cũ/mới và tính hai arena cùng lúc, trả một arena sau khi reader cũ thả.
4. Bind cache raw đã dựng đòi exclusive owner, từ chối Busy không giữ thêm charge; sau thả reader bind thành allocated.
5. Pressure lên cache warm hoặc quota cold thu hồi đúng phần tùy chọn, giữ document snapshot, dựng lại đúng sau áp lực.
6. Reader đang pin không bị thu hồi hay báo bytes đã free; thả reader mới cho phép request đang thiếu ngân sách được cấp.
7. Chỉ thiếu một byte thì bỏ đủ một cache và giữ cache thứ hai warm.
8. Hai thread tranh phần arena vừa thu hồi: giữ lease đến khi cả hai đã có kết quả; chỉ một request vừa, không tiêu trùng headroom.
9. Cold allocation recheck commit từ chối trước allocation, giữ document và thử lại được.
10. Commit chỉ vừa một arena trong hai quota đang pending: read thu hồi quota chưa dùng của scene khác thay vì từ chối giả.

Fixture pressure chừa đúng `Document::validation_scratch_usage`; admission document còn cần scratch ngoài managed live bytes. Lượt test đầu fill cả scratch nên đúng là vẫn bị Budget sau eviction; đã sửa cách tạo áp lực, không bỏ oracle reader/document/account. Đây là fixture reservation, không đo RSS hay cấp phát payload bằng số bytes reservation.

Các test app tiếp tục giữ snapshot/scene epoch, Undo/Redo, selection/file-destination và khả năng retry. Oracle RAM cũ được đổi từ “CPU charge luôn bằng trước” sang “chỉ quota tùy chọn đã thu hồi được giảm”; buffers/metadata nguồn vẫn phải khớp chính xác. Empty scene mới có owner arena chưa dựng, nên pending đó không còn bị kỳ vọng bằng 0.

Lượt cuối `build/standalone-validation/ui-artifacts/cache-pressure-final-20261105-042407` kết thúc exit 0: **257 passed, 1 ignored, 0 failed**. Log GPU không có `SKIP:`.

| Nhóm cuối | Passed / ignored | Elapsed gồm build (giây) |
|---|---:|---:|
| Core retained admission | 101 / 0 | 20,99 |
| Scene unit + instance admission + managed cache + shared instances + broad phase | 50 / 0 | 20,70 |
| Presentation | 10 / 0 | 49,36 |
| Hover | 14 / 0 | 9,74 |
| Scene memory | 7 / 0 | 1,55 |
| Picking + Gumball interaction/viewport + overlay | 47 / 1 | 39,93 |
| GPU offscreen publication | 28 / 0 | 14,07 |

Một overlay case vẫn ignored theo bộ test, không tính là pass. Lượt rộng `build/standalone-validation/ui-artifacts/cache-pressure-20261105-041541` kết thúc exit 0: replacement 8, STL unit 58, document/file/ordinary RAM 41 và STL integration 60 passed; 6 large RAM cases ignored. Những nhóm này chạy sau sửa weak-owner retirement, trước thay đổi hẹp cuối về lazy-read fresh recheck; lượt cuối phía trên kiểm lại đường recheck cùng các nhóm chịu ảnh hưởng. Không gọi tất cả lượt rộng là cùng source fingerprint cuối. Core native 5 và budget 4 đã pass trong `build/standalone-validation/ui-artifacts/commit-metadata-20261105-035620`; run đó còn có presentation oracle cũ fail nên không ghi cả run là pass. Các lỗi oracle trung gian đã sửa không được tính là pass cuối.

Fingerprint SHA-256 `ef8a3e3740afb35183b0d115c5cf71b6177773d31aa8a084be7a115fceb148be`: nối lần lượt UTF-8 đường dẫn + NUL + raw file bytes của 12 file sau; đây là định danh phạm vi source, không phải Git commit:

```text
crates/om9-core/src/transaction.rs
crates/om9-core/src/transaction/cache_pressure.rs
crates/om9-scene/src/scene.rs
crates/om9-scene/src/scene/picking_cache.rs
crates/om9-scene/src/scene/admission.rs
crates/om9-scene/src/picking.rs
crates/om9-scene/tests/managed_picking_cache.rs
crates/om9-scene/tests/shared_instances.rs
crates/om9-app/src/gpu_viewport.rs
crates/om9-app/src/ui.rs
crates/om9-app/src/session.rs
crates/om9-app/src/session/files.rs
```

Ledger, ba runtime owner M01 L2-01-02/03/06, README và checklist plan đã đồng bộ phạm vi CPU picking. `tools/public_source_audit.py` exit 0: 15.180 source files, 594 icon bindings, 593 SVG, errors rỗng. JSON, fingerprint, terminal summaries và 38 local Markdown links trong report/plan/alias đã kiểm tra. Prune qua `tools/prune_build_artifacts.py --apply` sau xác minh đích nằm trong checkout: xóa 4 run không còn được dẫn, giữ 10 mới nhất mỗi root và các run được hồ sơ dẫn; không xóa target ngoài checkout. Không chạy lại Rust sau thay đổi chỉ ở hồ sơ.

## Giới hạn và việc tiếp theo

- Reclaim mới áp dụng arena/quota picking đã đăng ký. Display mesh nằm trong document, GPU/upload, nhiều cache CPU khác và native vẫn theo P1/P2/R2; không coi chúng đã free chỉ từ flag nonresident.
- Thứ tự là registration order, chưa có chính sách recency/QoS để chứng minh không churn khi nhiều viewport tranh cache sát trần. Cold optional culling và critical picking chưa có mode admission khác nhau; cần đo workload sát trần và tránh render-only work đẩy cache tương tác liên tục.
- Registry bookkeeping dùng capacity/Arc layout estimate, không allocator/RSS certificate. External document/capture weak ownership, auxiliary/high-water, peer/journal/path/query scratch và source/version receipts ngoài phạm vi này còn audit; native peak trước callbacks chưa bị bound.
- Tài liệu/hình học gốc và Undo không bị xóa để cứu tác vụ thất bại. Một tác vụ thất bại có thể đã thu hồi cache tùy chọn; ảnh hoàn tất/GPU geometry owners giữ hợp đồng hiện có. Headless không thay bằng chứng desktop/Gumball chuột thật.
- Không chạy lại 6 large RAM cases hoặc release benchmark ở lượt này. Tổng ma trận tương lai chưa liệt kê hết, số batch còn lại chưa xác định.

Bước tiếp: đo/giảm churn sát trần và hoàn thiện owner/diagnostics R1, native envelope/peak R2, cùng worker UI P3b. P1–P5 vẫn mở; không đánh dấu toàn mục tiêu hoàn thành.
