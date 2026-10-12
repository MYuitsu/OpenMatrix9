# Box mặc định theo video — 2026-10-12

Người dùng làm rõ video là **Box mặc định**, không phải bốn option. Correction này thay thế yêu cầu chọn phương pháp bắt buộc của báo cáo [trước đó](2026-10-12-box-documented-workflow.md). Video đính kèm được đọc tại 0..7,4 giây; frame/reference giữ riêng trên C:, không nhập video hoặc ảnh Matrix vào source.

## Luồng hiện hành

Box mở ngay ở `First corner of base ( Diagonal 3Point Vertical Center ):`. Mặc định chọn hai góc đáy đối diện rồi Height: đúng ba lần chọn. Bốn option là nhánh tùy chọn, không phải bước bắt buộc. **3Point riêng vẫn là hai đầu cạnh rồi Width/Height**; Vertical và Center giữ hợp đồng đã có. Default không nhận số Length thay góc đối diện.

Ví dụ mặc định `Box → 0 → 10,20,0 → 5`; explicit3Point `Box → 3 → 0 → 10,0,0 → 20 → 10`. `0` ở bước điểm là gốc CPlane. Enum Corners nội bộ đại diện default, không có option thứ năm Corners trên CMD.

Rust dựng wire base (4 edges), wire box (12 edges), marker và signed bounds trên bản sao session; hover không thêm điểm nhận/checkpoint. C++ chỉ đưa các đoạn và marker vào Coin unpickable nodes trên bốn views. Top/Bottom ray song song trục Height dùng screen-up drag theo tỉ lệ world; default anchor là góc đáy đối diện. Side/perspective giữ closest-ray/axis. Geometry/units/finite limits, native one-transaction output, Undo/Redo, save/reload và cancellation giữ nguyên. Public ABI bổ sung hai copied-buffer functions, không đổi struct CommandOptions/capability; caller edges72/markers9 có kiểm canary/native bounds.

## Kiểm thử

Full Rust **1049/1049**, 137 suites, exit0. Native **652/652**, 10 hosts exit0 cùng module SHA `5f0eec4570b151bf4b9c5622889bda1d421a7b76b06d2e54909ecdec0ba8a773`. Thêm **23/23** kiểm tra visual trên cùng module bằng host hiển thị: actual QScreen cho thấy preview sạch, marker/khối trong cả bốn views. `QWidget.grab()` gây artifact OpenGL cả trên plugin cũ; không dùng các ảnh đó để tuyên bố lỗi render. Native default-video fixture kiểm click chuột thật, độc lập ray projection/volume/bounds, base trước click2, Top-height drag, unpickable preview cleanup, native Undo/Redo và FCStd.

| Fixture native | Checks |
|---|---:|
| box_default_video | 23 |
| box_documented_workflow | 31 |
| box_command_options | 26 |
| box_command_usability | 17 |
| box_command_history | 11 |
| solid_commands | 71 |
| solid_options | 252 |
| command_mnemonics | 181 |
| builder_command_options | 25 |
| rebuild_command_options | 15 |

Rà soát độc lập đã chỉ ra scene root SoNode/SoSeparator, accessor height-axis và negative-height marker normalization; đã sửa và kiểm bằng Rust/native trước completion. Build standalone OpenMatrix9Gui qua script quy định, reuse SDK/cache/App299; runtime/Rust/test profiles riêng trên C:, không root rebuild/copy host. [Machine evidence](2026-10-12-box-default-video.json) ghi hash, receipts và excluded attempts.

## Phạm vi và mở thử

Feature giữ partial/validated supported slice; không chứng nhận toàn bộ Matrix solver, associative History hay Box Builder/Styles. Bộ mnemonic/CMD–Builder được hồi quy bằng consumer hiện có. Skill Box nguồn và bản cài đã đồng bộ theo approval cập nhật skill và correction tiếp theo của cùng người dùng; Sphere/Cube và các giới hạn còn lại được giữ.

Đã mở `OM9 — Box mặc định theo video (GPU)`, PID28188, focused/visible/ready=true, đúng module và prompt First corner. Người dùng có thể test default ngay hoặc chọn D/3/V/C; có đường tròn mẫu Rebuild. Artifacts ở `C:/Users/nguye/AppData/Local/Temp/om9-box-video-reference-20261012`; actual clean preview: `visual01/height-screen.png`.
