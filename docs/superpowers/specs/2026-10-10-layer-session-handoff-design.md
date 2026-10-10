# Chuyển phiên làm việc OM9 ↔ Matrix9: layer, bộ màu và khóa

Status: proposed design; chưa triển khai hoặc nghiệm thu.

## Mục tiêu và quyết định của người dùng

Người thiết kế trang sức làm trong một phần mềm rồi chuyển sang phần mềm kia để tiếp tục. Khi chuyển qua 3DM hoặc Copy/Paste, cần giữ hình học hiện tại, tên/cấu trúc layer, bộ màu, khóa/mở và ẩn/hiện. Không cần dịch vụ đồng bộ thời gian thực, lịch sử dựng hình hoặc render.

Các lựa chọn đã chốt: hỗ trợ file và clipboard hai chiều; UI OM9 có gán layer, layer hiện hành, khóa/mở, ẩn/hiện; layer trùng đường dẫn lấy trạng thái nguồn và Undo được toàn bộ; Matrix9 dùng nút/lệnh chuyển riêng, giữ các phím tắt hiện hữu.

Làm trên candidate riêng xuất phát từ Phase3 đã nghiệm thu. Không tự tích hợp vào runtime chính hoặc sửa baseline Phase1–3.

## Hai phạm vi chuyển

1. **Copy phần chọn / Export Selected:** chuyển hình học hiện tại của tập chọn được phép xuất; không tự kéo thêm object chưa chọn. Luôn kèm toàn bộ bảng layer đang sống của nguồn, gồm layer trống, để giữ nguyên bộ màu.
2. **Chuyển cả phiên / Export Session:** chuyển toàn bộ object mô hình trong phạm vi bộ trao đổi hiện hỗ trợ, kể cả object đang khóa/ẩn, cùng toàn bộ bảng layer. Không dùng SelectAll để thu thập vì có thể bỏ sót object khóa/ẩn. Không tự xóa object có sẵn tại đích.

Chuyển phiên là thao tác gửi snapshot, không phải di chuyển/xóa nguồn hay cập nhật hai document liên tục. Dán vào document trống là luồng bàn giao cuối phiên; dán lặp vào document đã có vẫn là thêm object như Paste. Chưa làm cơ chế hợp nhất object theo UUID.

Không được báo thành công đầy đủ khi có object chưa hỗ trợ: preflight nêu loại/số lượng, mặc định dừng cả lần chuyển. Không chuyển xấp xỉ sang mesh hoặc bỏ sót âm thầm. Chế độ preserve-native đã có chỉ tiếp tục dùng theo hợp đồng hiện hành, không hứa object đó chỉnh sửa được trong OM9.

## Hợp đồng dữ liệu

- Layer: UUID nguồn, tên, parent, full path, RGB, locked/visible tại layer và persistent child state theo 3DM/Rhino5. Không đồng nhất layer chỉ vì cùng RGB; không dùng UUID nguồn làm khóa duy nhất ở document đích.
- Object: layer reference, object locked/hidden riêng, RGB riêng và color source ByLayer/ByObject. Effective locked = khóa object hoặc khóa một ancestor; effective visible = object visible và tất cả ancestor visible.
- Khi mở khóa/bật layer cha, khôi phục trạng thái riêng/persistent của con. Mở khóa layer không mở khóa object đã khóa riêng. ByLayer cập nhật màu theo layer; ByObject giữ màu riêng.
- Toàn bộ bộ màu theo bảng layer nguồn được mang qua, kể cả slot trống. 32 slot sidebar là bố cục mặc định; màu tài liệu nguồn thắng màu hardcode. Layer tùy chỉnh/nested phải truy cập được qua danh sách bổ sung; không ép về 32 slot.
- Legacy FCStd chỉ còn OM9Locked đã gộp mà không có provenance đủ tin cậy: migrate true thành khóa object, ghi cảnh báo một lần; không đoán rồi tự mở khóa. OM9Locked trở thành projection tương thích của effective state, không phải canonical field mới.

Match layer theo từng thành phần đường dẫn parent/name, so sánh ordinal không phân biệt hoa/thường để tương thích Rhino5; giữ spelling nguồn. Không match theo tên lá đơn độc. Nếu tài liệu có đường dẫn nhập nhằng sau quy tắc match thì dừng preflight. Không xóa layer chỉ có ở đích. Update RGB/lock/visibility của layer trùng path tác động cả object có sẵn ở đích và nằm trong cùng Undo.

## Layer hiện hành và thao tác UI

Đây là lựa chọn thiết kế OM9, chưa phải bằng chứng khôi phục nguyên xi gesture Matrix9 gốc.

