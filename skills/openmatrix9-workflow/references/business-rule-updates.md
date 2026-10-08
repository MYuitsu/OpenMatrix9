# Xác nhận cập nhật nghiệp vụ vào skill

Quy tắc người dùng yêu cầu ngày 2026-10-05: **Sau khi hoàn tất code và kiểm tra, nếu có đổi nghiệp vụ, hỏi người dùng có cập nhật nghiệp vụ đó vào skill không.**

Áp dụng cho các skill OpenMatrix9, kể cả khi chỉ một skill được dùng trong phiên làm việc.

- **Đổi nghiệp vụ** gồm thay đổi quy trình sử dụng, hành vi lệnh/chuột/phím, điều kiện đầu vào, tùy chọn hoặc giá trị mặc định, kết quả hình học, preview/commit/cancel, Undo/history hay quy tắc UI ảnh hưởng cách người dùng thao tác.
- Refactor, sửa build, tối ưu hoặc sửa lỗi để khôi phục hành vi đã thống nhất không tạo ra nghiệp vụ mới. Không hỏi cập nhật nghiệp vụ chỉ vì có thay đổi code.
- Hoàn thành phần code đã được giao, kiểm tra phù hợp và trình bày kết quả trước khi hỏi. Quy tắc này chỉ kiểm soát việc ghi nghiệp vụ vào skill, không yêu cầu xin lại phép để thực hiện công việc đã được giao.
- Chuẩn bị đề xuất cụ thể: hành vi trước/sau, bằng chứng kiểm tra, nội dung định ghi và skill liên quan. Gộp các thay đổi nghiệp vụ của cùng công việc vào một lần hỏi ở bước cuối. Nêu nguồn quy tắc và dẫn tới SKILL.md đang áp dụng khi hỏi.
- Chỉ sửa nội dung nghiệp vụ của skill khi người dùng đồng ý. Chưa trả lời không phải đồng ý; nếu chưa đồng ý hoặc từ chối, giữ nguyên quy tắc nghiệp vụ trong cả bản nguồn lẫn bản cài đặt. Vẫn ghi kết quả kỹ thuật và trạng thái đề xuất đang chờ/từ chối trong tài liệu tiến độ phù hợp.
- Nếu người dùng đã yêu cầu rõ việc cập nhật skill cho chính thay đổi đó, thực hiện trong phạm vi đã được cho phép, không hỏi lại. Đồng ý đổi hành vi trong code không tự động đồng nghĩa với đồng ý ghi thành quy tắc skill.
- Khi được đồng ý, cập nhật skill đúng nhóm nghiệp vụ, đồng bộ bản nguồn/bản cài đặt và xác thực. Không sửa các skill không liên quan hoặc coi lựa chọn riêng của OM9 là hành vi Matrix gốc đã được chứng minh.

Ví dụ câu hỏi sau khi đã code và kiểm tra xong: “Tôi đã đổi [hành vi trước] thành [hành vi sau], kiểm tra [kết quả]. Tôi đề xuất ghi [nội dung cụ thể] vào skill [tên/link]. Bạn có muốn cập nhật nghiệp vụ này vào skill không?”
