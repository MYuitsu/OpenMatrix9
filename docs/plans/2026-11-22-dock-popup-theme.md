# Kế hoạch chuẩn hóa dock, popup và theme OM9

Ngày 22-11-2026. Kế hoạch triển khai cho OM9 standalone trên Windows.

## Kết quả cần đạt

Mọi lệnh OM9 có bảng thao tác riêng và mọi khung workspace đều dùng cùng hệ thống bố cục. Người dùng có thể kéo bảng thành cửa sổ nổi hoặc thả vào dock trái hay phải. Phạm vi gồm cả Icon History, Main Menu, Display, **Snaps, Projects**, Info & Settings, Layers, vùng Builders và các Builder, bảng Properties, F6 và báo cáo có giao diện riêng. Vùng Builders nằm dưới Projects ở bố cục mặc định. Nội dung công việc đang mở tiếp tục hoạt động khi bảng đổi vị trí.

Người dùng có thể tạo nhiều **profile bố cục** có tên để chuyển cả workspace theo công việc. Ví dụ, một profile đặt Snaps phải và Projects trái, profile khác đặt Snaps trái và Projects nổi. Mỗi bảng nhớ vị trí riêng trong từng profile, kể cả bảng lệnh đang đóng tại thời điểm lưu.

Ảnh tham chiếu xác định thứ tự cột trái, cách xếp các bảng theo chiều dọc và vị trí Builder. Đề xuất dùng nhận diện xanh của OM9 làm theme chung, với sắc nền phân cấp cho cột dock, thân bảng, thanh tiêu đề và menu nổi. Giữ màu theo hợp đồng hiện hành của CMD và viewport. [Hợp đồng giao diện CMD và viewport](../../skills/openmatrix9-ui/references/command-grid-theme.md)

## Rà soát hiện trạng

| Thành phần | Phần có thể dùng lại | Phần cần làm |
| --- | --- | --- |
| Dock trái/phải và bảng nổi | `DockLayout` lưu thứ tự, kích thước và trạng thái vào `ui-layout.json`. `panel_ops` đã có đặt vào một vị trí, đổi cột, thả nổi và kiểm vùng làm việc | Đăng ký cả khung cố định và bảng lệnh vào cùng cơ chế kéo thả. Thêm danh sách profile có tên và profile đang dùng vào cấu hình bố cục. [Dock layout](../../crates/om9-app/src/dock_layout.rs) · [Panel operations](../../crates/om9-app/src/panel_ops.rs) |
| Builders | Ring Rail và Builder được host đã có điểm gắn vào sidebar. Gem Loader, Gem on Curve và Gem on Surface có phiên lệnh và preview hiện hữu | Đưa phần điều khiển từng Builder vào một khung có thể đổi vị trí. Dùng lại chính phiên lệnh khi di chuyển. [Sidebar](../../crates/om9-app/src/ui/chrome.rs) · [Gem UI](../../crates/om9-app/src/ui/gem.rs) |
| Theme | `chrome.rs` áp màu dock cục bộ và đặt style egui Light cho ứng dụng. Một số hộp thoại đặt lại `Visuals::light()` | Gom bảng màu và kiểu khung vào một nơi. Áp dụng cho cả dock, bảng nổi và widget con mở ngoài bảng như ComboBox. [Theme hiện tại](../../crates/om9-app/src/ui/chrome.rs) |

## Quy tắc giao diện và thao tác

