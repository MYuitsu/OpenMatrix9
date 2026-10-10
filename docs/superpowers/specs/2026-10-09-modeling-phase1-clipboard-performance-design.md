# Phase 1 bổ sung — Copy/Paste Rhino 5 ↔ OpenMatrix9 và tốc độ

Status: complete within scoped Phase1 extension;14 bound requirements pass. Actual Rhino11 fixtures/496 checks,19 FreeCAD reports/510 checks,48 native suites,80 Rust/58 Python. Two unavailable Rhino5 scalar probes on one127-face Brep are recorded; all SaveAs outputs pass independent native adaptive mass/topology/bounds reimport. Source/runtime/performance evidence: `docs/validation/modeling-phase-1-clipboard`.

## Ý định đã xác nhận

User yêu cầu thêm Copy/Paste và tối ưu tốc độ vào Phase 1, xác nhận hai chiều Rhino 5 ↔ OpenMatrix9 và bổ sung yêu cầu **tối ưu bằng đa luồng, mặc định dùng 60% số luồng CPU khả dụng thay cho giới hạn cố định 4 luồng**. Đích sử dụng: chọn hình học trong Rhino, copy sang OM9 và vẽ/sửa tiếp; chọn hình học hiện tại trong OM9, copy trở lại Rhino. Không yêu cầu history hoặc render.

Phase 1 qua file 3DM đã nghiệm thu 10 ca thực tế. Bằng chứng cũ giữ nguyên trong `docs/validation/modeling-phase-1`; nó chưa chứng nhận clipboard hoặc binary tối ưu mới. Phạm vi bổ sung có gate riêng và chỉ được gọi đạt sau kiểm tra hai ứng dụng thực tế.

## Lựa chọn kiến trúc

**Đề xuất: clipboard native mang archive 3DM, dùng lại converter và writer Phase 1.** Không cần người dùng chọn file trong luồng Copy/Paste. Transport Windows/Qt tách khỏi policy Rust, staging/import/export Python và native geometry binding hiện có. Tạo staging riêng khi adapter hiện tại cần đường dẫn; không viết lại converter hoặc đưa renderer vào luồng này.

Phương án thay thế là cầu nối chạy script/plugin trong Rhino và trao đổi file tạm. Nó dùng được converter hiện có nhưng tăng phụ thuộc cài đặt, điều phối ứng dụng và bảo trì. Đề xuất ưu tiên clipboard native; không tự chuyển sang phương án script và gọi đó là Ctrl+C/Ctrl+V tương thích nếu native format chưa qua Rhino 5 thật.

