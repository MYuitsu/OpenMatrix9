# Giao diện Rhino/Matrix — hợp đồng chi tiết cho OpenMatrix9

Ngày bổ sung: 2026-10-09. Các hành vi dưới đây được diễn giải từ code đã
decompile; các ca runtime chưa chạy. Phần này trình bày trực tiếp hành vi,
state và điều kiện triển khai, không kèm bảng dẫn file/dòng/hash của code
decompile. Giới hạn dữ liệu được tổng hợp tại [phạm vi đối chiếu](DECOMPILE_AUDIT.md).

## Khả năng FreeCAD và phần UI OM9 cần tích hợp

Đối chiếu này kế thừa mức UI/API của các chương RCORE. Source có thành phần
tương ứng chưa xác nhận bản FreeCAD cài trên máy đã có module/adapter đó.

| Phần UI | Nền tảng FreeCAD đã đối chiếu | Phần OM9 cần bổ sung |
|---|---|---|
| Dock/float panels, menu, focus | Gui/Qt có window, dock/menu và event lifecycle. | Layout schema, panel registry, owner, restore/fallback và trạng thái mở panel Matrix. |
| CMD, lựa chọn option, số/điểm | Host có command dispatch, selection và input/view APIs; Draft có workflow nhập riêng. | Một command session chung UI/CMD, typed options, locale/units, Cancel và parser/frame tương thích. |
| F6 context | Selection/object/subelement APIs là nền cho context. | Metadata gem/Builder, context snapshot, Pin, filter/menu registry, stale revision và dispatch. |
| Shade Mode | Gui/Part có draw styles, view providers và material/display support theo capability. | Mode registry, năm slot, active/all scope, unavailable feedback, preset shader và material policy. |
| Layers | Host có object appearance/visibility và cấu trúc group/layer theo module; chưa tương đương toàn bộ bảng Rhino layer. | Layer identity, hierarchy/effective lock, current layer, semantic roles và panel Matrix. |
| Project Manager | Document persistence/transactions là nền, chưa có bằng chứng codec/workflow Matrix Project trong host. | Versioned project store, assets, rows/slots/workflow, load/save/recovery và UI. |
| View Manager | View/camera APIs là nền để lấy và áp viewport state. | Saved-view store, identity/collision policy, projection/camera validation và atomic I/O. |
| Layout/Detail | TechDraw có pages/views/annotation/print; đây là representation riêng. | Adapter live Detail, page/model units, projection/scale/lock, snapshot/material và resource lifetime. |

## Mở panel và command routing

Các command mở Main, Layers, Snaps, Info & Settings và Project Manager gửi
yêu cầu sang cửa sổ shell Matrix. Giá trị success của wrapper chưa bảo đảm
panel đã hiện: nhánh thiếu window handle có thể không gửi thông điệp mà vẫn
kết thúc không báo lỗi. Phía nhận và dựng shell chưa nằm trong bộ export.

OM9 cần panel registry theo workspace, một instance cho mỗi panel, focus
panel đang mở và các trạng thái unavailable/opening/opened/failed. Mở panel
không tạo geometry. Menu/CMD đi vào cùng command router với capability thật;
alias gốc không tự làm command được kích hoạt. Chín icon History, thứ tự nhóm,
số slot Project và swatch layout vẫn cần UI evidence riêng.

## F6: context từ object và subobject

Brep edge được phân loại như curve, face như surface. Context object còn
đọc metadata ID/SubID, nhận diện gem và lập báo cáo metal/gem. Type/count
đơn thuần chưa đủ để dựng các nhóm hành động nghiệp vụ.

Snapshot OM9 gồm document/generation/revision, object identity, owner và
subelement, geometry type, semantic role, metadata đã resolve và trạng thái
unknown. Rust dựng context; Qt render và dispatch lựa chọn. Khi topology,
selection hoặc document thay đổi, context cũ không được dùng để thực thi.
ViewModel tạo menu chưa được xuất đầy đủ, nên thứ tự/filter của tám mode
chưa được xác nhận thêm bằng code caller.