- **Bảng lệnh và khung workspace** có tiêu đề, vùng kéo, thu gọn, đóng và cùng bố cục nút hành động ở ba vị trí. Có thể thả vào khe trước/sau một bảng ở dock trái hoặc phải, kéo giữa hai cột, hoặc kéo ra canvas để nổi. Trong lúc kéo hiện đường chèn và khung thả. Menu tiêu đề có Dock trái, Dock phải và Nổi để sử dụng bằng bàn phím. Menu View cho mở lại một khung đã ẩn. Bảng gắn với lệnh xuất hiện khi gọi lệnh đó.
- **Popup có nội dung làm việc riêng**, như F6, trình sửa thuộc tính và cửa sổ kết quả, có đủ ba vị trí. **Popup tạm theo thao tác**, như menu ngắn, danh sách thả xuống, tooltip và trình chọn đối tượng trùng, cùng theme và bám điểm mở. Hộp xác nhận giữ tính modal, hộp thoại hệ điều hành theo theme Windows. Kiểm kê ghi từng trường hợp để mọi giao diện lệnh đều có đường triển khai và ca thử.
- **Theme chung** định nghĩa font, khoảng cách, đường viền và bo góc, cùng màu nền, tiêu đề, chữ, icon, hover, focus, chọn, vô hiệu, Apply, cảnh báo và lỗi. Dock dùng tiêu đề gọn cho cột dọc. Bảng nổi dùng thêm viền và bóng để tách khỏi canvas. Kiểm tra chữ tiếng Việt và trạng thái tương tác ở mọi biến thể.
- **Kéo thả trong khi chạy lệnh** giữ tham số, đối tượng nguồn, điểm đã bấm, quá trình nền và preview. Thả sai vùng hoặc Esc trong lúc kéo đưa bảng về vị trí trước khi kéo. Tiêu đề, nút và thao tác kéo nhận sự kiện chuột trước viewport. Enter, Esc ngoài thao tác kéo, nút X và việc trả focus tuân theo vòng đời của từng lệnh.
- **Cửa sổ hẹp và DPI cao** tuân ngưỡng 1.200 điểm logic của dock phải. Nếu bảng đang thao tác nằm ở dock phải khi dock phải nhường chỗ, đề xuất tạm hiện nổi trong vùng thao tác và trở về dock phải khi có đủ chỗ. Vị trí lưu chỉ thay đổi khi người dùng chủ động di chuyển bảng. [Hợp đồng vùng dock](../../docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md)

## Profile bố cục

- Trong menu View, thêm **Profile bố cục** để tạo từ bố cục đang dùng, đổi tên, nhân bản, chuyển profile và chọn profile mở lúc khởi động. Reset Layout áp dụng cho profile hiện tại.
- Mỗi profile lưu dock trái/phải/nổi, thứ tự, kích thước, tọa độ cửa sổ nổi, trạng thái hiện/ẩn và thu gọn của từng khung. Vị trí khung lệnh được nhớ qua các lần mở, còn nội dung lệnh, bản vẽ, cấu hình Snap và dữ liệu Projects tiếp tục thuộc về các owner hiện có.
- Khi chuyển profile, cập nhật bố cục trong một lần rồi hiển thị lại các bảng đang mở ở vị trí trong profile đích. Bảng có phiên lệnh đang hoạt động vẫn tiếp cận được nếu profile đích ẩn nó. Giữ nguyên tham số, lựa chọn và preview. Hủy thao tác kéo đang dở trước khi đổi profile.
- Đọc layout schema v2 thành profile đầu tiên và giữ cách bố trí người dùng đã lưu. Khi màn hình thay đổi hoặc thiếu một add-on, lưu lại vị trí mong muốn và chọn vị trí hiển thị khả dụng. Khi điều kiện trở lại, bảng dùng vị trí trong profile.

## Thứ tự triển khai

