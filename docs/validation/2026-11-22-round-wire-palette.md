# Round Diamond Wireframe và bảng màu Gem

Ngày 22-11-2026. Phép đối chiếu dùng ảnh Top Matrix9 người dùng cung cấp và ảnh chụp từ cửa sổ OM9 trên Windows. Viên Round Diamond trong OM9 có đường kính 3,10 mm, do Gem Loader tạo dưới dạng B-rep.

## Hình học và nét vẽ

Đo trên mặt phẳng Top của mẫu Matrix9 cho thấy 64 vị trí quanh vành. Bán kính mặt bàn, các đỉnh sao trên và các điểm nối dưới lần lượt xấp xỉ 0,530, 0,745 và 0,200 lần bán kính ngoài. OM9 dựng các mặt từ kích thước và góc crown. Đường giữa U/V được cắt theo biên của từng mặt phẳng. Bốn mặt sao theo trục chính có hai nửa. Cạnh thật có độ dày và độ sáng thay đổi theo độ sâu camera. Đường U/V rộng một pixel, chỉ dùng để hiển thị. Wireframe dùng nền đen. [Đối chiếu DrawBrepWires và mật độ dây Rhino 5](2026-11-22-gem-placement.md#wireframe-round-diamond)

Khi chiếu trực tiếp mô hình tham chiếu lên ảnh Matrix9, riêng cạnh B-rep giải thích được 72,1% pixel nét xanh trong phạm vi 3 pixel. Thêm đường giữa U/V theo miền tham số từng mặt của mô hình tham chiếu cho 99,9%. Đây là phép đo trên nguồn Matrix9.

| Ảnh OM9, chuẩn hóa theo tâm và bán kính vành | Pixel nét Matrix9 có nét OM9 ở gần | Pixel nét OM9 có nét Matrix9 ở gần |
| --- | ---: | ---: |
| Bản trước, Round trong lần đo độ sâu | 61,4% | 53,7% |
| Bản sửa, Round Diamond trên Gem 03 | 93,0% | 91,7% |

Hai cột dùng khoảng cách tương đương 3 pixel ở bán kính tham chiếu 458 pixel. Mặt nạ chọn nét màu trong đĩa viên đá, bỏ hai trục màn hình để lưới và dấu gốc không ảnh hưởng phép đo. Bán kính OM9 trước và sau lần lượt là 256,5 và 390,5 pixel. Một phần nét quanh vành và bên trong các mặt vẫn lệch vị trí. Chưa đo sai số ảnh cho kiểu cắt khác. [Top OM9](../../build/standalone-validation/ui-artifacts/round-facets-palette-20261122/final/Round-Diamond-Top.png) · [Bốn góc nhìn](../../build/standalone-validation/ui-artifacts/round-facets-palette-20261122/final/Round-Diamond-four.png) · [Khi chọn](../../build/standalone-validation/ui-artifacts/round-facets-palette-20261122/final/Round-Diamond-four-selected.png)

## Màu khi tạo đá

Trên binary Windows cuối cùng, Gem Loader tạo lần lượt bảy viên Round cỡ 2, 3, 4, 5, 6, 2, 6 mm. Layer nhận được theo thứ tự là Gem 01, Gem 02, Gem 03, Gem 04, User 01, Gem 01, User 01. Tài liệu có năm layer và bảy bước Undo. Nếu tất cả hàng User đều đã có nhóm đá, OM9 dùng lại hàng User có ít nhóm nhất. [Kết quả chạy](../../build/standalone-validation/ui-artifacts/round-facets-palette-20261122/final/palette-result.json)

Các nhóm đá mới trong một tài liệu lần lượt nhận Gem 01 `#003CA0`, Gem 02 `#4178CD`, Gem 03 `#82B4FA`, Gem 04 `#AFD0FF`, rồi đến User 01, User 02 và các hàng User tiếp theo. Khóa nhóm gồm hình dạng, kiểu cắt, vật liệu và chiều dài × rộng danh nghĩa, làm tròn tới 0,001 mm. Màu hiển thị lấy từ layer. Việc tạo layer và đặt đá cùng nằm trong một bước Undo. File cũ chứa bốn màu Gem mặc định đời trước vẫn hợp lệ. [Trạng thái cửa sổ với ba nhóm đá](../../build/standalone-validation/ui-artifacts/round-facets-palette-20261122/final/desktop-fixture.json)

Kiểm thử bao gồm tạo lại cùng nhóm sau khi đổi kích thước, vật liệu và hình dạng, mã hóa và đọc lại archive có mesh, chuyển sang User, Undo/Redo, layer bị khóa, topo của B-rep và đường giữa mặt trong mesh. Bộ gem_assets, gem_brep, gem_panel và gem_placement đạt cùng các kiểm tra OCCT đi kèm (100 ca). Lượt gem_panel, layer_state và antialiasing đạt 40 ca, có lặp lại gem_panel. Lượt kiểm tra add-on rộng hơn còn hai lỗi ở việc chọn lệnh Union và dựng fixture Box qua menu. Kiểm tra nguồn công khai đạt.
