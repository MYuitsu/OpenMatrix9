# Command: icon, thao tác mặc định và tham số

Người dùng yêu cầu lưu quy tắc ngày 2026-10-26: khi implement command phải đồng thời map từ icon, thực hiện đúng thao tác mặc định và có tham số gạch chân phím tắt như Polyline. Áp dụng cho standalone OM9; đây là quy tắc triển khai/kiểm chứng của OM9, không tự chứng nhận hành vi Matrix gốc hoặc ký human UI gate.

## Phạm vi

Khi triển khai, port, sửa hoặc tiếp tục một command, coi **icon/binding → thao tác mặc định → tham số/input → kết quả thật** là cùng một luồng cần hoàn thiện. Không đánh dấu command hoàn tất chỉ vì có handler, dựng được geometry qua API, icon hiện ra hoặc menu mở được.

Task chỉ làm artwork/icon, tài liệu hoặc một phần đã được giao rõ vẫn giữ phạm vi đó; quy tắc này không tự cho phép triển khai kernel hay toàn bộ lệnh khác. Báo đúng phần đã làm và phần tích hợp còn thiếu. Command không có tham số hoặc dùng protocol chuyên biệt không phải nhận tham số giả để đủ checklist.

## 1. Map từ icon tới đúng implementation

- Xác định authored icon/asset key, vị trí menu/toolbar/sidebar/F6 liên quan, label/tooltip, exact feature/command ID và biến thể. Đọc binding thật trong catalog/dispatcher và reference milestone; không suy command chỉ từ hình dáng icon.
- Icon, menu, CMD và các entry point được hỗ trợ phải gọi cùng owner/dispatcher cho **cùng biến thể**, không có geometry hoặc parameter state riêng trong widget.
- Nếu icon có hành vi click trái/phải, dropdown hoặc modifier khác nhau theo contract, map và ghi rõ từng binding. Không tự đặt quy tắc click phải/double-click cho mọi icon; biến thể chưa được định nghĩa ghi “không áp dụng/chưa xác minh”.
- Giữ original IDs/names và license/authored-asset policy. Unsupported command vẫn visible-disabled với reason; không nối icon sang command gần giống chỉ để click có phản ứng.

## 2. Thao tác mặc định phải chạy đúng khi gọi lệnh

- Ghi mode ban đầu, giá trị mặc định, selection/context/unit/CPlane cần thiết, prompt đầu tiên và chuỗi input/preview/finish/cancel/Undo của command hoặc biến thể đó.
- Click icon phải khởi động đúng mode/action mặc định đã được contract quy định; không chỉ đổi selected icon hoặc mở một panel không thực thi. CMD/menu gọi cùng biến thể phải có kết quả tương đương.
- Biến thể chủ ý có default khác nhau được giữ riêng, ví dụ binding gọi một construction mode cụ thể. Không ép tất cả binding về một default để làm test giống nhau.
- Lấy defaults từ exact milestone/owner đã đối chiếu, không đoán theo Rhino/Matrix chung hoặc sao chép constructor của command khác. Default chưa rõ phải được ghi unknown và giải quyết trước khi tuyên bố command hoàn tất.

## 3. Tham số và phím tắt theo mẫu tương tác Polyline

