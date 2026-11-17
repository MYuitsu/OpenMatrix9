# OM9 — Kế hoạch quản lý bộ nhớ

Cập nhật: 2026-11-16. Sau lượt chỉ rà soát kế hoạch, người dùng đã cho phép triển khai **M1-R**, giữ nguồn chính xác và đầy đủ Undo; thử cảnh nhỏ trước rồi 512/768 Light/High, không Boolean. [Bằng chứng triển khai và các gate còn mở](../../../../docs/validation/2026-11-16-m1r-storage-receipts.md). Kế hoạch giữ chi tiết RAM/VRAM còn mở từ [PLAN.md](PLAN.md); yêu cầu này không mở toàn bộ M1–M5.

Lượt cuối đã qua đủ 10 cấu hình nhỏ/512/768 Light/High, nguồn chính xác và Undo còn đủ; fixture ghi rõ allowance 4M điểm, không đổi default 2M. Kiểm kê attribution tự động còn 5 lần ở 512 và 11 lần ở 768. M1-R chưa đóng: UI tạo cầu vẫn có đoạn 729/821 ms ở 768, High còn chậm publish khi bắt đầu chọn, chưa có registry O(delta) đầy đủ hoặc test burst/pressure toàn bộ. [Đọc decompile Rhino 5](../../../../docs/validation/2026-11-16-rhino5-mesh-capacity.md) chưa xác lập trần cảnh thực tế và không làm căn cứ đổi quota mặc định.

Phạm vi lịch sử ngày 10/11: [hàng đợi Core/Curve/Solid](PLAN_CORE_CURVE_SOLID.md) đặt A1 DupEdge trước A2 ExtractWireframe; B0 đo để quyết định B1–B3 thu hồi display chỉ còn trong lịch sử. Những quyết định đó không phải bằng chứng trạng thái source/display hiện tại. Đối chiếu [kế hoạch hoàn thiện source/display](PLAN_SOURCE_DISPLAY_COMPLETION_M01_M09.md) và ledger khi triển khai; giữ các đường Save/Open/Undo đang hỗ trợ. Ngưỡng lợi ích B0 vẫn thuộc plan Core/Curve/Solid; ngân sách độ trễ M1-R bên dưới chỉ dành cho quản lý cache/Undo.

**Cập nhật 2026-11-11:** phép đo sản phẩm ở profile 0,01 mm cho cận trên display của cả hai workload dưới ngưỡng 32 MiB; quyết định hoãn B1–B3. Profile sản phẩm thứ hai và các chi phí đầy đủ vẫn chưa đo; chưa có phục hồi source-only hoặc eviction. [Bằng chứng](../../../../docs/validation/2026-11-10-sphere-history-b0.md).

Ưu tiên A1-R DupEdge trong hàng đợi tính năng giữ nguyên phạm vi riêng. Yêu cầu hiệu năng hiện tại tập trung M1-R, không cần chờ eviction hoặc checkpoint, và không tự mở lại các gói tính năng đã có bằng chứng.

## M1-R — Giảm chi phí cache/Undo trên đường tương tác

**Trạng thái: đang triển khai và đo; chưa đóng M1-R hoặc đạt toàn bộ mục tiêu dưới đây.** Đã thêm storage receipts bất biến, cache aggregate có admission/thu hồi dùng chung và tối ưu lookup owner; chưa có registry O(delta) hoàn chỉnh. Không đặt lời hứa “nhanh tuyệt đối”: số lượng đối tượng, lịch sử, cache lạnh và áp lực RAM đều có chi phí. Mục tiêu là không quét lại dữ liệu bất biến trên mỗi lần kiểm tra, giới hạn công việc đồng bộ của UI và đo phần tăng theo workload.

### Bằng chứng và chỗ thiếu của kế hoạch cũ

[Console 16/11](../../../../docs/validation/2026-11-16-console-sphere-metrics.md): tại cầu 512, các lần kiểm tra retained owners được ghi log cộng 7.756,858 ms trong lệnh 7.867,601 ms, khoảng 98,6%; external 4.289,292 ms và internal 3.467,562 ms. Có 47 lần vượt ngưỡng ghi log 50 ms; đây là cận dưới số lần gọi, không phải toàn bộ số lần. Hai kiểm kê Undo cộng 220,396 ms nằm trong các pha đó, không cộng thêm vào tổng. UI có khoảng gián đoạn lớn nhất 8.094,25 ms trong lúc tạo, dù xoay cảnh sau đó đạt khoảng 29,73 FPS. Lượt 512 dừng trước kiểm chứng nguồn/Undo cuối nên chỉ là chẩn đoán; 512 High và 768 chưa chạy.