## F6: refresh, Pin, Escape và dispatch

Mở/refresh đi qua UI dispatcher; nhánh lập context có guard chống re-entry.
Selection đổi khi menu đang hiện làm rebuild context. Watcher dùng timer
một lần 400 ms và bỏ qua nhánh event khi nút trái còn giữ; controller còn có
đường kiểm lại selection bằng timer 750 ms sau mouse-down. Đây là hằng số
local của code đã đọc, chưa phải độ trễ bắt buộc của Rhino hoặc OM9.

Escape đóng cửa sổ chưa Pin. Chọn command cũng đóng cửa sổ chưa Pin trước
dispatch; cửa sổ đã Pin được giữ lại. Close menu và Cancel command có state
riêng. OM9 gộp event, kiểm revision, hủy watcher/timer khi đóng và giữ một
phiên command hợp lệ; không refresh Qt từ worker.

## F6: vị trí cửa sổ và màn hình

Vị trí thường dựa trên cursor với offset (-10,-100); Pin-on-startup dùng
vị trí đã lưu. Owner là main window của host và menu không hiện trên taskbar.
Nhánh clamp đã đọc kiểm mép trên, dưới và trái; chưa chứng minh kiểm mép phải
hoặc mọi trường hợp DPI.

OM9 kiểm cả bốn mép, panel lớn hơn working area, đổi DPI/font và tháo màn
hình. Logical/device coordinates phải chuyển đúng một lần; saved bounds có
schema và fallback còn thao tác được. Đây là yêu cầu bổ sung của OM9.

## Shade Mode: registry và năm nút

Dropdown lấy modes thực đang có của host, lọc Wireframe và một số mode nội
bộ/T-Splines. Wireframe được dùng trong nhánh toggle off. Tên preset trong
spec là vocabulary mục tiêu; availability cần mode ID, backend và options.

Năm nút ban đầu là Plastic, Shaded, Tech Shade, Ghosted và X-Ray. Preference
thay được slot nếu resolve tên hiện còn có. Mode mới ngoài năm mục làm dịch
danh sách và thêm cuối; chọn lại mode đã có giữ thứ tự. Không mô tả cơ chế
này như LRU tổng quát. Click thường áp active view; right-click áp mọi view
host liệt kê. UI hiển thị scope và đồng bộ theo active view.

## Shade Mode: tác động material trên layer chuẩn

Trước đổi mode, helper xử lý 32 định nghĩa layer chuẩn. Nhánh này tìm layer
hiện có theo tên và bỏ qua layer thiếu. Nó có thể tạo material nếu layer
chưa có, bổ sung environment texture nếu chưa tìm thấy, đổi tên material
theo định nghĩa layer, đặt texture decal mode và cập nhật material index.
Material hiện có có thể bị sửa; index dùng chung cần kiểm ảnh hưởng tới
các object/layer khác. Không thấy nhánh này sửa visibility, lock,
current layer hoặc màu layer đã tồn tại. Kết quả native modify và resource
availability chưa được nghiệm thu.

OM9 tách đổi viewport với material initialization. Nếu triển khai nhánh
material, phải công bố scope, kiểm resources, bảo toàn material tùy biến và
có transaction/rollback. Mẫu ViewState chỉ minh họa nhánh đổi viewport;
shape, selection và geometry History giữ nguyên ở nhánh đó. Shader parity
và bitmap thương mại chưa được thay bằng tên mode hoặc helper.

## Panel chuyển sang pick và phục hồi vị trí

Helper có owner host, nhánh modal hide/show và nhánh modeless ẩn cửa sổ,
focus vùng CAD, gọi pick rồi hiện lại. Code đã đọc chưa bảo đảm cleanup nếu
callback ném exception. OM9 giữ session panel→picking→panel; success, Cancel,
exception và document close đều phải kết thúc callback và phục hồi state
window hợp lệ.