Primary reference: [McNeel clipboard guide](https://developer.rhino3d.com/guides/cpp/retreiving-rhino-data-from-clipboard/) và [official Copy/Paste sample](https://github.com/mcneel/rhino-developer-samples/blob/7/cpp/SampleCommands/cmdSamplePaste.cpp). Actual Rhino5 `_CopyToClipboard` confirmed `Rhino 5.0 3DM Clip global mem`, containing a raw V5 archive. `_Copy` is the interactive geometry duplication command. Only owned test fixtures are captured.

## Luồng sử dụng

1. Rhino: chọn geometry, Ctrl+C. OM9: focus viewport hoặc cây model, Ctrl+V; nhận geometry độc lập ở world coordinates và có thể vẽ tiếp.
2. OM9: chọn geometry hiện tại, Ctrl+C. Rhino 5: Ctrl+V; nhận cả geometry nhập đã sửa và geometry mới được chọn. Không hồi sinh object đã xóa hoặc xuất toàn document khi chỉ chọn một phần.
3. Trong OM9, `Copy3dm`/`Paste3dm` qua menu và Command dùng cùng handler. Shortcut ưu tiên geometry chỉ trong workbench/model context phù hợp. Khi sửa text, Command input, ô thuộc tính hoặc dialog, Ctrl+C/Ctrl+V vẫn thuộc text control. Copy/Paste nội bộ dùng cùng archive adapter đối với geometry đủ capability.
4. Paste mặc định giữ world coordinates, chuẩn hóa đơn vị mm/cm đúng một lần. Không thêm chọn điểm đặt hoặc PasteSpecial trong phạm vi này.
5. Copy không sửa document hoặc Undo stack. Mỗi lần Paste tạo UUID độc lập trong một transaction Undo; Paste nhiều lần tạo nhiều bản độc lập. Recompute/refresh theo batch sau commit.

## Geometry và thuộc tính

Giữ phạm vi geometry đã đạt của working mode: CAD Point/Curve/BRep/Extrusion, native Mesh và PointCloud; placement, metadata cơ bản và signed-solid definition representation đã có. CAD có mesh hiển thị vẫn được copy bằng geometry CAD.

Không đọc display-mesh vertices hoặc toàn bộ cloud chỉ để xét command có khả dụng hoặc phân loại selection. Khi thật sự copy native Mesh/PointCloud đã chọn, dữ liệu vertices/points cần được serialize để truyền geometry. Không hứa truyền mesh mà không đọc dữ liệu hình học của nó.

Các block/reference chưa có modeling capability vẫn theo guard Phase 1; không tự thêm member editor hoặc clipboard support cho mọi type openNURBS. History/render/layout/userdata không được thêm vào working clipboard.

## Transport và lỗi

- Decode theo format được xác thực và kiểm tra kích thước/header trước đọc archive; áp dụng giới hạn archive hiện tại 512 MiB.
- Copy: chuẩn bị selection, serialize và reread validation trước khi publish; lỗi chuyển đổi giữ clipboard cũ và document nguyên vẹn.
- Paste: lấy snapshot clipboard ngắn, đóng clipboard trước conversion; source bị thay đổi trong lúc capture phải phát hiện hoặc đọc lại có giới hạn. Không giữ Windows clipboard lock trong lúc convert/recompute.
- Clipboard busy, không có Rhino geometry, payload truncated/sai length/version không hỗ trợ hoặc unsupported object: báo lỗi rõ, không tạo object dở dang, không mở hộp thoại Import file thay cho Paste.
- Không dereference tùy tiện đường dẫn từ text clipboard. Nếu actual Rhino transport có đường dẫn/OLE medium, phải xác nhận medium, copy snapshot vào staging do test/app sở hữu rồi import; không sửa/xóa file Rhino.
- Dọn staging do OM9 sở hữu khi success/failure/cancel. Giữ đúng Win32/OLE ownership của payload và Qt data source; clipboard còn dùng được khi source document đóng. Khả năng paste sau khi ứng dụng nguồn thoát cần được kiểm tra, không suy từ file lifetime.

## Tốc độ — đa luồng native, profile trước/sau

Luồng hiện tại trong `Gui/ThreeDmImportJobs.cpp` chạy tối đa 4 **process** QProcess riêng; đó là baseline song song hiện có, chưa chứng minh thread pool. Yêu cầu bổ sung là triển khai và đo **nhiều thread trong một process**, không gọi worker process cũ là đa luồng.

Code hiện tại ghi rõ lý do cách ly process: openNURBS có SubD serial numbers và diagnostic counters dùng process globals. Bản working đang bỏ SubD không tự chứng minh mọi đường openNURBS/OCCT an toàn. Vì vậy reader/model graph và archive stream giữ tuần tự ở thiết kế ban đầu; chỉ đưa operation vào thread pool sau audit call path/global state và stress test. Ưu tiên parallel conversion trên geometry độc lập đã snapshot và buffer preparation; không chỉ thay QProcess bằng std::thread quanh toàn bộ readArchive.

### Phân chia công việc

1. Luồng chính: capture/publish clipboard qua Qt/Win32, kiểm tra selection và snapshot dữ liệu cần thiết. Không giữ clipboard lock trong lúc chuyển đổi.
2. Thread pool native C++: chuyển đổi các root/member occurrences độc lập, chuẩn bị BRep/Mesh/Cloud và staging/serialization có thể tách. Tránh Python threads cho công việc CPU; Python binding chỉ điều phối, giải phóng GIL khi chờ đoạn native không gọi Python.
3. Mỗi task giữ geometry/model state và staging riêng, không dùng chung đối tượng mutable của openNURBS/OCCT. Init kernel trước fan-out. Dependency/definition graph được xác thực trước dispatch; expanded members thuộc cùng root không bị chia sai owner hoặc mất transform.
4. Ghép kết quả theo thứ tự root/member ban đầu, validate geometry/archive rồi publish clipboard hoặc commit transaction. Đọc đủ dependency trước khi tạo snapshot; không worker nào truy cập FreeCAD document, PyObject GUI, ViewProvider, clipboard hoặc QApplication widgets.
5. Luồng chính tạo object, cập nhật selection, Undo/Redo, recompute và refresh theo batch. Trong thời gian job native chạy, giao diện có progress/cancel; thread scheduling phải thực sự trả quyền điều khiển event loop, không chỉ tạo thread rồi block GUI bằng join.

### Giới hạn và rollback

- Pool có giới hạn động: `target_workers = max(1, floor(0.60 * N))`, với `N` là số logical CPU threads mà process được phép sử dụng, có xét affinity/giới hạn hệ điều hành. Nếu không xác định được N, dùng 1 worker. Không áp lại trần cố định 4 luồng. Ví dụ N=8/12/16/24/32 thì target=4/7/9/14/19. Quy tắc làm tròn xuống và fallback là quyết định thiết kế OM9 cho yêu cầu 60%.
- Số worker thực sự chạy không vượt target, số task độc lập sẵn sàng và ngân sách RAM. Ghi N/target/effective workers cùng lý do giảm vào report; pool tái sử dụng có giới hạn, không tạo thread theo số object. Geometry nhỏ chạy serial khi dispatch cost lớn hơn phần lợi. Benchmark serial, khoảng 30% và mức mặc định 60% trên cùng fixture, gộp các mức trùng nhau ở CPU nhỏ.
- Với một Mesh/Cloud lớn duy nhất, chỉ chia việc chuẩn bị buffers độc lập theo chunk khi phép ghép giữ topology/field order chính xác; không gọi đồng thời vào cùng instance Mesh/Shape để tạo facets hoặc sửa geometry.
- Cancel/error: ngừng cấp task mới, chờ task đang chạy hoặc cancellation point có giới hạn, join pool và dọn staging trước trả lỗi. Không detach thread; không publish clipboard dở dang hoặc commit một phần document. Worker chỉ gửi kết quả/progress qua đường có ownership rõ.
- Candidate phải stress-test kernel/thread safety. Đường chuyển đổi chưa chứng minh an toàn vẫn chạy serial hoặc giữ worker process cách ly; giới hạn đó được ghi rõ, không đổi nhãn thành thread-safe và không bỏ oracle để đạt tốc độ.
- Không chạy song song việc sửa document/Undo stack hoặc viết chung một archive stream. Writer cuối dùng kết quả đã ghép và kiểm tra đúng selection hiện tại.

Baseline cũ trên máy này: working import nhẫn 29 roots median khoảng 8.13 s; Mesh 20k triangles khoảng 0.87 s. Đây là số đo slice cũ, chưa phải baseline clipboard hoặc cam kết tốc độ.

Đo cùng fixture/binary/profile: warm-up tách riêng, ít nhất 5 lượt đo ổn định cho serial, khoảng 30% và mức mặc định 60% số logical CPU threads khả dụng, cùng baseline process nếu áp dụng; ghi median/max, bytes payload và thời gian từng đoạn capture/serialize/convert/bind/recompute/refresh. Ghi N, target/effective workers, CPU utilization và RAM cho fixture có kích thước xác định; không suy speedup từ số thread. Không dùng thời gian toàn probe gồm kiểm tra mass properties làm thời gian Paste.

Hotspots cần kiểm tra theo code hiện tại:

- Mesh import đang tạo facets qua nhiều lời gọi Python/native; thử batch construction khi profile chứng minh chi phí, giữ triangle/quad topology và orientation.
- Export preflight document/selection, serialize JSON/temporary BRep, validation lặp, native thread pool và GUI recompute/display: tối ưu đoạn thật sự chiếm thời gian. Global archive writing và document mutation vẫn được tuần tự hóa.
- Giảm số bản sao bytes, traversal và redraw dư. Chỉ bỏ validation trùng khi thay bằng validation tương đương được chứng minh; không bỏ IsValid, staging reread, atomicity hoặc oracle geometry.
- Cache chỉ xét sau khi có invalidation cho geometry/placement/delete/Undo/Redo/metadata. Không dùng snapshot cũ để tăng tốc copy geometry đã sửa.

Gate tốc độ: có baseline và after trên cùng điều kiện, chỉ nhận thay đổi có cải thiện lặp lại ở hotspot mục tiêu và không làm workflow khác chậm hơn ngoài độ dao động đo được. Không hứa hệ số tăng tốc trước profile.

## Khu vực dự kiến thay đổi

- Policy/capability và trạng thái operation: Rust trong OM9; geometry conversion giữ C++/OCCT/openNURBS.
- Clipboard transport Windows/Qt: adapter mới trong `Gui/`, tích hợp workbench lifecycle, menu và native keyboard context hiện có.
- Working staging/import/export: `ThreeDmModeling.py`, `ThreeDm.py` và command binding hiện tại.
- Test native decoder/ownership/policy, test Python staging, GUI FreeCAD shortcut/context/Undo/Redo, actual Rhino scripts và performance macro.
- Runtime mới, staging và evidence riêng; không ghi đè runtime/bằng chứng Phase 1 đã nghiệm thu trước khi candidate đạt. Không sửa parent FreeCAD Copy/Paste nếu module-level routing đủ đáp ứng.

## Nghiệm thu bổ sung

- [x] Capture actual Rhino5 `_CopyToClipboard`: format/medium/header/units/selection trên fixture sở hữu; encoder/decoder roundtrip và malformed controls đạt.
- [x] Rhino5 Copy → OM9 Paste → thêm/sửa geometry → OM9 Copy → Rhino5 Paste; actual document commands, geometry và hashes được ghi.
- [x] Curve/conic, solid/extrusion, point, cloud, Mesh20k và nhẫn thật; mm/cm, placement/reflection và selection subset đều đúng.
- [x] Repeated Paste độc lập; Undo/Redo, thiếu active document, empty/unsupported clipboard, malformed/busy/cancel rollback và clipboard lifetime đúng.
- [x] Shortcut trên viewport/model, menu/Command cùng handler; text controls không bị chiếm Copy/Paste.
- [x] CAD không bị tessellate để trao đổi; command availability không đọc point arrays nặng; selected Mesh/Cloud copy truyền đủ data.
- [x] FCStd save/reopen sau khi source clipboard/staging bị bỏ; current export và target SaveAs output reimport đạt.
- [x] Bounds 0.001 mm; area/volume max(0.001, 0.0001*abs(reference)); exact conic/point/cloud và native mesh signature giữ oracle trước. Instance bounds từ transformed member geometry, không cache display.
- [x] Profile trước/sau và số đo riêng Copy/Paste; regression các ca Phase 1 trên candidate cuối và actual Rhino5 trên cùng source/runtime bound đạt.
- [x] Thread pool native mặc định `max(1, floor(0.60*N))` có actual concurrent jobs khi đủ CPU/task/RAM; kiểm N=1/2/4/8/12/16/24/32 và unknown/affinity-limited, không còn trần cố định 4. Kết quả geometry/root/member order giống serial; không chỉ bật process worker cũ. Stress repeated copy/paste, exception/cancel/cleanup, RAM limits và GUI responsiveness đạt.
- [x] Cập nhật spec/checklist/README/progress/application evidence. Chỉ đóng phạm vi bổ sung khi native+FreeCAD+Rhino5 gates đạt.

## Trạng thái review

The user authorized inline execution of the agreed two-direction/60% design. One fresh final reviewer found one Important issue: restore the previous clipboard after a post-publication failure. An actual FreeCAD RED regression reproduced it; the fix materializes the previous MIME data and restores it only while this operation still owns the clipboard sequence. No Critical/Minor findings. The ledger retains rulings, verification and scope limits.
