# Sửa 21 Rust regression — 2026-11-05

Phạm vi: 21 failure được ghi tại [lượt workspace trước](2026-11-05-workspace-build-test-commit.md), trên branch `hieu-nang`, base `592757c25`. Đây là sửa lỗi và cập nhật fixture/oracle theo contract đã có; không tiếp tục triển khai toàn bộ kế hoạch bộ nhớ/viewport/export.

## Thay đổi và nguyên nhân

| Nhóm | Số failure cũ | Nguyên nhân và bản sửa |
| --- | ---: | --- |
| File widgets | 7 | UI ghi lại metadata sau khi đã nhận token picker, làm context của chính Save thành stale. Chỉ stage trước khi tạo operation; phản hồi picker không sửa context. Bốn kho metadata trở thành no-op khi payload không đổi; trạng thái UI mặc định chưa có resource không tạo giao dịch giả làm mất Redo khi hủy. Metadata thực sự đổi được commit cùng scene metadata qua đường incremental sẵn có, chia sẻ geometry buffer; lỗi admission không công bố một nửa. Giữ nguyên kiểm tra owner/token/revision và phản hồi cũ. |
| Status / Escape | 2 | Info/Warning như “Command canceled” ghi đè Error, sau đó success đổi sang Ready. Giữ Error đến khi acknowledge rõ ràng; Escape vẫn hủy lệnh đang chạy. Test chuẩn hóa thông báo acknowledge lỗi cũ trước khi kiểm tra Info mới. |
| Builder docking | 2 | Rect chẩn đoán lấy toàn bộ parent rail; test bấm “Builder” thực ra trúng nút khác và đo vị trí sai. Dùng phần layout còn lại tại điểm bắt đầu Builder. |
| Icon audit | 2 | Đếm lẫn 20 icon trang Main Menu với hàng Top11 gồm 11 nút. Tách phép đo, vẫn kiểm đủ 20/11, các trang/category, bounds, ô vuông, overlap và resize/DPI; Top11 fit theo chiều ngang được kiểm theo pixel thiết bị. Không tăng count kỳ vọng thành 31 để bỏ qua layout. |
| Escape Builder/Setting | 1 | Harness không tiêu thụ texture delta; sau khi sửa còn lộ fixture Prong chỉ có một gem. Xử lý delta và dùng hai gem có placement khác nhau để tạo hướng row hợp lệ. Vẫn kiểm hủy preview, không commit và không thêm Undo. |
| Layer Show thiếu ngân sách | 1 | Fixture dùng footprint ổn định làm trần cho đỉnh khởi tạo, nên lỗi trước khi mở dialog. Đặt pressure sau startup, chỉ chừa 8 KiB cho đường no-op; giữ kiểm epoch/Undo. Không nới production budget. |
| Mesh fill | 1 | Đếm CPU epaint Path không còn quan sát mesh trên GPU. Dùng đường render project thật, kiểm diện tích mặt cube và mặt che cả hai trục grid bằng depth. |
| UI grid | 1 | Tìm CPU LineSegment trong renderer GPU-only. Kiểm pixel GPU ở hai phía của X/Y cùng giới hạn grid và document/Undo không đổi. |
| Side View | 1 | Oracle màu cũ không đúng CPlane Y/Z đã duyệt. Kiểm cyan nằm ngang theo world Y, yellow đứng theo world Z và đúng endpoints. Không đổi hướng camera trong đợt này. |
| Pipe | 1 | Fixture B-rep không khai báo document tolerance nên exact snap bị từ chối đúng. Dùng template tolerance như New, kiểm chuyển prompt sau hai radius pick; giữ oracle thể tích và Undo. Không thêm dung sai đoán hoặc nới sai số thể tích. |
| Transform blocker | 1 | Test còn coi ArrayCrvOnSrf/026 là chưa hỗ trợ dù đã có owner. Kiểm 026 available, giữ blocker độc lập 019/037 và các phép từ chối cũ. |
| Plugin attributes | 1 | Test giả định màu ban đầu None, trong khi Entity hiện lưu màu mặc định. So sánh dữ liệu getter với attributes thật của source; giữ kiểm staged edit/commit/Undo. |
| **Tổng** | **21** | Danh sách đối chiếu từng test gốc và test hiện tại nằm trong JSON kết quả. |

## Kiểm chứng