Saved bounds đi cùng font family/size theo plugin và loại window. Restore
có thể bị từ chối khi dữ liệu thiếu hoặc font khác; width/height chỉ phục
hồi cho window cho resize, rectangle được đưa về màn hình gần nhất. OM9
thêm version/DPI context và fallback. Window preference không thuộc shape
hoặc geometry transaction.

## Ô số và parser command

Control số đã đọc có value/min/max nullable, increment 0.1, bốn chữ số sau
dấu thập phân, parse CultureInfo và range validation. Text rỗng trong nhánh sync dùng
default hoặc giá trị non-null trước đó. Những defaults này chỉ thuộc control;
không xác nhận tolerance document hoặc defaults mọi Builder.

OM9 giữ riêng raw text, parsed quantity, error và accepted value. Mỗi control
quy định units/range, Enter/focus loss, preview và Cancel. Grammar tọa độ
CPlane/world/relative là contract command riêng; decimal control chưa chứng
minh chấp nhận mọi unit suffix hoặc cú pháp tọa độ. Template/bindings và hành
vi nhập lỗi vẫn cần fixture UI.

## View Manager: action và nguồn camera

Command lấy NamedViews trong saved-view store; Add/Delete luôn có, Load chỉ
thêm khi có mục đã lưu. Save/delete/load nhận tên qua command input. Save
dùng active view. Biến ưu tiên Perspective khác được dùng giữ/khôi phục tên
viewport; không thay active source của thao tác save.

Helper save tạo named view tạm trong document, đưa ViewInfo vào archive,
giữ mapping tên thân thiện→tên viewport và thumbnail 35×35 từ active view,
ghi archive version 5 rồi xóa named view tạm. Đường UI có hỏi overwrite và
xóa entry cũ; đường command truyền script=true bỏ qua nhánh này. Chưa thể
khẳng định script path luôn giải quyết tên trùng hoặc không có duplicate.

UI có Match Name, Fit Match và Current Viewport, mặc định Match Name. Hai
mode đầu thử tên viewport được lưu; Fit Match còn tìm Perspective hoặc
view có orientation class theo trục chính. Không tìm được thì fallback
active viewport. Helper áp ViewInfo và giữ tên target; nhánh tìm được view
trả về trước bước cuối activate/redraw, nên phải nghiệm thu focus/redraw
từng đường. Load không luôn áp vào active viewport.

Load/list helper còn ghi archive và gọi synchronize để ghi lại; đó chưa
phải thao tác read-only trong code gốc. Delete tên không tồn tại có thể
kết thúc im lặng. Command không kiểm mọi kết quả write/apply và vẫn có thể
trả success. OM9 quy định read/list, Save/Delete, collision, camera target,
cleanup và Cancel rõ; atomic I/O và error feedback dựa vào kết quả thực.

## Layout: live Detail và snapshot

Form khởi tạo Detail bật. Chọn thumbnail chỉ xử lý trong page view. Một
selected PageSpace Brep với Detail tắt đi vào nhánh thay ảnh; một selected
PageSpace Detail với Detail bật đi vào nhánh copy projection/commit viewport.
Selection khác có thể dẫn tới thêm item mới.

Trong handler thumbnail, nhánh ảnh capture 1600×1200 pixel và nhúng bitmap/
material trên PageSpace surface. Chỉ nhánh thêm ảnh mới, khi resolve được
source view và object, ghi metadata camera/projection/CPlane/lens/viewport
size; thay material ảnh đang chọn không làm mới metadata đó. Nhánh live
Detail tạo rectangle ban đầu 100×80 page units rồi copy projection nguồn
và commit. Đây là giá trị của handler thumbnail, không phải paper size,
scale hoặc resolution mọi thao tác layout.

