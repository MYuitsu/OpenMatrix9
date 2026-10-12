# Dùng Gumball trong bốn ô nhìn

Chọn object, gõ **`Gumball On`** ở Command rồi nhấn Enter. Gumball xuất hiện
đồng thời ở cả bốn ô. Bạn có thể thao tác ngay ở bất kỳ ô nào.

Muốn thao tác cả cụm, giữ **Shift** và click thêm object, hoặc kéo khung chọn
quanh nhiều object. Cả selection dùng **một Gumball chung**; không cần tạo
Group. Move dịch cả cụm, Rotate xoay cả cụm quanh tâm chung, Scale đổi cả kích
thước object và khoảng cách tới tâm chung. Một Ctrl+Z hoàn tác cả thao tác.
Ở Object Alignment, selection nhiều whole object dùng trục của CPlane active
và tâm bounds của cả cụm; một object riêng vẫn dùng frame Object của nó.

| Tay nắm | Thao tác |
|---|---|
| Mũi tên đỏ / xanh lá / xanh dương | Di chuyển theo X / Y / Z |
| Ô vuông hai màu | Di chuyển trên một mặt phẳng |
| Ô nhỏ tại tâm | Di chuyển tự do |
| Cung tròn màu | Xoay quanh trục đó |
| Ô vuông rỗng ở biên object | Co hoặc giãn theo trục; giữ Shift để scale đều |
| Vòng trắng cạnh tâm | Mở menu Gumball |

Kéo tay nắm để xem preview ở cả bốn ô. Thả chuột để áp dụng; **Ctrl+Z** sau
khi áp dụng hoàn tác cả thao tác.

Muốn nhập chính xác, click tay nắm rồi nhập số: Move **`5 mm`**, Rotate **`45`**
độ hoặc Scale **`1.25`**. Nhấn Enter để áp dụng. Nếu đang giữ chuột kéo,
Enter chỉ đặt ràng buộc; thả chuột mới áp dụng. Esc đóng ô nhập số, Esc tiếp
hủy thao tác.

Giữ **Alt** khi click rồi nhập số để biến đổi một bản sao. Khi đang kéo, nhấn
Alt một lần bật Copy, lần nữa tắt. Giữ **Ctrl ngay từ lúc nhấn chuột** để chỉ
đổi widget/pivot. Thêm Ctrl sau khi đã kéo mũi tên và giữ tới lúc thả để Extrude
một curve/surface phẳng có hỗ trợ.

Trong menu vòng trắng, chọn **World / Object / CPlane** để đổi hệ trục;
**Relocate** đổi pivot, **Reset** đặt lại pivot. Relocate bằng Command nhận
origin, điểm hướng X rồi Enter hoặc điểm hướng Y. Ví dụ `w10,0,0` là điểm
world; điểm không có `w` dùng CPlane hiện hành và đơn vị document.

**Smooth** kéo liên tục; **Snappy** dùng Osnap đang bật. **Drag strength**100%
là bình thường,50% dịch bằng nửa khoảng kéo. **Settings** đổi màu/kích thước;
**Restore defaults** khôi phục thiết lập Gumball. **Rotate view around gumball**
đặt tâm orbit tại pivot. Gõ **`Gumball Off`** để tắt.

Một số owner parametric hoặc kiểu Scale không biểu diễn được sẽ báo lý do
và giữ model trước thao tác. [Phạm vi đã kiểm chứng](features/OM9-INFO-020.md).

Phạm vi core được duyệt đã qua native cohort cuối ngày 2026-10-12. Scale một
primitive có thể chỉnh dimensions và giữ expressions. Khi native owner không
biểu diễn trực tiếp phép co/giãn, kết quả liên kết giữ definition và dependency
của nguồn thay vì thay nguồn bằng một shape rời; Alt giữ nguyên nguồn. Object
được ẩn sau khi tạo kết quả vẫn giữ definition để kết quả cập nhật khi nguồn đổi.
Một graph hoặc frame không hỗ trợ sẽ báo lý do và hủy cả thao tác.

Object alignment trong phạm vi đã kiểm chứng nhận Mesh vertex/edge/facet qua **Ctrl+Shift-click**
ở ô nhìn; component mơ hồ, suy biến hoặc facet không phẳng sẽ bị từ chối.
Typed Camera/Light là object trong document với vị trí/hướng và dimensions
vật lý, có thể chọn như object khác; chúng không chứng nhận renderer hoặc
widget camera đang điều khiển viewport.

Pivot đã Relocate được nhớ trong phiên cho một owner còn nguyên. Khi đóng
tiến trình, cache này kết thúc; mở lại FCStd giữ geometry/native object nhưng
không khôi phục widget pivot. Reset hoặc chọn Alignment rõ sẽ đặt lại frame.
[Candidate, phạm vi và gates](validation/2026-10-12-gumball-core-release-candidate.md).

[Phạm vi core đã kiểm chứng và giới hạn host](validation/2026-10-12-gumball-core-completion.md). Xem trạng thái build và kiểm thử trên FreeCAD bản cài đặt tại [GitHub Actions](https://github.com/MYuitsu/OpenMatrix9/actions/workflows/release-windows.yml) của đúng commit.