Kết quả và source hashes: [JSON](2026-11-05-fix21-regressions.json). Log nằm trong [batch fix21](../../build/standalone-validation/ui-artifacts/fix21-20261105/). Thông số build giữ nguyên target ngoài checkout `E:\om9-surface-snap-validation`, OCCT 8.0.1 SDK/runtime đã pin, `--locked -j 2`, dev debug 0 và incremental 0.

<!-- VERIFIED_RESULTS -->
**21/21 failure cũ đã đạt.** Kết quả mới nhất theo từng test liên quan: **669 passed / 0 failed / 1 ignored**. Tính theo (crate, target, test), lấy kết quả mới nhất khi một test được chạy lại, không cộng retries. Lượt gộp app lib/14 integration có 611 passed/1 failed/1 ignored; sau sửa scene metadata chỉ chạy lại 5 integration liên quan: 33 passed/0 failed. Pipe: 1 passed; plugin staging: 55 passed. Không có nhánh SKIP/no-GPU trong các log này.

Release build cuối exit 0, 411.64 giây. Lượt gộp app 289.50 giây; lượt sửa metadata 53.99 giây. Release create/inspect cùng exit 0, 4 object, restoration ready, persistent hash khớp. Nguồn và executable có SHA-256 trong JSON. Các test ignored không được tính là passed.
<!-- END_VERIFIED_RESULTS -->

Không chạy lại full workspace sau từng sửa nhỏ. Giữ các attempt đỏ: Escape sau khi xử lý texture delta còn thiếu gem row; hai lượt lib không compile vì test truy cập helper private; test chuẩn hóa status ban đầu chưa acknowledge Error. Lượt gộp phát hiện thêm regression mới: metadata commit không đồng bộ scene, khiến Point sau đó báo `Stale("command scene")`. Dừng build release chưa hoàn tất của nguồn đó, sửa đường commit và chỉ chạy lại năm integration lưu file/persistence liên quan. Kết quả Pipe, plugin và các phần độc lập đã đạt được tái sử dụng; không retry toàn workspace hoặc toàn bộ 510 lib tests.

Ba regression mới kiểm metadata mặc định, cả bốn resource đã nhúng và admission failure: stage không đổi không thay snapshot/mất Redo; metadata đổi giữ scene epoch khớp publication, chia sẻ cùng geometry buffer và cho phép vẽ Point tiếp; khi thiếu bộ nhớ giữ document/scene/Redo để retry. Test persistence hiện có tiếp tục kiểm display, F6, pictureframe và Job Bag.

## Giới hạn và việc tiếp theo

- GPU offscreen và egui headless không chứng minh ảnh được trình chiếu, độ trễ chuột, WPR hoặc độ mượt desktop.
- Không đổi hình học/kernel, meshing tolerance hay định dạng STL/3DM. Không chứng nhận độ chính xác toàn miền từ các regression này.
- Giữ lịch sử baseline 4.497 passed/21 failed/183 ignored; chỉ 21 failure cùng nhóm liên quan được đóng bằng bằng chứng mới. Các ignored ngoài nhóm vẫn chưa được chứng minh.
- Native source inventory bị packet preflight từ chối và tool fixtures thiếu PDF của lượt trước nằm ngoài 21 Rust failure này, chưa được sửa hoặc gọi là đạt.
- Metadata UI thực sự thay đổi vẫn được stage trước Save như thiết kế hiện tại. Đây không phải thiết kế lại transaction của toàn bộ giao dịch Save/Cancel khi có UI metadata chưa đồng bộ.
- Không thay phiên OM9, không mở bản vẽ người dùng, không push/publish hoặc sửa quy tắc trong skill. Các sửa khôi phục contract đã được ghi nhận.

Sau đợt này không có lý do chạy lại broad suite vì chỉ sửa tài liệu. Khi tiếp tục phần viewport/export, dùng milestone và lượt test gộp trong PLAN hiện có; không mở lại 21 lỗi đã có bằng chứng nếu không có thay đổi liên quan.

Kiểm tra hồ sơ: 21 ánh xạ test gốc → passed, JSON milestone/ledger và ID duy nhất, các link local đạt. Public source audit exit 0: 15200 files, 594 bindings, 593 SVG, errors rỗng.

Vệ sinh cuối: diff whitespace và file mới đạt; source hashes khớp bản đã ghi. Prune exit 0, không có run đủ điều kiện xóa; giữ 10 mới nhất và các run được dẫn, không xóa target ngoài checkout. Không có code/test/build chạy lại sau cập nhật hồ sơ.
