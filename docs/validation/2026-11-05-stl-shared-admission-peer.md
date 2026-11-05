# STL capture dùng admission peer sẵn có — 2026-11-05

Phạm vi R1a: bỏ việc dựng coordinator/account/identity rỗng rồi adopt chỉ để tạo admission peer cho capture STL. Rust tiếp tục sở hữu chính sách; không đổi FFI, mesher/profile, geometry, Undo semantics, input/Gumball hoặc phiên desktop đang mở. Đây là một phần R1a, không phải đóng toàn kế hoạch.

## Lỗi tái hiện và thay đổi

`capture_stl_selected_with_sample` trước đây gọi `World::new([])` → `TransactionCoordinator::new` → `adopt_external_captures`. Các bước này tạo identity/account riêng rồi bỏ account để ghép nguồn. Admission của peer vẫn tính identity ledger riêng và mang authority khác coordinator nguồn.

Run đỏ chuẩn `build/standalone-validation/ui-artifacts/stl-peer-20261105-061149` chạy đúng hai test mới, cả hai thất bại, exit 101. Với cùng registry và dữ liệu đang sống, peer cũ tính hơn peer tham chiếu 40 byte metadata trong fixture. Khi xin đúng headroom đã được peer chung chấp nhận, nó thu hồi CpuCache quota từ 1960 xuống 1584 byte để bù. Đây là sự thu hồi không cần thiết trong tình huống sát trần, không phải bằng chứng tiết kiệm RAM lớn hay FPS tăng. Run trước `stl-peer-20261105-061129` cũng tái hiện authority khác và chênh 40 byte. Lượt đầu `061050` dùng sai bộ lọc, chạy 0 test và script từ chối: không tính là red/pass.

Sửa production dùng trực tiếp `self.coordinator.memory_admission_peer()?` đã có. Tái dùng account, document registry, sampler/policy và export permit chung; dùng empty identity ledger đã có owner ở account. Không tạo coordinator root/account/identity allocation mới cho việc ghép này và không sao chép world/journal. Peer vẫn là một giá trị coordinator riêng dùng để admission; đây không phải khẳng định mọi metadata/capture plumbing đã được hạch toán.

## Kiểm chứng

Windows x64, default Cargo features, OCCT 8.0.1 SDK/TBB/jemalloc như trước; target riêng `E:\om9-surface-snap-validation`, `-j 2`, dev debug=0/incremental=0. Không thay kernel hoặc profile dung sai. Script `build/run-stl-peer-20261105.ps1 -Full` giữ log trong `build/standalone-validation/ui-artifacts/stl-peer-20261105-061229`.

| Nhóm | Kết quả cuối |
|---|---:|
| Peer mới, nằm trong STL unit bên dưới | 2 pass |
| Toàn STL unit | 60 pass |
| Replacement budgets | 8 pass |
| Document/file/ordinary RAM | 12 + 20 + 9 pass, 6 large ignored |
| STL integration, adapter native thật | 60 pass |

**169 distinct test pass, 6 ignored, 0 failed**; không cộng lại hai peer tests nằm trong 60 STL unit. Hai fixture mới là mesh một tam giác, fixture thứ hai thêm một Point rồi Undo/Redo; không gọi 169 là số fixture hay phần trăm hoàn thành. Các reservations sát trần không cấp phát payload tương ứng, không là đo RSS/peak thật.

Oracle mới: cùng yêu cầu sát ceiling không làm mất quota cache; authority nguồn đúng; peer world/Undo/Redo rỗng; hai captures giữ đúng các revision trước/sau edit/Undo/Redo và thấy cùng registry; nguồn hiện hành cùng allocation, không clone document; xuất capture cũ giữ một tam giác/source epoch cũ, không đổi document hiện hành; bỏ capture đầu không làm peer còn lại mất tài khoản/lease. Suite hiện có kiểm failure/cancel, permit, New/Open/reopen/restore và destination preservation.

Runtime elapsed được lưu trong terminal receipts của lượt: peer gồm build 26,59 s, STL unit 1,67 s, replacement 4,63 s, lifecycle gồm build 29,47 s; STL integration 115,22 s gồm build, runtime 111,54 s. Không đo hiệu năng release/desktop mới hoặc so tốc độ tạo peer. Không chạy GPU vì không đổi renderer/input/picking; hồi quy STL thật được chạy vì thay đường admission dùng chung.

Source raw-file SHA-256 của `crates/om9-app/src/session/stl.rs`: `52372d09cc12b9a74b58f0cab97875131334e4c7691bf3facdeda93754c685c4`. Đây không phải Git commit hoặc fingerprint toàn worktree. Plan/alias, README, live ledger và M01 runtime records được đồng bộ đúng phạm vi. Public-source audit exit 0: 15.189 source files, 594 icon bindings, 593 SVG, errors rỗng. Đã kiểm tra fingerprint, terminal summaries, 37 local links, JSON ledger và ba runtime owner files; `git diff --check` trong scope đạt. Prune qua tool dự án sau kiểm tra resolved targets trong checkout đã bỏ 2 run không còn được dẫn, giữ 10 mới nhất mỗi root cùng các run được hồ sơ dẫn; không xóa target ngoài checkout. Log: `build/standalone-validation/stl-peer-prune-20261105.log`. Không chạy lại Rust sau thay đổi chỉ ở hồ sơ.

## Còn mở và bước tiếp theo

`scene_auxiliary_bytes` và `scene_high_water` vẫn là các estimate của capture; chưa thay bằng owner sống. Đây là phần chính còn lại của R1a. Coordinator/peer/journal/account-retirement inventory toàn bộ vẫn thuộc R1b; chia sẻ peer không tự chứng minh toàn tổng RAM/RSS đã đúng. Diagnostic source/job provenance ngoài authority của peer vẫn còn thiếu, không coi peer epoch 1 là epoch hiện hành của tài liệu; receipt giữ source epoch chụp riêng.

Native allocation trước Rust callback, native envelope/peak R2a-STL, UI STL worker, display/v5, GPU budget/batching, các gate chính xác STL/3DM, checkpoint và release/desktop đều còn mở. Không có batch đo 3DM/openNURBS mới, không đóng exchange packages; tổng batch tương lai chưa xác định. Tiếp tục R1a bằng owner sống cho auxiliary capture và vòng đời edit/drop/replacement, không triển khai lại peer construction.

[Plan](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md).