“Không quét toàn history mỗi frame” chưa đủ: hiện vẫn kiểm kê nhiều lần trong một lệnh. Chu kỳ kiểm kê Undo 100 MiB không thay thế admission của dữ liệu đang sống. [Cache proof 02/11](../../../../docs/validation/2026-11-02-undo-validated-owner-cache.md) đã tránh một phần kiểm chứng cấu trúc nhưng vẫn tính storage mỗi lần. [Index 06/11](../../../../docs/validation/2026-11-06-undo-inventory-scaling.md#follow-up-admitted-per-document-storage-index) đã giảm chi phí một số đường; không viết lại phần đó hoặc suy từ fixture tám bước Undo rằng lịch sử 512 bước đã ổn.

| Đường code đã đối chiếu | Chi phí còn thấy trong code | Hướng xử lý có giới hạn |
|---|---|---|
| `transaction.rs::validate_retained_owners_with_external_except` | Tính external rồi internal lại ở mỗi admission | Tái dùng summary hợp lệ; vẫn kiểm hạn mức và reservation hiện hành |
| `external_storage_usage` | Quét owner trước đó, tài liệu/lịch sử và lease để khử trùng source/topology | Tra chỉ mục allocation dùng chung tại coordinator hiện có, cập nhật theo owner thực thay đổi |
| `validate_internal_retained_owners` | Dựng scratch index, duyệt tài liệu và tính storage nhiều lượt; fallback có thể bậc hai | Cache manifest bất biến và aggregate; đo rõ fallback, không chỉ tối ưu nhánh thuận lợi |
| `undo_memory.rs::note_undo_memory_growth` và refresh/trim | Ước lượng baseline, kiểm kê và invalidation có thể làm việc lặp | Gắn lý do invalidation; dùng delta thực hoặc cận trên có chứng minh, tái dùng inventory cùng trạng thái |
| `commit_with_guard` | Kiểm tra tổng trước publication trên đường đồng bộ | Chuẩn bị bằng chứng trước; commit chỉ đối chiếu thế hệ, phần tăng và chốt nguyên tử |

### Thiết kế ưu tiên: tái dùng bằng chứng, cập nhật phần thay đổi

1. **Summary theo phiên bản bất biến.** Tái dùng identity/Arc, proof và `BrepStorageIndex` hiện có. Tính storage/manifest cho phiên bản mới một lần; bản đã công bố giữ summary có kiểm soát lifetime. Manifest phân biệt source BRep, topology, native payload, display, resource và metadata. Không lấy hash toàn bộ hình học mỗi lần tra cache; không dùng riêng địa chỉ pointer có thể tái sử dụng. Chỉ dùng summary cho owner bất biến, hoặc phải có generation thay đổi đồng bộ với mutation. Cache proof cấu trúc và cache byte có điều kiện hiệu lực riêng.
2. **Chỉ mục owner dùng chung trong coordinator.** Ghi allocation identity, số byte và owner/pin theo lớp; allocation được tính một lần dù current/Undo/Redo/worker/scene cùng giữ. Tái dùng registry/reservation hiện hữu, không thêm một hệ thống quota hoặc lịch sử Undo thứ hai. Document metadata, vector capacity, Arc/Weak control block và chính index/cache phải được hạch toán. Quy tắc alias/resource hiện tại phải được bảo toàn bằng đối chiếu với bộ kiểm kê tham chiếu.
3. **Cập nhật tăng/giảm tại ranh giới sở hữu.** Reserve, chuyển pending sang allocated, capture/drop, commit, Undo/Redo, New/Open, xóa nhánh Redo và trim đều cập nhật delta trong cùng giao dịch hoặc giao thức đồng bộ đã chứng minh. Release chỉ giảm charge khi owner thực sự kết thúc; Weak chết còn giữ control block phải xử lý riêng. Không dùng `strong_count()` như sổ kế toán hay trừ RAM chỉ vì đã gửi Cancel. GPU còn chờ fence vẫn là live. Bộ nhớ trì hoãn giải phóng vẫn phải được tính.
4. **Tách summary bền khỏi reservation thay đổi nhanh.** Thay đổi pending hoặc lease không được tự động bắt tính lại manifest của mọi document. Kết quả aggregate phải gắn thế hệ world/history, external owner, reservation và scope candidate/excluded thực tế; lập bảng từng nơi làm chúng thay đổi trước khi cache. Nếu một token hiện có không bao quát đủ, bổ sung tại đúng owner. Thay đổi giới hạn phải kiểm lại với `Limits` mới; đổi probe RAM/commit không được dùng lại quyết định admission cũ.
5. **Chuẩn bị nặng ngoài UI, commit ngắn.** Tái dùng worker hiện có nếu cần dựng summary/cache lạnh hoặc dọn owner lớn. Chỉ worker dùng snapshot đã reserve/pin; trước publication đối chiếu generation và quota hiện hành trong cùng cơ chế chống race. Nếu đã cũ, chuẩn bị lại có giới hạn và vẫn xử lý sự kiện UI; không bận lặp hoặc giữ mutex qua phép quét lớn. Việc chuyển worker là bước sau khi bỏ quét lặp, không thay thế sửa thuật toán.
6. **Cache có giới hạn và thu hồi theo sự kiện.** Thu hồi mục dẫn xuất không còn pin, theo áp lực bộ nhớ hoặc thay đổi owner; không timer quét toàn scene/history. Giới hạn hàng đợi job, gom các yêu cầu cùng phiên bản, bỏ kết quả cũ có kiểm tra source. Di chuyển/selection chỉ vô hiệu hóa phần phụ thuộc cần thiết; camera không làm invalidation source hay Undo. Việc free/trim hàng loạt cũng phải đo và chia việc an toàn nếu gây đứng UI, vẫn giữ charge tới khi thực sự giải phóng.

Đích độ phức tạp: đọc tổng đã hợp lệ O(1); cập nhật registry phụ thuộc số allocation/owner **thực sự thay đổi**, tra cứu theo loại index đã chọn. Chưa hứa toàn lệnh Sphere O(1): tạo snapshot/metadata hiện tại vẫn có thể O(N), và bỏ một nhánh Undo lớn có nhiều owner phải xử lý. Nếu một bước vẫn phải liệt kê mọi tham chiếu của snapshot, ghi rõ O(N), không gọi đó là O(delta). Chưa thay định dạng journal hoặc chuyển toàn model sang persistent collection nếu số đo chưa chứng minh cần.

### Thứ tự triển khai và kiểm chứng

- [ ] **R0 — Chốt baseline và nguyên nhân gọi lặp.** Tái dùng log 16/11; bổ sung bộ đếm mọi lần gọi (không chỉ >50 ms), hit/miss theo lý do, owner/byte đã duyệt, rebuild/fallback, lock wait/hold và drop/free. Tách thời gian tự thân với pha lồng nhau. Đối chiếu 1.024 full inventories ở 512 Undo và các đường capture/trim gây invalidation; không quy mọi chi phí cho riêng kiểm kê Undo.
- [ ] **R1 — Summary và tái sử dụng cùng trạng thái.** Bắt đầu đúng đường Sphere/admission trong `transaction.rs`, storage trong `model.rs` và `transaction/undo_memory.rs`; kiểm mọi caller bị ảnh hưởng bởi helper chung. Không vô hiệu hóa validator để đạt FPS. Cache cold/miss và lỗi cấp phát phải có đường an toàn. Nhánh fallback chính xác có thể chậm: ghi metric, không tuyên bố hoàn tất nếu nó còn chặn UI nhiều giây trong workload mục tiêu.
- [ ] **R2 — Delta owner và publication.** Nối điểm acquire/release thực đã kiểm, so tổng với phép kiểm kê đầy đủ ở test trước khi dùng tổng làm admission. Worker, candidate bị loại, stale result, Redo branch và mức giới hạn đổi không được làm thiếu charge. Chỉ tối ưu UI/worker/drop tiếp ở pha còn vượt ngân sách sau R1.
- [ ] **R3 — Kiểm đúng trước, đo mượt sau.** Test đối chiếu summary/registry với phép kiểm kê độc lập sau chuỗi create/edit/delete/Undo/Redo/branch/clear; cùng allocation và allocation khác nhưng byte giống nhau; nguồn thay đổi cùng kích thước; native/resource/display/topology aliases; owner cuối được worker giữ; release đồng thời; giới hạn chặt/overflow/generation hết miền; lỗi/hủy và New/Open khi job còn sống. Giữ cùng kết quả nhận/từ chối theo hợp đồng ngân sách, hình học/ID và hai stack sau lỗi. Cache/index phải nằm trong admission; không được tạo đỉnh old+new không reserve. Cold miss không được silently cho phép commit chưa kiểm chứng.
- [ ] **R4 — Console theo từng cỡ.** Cảnh nhỏ 16/64 Light và High trước; 128 rồi vùng 448–512; chỉ sau khi UI còn đáp ứng mới chạy đủ 512/768 Light/High. Không Boolean, tạo từng Sphere qua đường thường, giữ đầy đủ Undo như yêu cầu hiện tại. Kiểm source/Undo ở checkpoint riêng với số đo tương tác; bộ kiểm chứng cuối của harness không được nhập vào thời gian lệnh sản phẩm. Có cả lượt paced để chứng minh mỗi cầu được publish và lượt tốc độ thao tác thông thường để kiểm lag/hàng đợi, không suy một từ một.

Gate R3 gồm nguồn chính xác bất biến giữa Light/High, Undo/Redo đúng và quota/rollback đúng; tốc độ không được bù cho lỗi accounting. Differential test phải có oracle không gọi lại chính fast path. Census đối chiếu trong test hoặc chẩn đoán riêng; không đưa trở lại mỗi lệnh release. Khi trạng thái cache không chứng minh được an toàn, giữ ảnh/document trước và chuẩn bị lại hoặc báo lỗi hiện có, tuyệt đối không chấp nhận ước lượng thiếu làm trần managed.

### Ngân sách độ trễ và cách báo cáo

Các số dưới đây là **mục tiêu nghiệm thu đề xuất cho máy console và fixture đã nêu**, chưa có kết quả đạt. Không phải cam kết cho mọi model/native call. Không thay giới hạn FPS sản phẩm trong lượt tối ưu cache/Undo.

| Chỉ số cần đo | Mục tiêu |
|---|---|
| Tổng thời gian tự thân của quản lý cache/Undo trên UI trong một lệnh, gồm lock và cleanup | p95 ≤ 4 ms, p99 ≤ 8 ms; đoạn chặn liên tục tối đa ≤ 33,3 ms trong workload mục tiêu |
| Tái kiểm cùng tập owner/phiên bản/giới hạn | Không duyệt lại payload hoặc dựng lại index toàn history; đếm được cache hit và phần kiểm quota hiện hành |
| UI trong cả lúc tạo, chọn/xóa, Undo/Redo và resize | Khoảng callback p99 ≤ 50 ms, không gap >100 ms do quản lý cache/Undo; báo mọi gap toàn app và nguyên nhân chưa rõ |
| Khung hình hoàn tất khi đang tương tác, cap hiện tại 30 Hz | Khoảng 28–30 FPS; báo frame-time p50/p95/p99/max và số gap >100/500/1.000 ms, không chỉ FPS trung bình sau khi tạo xong |
| Thời gian thấy cầu mới | Tách command-start→commit, commit→ảnh đủ mọi viewport, và start→ảnh; nếu publish vẫn chậm thì mở việc tại render, không gọi cache/Undo đã giải quyết toàn bộ |
| RAM và lịch sử | Báo unique live, pending, index/cache bytes, Undo estimate/census, WS/private/commit, VRAM và số bước Undo/Redo; đủ 512 hoặc 768 bước tạo, không âm thầm trim để lấy số đẹp |

Đo độc lập hai trục: giữ số đối tượng cố định và tăng lịch sử; giữ lịch sử ngắn cố định và tăng số đối tượng, sau đó đo workload đầy đủ cả hai. Với phép kiểm cùng trạng thái, lượng payload duyệt phải bằng 0 sau warm-up, thay vì chỉ dựa vào hệ số tăng thời gian. Báo cache lạnh riêng. Microbenchmark ít nhất ba lần lặp; desktop chỉ lặp sau khi lượt đầu ổn và ghi rõ số mẫu, không coi một lần là thống kê ổn định.

So A/B phải dùng baseline và bản sửa có source/build fingerprint, cùng fixture, profile, Undo, cửa sổ, DPI, GPU, scheduler và cấu hình telemetry. Dùng log cũ cho chẩn đoán, không ghép khác điều kiện thành speedup chính xác; không bắt buộc tái chạy cảnh baseline gây treo lớn chỉ để lấy đối chứng. Chốt performance bằng bộ đếm nhẹ, tắt log chi tiết; báo overhead instrumentation từ một phép đo riêng. Windows foreground/occlusion, client size, render-target extent, DPI, `IsZoomed` và trạng thái hung phải được ghi để kiểm lỗi maximize không scale. Nếu lại không phản hồi, dừng test do mình sở hữu, giữ log và không chạy cỡ lớn hơn; không tác động phiên người dùng.

**Điều kiện đóng M1-R:** R3 đúng, R4 qua các cỡ được chọn, ngân sách latency đạt hoặc phần chưa đạt được giữ mở với nguyên nhân; có đối chiếu RAM và đầy đủ Undo. Không dùng việc build thành công, ảnh đẹp, FPS orbit hoặc hoàn tất plan để thay bằng chứng này. M2–M5 vẫn là backlog độc lập, không kéo vào gate M1-R.

## Đã làm đến đâu

**Lựa chọn của người dùng 2026-11-12:** sau cảnh báo và đồng ý, tự bỏ các bước Undo cũ nhất đủ để thử lại; không xóa nếu từ chối. Luồng ExtractWireframe qua CMD đã kiểm có giới hạn tại [bằng chứng](../../../../docs/validation/2026-11-12-memory-recovery-choice.md).

| Hạng mục | Trạng thái có bằng chứng | Giới hạn |
|---|---|---|
| Undo | Mặc định 1024 MiB, cấu hình tối đa 50% RAM vật lý; kiểm kê sau 100 MiB tăng trưởng được hạch toán hoặc sớm hơn khi cần. [Báo cáo 02/11](../../../../docs/validation/2026-11-02-undo-ram-cap-and-inventory.md). | Quota giữ lịch sử là mềm vì vẫn giữ số bước tối thiểu; không phải toàn bộ RSS. Không cấp trước 1024 MiB. |
| RAM phiên tương tác | Đã có coordinator, reservation pending/allocated, kiểm tra RAM vật lý và commit; đường Save/New/Open tương tác đã kiểm với dữ liệu lớn hơn 256 MiB. [Chuyển tiếp](../../../../docs/validation/2026-11-04-interactive-ram-transition.md), [pending/allocated](../../../../docs/validation/2026-11-05-pending-allocated-ram.md). | Chỉ các owner/caller đã nối và kiểm chứng; `Limits::default()` vẫn còn cho các ngữ cảnh khác. Không khẳng định toàn tiến trình bị chặn cứng. |
| Cache/scene và xuất nền | Một số owner scene/picking đã được hạch toán; Save và STL cấp theo pha, xuất nền đã có kiểm thử giao diện. [Cache](../../../../docs/validation/2026-11-05-picking-cache-pressure-reclamation.md), [STL](../../../../docs/validation/2026-11-05-stl-staged-memory.md), [desktop](../../../../docs/validation/2026-11-05-stl-background-ui.md). | Chưa đủ bằng chứng cho mọi owner, workload lớn, native OCCT hoặc mọi đường capture. |
| VRAM | Có bằng chứng truy vấn adapter DX12/Vulkan được chọn: [DX12](../../../../docs/validation/2026-11-03-dx12-selected-adapter-memory.md), [Vulkan](../../../../docs/validation/2026-11-03-vulkan-selected-device-memory.md). | Chưa chứng minh toàn bộ ngân sách cache, giải phóng sau GPU fence và vẽ đủ theo lô trên luồng thật. |
| Checkpoint 512 MiB | Quy tắc đã thống nhất trong kế hoạch. | Chưa triển khai; không phải cơ chế làm hình học gốc chính xác. |

Ngày 02/11 là mốc có báo cáo rõ ràng cho đợt Undo/RAM này, không nhất thiết là ngày đầu tiên dự án có quản lý Undo. Đến 06/11 là khoảng 4 ngày lịch; hồ sơ không chứng minh số giờ làm việc thực tế hay thời gian chạy liên tục. Không suy phần trăm hoàn thành từ số test, file hay ngày.

## Hợp đồng giữ nguyên

- Hạn mức dữ liệu được quản lý: `unique_live + pending <= floor(60% × RAM vật lý tổng)`. Undo là một phần trong đó, không cộng thêm toàn bộ quota. Đây không phải giới hạn cứng cho toàn bộ RSS/native heap.
- Trước cấp phát mới, kiểm tra riêng RAM khả dụng sau dự phòng `max(256 MiB, 10% RAM tổng)` và commit Windows sau dự phòng 256 MiB. Đây là số đo tức thời, không bảo đảm tránh mọi OOM. Không trừ hai lần bộ nhớ đã sống; tính đỉnh old/new, scratch và các reservation đồng thời.
- 256 MiB có thể là giới hạn local, mức dự phòng, hoặc GPU fallback; không nhập nhằng ba ý nghĩa. Giữ giới hạn cấu trúc, overflow và parser phù hợp từng đường.
- Cấu hình Undo 1024 MiB mặc định, tối đa 50% RAM tổng; quota mềm có thể vượt khi giữ bước tối thiểu. Kiểm kê 100 MiB là kiểm kê Undo, không phải tính lại hình học. Thiếu RAM không được âm thầm hạ dung sai, xóa Undo/Redo để cứu giao dịch thất bại hoặc làm đổi file đích.
- GPU Auto tối đa 50% VRAM đúng adapter và trong ngân sách OS sau dự phòng 10%; cho cấu hình thấp hơn. UMA hạch toán RAM chung một lần; không xác minh được dùng fallback 256 MiB và báo rõ. Giữ độ phân giải, AA và hình học; nếu không đủ thì vẽ đủ theo lô, chỉ công bố ảnh hoàn chỉnh, hoặc báo thiếu bộ nhớ nếu một lô tối thiểu không vừa.
- Checkpoint 512 MiB chỉ kiểm chứng nguồn thay đổi trong phiên; không tự lưu, không thay đổi Undo và không quyết định độ chính xác. Hình học phải hợp lệ theo dung sai khi commit; export luôn kiểm nguồn của riêng nó.

## Thứ tự làm và điều kiện hoàn thành

### M1 — Nền RAM và Undo cùng source/display; mở rộng theo từng lệnh

Đối với B0–B3 trong [hàng đợi chính](PLAN_CORE_CURVE_SOLID.md), đo unique history-only display trước; chỉ tiếp tục khi đạt ngưỡng lựa chọn. Giữ source/proof và record phục hồi, chỉ bỏ display không còn current/scene/worker/snapshot pin. Dùng reclamation hiện có, không timer/quét mỗi frame. Worker chuẩn bị document/scene đầy đủ rồi Undo/Redo mới công bố nguyên tử; lỗi giữ world và hai stack. Save/Save As v4 chụp document đang công bố, kiểm lưu/mở lại sau restore lạnh; v5 và eviction current document ở backlog. Không tạo lịch sử thứ hai cho mesh cache. A chỉ sửa/kiểm admission và lifetime của caller bị ảnh hưởng, không phải hoàn tất mọi owner hoặc eviction trước.

- [ ] Rà đúng các caller Open/Save/edit/STL còn thiếu admission, owner và lifetime; tái dùng coordinator, thống kê phần shared một lần. Không viết lại phần đã có bằng chứng, không quét toàn history mỗi frame.
- [ ] Kiểm tra quota Undo mềm so với trần managed, thông báo rõ actual/quota và nguyên nhân từ chối; xử lý probe lỗi, trần hiệu lực và phục hồi theo bằng chứng. Không tự đổi hành vi minimum steps.
- [ ] Tính reservation cho phần tăng thêm và tác vụ đồng thời; giữ lease tới khi owner/worker kết thúc. Khi thất bại hoặc hủy, giữ document, Undo/Redo và file đích. Thu hồi cache tùy chọn trước khi từ chối nếu an toàn.
- **Gate gộp:** shared ownership, pending đồng thời, vượt quota mềm, RAM/commit thấp, New/Open/Undo và export khi worker còn sống, thất bại/hủy. Thêm một fixture model thực lớn hơn 256 MiB khi điều kiện cho phép; phân biệt test mô phỏng và đo RAM thật. Báo phần không hạch toán và chênh lệch với commit/working set.

### M2 — Đo và giới hạn rủi ro native OCCT

- [ ] Đo peak theo pha cho xuất STL/tessellation trước, cùng fixture nhẹ/nặng/lặp; so managed counters với commit và working set. Ghi cấu hình, thời gian, sai số và kết quả; không suy hệ số cố định từ một hình cầu.
- [ ] Dựa trên phép đo, giới hạn số job nặng và reserve phần có thể kiểm soát trước native call; giữ lease đến khi call thực sự kết thúc. Phân biệt ước lượng có dự phòng với trần thực thi được. Nếu OCCT không cấp theo quota/chunk thì ghi rõ phần native chưa chặn; chỉ cân nhắc process isolation khi cần trần cứng và đã đo overhead.
- **Gate gộp:** xuất thành công/hủy/lỗi giữ file đích và nguồn, không trả lease sớm; báo native peak và các allocation chưa kiểm soát. Không tuyên bố hết OOM chỉ vì admission đạt.

### M3 — Ngân sách cache GPU trên đường vẽ thật

- [ ] Dùng đúng adapter hiện hành; tính buffer/staging/target màu, depth, MSAA và tài nguyên còn chờ fence. Giữ pipeline/buffer dùng chung, camera-only không upload lại geometry.
- [ ] Áp Auto/giới hạn người dùng theo hợp đồng; UMA và fallback thể hiện rõ. Khi vượt ngân sách, chia lô đúng depth/pass, overlay sau cùng; publication và picking khớp ảnh hoàn tất, camera mới nhất không tạo hàng đợi vô hạn.
- **Gate gộp:** offscreen ép ngân sách nhỏ chứng minh đủ object, không đổi độ phân giải, fence và hủy kết quả cũ; kiểm picking/Gumball nếu publication đổi. Desktop đo riêng; nếu target và lô tối thiểu không vừa thì giữ ảnh trước và báo lỗi.

### M4 — Checkpoint kiểm chứng 512 MiB

- [ ] Đếm payload hình học mới đã chấp nhận theo phiên bản nguồn, kể cả thay thế cùng kích thước; camera/selection/cache không tính và Undo/Redo không đếm lại phiên bản đã gặp.
- [ ] Gộp yêu cầu, worker kiểm phần phụ thuộc thay đổi từ nguồn bất biến; chỉ chốt mốc khi thành công, bỏ kết quả cũ khi nguồn thay đổi. Ưu tiên tương tác rồi export; không khẳng định có thể ngắt native call đang chạy.
- **Gate gộp:** dưới/đúng/vượt ngưỡng, lệnh lớn, Undo/Redo, nguồn đổi, lỗi/hủy và kết quả export độc lập checkpoint. Không phát lại 256 lệnh hoặc sao chép cả model làm checkpoint.

### M5 — Nghiệm thu release

- [ ] Gộp một batch sau các mốc được chọn: cùng release, fixture/GPU/DPI/cửa sổ; đo peak managed, commit, working set và VRAM nếu truy vấn được; đối chiếu trước/sau, latency và mức từ chối. Đo desktop/WPR riêng khi có điều kiện, không suy từ headless.
- [ ] Ghi kết quả, skips và giới hạn vào ledger; giữ phiên OM9 đang mở. Build ngoài checkout, `-j 2`, kiểm runtime DLL; audit nguồn và dọn artifact theo quy định khi có batch build.

## Chống lặp

Mỗi mốc ghi đầu vào, file/caller sửa, bằng chứng hiện có và gate; hoàn thành rồi mới chuyển mốc. Dùng lại kết quả phù hợp cùng source/cấu hình; sau sửa chỉ chạy nhóm lỗi và phần bị ảnh hưởng, rồi một batch gộp tại gate. Không lặp audit hoặc test xanh vì mất phiên quan sát; kiểm tra tiến trình/log trước. Hai lượt liên tiếp không tạo tiến triển thì xác định blocker hoặc chọn công việc cụ thể thay vì đọc lại toàn bộ. Không mở rộng sang mọi lệnh chỉ vì tìm thấy cùng từ khóa. Các mốc còn lại không phải cam kết thời gian hay phần trăm hoàn thành.
