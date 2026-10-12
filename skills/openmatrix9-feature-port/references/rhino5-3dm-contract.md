# OM9-FILE-012 — quy tắc 3DM đã được duyệt

Người dùng đồng ý ghi quy tắc này vào skill ngày 2026-10-06, sau khi đã triển khai và kiểm chứng. Đây là quyết định nghiệp vụ OpenMatrix9. Trước đây importer từ chối block; hành vi hiện tại bung block nhúng thành hình học CAD/mesh tương ứng.

- Import block nhúng bằng cách giải định nghĩa và bung đệ quy các instance, kể cả block lồng nhau. Ghép transform theo thứ tự cha * con; áp dụng đúng vị trí, xoay, scale không đều và phản chiếu. Giữ hình học CAD, không tự chuyển CAD thành mesh để né lỗi chuyển đổi.
- Hình học thuộc định nghĩa chỉ được nhập thông qua các instance đang sử dụng nó. Không nhập thêm bản gốc của định nghĩa vào cấp trên cùng và không làm mất các bản đặt khác nhau của cùng một block.
- Giữ màu kế thừa từ block cha khi đối tượng dùng màu từ parent; kết hợp trạng thái hiển thị/khóa của cha và con. Xuất hình học đã bung; chưa giữ cấu trúc định nghĩa/tham chiếu block của Rhino. Phải nêu rõ giới hạn này khi người dùng cần chỉnh sửa block tiếp trong Rhino.
- Định nghĩa hoặc thành viên bị thiếu, block ngoài không có hình học nhúng, vòng lặp định nghĩa và transform không hợp lệ phải báo lỗi trước khi sửa document. Không bỏ qua block lỗi rồi báo import thành công.
- Nghiệm thu phải có import → export Rhino 5 → import lại trong FreeCAD, kiểm tra số đối tượng, metadata, topology hợp lệ, bounds hình học và độ lệch diện tích/thể tích. Kiểm tra thêm lưu, đóng và mở lại FCStd giữ đủ hình học CAD hợp lệ. Dùng các file 3DM người dùng cung cấp làm fixture bất biến; lưu output ở thư mục test/build riêng.
- Với NURBS phức tạp, dùng tích phân thích nghi cho diện tích/thể tích và bounds hình học thay vì chỉ dựa vào `Shape.Volume` hoặc bounds từ tessellation. Không nới ngưỡng nghiệm thu chỉ để làm test qua. Bằng chứng archive version 5/50 không thay thế việc kiểm tra trong ứng dụng Rhino 5.

Trong checkout OpenMatrix9, đọc `docs/features/3dm-support-matrix.md` để xác định loại dữ liệu đã hỗ trợ và giới hạn còn lại; đọc `docs/validation/2026-10-06-3dm-blocks.md` để đối chiếu bằng chứng. Không suy từ hai fixture đạt test rằng mọi chức năng openNURBS đã được hỗ trợ. Các tài liệu này là đường dẫn tương đối với checkout, không phải thư mục skill đã cài.

## Phase 1 — Copy/Paste và worker

Người dùng đồng ý lưu bổ sung này ngày 2026-10-09, sau kiểm chứng trong Rhino 5 và FreeCAD. Phase 1 trước đây ưu tiên Import/Export Selected qua file 3DM; nay có thêm clipboard hai chiều để chuyển hình học rồi vẽ tiếp, không cần lịch sử Rhino hay render. Đây là lựa chọn OpenMatrix9 trong phạm vi hình học Phase 1 đã nghiệm thu.

- Copy/Paste Rhino 5 ↔ OM9 dùng hình học đang chọn. Copy lấy trạng thái hiện tại, kể cả đối tượng đã chỉnh sửa hoặc mới vẽ; không dùng lại hình học nguồn cũ thay cho kết quả hiện tại. Ctrl+C/V trong viewport/model tree gọi lệnh hình học; ô nhập chữ giữ shortcut văn bản thông thường.
- Paste tạo các đối tượng độc lập, có thể chỉnh sửa, trong một transaction Undo. Một lần Undo gỡ toàn bộ lần Paste; Redo khôi phục lần đó. Không yêu cầu tái tạo lịch sử Rhino hoặc render để vẽ tiếp.
- Worker mặc định là `max(1, floor(0.60 * N))`, với `N` là số luồng CPU logic khả dụng cho tiến trình. Giảm theo số task độc lập và RAM khả dụng; không áp trần cố định 4 worker. Ví dụ 24 luồng khả dụng cho mục tiêu 14 worker trước các giới hạn task/RAM. Đây là ngân sách số worker, không bảo đảm dành đúng 40% CPU/RAM cho phần mềm khác.
- CAD có display mesh vẫn được phân loại là CAD; không biến CAD thành mesh để copy hoặc để né xử lý hình học. Mesh thực và point cloud là các loại dữ liệu riêng. Xét khả dụng/phân loại bằng metadata, không đọc toàn bộ point arrays hoặc tessellation chỉ để bật lệnh. Khi thực sự Copy mesh/cloud, vẫn phải tuần tự hóa dữ liệu hình học cần thiết.

Khi bảo trì hoặc kiểm tra, đọc `docs/validation/2026-10-09-modeling-phase-1-clipboard.md` và bộ bằng chứng tại `docs/validation/modeling-phase-1-clipboard/` trong checkout. Bằng chứng này chỉ nghiệm thu phần mở rộng Phase 1; không đóng Phase 2–5 hay chứng minh full openNURBS.
