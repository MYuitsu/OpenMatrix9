# Gem Loader, Gem on Curve và Gem on Surface

## Phạm vi triển khai

| Lệnh | Đường xử lý và hình học |
| --- | --- |
| Gem Loader | Menu, CMD và F6 dùng phiên đặt đá của Controller. F4 là gốc thế giới khi không có điểm hay đá đích. Phiên nhận nhiều điểm trong viewport hoặc CMD, hỗ trợ Undo của điểm, Percentage, mẫu từ kho Gem Styles và chuyển sang Gem on Surface. Apply dồn các điểm vào một giao dịch. Thay đá giữ định danh, vị trí và hướng của đối tượng trong các trường hợp hỗ trợ. |
| Gem on Curve | Chọn đường trước hoặc sau khi mở lệnh. Tham số Start/End biểu diễn phần trăm chiều dài cung. Spacing là khe dọc theo đường giữa các biên girdle chiếu lên khung đặt đá. Step nhận bước tâm rõ ràng. Ba cách chia Spacing, Size Gems và Fixed chia lại hàng theo quy tắc tương ứng. Tay nắm Start/End dùng cùng bộ tham số với CMD. Recipe của hàng lưu nguồn và tham số để F6 sửa sau khi mở lại tệp. Đá còn lại giữ ID theo thứ tự hàng. |
| Gem on Surface | `gvOrientGems` vào phiên đặt đá trên mặt. Điểm hover và click dùng tia từ camera, UV của bề mặt NURBS nguồn, biên trim và pháp tuyến có hướng. Click giữ đá trong preview cho đến Apply, trừ Copy=No kết thúc tại lần đặt đầu. Có chế độ Grid và U/V khi yêu cầu. Phiên kiểm lại dấu nguồn trước khi ghi. |

## Đối chiếu Matrix9

Manual Matrix 8 Book 2 tr. 176–177, 177–181, 208–209 mô tả ba lệnh. Bản cài Matrix9 có menu Gems ở các vị trí 1, 2, 11 và gọi `_gvBuilderGemLoader`, `_gvGemOnCrv`, `_gvOrientGems` từ F6. Form VB6 và định nghĩa Ghidra được đối chiếu theo địa chỉ.

| Thành phần VB6 | Thủ tục | VA Ghidra | Phạm vi bằng chứng |
| --- | --- | --- | --- |
| frmICBuilderGemLoader | Form_Load | `0175F0F0` | Cấu hình Builder Gem Loader. |
| frmICBuilderGemOnCrv | Proc_97_15_16FD1E0 | `016FD1E0` | Nhánh gọi Gem on Curve và macro Browser. |
| frmICBuilderGemOnSurface | ctrInSurface_UnknownEvent_B | `016C3250` | Sự kiện nhận mặt và gọi `_gvOrientGems`. |

Đọc 57 tệp 3DM cục bộ trong `Library/ICGems`. Mỗi tệp chứa một B-rep. Có 22 Diamond, 14 Simple, 11 Gem, 9 Cabochon và 1 Pearl. Tám Diamond và mẫu Pearl dùng đơn vị None. So sánh 29 tổ hợp mà thư viện OM9 dựng được sau khi cấp cùng kích thước bao, sắp lại XY theo chiều dài và chiều rộng, và lấy trung bình XY cho mẫu ghi là tròn. Cả 29 thân OM9 kín. Sai lệch kích thước bao lớn nhất là 1,8 × 10⁻¹⁵ theo số đo đã chuẩn hóa. Cao độ hai đầu theo Z lệch tới khoảng 0,934 đơn vị trên mẫu nguồn. So sánh này kiểm kích thước bao và mốc Z, chưa đo đồng dạng toàn bộ đường bao, mặt table hay topology facet. Các mẫu thương mại nằm ở thư viện cục bộ. OM9 dựng và phân phối tài sản của dự án.

