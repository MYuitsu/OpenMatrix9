# Hình học nguồn và viewport trong cùng bàn giao

Người dùng yêu cầu ngày 2026-11-13: khi triển khai hoặc sửa lệnh CAD, phải làm cả phía viewport và phía dữ liệu hình học chính xác. Đây là yêu cầu cho công việc sau này; cập nhật hướng dẫn không tự khởi chạy các lệnh còn thiếu trong inventory.

## Phạm vi thực hiện

- Trước khi sửa, xác định nguồn canonical của input/output: Curve, NURBS, B-rep/topology, placement hoặc mesh do người dùng tạo/nhập. Truy vết caller tính toán và viewport/preview/picking/snap liên quan; không chỉ sửa helper kernel hoặc hình vẽ.
- Trong cùng bàn giao, thực hiện phần hình học nguồn và phần viewport chịu ảnh hưởng, cùng liên kết giữa chúng. Nếu một phía đã đáp ứng, tái dùng và xác minh đường hiện có; ghi căn cứ không cần sửa. Không âm thầm để phía còn thiếu thành công việc tương lai rồi báo hoàn tất lệnh.
- Phía nguồn: đúng input, topology/trim, miền tham số, placement, units và dung sai phép tính. Nêu rõ kết quả chính xác, xấp xỉ theo dung sai hay mới kiểm bằng lấy mẫu. Đọc từ nguồn không tự chứng minh thuật toán chính xác.
- Phía viewport: preview, hình sau commit, Wireframe/Shaded và chọn/bắt điểm phải khớp nguồn trong phạm vi lệnh. Đoạn/tam giác hiển thị có thể tìm ứng viên; cạnh/mặt dùng cho CAD phải được gắn và xác minh với đúng nguồn, phiên bản, scene/camera liên quan. Không dùng index hiển thị như identity nguồn nếu chưa chứng thực; loại kết quả stale.
- Dung sai tính toán/xuất không bị thay bởi độ mịn viewport. Với ExtractWireframe, Silhouette và lệnh phụ thuộc cách nhìn/hiển thị theo hợp đồng, xác định riêng yếu tố được phép phụ thuộc (tập đường, density, hướng nhìn) và độ chính xác hình học kết quả. Không ép mọi kết quả bất biến khi hợp đồng cho phép chúng thay đổi.
- Mesh tác giả/nhập có thể chính là nguồn; giữ semantics của lệnh mesh. Preview hoặc mesh dựng tạm cho phép tính phải có vai trò và sai số riêng, không mặc định là mesh viewport. Không tự tạo hai pipeline, cache hay representation mới khi có thể tái dùng đường đã đúng.

## Kiểm chứng và báo cáo

- Dùng oracle hình học độc lập phù hợp cùng kiểm preview/scene và thao tác chọn liên quan; menu/CMD dùng cùng owner. Khi độ mịn display có thể ảnh hưởng lệnh, kiểm hai cấu hình hiển thị với cùng nguồn/dung sai tính toán và cấu hình xuất khi liên quan; đối chiếu bất biến hoặc thay đổi được phép theo hợp đồng.
- Với trạng thái được lưu hoặc thay đổi, kiểm lưu/mở, Undo/Redo và lỗi/hủy trong phạm vi bị tác động; hình học và scene sau publication phải tương ứng, không tạo kết quả một phần. Không mở lại mọi bộ kiểm thử cho sửa nhỏ không liên quan.
- Phân biệt kiểm kernel/controller, sự kiện egui mô phỏng, GPU ngoài màn hình và desktop thật. Scene đúng không chứng minh GPU đã present; ảnh đẹp không chứng minh CAD đúng. Nếu thiếu một tầng cần thiết thì ghi rõ mức bàn giao và phần chưa xác nhận; không tự mở cửa sổ desktop ngoài yêu cầu.
- Hồ sơ bàn giao ghi riêng: phía nguồn, phía viewport và liên kết chọn/phiên; mỗi phía có bằng chứng hoặc căn cứ không bị ảnh hưởng. Chỉ kết luận hoàn tất trong phạm vi đã kiểm cả hai phía. Không gắn việc này với điều kiện phải hoàn tất source-only/eviction của lịch sử trước.

Tra danh sách nhóm lệnh liên quan trong checkout: `docs/validation/2026-11-13-curve-solid-source-display-inventory.md`. Danh mục là nơi chọn việc có phạm vi, không phải xác nhận mọi lệnh đã chạy hay yêu cầu tự động triển khai toàn bộ.
