# Box — quy trình theo tài liệu, 2026-10-12

**Superseded for the default Box workflow:** the user clarified the video shows default Box, with optional methods. See [current correction](2026-10-12-box-default-video.md). Earlier counts remain historical.


Sửa theo yêu cầu trực tiếp của người dùng: bỏ Box thường/Corners và bắt buộc chọn phương pháp. Đây là correction thay thế nghiệp vụ Corners/default và side Length tại endpoint trong các báo cáo trước; không thay đổi tài liệu tham chiếu gốc.

## Hành vi hiện tại

- Box mở ở `Box. Choose a method ( Diagonal 3Point Vertical Center ):`.
- `3Point`: chọn đầu cạnh, chọn cuối cạnh, pick/type Width, pick/type Height. `3` hoặc click chọn phương pháp. Điểm cuối phải là tọa độ/point; `10`, `10mm`, `Length=10` bị từ chối ở bước này.
- `Vertical`: hai đầu cạnh trong CPlane đã chụp; Width dọc pháp tuyến của CPlane để đáy vuông góc CPlane. Height theo pháp tuyến đáy đứng.
- `Center`: chọn tâm, chọn góc đối diện hoặc nhập Length toàn phần. Đường nhập số tiếp tục Width rồi Height; góc pick xác định hai extent tâm–góc và nhân đôi.
- `Diagonal`/`Cube` giữ solver đã có. `Corners` và `Mode=Corners` không còn là tùy chọn tương tác. Giá trị enum legacy chỉ giữ tương thích dữ liệu/capability cũ.
- `0` chỉ được nhận làm điểm gốc CPlane tại bước nhập điểm sau khi chọn phương pháp. Chưa chọn phương pháp thì `0` không tạo checkpoint/hình và giữ draft để sửa.
- Width Enter dùng Length; Height Enter dùng Width. Phím tắt một ký tự, click cả tên, focus CMD, lịch sử, invalid draft, input Undo, native Undo/Redo và FCStd vẫn giữ.

Ví dụ hợp lệ: `Box → 3 → 0 → 10,0,0 → 20 → 10`; hoặc `Box → C → 0 → 10 → 20 → 10`. Trong 3Point, `10` không thay cho `10,0,0`. Right-click icon chọn 3Point trực tiếp.

## Thực thi và bằng chứng

Rust sở hữu chọn phương pháp, điều kiện point/scalar, state history, construction frame/math và capability/preview guards. Không đổi public ABI. C++ chỉ sửa bridge khi callback native thay thế lệnh đang publication: lỗi cũ vào history, bỏ draft cũ, giữ prompt lệnh mới; lệnh Curve thay thế cũng được kiểm tra.

Full Rust: **1042/1042**, 135 suites, exit 0. Native cuối cùng: **1195/1195**, **17 hosts exit 0**, cùng plugin SHA `200bbfd573ea96562bcfcbebc1e836d877e31557c35106e48a8eebcb73ac0901`. Fixture mới có 31 checks gồm click, mnemonic, actual viewport picks, native geometry, Undo/Redo và reload. Callback fixture tái hiện hai lỗi replacement trước sửa rồi qua trên bản cuối. Rà soát độc lập đã kiểm state/mode/Undo/Center và bridge callback.

| Fixture native | Checks đạt | Exit |
|---|---:|---:|
| rhino_core_solid_callback | 156 | 0 |
| rhino_core_solid_publication | 86 | 0 |
| rhino_core_solid_construction | 67 | 0 |
| box_documented_workflow | 31 | 0 |
| box_command_options | 26 | 0 |
| box_command_usability | 17 | 0 |
| box_command_history | 11 | 0 |
| solid_commands | 71 | 0 |
| solid_options | 252 | 0 |
| rhino_core_native_solid_binding | 88 | 0 |
| command_mnemonics | 181 | 0 |
| builder_command_options | 25 | 0 |
| rebuild_command_options | 15 | 0 |
| command_line_options | 26 | 0 |
| core_invalid_cplane_tools | 14 | 0 |
| core_extended_input | 58 | 0 |
| layer_panel | 71 | 0 |

Chi tiết hash, nguồn, App/Gui đã nạp, stdout và artifact paths: [machine evidence](2026-10-12-box-documented-workflow.json). Artifacts nằm trên C: trong `C:/Users/nguye/AppData/Local/Temp/om9-box-documented-20261012`; mô hình native FCStd được giữ ở đó. Các lần RED, lỗi setup thiếu environment và assertion prompt cũ bị loại khỏi tổng cuối; không bỏ kiểm hình học/state để đạt xanh.

Build plugin qua `build/build-om9-plugin.ps1`, dùng cache duy nhất `build/openmatrix9-history-cage`, Rust target/runtime trên C:, SDK chung `build/relWithDebInfo`. App299 đã có inputs/source snapshot; hoàn tất lại compile-object/link đã cấu hình, quan sát exit0, dùng cặp DLL/import cùng thư mục và validator 190 source rows. Không reconfigure root hoặc copy runtime FreeCAD. SHA/nguồn là evidence của candidate, không chứng chỉ ABI/compiler đầy đủ. Không claim clean logs: legacy fault fixtures có warning khi trực tiếp clear transcript và các fault tạo diagnostic dự kiến.

## Phạm vi còn mở

Feature giữ `partially_implemented / validated_supported_slice`. Associative History, đầy đủ Box Builder/Styles, solver Matrix riêng và toàn bộ foundation acceptance chưa được chứng nhận bởi correction này. Đồng bộ CMD–Builder được hồi quy bằng consumer Builder/Rebuild hiện có, không claim Box có panel Builder mới.

Người dùng đã đồng ý “Có, cập nhật skill”. Đã lưu quy tắc Box vào `skills/openmatrix9-feature-port/references/solid-box-sphere-contract.md` và đồng bộ bản cài trong `C:/Users/nguye/.codex/skills/openmatrix9-feature-port/references/solid-box-sphere-contract.md`: bắt buộc chọn phương pháp, bỏ Corners, hai endpoint cho 3Point/Vertical và hai đường nhập Center. Hash hai bản khớp; chỉ sửa phần hợp đồng Box, giữ phần Sphere/Cube và giới hạn hiện có.

## Cửa sổ native để người dùng thử

Đã mở `OM9 — Box theo tài liệu`, PID81336, plugin đúng SHA của 17 fixtures cuối, `ready/visible/focused=true`, prompt bắt buộc chọn phương pháp. Có đường tròn mẫu Rebuild; các lệnh Box/Sphere/Circle/Rebuild vẫn thử được trong cùng cửa sổ. Receipt và ảnh ở `manual01/`; cửa sổ giữ mở cho người dùng.
