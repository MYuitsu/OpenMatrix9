# Rà soát checklist viewport/export và RAM — 2026-11-05

Phạm vi: yêu cầu người dùng đánh giá lại plan, bỏ việc đã hoàn tất khỏi checklist và bổ sung phần RAM chưa hợp lý. Chỉ sửa tài liệu, không sửa runtime, không mở/đóng OM9, không chạy lại Rust/GPU/release. Không nâng trạng thái triển khai milestone từ lần rà soát này.

## Những phần không còn là việc cần triển khai lại

- Undo mặc định 1024 MiB, cấu hình tối đa 50% RAM và inventory 100 MiB: [bằng chứng](2026-11-02-undo-ram-cap-and-inventory.md).
- Đường tương tác tổng hợp vượt 256 MiB đã có release fixture, sửa/Undo/Redo/save/reopen: [bằng chứng](2026-11-04-interactive-ram-transition.md). Không kết luận mọi codec/native path đã hết giới hạn.
- Buffer scene startup/New/Open, full candidate/plugin và Undo/Redo đã có admission/owner transfer và test: [startup](2026-11-05-initial-scene-buffer-registration.md), [candidate](2026-11-05-full-candidate-buffer-publication.md), [history](2026-11-05-history-scene-buffer-registration.md).
- Nguồn và display snapshot B-rep đã chia sẻ; một NativeBrep fixture đã chứng minh STL không lấy mesh display: [chia sẻ](2026-11-04-brep-shared-display.md), [export](2026-11-04-native-brep-display-export-separation.md). Display vẫn nằm trong model/archive v4, chưa phải cache độc lập/v5.
- STL profile, guard f64/f32 T/10 và ca ba trục gần 100.000 mm, staging readback/triangle decode đã có: [precision](2026-11-03-stl-f64-quantization.md), [staging](2026-11-03-stl-staging-readback.md). Không chứng minh tổng sai số toàn miền, self-intersection hoặc containment.
- Capture/session invalidation/worker permit/cancellation đã có nền tảng: [session](2026-11-04-stl-capture-session-identity.md). UI chưa nối worker; giữ thành việc tích hợp, không viết lại nền tảng.
- Lỗi Vulkan fallback trên adapter đã xác định và sphere peak gần 1 GB đã có sửa/đo giới hạn: [GPU](2026-11-04-gpu-vulkan-luid-replay.md), [sphere](2026-11-03-sphere-stl-mesher-memory.md). Không mở rộng kết quả sang mọi GPU/OCCT shape.

## Đối chiếu code và phần RAM cần bổ sung

1. `crates/om9-core/src/budget/memory.rs`: `MemoryBudget::reserve` giữ pending cho cả vòng đời reservation; comment thừa nhận có thể tính trùng RAM đã allocated. `crates/om9-core/src/transaction.rs`: `ExternalMemoryReservation::recheck_physical` lấy tổng các lớp accounted làm pending; production còn `reserve/grow/adopt` riêng. Vì vậy R1 phải sửa semantics ở đúng tài khoản đang dùng, không chỉ helper độc lập. Rà soát này chưa đo mức false refusal trên máy thật.
2. `crates/om9-app/src/session/stl.rs`: cả synchronous captured export và detached export dùng `reserve_remaining_external_memory_with_held`. `crates/om9-app/src/session/files.rs::save_archive_for_batch_guard` cũng giữ toàn headroom. R2 chuyển sang envelope/giai đoạn có căn cứ và giữ old/new/scratch overlap; thêm acceptance edit đồng thời khi còn ngân sách thật.
3. `scene_high_water`, auxiliary captures, metadata, lazy picking caches, snapshot input và allocator capacity cần phân biệt bytes còn sống với peak lịch sử. Transfer vào registry không được tính tiếp ở held usage; hủy/eviction không trả bytes khi worker còn giữ.
4. `crates/om9-native-owner/src/system_memory.rs` hiện đọc RAM vật lý bằng `GlobalMemoryStatusEx`; chưa có cặp tổng commit hệ thống `CommitTotal/CommitLimit` ở adapter này. Plan thêm kiểm tra commit headroom riêng cho tác vụ nặng, giữ nguyên trần managed 60% RAM vật lý. Đây là yêu cầu cần triển khai/kiểm chứng, không phải cam kết chống OOM.
5. Undo quota là phần bên trong ngân sách tổng, không phải thêm 1024 MiB hoặc 50% vào 60%. Cache thu hồi phải tính đến owner cuối/fence, không chỉ cờ nonresident. Không xóa lịch sử để cứu giao dịch thất bại.

## Code mới chưa được đóng bằng chứng

Có `DocumentSnapshot::restore_with_admission`, helper `build_reserved_document_replacement`/`publish_reserved_replacement`, đường Snapshot/Master dùng helper và các test pressure/observer/old worker. Có test `first_incremental_edit_registers_buffers_without_viewport_work` và thay đổi incremental buffer owner. Chưa có báo cáo cuối bao phủ toàn bộ source hiện tại; chỉ sự hiện diện của code/test không đủ để bỏ khỏi R0. Lần rà soát này không phục dựng kết quả pass từ lệnh kiểm thử bị gián đoạn hoặc output không còn đầy đủ.

## Bàn giao và kiểm tra tài liệu

Plan chính là [PLAN.md](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md); `PLAN-REVISED.md` tiếp tục trỏ vào cùng checklist. R0 chỉ chốt thay đổi publication mới; R1 sửa accounting và R2 giảm reservation quá rộng là các bàn giao riêng trước P1–P5. Các yêu cầu v5, cache GPU/batching, global STL accuracy, 3DM và checkpoint vẫn mở; Mesh Repair giữ AFTER riêng.

Kiểm tra đã chạy: liên kết local của cả ba file Markdown hợp lệ; ledger parse JSON được và ID lần rà soát xuất hiện đúng một lần. Public-source audit: 15.170 source files, 594 icon bindings, 593 SVG, không có lỗi. Workflow status xác nhận router standalone và ledger hiện hành; output lớn không được dùng làm bằng chứng nghiệm thu. Kiểm tra JSON ban đầu dùng giả định mọi entry cũ có `id` nên báo `KeyError`; đã sửa phép kiểm để giữ nguyên các entry lịch sử không có trường đó và chạy lại thành công. Không có phép đo hình học, RAM runtime, desktop hoặc chứng nhận độ chính xác mới.
