# Phạm vi bổ sung specs từ code Rhino/Matrix

Ngày đối chiếu: 2026-10-09. Đã khảo sát bộ export người dùng cung cấp tại
`L:/rhino5` và đọc có chọn lọc các đường code phục vụ nền CAD và giao diện.
Đã bổ sung cả 12 chương RCORE, tám feature UI và [hợp đồng UI chi tiết](UI_DETAILS.md).
Các phần decompile trình bày hành vi trực tiếp, không kèm bảng tham chiếu
file/dòng/hash theo yêu cầu người dùng.

## Bộ dữ liệu và mức xác nhận

Bộ export sau lần xuất lại có 3.530 file, gồm 3.394 C# files, năm project dnSpy,
bốn file native C export và header bổ sung cho rhcommon_c. Năm nhóm managed là RhinoCommon, Rhino_DotNet, RhinoWindows,
Rhino.WindowsAPI và plugin Matrix. Số file được kiểm kê không đồng nghĩa
mọi thuật toán trong tất cả file đã được đọc hoặc phục dựng hoàn chỉnh.

| Nhóm code | Đã dùng để làm rõ | Giới hạn |
|---|---|---|
| Managed implementation | UI events, guards, lựa chọn, defaults local, thứ tự thao tác, ownership/cache và kết quả trả về. | Gọi native hoặc dependency bên ngoài chưa xác nhận thuật toán bên trong hay kết quả runtime. |
| Chú thích SDK trong export | Semantics overload, option, precondition và scope được API mô tả. | Chú thích là hợp đồng API; không nâng thành nhánh runtime đã được kiểm chứng. |
| Rhino native pseudocode | Một số nhánh History, Rebuild/Fit và behavior có symbol rõ. | Decompiler có warnings; type/offset/call mapping cần kiểm thêm, nhiều nhánh chỉ là giả thuyết có giới hạn. |
| openNURBS native pseudocode | Serializer settings, units, layers/materials và HistoryRecord. | Archive/representation support chưa cung cấp toàn bộ modeling kernel hoặc replay engine Rhino. |
| RDK bridge pseudocode | Một số bridge scene/content/ground-plane và boundary. | Bridge chuyển tiếp sang RDK; chưa chứng minh renderer, shader hoặc ảnh tương đương. |
| Native bridge rhcommon_c xuất lại | Null/range guards, thứ tự tham số Rebuild/Fit, validity selectors và một số conversion/ownership paths. | Virtual calls và reconstruction của decompiler vẫn cần đối chiếu; thân bridge chưa xác nhận kết quả solver khi chạy. |

RhinoCommon có assembly version 5.1.30000.17 và file version 5.14.00522.08390.
Matrix assembly metadata ghi DotMatrix 1.0.0.0; dấu vết cấu hình/debug chứa
nhiều nhãn phiên bản. Chúng chưa đủ chứng minh mọi binary thuộc cùng một
build Matrix. Chưa có hash binary gốc hoặc cấu hình export để đối chiếu với
runtime; hash export chỉ nhận diện file văn bản đã đọc.

## Các chỉnh sửa hành vi quan trọng

- Ortho/Planar dựa vào base point của phiên input; không luôn là điểm accepted trước đó.
- Unit-system metadata và thao tác scale geometry phải được phân biệt; model/page units có scope riêng.
- Curve Rebuild có managed clamp riêng; Surface Rebuild không mặc nhiên có cùng clamp.
- History record/version/ObjRef và object metadata có thể cùng cần thiết cho replay Builder.
- Nhánh History Update Off đã đọc xóa pending queue; OM9 giữ dirty là policy riêng cần nghiệm thu.
- Numeric scale của Detail chỉ áp dụng projection parallel; perspective không có tỷ lệ kỹ thuật đó.
- F6 cần context subobject, semantic metadata, Pin, dispatcher và stale-selection handling.
- Shade Mode có năm slot, active/all scope và helper material có thể sửa material/layer links.
- Layout ảnh là bitmap/material snapshot; metadata camera không biến ảnh thành live Detail.
- View Manager phải phân biệt active source khi save, store mutation và apply camera.