Kiểm tra tiếp metadata RhinoScript trong 57 tệp xác nhận có 1.476 cặp kích thước preset. COM GetGemSizeList đọc các cặp này theo kiểu cắt và dáng đá. [Đối chiếu cấu trúc và số lượng](2026-11-22-matrix9-gem-catalog.md).


## Wireframe Round Diamond

- Mã RhinoCommon của Rhino5 và bản decompile cục bộ xác nhận `DisplayPipeline.DrawBrepWires` chuyển B-rep và `wireDensity` vào wrapper `CRhinoDisplayPipeline_DrawBrep` (RVA `0x44E70` của `rhcommon_c.dll`), rồi đến `CRhinoDisplayPipeline::DrawBrep` (RVA `0x4D33D0` của `Rhino.exe`). `DrawMeshWires` dùng API khác. `ON_3dmObjectAttributes::m_wire_density` phân biệt cạnh biên với đường đẳng tham số theo nút và mật độ nội bộ. Kiểm đọc `Library/ICGems/Diamond/Round.3dm` cục bộ cho thấy `WireDensity = 1`, 127 mặt B-rep và 286 cạnh. Lượt nhập tệp đối chiếu giữ nguyên nguồn riêng vì có userdata thương mại. Số đếm và mật độ được đo trực tiếp bằng bộ đọc 3DM. [Mã RhinoCommon 5](https://github.com/mcneel/rhinocommon/blob/master/dotnet/rhino/rhinosdkdisplaypipeline.cs#L1080-L1117) · [Tài liệu API](https://developer.rhino3d.com/api/rhinocommon/rhino.display.displaypipeline/drawbrepwires)
- Bản OM9 trước sửa tạo đường bao 16 đoạn và chia mặt bàn thành quạt tam giác. Chuyển sang OCCT từng tam giác làm đường chéo mặt bàn trở thành cạnh B-rep. Bản mới sinh thân Round Diamond đối xứng tám hướng từ các mặt cắt do OM9 dựng, với đai 32 đoạn và 89 mặt nguồn. Mặt bàn bát giác là một mặt B-rep. Preview và mesh bỏ các đường chéo dùng để biểu diễn mặt bàn, trong khi các cạnh B-rep đến từ chính mặt nguồn. Giữ logic hình học trong Rust. C++ chỉ nhận các vòng cạnh phẳng và dùng lại phép may vỏ OCCT thành khối kín. Lưới CPlane tiếp tục có bước phụ 1 mm, vạch chính 5 mm và biên ±20 mm.
- Dựng trên `L:/om9-target` với `-j 2`. Có 98 ca đạt trong `gem_panel` (28), placement app (21), `gem_assets` (24), placement core (22) và `gem_brep` (3). Kiểm ba cỡ Ø1,00, Ø3,10 và Ø11,20 mm đều có tám cạnh thật ở mặt bàn trên cả hai đường hiển thị. Kiểm thêm thể tích B-rep theo mesh, từ chối mesh mở, F4, preview, Apply và Undo.
- Ảnh Windows góc Top của viên OM9 Ø3,10 mm cho thấy chùm đường chéo một phía đã biến mất. So với ảnh Matrix9 được cung cấp, đai OM9 vẫn chia đoạn khác và các đường đẳng tham số còn phụ thuộc cách tham số hóa mặt. Bản dựng chụp sau sửa có SHA-256 `275b31f592a80803ea17b7607dbf3733a788b86fb6c618b4636202fa66d875a3`. [OM9 trước sửa](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/round-wireframe/om9-before.jpg) · [OM9 sau sửa](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/round-wireframe/om9-after.jpg)

## Kiểm chứng Windows

- Dùng Cargo target `L:/om9-target` và `-j 2` với cấu hình sản phẩm OCCT. Bốn bộ `gem_panel`, `gem_placement` cấp app, `gem_assets` và `gem_placement` cấp core lần lượt đạt 25, 19, 21 và 22 ca trong lượt tích hợp. Lượt app placement cuối đạt 20 ca sau khi bổ sung replay viewport. Ca app gồm Loader với CPlane đã dời, macro đặt hàng, sửa F6 sau lưu/mở `.om9`, hai tia đặt cùng vị trí trên mặt có lỗ trim và Undo cả lượt.
- Hai cửa sổ Windows riêng dùng profile standalone và profile bật add-on Matrix. Mỗi profile chạy một Loader qua feature ID, Gem on Curve qua macro có dấu gạch dưới, sửa hàng bằng action F6 và Gem on Surface qua `_gvOrientGems _Grid=2`. Mỗi Loader thêm 1 thân, hàng Curve từ 6 còn 4 thân sau sửa, Grid thêm 4 thân. Các lần tạo tăng đúng một mục Undo. Receipt đi qua cầu nối của cửa sổ đang chạy. [Standalone](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/profile-standalone/coverage.json) · [Matrix](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/profile-matrix/coverage.json)
- Replay egui chọn mặt NURBS trong viewport, bấm đặt một viên, xác nhận chưa có đá trong document trước Enter rồi so tọa độ sau commit với giao điểm tia camera và mặt phẳng. Ca replay kiểm widget và đường gọi hình học.
- Cargo check chạy thành công khi tắt feature mặc định. Lệnh audit nguồn công khai trả 0 lỗi. Công cụ giữ artifact tìm thấy 0 run cũ đủ điều kiện xóa.

## Giao diện Builder

- Gem Loader mở lưới Gem Shape từ nút đầu nhóm Gems. Mũi tên và danh sách đổi giữa Diamond, Simple, Gem, Cabochon và Pearl. Chọn hình mở trang Size và cập nhật preview. Ô hình lấy đường bao từ hình học OM9. Bảng giữ Length, Width, Depth, Percentage và Keep Original Size. Pearl tròn có hai vòm đối xứng. Đường bao Cushion, Pear và Marquise được sửa ở bộ sinh hình.
- Gem on Curve có chọn đường, các nút chọn tham số, chế độ chia hàng, taper và hướng culet. Gem on Surface có chọn mặt, đường dẫn và đá, các nút chọn thông số cùng Copy, Snap, Flip, Start và Reset. OM9-GEM-002 và OM9-GEM-011 mở Builder từ nhóm Gems. Hai lệnh dùng dịch vụ Rust khi có hoặc không có add-on.
- Tạo nhiều viên B-rep trong một lần Apply cấp nguồn và định danh riêng cho từng viên. Undo và Redo dành cho cả lượt.
- Windows standalone: bấm menu Gems rồi Gem Loader, đổi qua cả năm kiểu cắt, chọn Round trong Pearl, chọn 8 mm và Apply. Cửa sổ ghi đúng một thân B-rep Round Pearl tại gốc thế giới và một mục Undo. Bấm trực tiếp biểu tượng Curve và Surface đều mở bảng Builder. [Video Loader](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/gallery-video/OM9-Gem-Loader.mp4) · [Ảnh bảng chọn](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/gallery-video/OM9-Gem-Loader.png) · [Trạng thái sau Apply](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/gallery-video/pearl-desktop-state.json).
- Clip Windows standalone ghi ba lượt trên cùng cảnh mẫu. Gem Loader chọn Oval Diamond 6 × 4 mm trên trang cỡ và đặt tại F4. Gem on Curve hiện Builder và rải 9 viên cỡ 3,50 mm với khe 0,65 mm khi điều khiển qua cầu nối của ứng dụng. Gem on Surface chọn mặt NURBS nghiêng bằng chuột rồi bấm đặt 4 viên Oval với góc 20°. Trạng thái cuối có 14 viên, 2 đối tượng nguồn và 3 lượt Undo. [Clip thao tác](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/combined-video/OpenMatrix9-Gem-Loader-Curve-Surface.mp4) · [Trạng thái cuối](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/combined-video/video-result.json).


- Lượt kiểm tra giao diện và catalog đạt 119 ca Rust trong bảy bộ, gồm Loader, Curve, Surface, asset, edit và định tuyến menu khi bật hoặc tắt add-on. Bản không bật feature mặc định qua cargo check. Audit nguồn công khai trả 0 lỗi.

## Ô hình trong bảng đặt đá

- Gem on Curve dùng nút hình Curve để chọn đường nguồn và ô minh họa cho Start, End, Spacing, Z Offset, Roll, Start Size và End Size khi bật taper. Gem on Surface dùng nút hình Surface để chọn mặt và ô minh họa cho Angle, Offset, Scale và Spacing. Hình đá, nguồn và mũi tên do OM9 vẽ. Ô hiện hành giữ viền sáng và chọn thanh giá trị ngay bên dưới khi đổi vị trí bảng.
- Windows standalone ở 100% DPI. Mở cả hai lệnh qua menu Gems và dùng nút hình để chọn nguồn. Curve đổi khe từ 0,20 lên 0,75 mm ở dock phải, preview đổi từ 22 còn 16 viên và Apply ghi cả hàng. Surface giữ ô Offset khi thả nổi, nhận 0,4 mm và đặt một viên trên mặt nghiêng. Sau Apply tài liệu có 19 đối tượng và hai bước Undo. Undo gỡ lượt Surface. [Curve dock trái](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/placement-graphics/curve-left.jpg) · [Curve dock phải](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/placement-graphics/curve-right.jpg) · [Curve dạng nổi](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/placement-graphics/curve-floating.jpg) · [Surface dock trái](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/placement-graphics/surface-left.jpg) · [Surface dạng nổi](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/placement-graphics/surface-floating.jpg) · [Trạng thái sau Apply](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/placement-graphics/desktop-applied.json).
- Dựng target `L:/om9-target` với `cargo test -j 2 -p om9-app --test gem_placement`. Hai cửa sổ đang mở giữ executable chính nên chạy trực tiếp test binary vừa liên kết, cùng DLL từ thư mục debug. Cả 21 ca đạt, gồm giữ nguồn và preview của cả hai bảng ở dock trái, phải và nổi tại 100%, 125%, 150% qua egui, cùng Apply, Undo và Cancel. Desktop chạy executable song song có SHA-256 `d007f004dbffc18e811e064c81e1a0f7bab14b231a2a1623b48c6c75d6e20f2d`.
- Clip 54 giây trên cùng bản dựng và một profile Windows riêng. Kéo tiêu đề Gem Loader trái → phải → trái → nổi, giữ preview trong suốt các lượt thả. Chọn Oval 6 × 4 mm và Apply tại F4. Curve chọn nguồn và chỉnh bằng các ô hình, ghi 9 viên cỡ 3,50 mm với khe 0,65 mm. Surface chọn mặt qua ô hình, chỉnh Scale thành 4,40 mm rồi bấm bốn vị trí trên mặt nghiêng. Trạng thái cuối có 14 viên, hai nguồn và ba mục Undo. [Clip kéo dock và đặt đá](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/docking-video/OpenMatrix9-Gem-Dock-Curve-Surface.mp4) · [Khung kiểm tra](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/docking-video/review.jpg) · [Trạng thái cuối](../../build/standalone-validation/ui-artifacts/2026-11-22-gems/docking-video/final-state.json).

## Còn mở

- Gem on Curve còn Object và Browser. Tay nắm viewport hiện có cho hai đầu hàng. Cần kiểm các tay nắm còn lại, clearance trong không gian 3D trên đường cong gắt và thao tác Style cho cấu hình cả hàng.
- Gem on Surface dùng NURBS và NURBS có trim. Cần hỗ trợ chọn và lưu nhận dạng từng mặt trong B-rep nhiều mặt, kiểm nghiệm trên mặt cong có trim, hai mức hiển thị, phím tắt và thao tác chuột trong profile bật add-on Matrix.
- Thư viện OM9 còn thiếu một số biến thể dáng riêng của Matrix9. Nghiệm thu M05-UI cho cả ba lệnh còn mở.
