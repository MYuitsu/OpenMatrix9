# Rectangle — sửa hướng kéo và tọa độ CMD (2026-10-26)

Target: standalone Rust OM9, Windows x64, `OM9-CURVE-004` / `OM9-L3-03-16-004`. Đây là phần sửa mã sau [chẩn đoán](../2026-10-26-rectangle-diagnosis/README.md); lần cập nhật skill trước đó không sửa sản phẩm.

## Thay đổi đã kiểm chứng

- Trước: Corners lấy trị tuyệt đối của thành phần CPlane rồi luôn dựng theo +X/+Y; ba hướng kéo có thành phần âm bị phản chiếu. Sau: độ dài vẫn dương, hai trục dựng giữ dấu hướng kéo theo CPlane. `RectPending::corner_frame` được cả preview và commit dùng chung. Center giữ kích thước/semantics hiện có; Square và Length cố định giữ phía con trỏ.
- Trước: `Rectangle → 3,7,0` bị từ chối trước khi parser tọa độ chạy. Sau: tọa độ ở stage nhận điểm đi qua `om9_core::coords::parse_input` và `resolve_with_base`, rồi `accept_world_point` như mouse. Hỗ trợ grammar chung local/world/relative và page units; assignment option giữ đường xử lý hiện có. Input sai giữ pending để sửa.
- Không đổi ID, icon hay default mode. Không sửa native adapter hoặc GPU backend. Không bật Rounded hay mở rộng nghiệm thu các mode ngoài phạm vi đã thử.

## RED → GREEN và build

Nguồn regression thường trực: `crates/om9-app/tests/rectangle_regressions.rs`.

1. Trước sửa: **5/5 regression FAIL**, xem [red.log](red.log).
2. Sau sửa: **42 tests PASS** trong 5 suite, xem [green.log](green.log): `rectangle_regressions` 5, `rect` 16, `command_values` 7, `command_options_interface` 9, `curve_menu` 5.
3. Regression hướng kéo thực thi 24 case: bốn tổ hợp dấu × ba CPlane (XY, Front, XY xoay/dịch origin) × hai cách gọi menu/CMD, cùng world picks. Điểm đầu khác gốc và khác cao độ CPlane. Kiểm đỉnh canonical, preview/commit parity, một entity/scene object visible và epoch đồng bộ. Các test riêng kiểm local/relative với cm, world coordinates chiếu cao độ, Square/Length hướng âm, sai/retry/degenerate/Undo/Redo/Cancel.
4. `cargo build -p om9-app --bin om9 -j 2`: PASS, xem [build.log](build.log). `CARGO_TARGET_DIR=L:/om9-target`; product default features. Không có Rust source mới hơn binary tại kiểm tra cuối, bản staging và bản debug có cùng SHA-256.

## UI thật qua Computer Use

Chạy executable vừa build trong profile layout riêng; PID/path/hash trong [runtime-launch.json](runtime-launch.json). Binary debug có thể mở trực tiếp tại `L:/om9-target/debug/om9.exe`; bản staging dùng runtime DLL directory của build debug. Lần thử staging chỉ có SDK bin trên PATH thoát loader `0xC0000135`; thêm thư mục runtime debug đã có giải quyết khởi động, không sửa native code.

Đã thao tác trên cửa sổ có fingerprint:

- Click icon Rectangle trong Curve; chọn góc đầu tại pixel cửa sổ `(515,300)`, kéo tới `(680,490)` trong Looking Down. Preview xuống phải đúng vị trí; click chốt tạo hình cùng phía và prompt về `Command:`. Geometry hiển thị ở Looking Down và Perspective, hai hình chiếu cạnh hiển thị đường cạnh tương ứng.
- Click CMD, nhập bằng bàn phím `Rectangle`, `3,7,0`, `-7,-13,0`, mỗi bước nhấn Enter. Góc đầu được nhận, góc đối diện chốt thành hình ở đúng phía âm. Cửa sổ hiện hai rectangle đã tạo qua hai route.
- Lưu bốn ảnh ứng dụng thực dưới `implementation/evidence/AFTER/rectangle-signed-corners-20261026/`. Thao tác Ctrl+S đã mở dialog; bước set-value của helper không hoàn tất và dialog không còn ở quan sát tiếp theo. Không có file fixture được lưu: không tính lần này là bằng chứng save/reload; không chụp hay đưa dialog/thư mục cá nhân vào evidence công khai.

## Giới hạn và bước tiếp theo

Ảnh trước/sau chốt xác nhận hình có hiển thị tại những frame quan sát; không phải phép đo frame/present liên tục để loại trừ mọi chớp hình. Triệu chứng biến mất thoáng qua trong video cũ chưa có nguyên nhân GPU được xác nhận. Không thay đổi backend để suy đoán sửa triệu chứng đó.

Chưa nghiệm thu toàn bộ options/3Point/Vertical/Rounded, đổi CPlane giữa command, save/reload, macOS hoặc human M03-UI. 16 test cũ của các mode hiện có vẫn qua; không mở rộng kết luận từ các pass này. Không chạy measurement openNURBS, không phát hành release hoặc thay shortcut/launcher người dùng. Cửa sổ bản sửa được giữ mở để người dùng thử. Nếu còn triệu chứng chớp hình, replay đúng binary có fingerprint và đo preview→scene→GPU present riêng.
