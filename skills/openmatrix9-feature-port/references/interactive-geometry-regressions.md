# Hướng kéo, nhập điểm và preview của lệnh tương tác

Người dùng duyệt lưu ngày 2026-10-26 sau khi kiểm tra lỗi Rectangle. Áp dụng khi implement/port/sửa lệnh nhận điểm, hình học có hướng hoặc đường nhập điểm/preview dùng chung; chọn các case phù hợp với exact owner. Không đổi default hay semantics của command khác theo ví dụ Rectangle.

## Giữ dấu trong hệ tọa độ dựng hình

- Top 11 Move: tùy chọn `Vertical` khi bật chiếu độ dời có dấu lên pháp tuyến của CPlane hiện hành; điểm nhập CMD và điểm chọn đi qua cùng handler Move. Người dùng duyệt lưu quy tắc ngày 2026-10-28; chứng cứ và giới hạn ở `docs/validation/2026-10-28-top11-move-vertical.md`.
- Top 11: cho phép bắt đầu lệnh trước khi chọn đối tượng; Rotate có `Copy` và nhận góc số hoặc các điểm tham chiếu; Mirror X/Y theo CPlane hiện hành. Join hai BRep dùng dung sai tài liệu và ghi kết quả trong một giao dịch Undo. Người dùng duyệt lưu ngày 2026-10-28; phạm vi đã kiểm và các giới hạn ở `docs/validation/2026-10-28-top11-selection-transforms-join.md`.
- Top 11: Duplicate có `InPlace` tại prompt; cho phép bỏ đối tượng hoặc cutter đã chọn; Split Point trên đường thẳng dùng dung sai tài liệu; Join nhiều BRep chỉ commit sau khi nối được toàn bộ. Người dùng duyệt lưu ngày 2026-10-28; phạm vi kiểm và giới hạn ở `docs/validation/2026-10-28-top11-split-join-followup.md`.
- Top 11 Split nhiều curve bằng một hoặc nhiều cutter: giữ nguyên curve không giao và ghi toàn bộ kết quả trong một giao dịch Undo. Người dùng duyệt lưu ngày 2026-10-28; chứng cứ và phạm vi hiện hỗ trợ ở `docs/validation/2026-10-28-top11-split-multiple.md`.

- Tách **độ dài** khỏi **hướng**. Giữ các thành phần có dấu của độ lệch trong CPlane/frame đang dùng; chỉ lấy trị tuyệt đối cho đại lượng cần không âm. Không dùng `.abs()` rồi dựng lại theo hai trục dương làm mất phía con trỏ. Không thay bằng phép xét dấu world-X/Y khi CPlane xoay.
- Rectangle Corners mặc định, không Square/Length hay ràng buộc khác: hai điểm được chọn là hai góc đối diện của hình. Điểm thứ hai chiếu lên mặt phẳng dựng phải là một đỉnh của cả preview và hình canonical sau commit. Dấu của Width nhập số, Center, 3Point, Vertical và các ràng buộc theo exact owner; không suy ra quy tắc mới từ Corners.
- Preview và commit dùng chung phép dựng/validation hình học có hướng do Rust owner quản lý. UI chỉ chuyển input và vẽ kết quả; không duy trì công thức dựng khác cho preview để tạo ra hai hành vi lệch nhau.

## Điểm nhập qua CMD phải tới cùng owner

- Phân loại option, giá trị đang được hỏi và tọa độ trước khi từ chối input. Khi stage nhận điểm, tọa độ hợp lệ đi qua parser tọa độ/units/frame chung rồi cùng đường nhận điểm như mouse/snaps. Không để handler tham số trả lỗi “needs a first corner” trước khi parser có cơ hội đọc tọa độ góc đầu.
- Nếu đang nhập value của một option, giữ quyền sở hữu của stage đó; không diễn giải lại thành điểm tự do. Input sai giữ trạng thái hợp lệ để sửa; không tạo entity hoặc transaction một phần.
- Kiểm tra gọi `Rectangle` qua CMD và nhập **tọa độ** là hai việc khác nhau. Gọi CMD rồi dùng `accept_world_point` chỉ chứng minh invocation + world picks, chưa chứng minh parser tọa độ CMD.

## Oracle để bắt lỗi tương tự trước nghiệm thu

- Với Corners có thể kéo hai phía trên hai trục, kiểm đủ bốn tổ hợp dấu `(+,+),(-,+),(+,-),(-,-)` từ một điểm đầu khác gốc. Đối chiếu đỉnh thật hoặc bounds có dấu với oracle độc lập; số cạnh, độ dài, diện tích hay kích thước bbox đơn thuần không chứng minh vị trí đúng. Khi đổi phép dựng/frame, thêm CPlane xoay và các mode/ràng buộc bị ảnh hưởng theo semantics đã được xác minh.
- Fixture Rectangle cơ sở: A=`(3,7,0)`, B lần lượt `(13,27,0)`, `(-7,27,0)`, `(13,-13,0)`, `(-7,-13,0)`. Mỗi kết quả Corners phải chứa A và B; không chấp nhận cả bốn lần đều có góc đối diện `(13,27,0)`.
- Với mỗi route được tuyên bố hỗ trợ, kiểm preview trước commit, hình học canonical sau commit và không có entity thừa. Kiểm tọa độ CMD ở điểm đầu và các stage nhận điểm tiếp theo, input sai/retry, Cancel và Undo theo phạm vi thay đổi. Menu/mouse/CMD cho cùng biến thể dùng một implementation; dispatcher test và click UI được ghi thành hai loại bằng chứng riêng.
- Lỗi đã tái hiện phải có regression thực sự đỏ trước sửa và xanh sau sửa; giữ regression phù hợp trong suite thường trực khi sửa sản phẩm. Không bỏ/ignore test lỗi hoặc coi baseline xanh là đủ khi baseline thiếu hướng âm. Với nhiệm vụ chỉ chẩn đoán/skill, lưu oracle và trạng thái FAIL, không tuyên bố đã sửa code.
- Nếu khung biến mất khi chuyển preview → scene, kiểm model/entity, scene visibility/epoch và redraw/present của executable có fingerprint. Model/scene đúng không chứng minh GPU đã hiển thị; video chớp hình không đủ để kết luận entity bị xóa. Giữ triệu chứng redraw chưa rõ nguyên nhân ở trạng thái chưa xác minh.

## Bằng chứng của lỗi thúc đẩy quy tắc

Resolve checkout OpenMatrix9 rồi xem `docs/validation/2026-10-26-rectangle-diagnosis/README.md` và `results.json`: 16 test cũ qua; batch chẩn đoán có 8 case hướng kéo, 6 sai, cùng một fixture CMD từ chối điểm đầu. Đây là bằng chứng lỗi tồn tại lúc đo, không phải kết quả sửa. Cập nhật skill chỉ bổ sung quy tắc triển khai/kiểm chứng; không đóng lỗi Rectangle hoặc manual gate.
