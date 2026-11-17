# Quy tắc skill: tách nguồn và preview theo từng lệnh

Người dùng yêu cầu rõ ngày 2026-11-16 cập nhật skill: khi kiểm tra một lệnh,
tái dùng và đánh dấu phần tách đã có; triển khai phần thiếu theo luồng preview
nhẹ trước, nguồn chính xác tính riêng và cập nhật viewport khi hợp lệ; duy trì
một danh sách theo lệnh. Đây là sự cho phép ghi chính quy tắc này vào skill,
không cần hỏi lại xác nhận nghiệp vụ.

Trước: hợp đồng yêu cầu làm nguồn và viewport trong cùng bàn giao, chưa bắt
buộc phân biệt preview rẻ trước exact với Light/High và chưa có danh sách sống
theo lệnh. Sau: hợp đồng quy định rà bốn phần, tái dùng code có sẵn, bổ sung
phần thiếu đúng lệnh được giao và ghi trạng thái có bằng chứng. Chỉ đọc/chỉ
chạy test rõ ràng vẫn giữ scope đó. Lượt skill-only này không sửa runtime,
không tự chạy cả danh mục lệnh và không đóng các gate sản phẩm còn mở.

Quy tắc chuẩn nằm tại
[source-and-viewport-contract.md](../../skills/openmatrix9-workflow/references/source-and-viewport-contract.md).
Sáu entrypoint workflow, feature-port, UI, workspace-contract, build-validation
và headless-testing dẫn vào cùng quy tắc; không nhân bản một hợp đồng đầy đủ
ở sáu nơi. Bản nguồn `skills/` và bản cài Codex đã được đồng bộ đúng bảy file
(sáu SKILL.md và một reference). Không thay invocation policy hoặc cấu hình UI
của skill.

Danh sách duy nhất:
[command-source-viewport-checklist.md](command-source-viewport-checklist.md).
Phần đã có của Union và Sphere được đánh dấu để tái dùng nhưng hai dòng còn
MOT_PHAN. Năm lệnh khác là CHUA_RA theo hợp đồng mới, không phải năm lệnh
không có code. Tổng phần còn lại của toàn ứng dụng chưa được thống kê.

Rà thủ công cách quyết định theo quy tắc (không phải chạy sản phẩm):

- Union có preview và exact song song: tái dùng, kiểm phạm vi mới và giữ lỗi
  High 512 chưa đóng; không tạo lại worker chỉ vì đang kiểm tra lệnh.
- Sphere có nguồn/Light/High và Undo đã đo: giữ phần đạt, truy vết proxy trước
  exact; CHUA_RA không tự chuyển thành THIEU_CAN_BO_SUNG nếu chưa đọc route.
- Lệnh đã chứng minh xong tức thời hoặc không tạo hình: ghi căn cứ hạng mục
  KHONG_AP_DUNG, không thêm thread hình thức hoặc bỏ kiểm nguồn liên quan.
- Yêu cầu chỉ phân tích/chỉ chạy test: ghi thiếu vào danh sách, không triển
  khai ngoài scope; kiểm tra lệnh thông thường bao gồm bổ sung đúng lệnh đó.

Kiểm chứng: `quick_validate.py::validate_skill` qua 12/12 bản nguồn/bản cài
trong Python UTF-8; bảy cặp file khớp byte. Liên kết hợp đồng giữa các skill,
liên kết danh sách/evidence và ID runtime trong các dòng khởi tạo được kiểm.
Validator chỉ kiểm cấu trúc skill, không chứng minh agent tương lai tuân thủ
quy tắc hay thuật toán CAD đã đúng. Không thêm/chạy Rust, native, GPU hoặc
desktop test cho thay đổi hướng dẫn này. Public source audit qua với 16.117
source files, 594 icon bindings, 593 SVG, không lỗi; kiểm diff whitespace qua.