Change Display Modes là handler khác, xử lý nhiều item và có thể chuyển
Detail↔ảnh. Placement dùng bounds của item, capture dựa vào viewport size
và tăng kích thước ở nhánh V-Ray; không dùng cố định 1600×1200/100×80.

| Item hiện tại | Detail/mode | Nhánh Change Display Modes đã đọc |
|---|---|---|
| Detail | Detail bật, mode khác V-Ray | Đổi display mode trên Detail. |
| Detail | Detail tắt hoặc mode V-Ray | Tạo ảnh thay thế rồi xóa Detail cũ. |
| Brep ảnh có camera metadata | Detail bật | Tạo live Detail từ metadata/bounds rồi xóa ảnh cũ; nhánh này không có cùng điều kiện loại V-Ray như nhánh Detail. |
| Brep ảnh có camera metadata | Detail tắt | Capture lại ảnh theo camera đã lưu. |

Thumbnail với Detail bật vẫn có thể tạo/cập nhật live Detail khi tên mode
V-Ray được chọn, qua resolve mode hoặc fallback Shaded. Vì vậy OM9 phải
công bố policy renderer/Detail nhất quán theo capability; chưa được suy
rằng mọi chọn V-Ray đều tạo ảnh. Conversion tạo object ID mới rồi xóa ID cũ;
identity mapping, dependencies và Undo là contract cần kiểm riêng.

Numeric scale paper:model chỉ dùng cho Detail parallel; perspective trả
ratio 0 và từ chối thao tác SetScale ở wrapper. Projection, camera lock,
vị trí item và scale là các trạng thái riêng. OM9 resolve nguồn/resource
trước mutation, rollback lỗi/Cancel và lưu đúng live/snapshot type.

## Các ca UI cần nghiệm thu — chưa chạy

| ID | Thao tác | Điều kiện cần giữ |
|---|---|---|
| DUI-T01 | Mở Main/Layers khi chưa có panel adapter. | Báo unavailable/failed; không success giả hoặc tạo geometry. |
| DUI-T02 | F6 với edge, face, gem và selection hỗn hợp. | Owner/subelement/revision và semantic roles hợp lệ; metadata thiếu rõ. |
| DUI-T03 | F6 Pin on/off, Escape và chọn command. | Close/menu dispatch đúng; không tạo hai phiên command hoặc mất selection ngầm. |
| DUI-T04 | Kéo selection liên tục rồi đóng document. | Event gộp, callback cũ bị bỏ, watcher/timer được hủy và Qt chỉ cập nhật trên UI thread. |
| DUI-T05 | F6 ở bốn góc, DPI khác và tháo màn hình. | Panel còn thao tác được, Pin restore/fallback đúng và kiểm cả mép phải. |
| DUI-T06 | Click shade trái/phải, toggle và material/resource thiếu. | Active/all scope đúng; shape giữ nguyên; material action explicit và lỗi không partial mutation. |
| DUI-T07 | Chọn A..E, chọn lại B rồi thêm F. | Policy năm slot đúng, registry không có mode giả và availability rõ. |
| DUI-T08 | Panel pick success/Cancel/exception/document close. | Owner/focus/visibility hợp lệ; callback và preview không sống quá session. |
| DUI-T09 | Ô số khác locale, text rỗng, vượt range và focus loss. | Raw text/accepted/error tách biệt, units đúng và số chữ số không thay tolerance. |
| DUI-T10 | Store view rỗng/có tên, save orthographic active, load/cancel. | Load availability đúng, capture active view đúng, camera/store theo policy và lỗi apply/I/O rõ. |
| DUI-T11 | Layout thumbnail và Change Display Modes với Detail on/off, V-Ray, selection một/nhiều item. | Branch/type đúng policy, metadata stale rõ, conversion identity và rollback/Undo được kiểm; resolution không suy từ handler khác. |

Các ca này bổ sung nghiệm thu feature và RCORE; đọc code chưa xác nhận pass.
