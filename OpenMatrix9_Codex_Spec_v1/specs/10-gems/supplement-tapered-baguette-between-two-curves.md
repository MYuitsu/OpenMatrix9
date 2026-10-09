# Tapered Baguette Between Two Curves

Alias tương thích `gvTaperBagBetweenTwoCurves`. Workflow bổ sung chưa có ID
trong catalog 607 feature; không gộp vào Baguette Channel hoặc Gems Between 2
Curves. Trạng thái triển khai OpenMatrix9 chưa xác định.

Mục tiêu là bố trí một baguette thuôn giữa hai curve với khả năng chỉnh độc lập
vị trí mỗi đầu và kích thước viên đá. Người dùng gán hai curve vào Builder,
chỉnh preview rồi Enter để tạo kết quả. Đá đã tạo có thể tiếp tục chỉnh qua
Baguette Builder; adapter phải giữ identity và tham số thay vì chỉ tạo mesh rời.

| Tham số | Ý nghĩa |
|---|---|
| Length | Chiều dài theo mm; điều khiển từ ô số, slider hoặc handle ngang trên girdle. |
| Width Bottom / Width Top | Hai chiều rộng đầu baguette theo mm, chỉnh riêng. |
| Depth | Độ sâu table–culet theo mm; handle đứng tại table. |
| Girdle Thickness | Bề dày girdle theo mm; handle đứng tại girdle. |
| Point A / Point B | Vị trí mỗi đầu dọc curve, nhập theo phần trăm hoặc kéo handle cầu trên curve. |
| Flip | Đảo hướng đặt baguette. |
| Reset | Trả cấu hình về bộ default đã được Builder xác lập. |

Solver cần resolve hai curve, đánh giá vị trí A/B, tạo frame đặt đá và hình
baguette theo kích thước. Kiểm tra curve rỗng, hai đầu trùng nhau, tham số ra
ngoài miền và hình không hợp lệ. Miền hợp lệ, default số và quy tắc cập nhật
History chưa xác định; không tự suy ra. Preview/cancel không sửa document;
commit phải nguyên tử và nghiệm thu Undo/Redo, save/reload cùng chỉnh sửa sau tạo.