- Dùng contract chung `CommandOptionInterface`/active command input của OM9; owner sở hữu values, availability và input stage. Không thêm bảng phím tắt/parser hoặc mutable parameter cache riêng trong UI cho từng command.
- Prompt hiển thị các tham số **của command đang chạy**, đúng stage và current value; tham số có accelerator phải gạch chân đúng chữ được gán. Dùng cùng snapshot cho label, underline và click target; không dò substring mơ hồ như Close trong PersistentClose.
- Nhấn accelerator, click token và nhập full name/assignment được hỗ trợ đều đi qua cùng owner. Toggle đổi đúng một lần; Choice chọn/cycle theo semantics của owner; Number/Integer/Point/Vector/Text mở bước nhập đúng tham số, validate rồi cập nhật value/preview hoặc commit khi owner đủ input. Không submit `Name=` thiếu value rồi coi lỗi parser là luồng nhập bình thường.
- Shortcut phải duy nhất trong stage hoặc không quảng bá chữ gây xung đột. Giữ numeric/coordinate input, IME/paste, nonempty draft, popup và focus của editor khác; giữ phím không toggle liên tục. Giải xung đột với snap keys theo owner hợp lệ.
- Không lấy repeat slot/lệnh trước làm active owner. Chuyển command/run phải bỏ option/hit target/input state cũ. Ví dụ: ở Start of Polyline không được xuất hiện BothSides của Line; Length của Polyline chỉ xuất hiện sau first point trong Line mode.
- Giữ semantics/units/defaults riêng của command. Polyline là mẫu tương tác, không phải danh sách option để sao chép. Direction=U/V khác vector Direction; set choice khác toggle; âm/0 và commit timing theo exact owner.
- Invalid input giữ stage/draft và state hợp lệ để sửa; Esc/pending Undo/document Undo theo contract; không tạo transaction chỉ vì đổi option. Giữ một protected/copyable transcript với live input pinned, không thêm editor hay hàng button lặp lại ngoài luồng prompt chuẩn.

## 4. Bằng chứng tối thiểu để gọi command đã tích hợp

Với command nhận điểm/hình học có hướng hoặc thay đổi parser/preview, đọc thêm [oracle hướng kéo, nhập điểm và preview](interactive-geometry-regressions.md), người dùng duyệt 2026-10-26. Kiểm tọa độ CMD thực và vị trí hình học theo các dấu/frame liên quan trước khi kết luận các routes tương đương.

Trong record của command, ghi đủ:

| Mục | Bằng chứng cần có |
|---|---|
| Icon/binding | Asset key, entry point, feature ID, trigger và biến thể thực tế |
| Default action | Selection/preconditions, initial mode/values, first prompt và expected input sequence |
| Options | Parameter/type/current value, stage availability, accelerator gạch chân và owner xử lý |
| Routes | Icon/menu/CMD cho cùng biến thể; keyboard/click/typed parameter cho cùng kết quả |
| Kết quả | Actual geometry hoặc effect đúng loại; preview/commit/cancel/Undo phù hợp; invalid/retry không phá model |
| Runtime | Đúng executable path/PID/hash/profile; screenshot/click evidence của bản đã build, không nhầm release cũ với debug mới |
| Giới hạn | Unsupported/unknown, test baseline hỏng, fixture ignore và human gates chưa ký ghi riêng |

Kiểm cả default lẫn các parameter/mode được tuyên bố hỗ trợ. Một Circle Center hoặc một toggle thành công không chứng minh tất cả tangent/multi-point modes. Nếu thiếu một trục icon/default/options, ghi partial/bridge/missing integration cụ thể, không nâng toàn command thành done.

Đồng bộ technical evidence/ledger và milestone theo scope đã kiểm. Người dùng đã yêu cầu lưu **chính quy tắc này**, không hỏi consent lại cho việc ghi nó; thay đổi nghiệp vụ khác vẫn theo [quy tắc cập nhật nghiệp vụ](../../openmatrix9-workflow/references/business-rule-updates.md).

## Hướng dẫn sâu khi chuẩn hóa nhiều command

Resolve checkout rồi đọc gói repo-relative `implementation/design/M01/command-options-polyline-standard-20261026/README.md`. SPEC, owner cards, task packages và acceptance templates giúp chia việc theo family. Đó là đặc tả/historical source inventory, không phải implementation hoặc số lượng command đã được nghiệm thu; đối chiếu code hiện tại trước khi dùng các counts/API đề xuất.

Yêu cầu lưu skill này phê duyệt quy tắc tích hợp icon/default/options; không mặc nhiên phê duyệt mọi API, mode mới hoặc thay đổi geometry được đề xuất trong gói đặc tả.
