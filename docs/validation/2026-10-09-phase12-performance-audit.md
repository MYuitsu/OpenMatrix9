> Cập nhật: thiết kế bounded đã được duyệt và đợt tối ưu đã kiểm chứng; xem [kết quả mới](2026-10-09-phase12-performance-complete.md). Nội dung dưới đây lưu baseline trước sửa.

# Phase 1/2 — baseline trước tối ưu bổ sung

Yêu cầu: tăng hiệu năng Phase1 và Phase2, Rust là ưu tiên số1, giữ 60% số luồng khả dụng, hình học CAD và toàn bộ contract Copy/Paste/Undo/Redo/Cancel/Rhino5 đã nghiệm thu. Các phép đo dưới đây chạy trên runtime hiện tại, không phải kết quả của code tối ưu mới.

## Code hiện tại

- `Gui/ThreeDmModeling.cpp` đã xử lý dựng/validate/staging BRep bằng worker native, giải phóng GIL ở binding. `rust/Cargo.toml` đã dùng release opt-level3, thin LTO, codegen-units1. Tăng compiler flags hoặc chuyển tên file sang Rust không tự tạo lợi ích end-to-end. OCCT/openNURBS vẫn là kernel native.
- `ThreeDm.py::_mesh_from_rows` đã dùng một lần gọi Mesh bulk. `ThreeDm.py::_stage_current_geometry` vẫn lấy topology, tính signature, transform qua các wrapper Python và tạo dữ liệu JSON lớn; `ThreeDmPointCloud.py` vẫn chuyển property arrays/JSON. Các phần này cần đo riêng nếu mở rộng migration; không thể suy mọi thời gian Copy/Paste đều ở Python.
- `ThreeDm.py::_write_geometry_atomic` còn encode JSON có whitespace mặc định để truyền request cho native writer. Có thể giảm kích thước request trong cùng API, không đổi số/geometry.
- `rust/src/phase2_snap.rs::start_query` scan toàn bộ candidate cache để lọc object gần. Nó chỉ clone điểm của object được chọn, không clone toàn index, nhưng vẫn cấp phát/copy candidate vectors; `cached` và publish còn clone candidate payload. Cache cap4096 entries/131072 points. Có thể chia sẻ snapshot bất biến trong safe Rust và dùng lookup theo object/mode, giữ generation/budget.
- Native index rebuild/camera projection không được định lượng bởi warm-query gate. Chưa có bằng chứng cho tốc độ cold rebuild, nên không gán p95 warm cho lần đổi camera/document.

## Phép đo mới

Runtime: `H:/FreeCAD-src/build/om9-phase2-rust-sdk`; source/runtime manifest được kiểm trước mỗi launch và report được bind sau exit0. Các benchmark chạy tuần tự, có profile riêng, không sửa bản vẽ của người dùng. [Tổng hợp lần chạy](H:/FreeCAD-src/build/phase12-performance-baseline/b335de75890f428aa078c4708e6bf278/summary.json).

| Batch hiện hữu chạy lại | Dataset/scenario | Kết quả |
|---|---|---|
| modeling_clipboard_profile | Nhẫn nguồn bất biến29 root; mesh20k facets; mỗi mode1 warm-up+5 mẫu đo | 34 kiểm tra đạt |
| modeling_snap_performance_smoke | Scene10k CAD xa, kiểm cả dense64-object budget;500 warm queries | 5 kiểm tra đạt |
| modeling_snap_heavy_smoke | Scene mesh1.000.002 points/333.334 facets +cloud1.000.000 points +CAD;500 warm queries và chuột thật | 9 kiểm tra đạt |

Tổng: **3 batch hiện hữu được đo lại, 4 dataset/scenario chính, 48 kiểm tra đạt**. Số object, lần đo, truy vấn và assertions không phải số batch.

Các report:

- [Profile](H:/FreeCAD-src/build/om9-dev/build/modeling_clipboard_profile-1/915593bd64b8484ea11b458c136abdee/results.json).
- [Snap CAD](H:/FreeCAD-src/build/om9-dev/build/modeling_snap_performance_smoke-1/6e86fdc37ec54ff189e708edb9f72a30/results.json).
- [Snap mesh/cloud](H:/FreeCAD-src/build/om9-dev/build/modeling_snap_heavy_smoke-1/8859b7ed79254418b6ef04805990d21d/results.json).

| Metric | Baseline mới |
|---|---:|
| Mesh20k binder cũ từng facet, median | 1,868921s |
| Binder bulk hiện tại, median | 0,048838s |
| Nhẫn29 root chuẩn bị serial, median | 12,224794s |
| Nhẫn chuẩn bị ở30%, median | 3,706537s |
| Nhẫn chuẩn bị ở60%, median | 3,448141s |
| Nhẫn60%: SDK prepare/assemble_validate/stage, median từng stage | 0,830/1,868/0,661s |
| CPU process median theo24 logical CPUs | 26,29% |
| Process peak working set trong nhóm60% | 557,04MiB |
| Warm snap CAD10k remote, p95/median | 0,020300/0,018600ms |
| Warm snap million mesh/cloud scene, p95/median | 0,066700/0,049050ms |

CPU24 luồng; target/effective/peak14 worker, giữ policy floor(60%N) và giới hạn tasks/RAM. Chênh số với lần nghiệm thu cũ là biến thiên đo của binary hiện tại, không phải speedup từ code mới. Các số binder/native preparation không phải thời gian Copy/Paste toàn bộ. Source-level exclusions và counter literal0 không được nâng thành bằng chứng instrumentation độc lập.

## Đề xuất đợt sửa bounded, chờ duyệt thiết kế ngắn

Giữ kiến trúc ownership và các API/ABI đã nghiệm thu. Phase1 làm gọn JSON request lớn của đường import/export/clipboard phù hợp, đo bytes/serialization/native write và full Copy/Paste, không đổi geometry values hoặc schema/legacy FCStd. Phase2 thay candidate Vec clone bằng shared immutable Rust snapshots với lifetime/generation giữ nguyên, index cache theo object để không scan các object ngoài query, bảo toàn residual budget/modes/cache caps và incomplete fallback. Snapshot Rust không chứa con trỏ Qt/document/OCCT.

Kiểm chứng: unit/native/host RED→GREEN cho serialization equivalence, stale handles/cache invalidation/budgets/modes; thêm workload candidate cache lớn để đo allocation/median/p95; chạy lại mesh/cloud/ring và thực tế Rhino5 Copy/Paste/SaveAs/reimport trên runtime riêng. Chỉ ghi speedup sau A/B cùng fixture/binary/options, warm-ups tách riêng, báo cả RAM và end-to-end. Nếu không đo được lợi ích hoặc có regression, giữ baseline. Không hứa hệ số tăng tốc trước khi đo.

Hướng mở rộng tiếp theo là typed-buffer migration cho mesh/cloud và cold-index geometry snapshots; chúng thay native/Rust boundary và cần thiết kế kiến trúc riêng khi phép đo chứng minh cần thiết. Đợt bounded không tự chuyển toàn bộ Python hoặc toàn CAD kernel sang Rust.

Baseline đã đo xong; **0 batch đã chuẩn bị còn chờ,7 gói full openNURBS vẫn mở,tổng batch tương lai chưa xác định**. Các benchmark ứng viên chỉ là kế hoạch, chưa tính thành batch đã chuẩn bị. Tiếp theo duyệt thiết kế bounded trước product edits theo Superpowers brainstorming. Phase1/2 nghiệm thu cũ và runtime/snapshot vẫn nguyên vẹn; không commit/push/ghi đè bản cài chính.