Những yêu cầu như rollback nguyên tử, cleanup khi exception, kiểm đủ bốn mép
màn hình, schema migration và material scope là contract triển khai OM9 khi
code gốc chưa chứng minh bảo đảm đó. Không trình bày chúng như behavior Rhino
đã phục dựng hoàn chỉnh.

## Phần còn thiếu để tăng độ chính xác

Sau khi xuất lại, rhcommon_c.dll.c có 3.603.082 byte và 123.145 dòng LF;
header đi kèm có 545.387 byte và 9.160 dòng LF. Đã đọc được thân hàm Rebuild,
Fit, validity và các bridge khác. Ghi chú trước đây về bản 330 dòng chỉ có
kiểu/PE được thay bằng kết quả này. File lớn hơn chưa chứng minh mọi export
đều decompile thành công hoặc mọi virtual call đã có kiểu chính xác.

Kiểm kê cấu trúc thấy 2.763 tên export có annotation gắn với khối thân hàm,
cùng 6.236 warning comments. Đây là xác nhận độ phủ văn bản, không phải
chứng nhận mọi function body được phục hồi đúng. Đã làm rõ thứ tự tham số
Rebuild/Fit, native normalization riêng curve/surface, unit-scale dispatch,
validity selectors, face-index guards và các boundary CPlane/GetPoint/GetObject.

| Export/implementation còn cần | Phần bị giới hạn |
|---|---|
| MatrixWPFControls | ViewModel/renderer/filter của F6 và custom WPF UI. |
| MatrixControls | Bindings và behavior của custom widgets. |
| MatrixCommon | Settings/helpers và các semantics phụ thuộc lớp chung. |
| MatrixTranslation | Localization, option text và mnemonic mapping. |
| PIMData | Project data/codec và workflow storage. |
| Receiver/cửa sổ shell Matrix | Layout Main/Layers/Snaps/Info/Project và phía xử lý yêu cầu mở panel. |

Các dependency trên chưa có export tương ứng trong bộ đã cung cấp. Điều
này mô tả độ phủ dữ liệu hiện tại, không xác nhận chúng thiếu chức năng ở
ứng dụng gốc hoặc chưa được cài trên máy người dùng.

## Phạm vi nghiệm thu

Lần này cập nhật tài liệu và hợp đồng. Fixture native, UI runtime, geometry,
Rhino 3DM roundtrip, renderer và build integration có trạng thái `not_run`.
Capability FreeCAD và trạng thái native 607 feature không được nâng chỉ vì
đã đọc decompile. Mẫu Rust giữ nguyên; spec mô tả supported slice cần viết
adapter/solver và kiểm chứng trước khi bật command.

Kết quả kiểm cấu trúc, tính nhất quán catalog/manifest và source audit được
ghi tại [GUIDANCE_VALIDATION](../../GUIDANCE_VALIDATION.md). Bảng capability
và số fixture hiện hành nằm trong [README nhóm 00](README.md).

SOURCE_BASELINE giữ snapshot của lần đối chiếu FreeCAD trước đó. Sáu file
OM9 được snapshot này dẫn tới đã đổi sau snapshot; source lõi FreeCAD trong
bản ghi vẫn khớp. Lần bổ sung decompile giữ nguyên phân loại capability và
không tái xác nhận native runtime từ những chỉnh sửa OM9 khác. Khi triển khai
feature, cần đọc lại source/implementation record hiện hành trước khi đóng
capability; baseline không chứng nhận working tree hoặc binary mới nhất.

## Vị trí nguồn riêng sau khi tách khỏi workspace

Thư mục code tham chiếu Matrix và ba PDF gốc đã được bảo toàn ngoài
FreeCAD-src sau khi đối chiếu SHA256; các tên nguồn trong ghi nhận lịch sử
được giải quyết qua registry cục bộ theo
[quy tắc nguồn riêng](../../../docs/private-reference-policy.md). Bộ Rhino
ở ngoài workspace từ trước vẫn được giữ nguyên. Chưa diễn giải toàn bộ
nguồn; kho riêng còn đầy đủ để đọc các phần cần thiết sau này.

Audit sau khi tách nguồn trả về exit code 0 với không có lỗi. Các con số
290 ở bản ghi trước là kết quả lịch sử, không phải trạng thái audit sau
thao tác này. Kiểm thử native và hình học vẫn chưa chạy.