1. **Kiểm kê và chốt baseline.** Đối chiếu danh mục lệnh và các điểm tạo UI với danh sách khung workspace. Ghi từng bảng lệnh, khung cố định, popup, report, hộp thoại và UI do Windows sở hữu cùng ID ổn định, owner, đường mở, hành vi dock, focus và đóng. Dùng Snaps, Projects, Gem Loader, Ring Rail, Layers, F6 và một hộp xác nhận làm mẫu ảnh trước thay đổi.
2. **Theme và khung chung.** Dùng style egui và các hàm vẽ có sẵn trong `chrome.rs` làm điểm chuyển đổi. Tạo một đường tạo khung bảng công cụ, một đường cho dialog và một đường cho popup bám điểm mở, cùng đọc theme OM9. Kiểm theme truyền tới title bar, ComboBox, scrollbar và tooltip. Mở rộng kiểm tra ID cửa sổ để phát hiện các popup tạo ngoài khung chung, với ngoại lệ có ghi owner.
3. **Mở rộng dock và lưu profile.** Tái dùng vùng thả, chỉ báo kéo và thao tác bố cục. Dùng cùng cơ chế nhận diện cho khung có sẵn, bảng lệnh và UI add-on do OM9 host, với khóa ổn định độc lập với tiêu đề dịch. Giữ `BuilderSlot` là vùng mặc định bên trái. Mở rộng lưu bố cục để đọc schema v2, quản lý profile và lưu nguyên tử khi đổi profile hay di chuyển bảng.
4. **Thử cả khung cố định và lệnh.** Chạy chu trình dock trái → dock phải → nổi → dock trái cho Snaps và Projects với dữ liệu thật, cho Gem Loader khi có preview và cho F6. Kiểm Ring Rail và Builder add-on qua cùng cơ chế. Tạo hai profile với vị trí khác nhau và chuyển qua lại trong phiên thao tác. Mỗi lần đổi chỗ chỉ hiển thị một bản của bảng.
5. **Chuyển toàn bộ danh mục.** Hoàn tất các khung workspace, sau đó tất cả Builder và UI lệnh, Properties, History, Styles, Profile Library, Add-ons, Project Manager, cài đặt và report. Áp theme cho hộp thoại, menu, tooltip và popup trong viewport. Mỗi mục trong danh sách bước 1 có đường mở và ca kiểm tương ứng. Các bảng dock được kiểm ở cả ba vị trí, gồm những lệnh hiện đặt UI trong một popup riêng.
6. **Nghiệm thu và ghi bằng chứng.** Mở rộng `shell_docking`, `dock_layout_persistence`, `dock_layout_default`, test Builder và replay egui hiện có. Dựng và kiểm trên Windows với `L:/om9-target` và `-j 2`. Quay chu trình kéo ở DPI thật cho cả khung workspace lẫn lệnh, rồi chuyển profile và khởi động lại. Chạy public source audit và cập nhật đúng M01 cùng các mục đã thử.

## Điều kiện nghiệm thu

| Ca thử | Kết quả cần quan sát |
| --- | --- |
| Kéo/đổi cột/đổi thứ tự với chuột thật | Snaps, Projects và bảng lệnh nổi đều có đường chèn đúng và một bản đang mở. Snap đang bật, project đang chọn và thao tác bên trong bảng tiếp tục hoạt động. Click trên bảng không đặt điểm vào bản vẽ. |
| Chuyển vị trí hoặc chuyển profile khi lệnh đang preview | Kiểm nhóm Gems cùng một lệnh ngoài nhóm Gems. Nội dung, ID nguồn và số điểm còn nguyên. Apply tạo đúng kết quả và một bước Undo. Cancel dọn preview. |
| Theme của dock, bảng nổi và widget con | Trạng thái hover, focus, chọn và vô hiệu đọc rõ trên cả hai dock và cửa sổ nổi. Toàn bộ popup trong danh sách có kết quả kiểm. |
| Hai profile, khởi động lại và Reset Layout | Tạo hai bố cục khác nhau, đổi qua lại, mở lại cửa sổ và so từng khung. Reset một profile giữ bố cục profile kia. Tệp schema v2 mở được và bố cục mặc định giữ thứ tự từ Icon History đến Projects rồi Builders. |
| 100%, 125%, 150% DPI, màn hình thứ hai và cửa sổ qua ngưỡng dock phải | Tiêu đề luôn kéo được, nút Apply/Cancel vẫn tiếp cận được, bảng bên phải có thể phục hồi sau khi cửa sổ rộng lại. |
| Menu, hộp modal và thao tác phím | Menu đóng đúng chỗ, modal chặn thao tác phía sau, gõ trong bảng và kết thúc lệnh trả focus theo hợp đồng CMD/viewport. |

Các mục M01 phụ trách vùng dock, thao tác panel, reset, persistence và Builder Slot là `OM9-L3-01-01-002/003/004/005/008`. Gắn phần thứ tự sidebar và thao tác khối menu vào `OM9-L3-03-07-001/006/007`. Kế hoạch này ghi nhận kết quả rà mã. Trạng thái nghiệm thu từng mục sẽ dựa trên kết quả thực thi ở bước 6. [M01 Docking](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell Docking/L2.json>) · [M01 Sidebar](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-07 - Febe Matrix Main Menu Iconhistory And Docking/L2.json>)
