# Kiểm tra thủ công Project Manager — chưa nghiệm thu

Bản kiểm tra dùng release `c14055798c269947f5302be33265c522c499a14ec2ab5e71451a1ab0a1d25e1a`, một bản sao Box Native BRep và catalog/layout riêng trong `build/standalone-validation/ui-artifacts/2026-11-18-projects-offscreen/manual-project-manager-v7`. Binary này chưa có sửa đổi câu thông báo khi v5 từ chối BRep được thực hiện sau khi build; hành vi codec không đổi. Không dùng cửa sổ debug hoặc catalog đang làm việc. Lượt kiểm tra này do người dùng thực hiện; chưa có kết quả hay chữ ký nghiệm thu.

Mở PowerShell và chạy lệnh sau. Cửa sổ OM9 release là cửa sổ tương tác riêng; chạy lại cùng lệnh sau khi đóng để kiểm tra catalog qua khởi động lại. Launcher từ chối nếu binary release đã đổi.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File 'H:\OpenMatrix9\build\standalone-validation\ui-artifacts\2026-11-18-projects-offscreen\manual-project-manager-v7\launch-review.ps1'
```

Đối chiếu trên màn hình thật, ghi lại **đạt / lỗi / chưa thử**, phiên bản Windows, kích thước cửa sổ và tỉ lệ hiển thị thực tế. Nếu có lỗi, ghi thao tác trước lỗi và nội dung thông báo; không đánh dấu đạt chỉ vì ứng dụng mở được.

1. Quan sát Projects: danh sách project bên trái, các ô đánh số phía trên, nút Capture/Save/X, thanh cuộn ngang, nút thu gọn/mở rộng và năm nhóm Master, Creation, Parts, Render, Output phía dưới. Thử kéo thanh cuộn, thu gọn/mở rộng, đổi kích thước cửa sổ và đọc tooltip; xác nhận các nút không đè nhau hoặc bị cắt.
2. Chọn Box, nhấn nút có tooltip **Capture selected objects** của ô đầu tiên, đợi thumbnail; xóa Box khỏi bản sao document, nhấn ô đó để chèn lại. Kiểm tra hình xuất hiện đúng vị trí; Undo xóa lần chèn và Redo khôi phục. Lưu bản sao document và mở lại để kiểm tra, sau đó đóng ứng dụng và mở lại bằng cùng launcher để xác nhận catalog còn bag.
3. Với từng nhóm Master/Creation/Parts/Render/Output, dùng nút có tooltip **Capture the whole document into this version**, thử nút thêm version và mở snapshot. Khi document có thay đổi chưa lưu, kiểm tra hộp thoại Save/Discard/Cancel; Cancel giữ nguyên document. Không cần ghi đè tài liệu bên ngoài thư mục kiểm tra.
4. Nhấn Mngr, chọn project, mở Info và Files, kiểm tra preview và thumbnail; thử Search/Reset, Show All và filter đại diện. Kiểm tra thay đổi Info có Save/Cancel, tìm lại project sau khi mở lại. Thử các nút import/export bằng tệp thử trong thư mục kiểm tra và kiểm tra Cancel không thay dữ liệu.
5. Lặp kiểm tra bố cục và các nút chính tại tỉ lệ hiển thị Windows 100%, 125% và 150% nếu các mức đó khả dụng; ghi rõ mức chưa thử. Kiểm tra cửa sổ nhỏ và floating/docking nếu môi trường hỗ trợ.

## Change root với cấu hình riêng

Đóng cửa sổ kiểm tra ở trên, rồi chạy launcher thứ hai. Launcher này đặt `APPDATA` riêng cho tiến trình thử và không đặt `OM9_PROJECT_CATALOG`, để nút Change hoạt động mà không dùng cài đặt catalog thông thường.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File 'H:\OpenMatrix9\build\standalone-validation\ui-artifacts\2026-11-18-projects-offscreen\manual-project-manager-v7\root-review\launch-root-review.ps1'
```

Trong cửa sổ thử, tạo một project hoặc Capture một bag. Mở Mngr, nhấn **Change…** và tại hộp chọn thư mục chỉ chọn thư mục thử đã chuẩn bị:

```text
H:\OpenMatrix9\build\standalone-validation\ui-artifacts\2026-11-18-projects-offscreen\manual-project-manager-v7\root-review\destination
```

Kiểm tra project và bag vẫn hiện sau chuyển; đóng và mở lại bằng launcher thứ hai để kiểm tra lại. Có thể thử Cancel trên hộp chọn trước khi chuyển và xác nhận project vẫn còn. Ghi kết quả; không chọn thư mục project đang làm việc. Đây là một lượt tương tác người dùng, không chứng minh đầy đủ các trường hợp cạnh tranh hoặc lỗi hệ thống tệp.

Đây là checklist có hướng dẫn, chưa phải bằng chứng rằng mọi trường hợp hình học, worker, filter hay hộp thoại native đã đạt. Không thay đổi hoặc ký manual M01-UI khi chưa có kết quả thực tế.