- Click tên/swatch đặt layer hiện hành; arrow gán toàn bộ tập chọn sang layer đó; lock và visibility toggle riêng.
- Không gán object đang khóa hoặc gán vào layer khóa. Batch lẫn object không hợp lệ bị từ chối toàn bộ.
- Khóa/ẩn layer hiện hành: đổi sang layer dùng được trước rồi áp dụng trạng thái. Ưu tiên layer hiện hành từ nguồn khi nhận và layer đó effective unlocked/visible; nếu không, giữ layer đích nếu hợp lệ, rồi layer hợp lệ đầu tiên theo full path. Nếu không có, tạo layer làm việc OM9 Transfer Work với hậu tố tránh trùng, RGB #B4B4B4; không tự mở khóa/bật layer nguồn. Báo layer hiện hành mới.
- Mặc định đề xuất: layer đang khóa nhưng nhìn thấy vẫn làm tham chiếu snap; không được chọn để sửa. Layer ẩn không snap. Đây là giả định thiết kế có thể đổi khi duyệt.
- Guard chung chặn sửa curve/surface/BRep, transform, delete và gán layer qua menu/CMD/tree/property trong các luồng OM9 được hỗ trợ. Tái kiểm tra generation ngay trước commit để chặn editor đã mở trước khi khóa. Không tuyên bố sandbox cho mã Python tùy ý hoặc plugin bên thứ ba.

## Kiến trúc và vận chuyển

Safe Rust sở hữu layer model, validation, merge plan, effective state, index membership và policy. C++ chỉ adapter FreeCAD/Qt/openNURBS. C# chỉ adapter RhinoCommon cho plugin Matrix9/Rhino5; ghi rõ ngoại lệ API native. Python chỉ script kiểm thử/bootstrap.

3DM ghi bằng layer/object fields native, bao gồm parent/persistent state và bảng layer trống. Clipboard tiếp tục có payload hình học Rhino5 native đã được nghiệm thu, thêm format OM9.LayerSession.v1 chứa metadata có version và liên kết kiểm chứng với payload hình học. Không thay payload thành text hoặc đường dẫn file. Wrapper nhận metadata mở rộng để giữ layer trống/bộ màu đầy đủ và thực thi source-wins; clipboard Rhino thuần thiếu format mở rộng vẫn được đọc theo các native facts có sẵn, kèm thông báo nếu không chứng minh đủ bộ màu trống.

Lệnh dự kiến trong Matrix9: OM9Copy3dm, OM9CopySession, OM9Paste3dm, OM9Import3dm, OM9ExportSelected3dm, OM9ExportSession3dm. Kiểm tra collision và kiến trúc Rhino host trước build; không hook Ctrl+C/V. Các lệnh riêng phải cho phép Copy/Export từ tập object chọn sẵn bằng tree/API mà không mở khóa nguồn; chuyển phiên thu thập trực tiếp cả object khóa/ẩn.

Receive có preflight metadata + geometry, apply trong transaction host và một Undo. Không gọi BeginUndoRecord như thể đó là rollback: plugin command phải dùng record của chính command và lưu before-images của layer/object/active layer để phục hồi khi lỗi. Không phát sinh thao tác Undo mù làm mất thao tác người dùng trước đó. Clipboard nguồn được giữ nếu publish mới thất bại.

Toggle layer/màu chỉ đọc metadata/membership; không trích mesh vertices, point cloud points, serialize BRep hoặc remesh. Worker vẫn max(1, floor(0.60 × logical CPU)), giới hạn thêm theo task/RAM; mutation document chạy GUI thread. Log riêng chuẩn bị, hình học, bảng layer, clipboard, commit, redraw và tổng.

## Nghiệm thu

Rust tests + native 3DM tests + FreeCAD runtime + Matrix9 thực sự đã nạp trên Rhino5. Rhino5 sạch chỉ là gate trung gian. Kiểm cả hai chiều, Selected và Session, file và clipboard, layer trống, nested cùng tên lá, own/object lock, ByLayer/ByObject, source-wins trên document có sẵn, Undo/Redo/Cancel, lưu FCStd/reopen và active-layer fallback.

Fixture trang sức: nhẫn/BRep, curve, mesh/cloud nặng; chuyển OM9 → Matrix9, mở khóa một layer, chỉnh geometry, đổi trạng thái, chuyển về OM9 và tiếp tục dựng. So sánh hình học bằng oracle và trạng thái riêng/effective; không chỉ nhìn màu hay đếm object. Replay Phase1–3 trên candidate cuối cùng và ghi source/runtime hashes. Chỉ công bố slice đã có bằng chứng, không gọi là full openNURBS.

## Nguồn đối chiếu

- OM9-LAYER-001: ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-layer-001-layer-management.md. Gesture chưa được normalized đủ; lựa chọn UI trên là thiết kế OM9.
- Baseline Gui/MatrixSidebar.cpp: slot màu tĩnh, lock/visibility chưa hoạt động.
- Gui/ThreeDmArchive.cpp và ThreeDm.py: import hiện gộp layer/object flags, chưa có layer table độc lập xuyên pipeline.
- RhinoCommon.xml cài tại C:/Program Files (x86)/Rhinoceros 5/System: persistent child visibility/locking, current layer phải normal; BeginUndoRecord có thể trả 0 nếu command đã ghi Undo.
