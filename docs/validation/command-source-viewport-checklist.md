# Danh sách kiểm tra nguồn chính xác và viewport theo lệnh

Cập nhật 2026-11-16. Danh sách sống, dùng cùng
[hợp đồng đã được người dùng duyệt](../../skills/openmatrix9-workflow/references/source-and-viewport-contract.md).
Mỗi lần được giao một lệnh, rà dòng cũ và code hiện tại trước khi bổ sung;
giữ evidence cũ có phạm vi, cập nhật mode/route và kết quả mới vào chính dòng.

CHUA_RA = chưa rà đủ theo hợp đồng này; THIEU_CAN_BO_SUNG = đã xác định thiếu
ở route cụ thể; MOT_PHAN = có triển khai nhưng còn phần thiếu hoặc kiểm chứng;
DA_XAC_MINH = đủ hạng mục áp dụng trong phạm vi ghi rõ. KHONG_AP_DUNG dành
cho từng hạng mục có căn cứ. `[x]` bên dưới đánh dấu phần đã triển khai để
tái dùng, **không** thay thế trạng thái nghiệm thu cả lệnh.

## Phần có sẵn đã đối chiếu

| Lệnh / ID runtime | Đã triển khai, cần tái dùng | Còn phải bổ sung hoặc kiểm chứng | Trạng thái theo hợp đồng mới | Evidence |
|---|---|---|---|---|
| Boolean Union — `OM9-SOLID-003` | [x] Worker nguồn chính xác; [x] mesh preview có giới hạn chạy song song; [x] preview là render tạm; [x] guard trước final source/scene commit với một Undo trong fixture đã kiểm | Native 512 High từng lỗi worker cuối; preview không tức thời; chưa đủ topology/placement/mode, áp lực, lifecycle và persistence tổng quát | MOT_PHAN; tái dùng pipeline hiện có, không viết lại vì tên gọi | [Code preview](../../crates/om9-app/src/session/boolean_union_worker/mesh_preview.rs), [start/poll/commit](../../crates/om9-app/src/session/boolean_union_worker.rs), [64/128 và 512](2026-11-15-speculative-boolean.md) |
| Sphere — `OM9-SOLID-014`, cảnh tạo cầu của audit | [x] Nguồn canonical riêng; [x] Light/High trên cùng nguồn; [x] publication từng cầu và giữ đủ Undo trong batch đã đo | Chưa rà/khẳng định có proxy mới trước khi nguồn exact của Sphere xong; chưa phủ mọi mode. Kiểm cảnh full Undo không chứng minh cold restore hoặc `.om9` v5 BRep round-trip | MOT_PHAN; preview trước exact = CHUA_RA, không tự kết luận đã thiếu hay đã đủ | [M1-R: 10 cấu hình, không Boolean](2026-11-16-m1r-storage-receipts.md), [profile](../../crates/om9-app/src/display.rs), [ownership hiện tại](2026-11-16-source-viewport-current-status.md) |

## Hàng chờ rà theo hợp đồng mới

Những lệnh này có code/bằng chứng cũ; CHUA_RA không có nghĩa chưa triển khai
lệnh. Chưa được nâng trạng thái từ việc dùng chung worker hoặc giống tên.

| Lệnh / ID runtime | Trạng thái | Điểm bắt đầu khi được giao |
|---|---|---|
| Box — `OM9-SOLID-012` | CHUA_RA | Nguồn và các mode primitive, proxy/preview, commit và Undo |
| Boolean Difference — `OM9-SOLID-001` | CHUA_RA | Worker nguồn, preview sớm độc lập, input/cutter và DeleteInput |
| Boolean Intersection — `OM9-SOLID-002` | CHUA_RA | Worker nguồn, preview sớm, kết quả rỗng và binding |
| Boolean Split — `SolidBooleanSplit` | CHUA_RA | Worker nhiều tập, preview sớm và một transaction cho kết quả |
| Boolean Two Objects — `OM9-SOLID-004` | CHUA_RA | Phân biệt preview năm mode bằng kernel với preview rẻ trước exact; cancel/stale và commit |

Các lệnh khác được thêm vào khi được giao, dùng ID/alias từ dispatcher và
milestone thật. Đây chưa phải toàn bộ danh mục; tổng số lệnh chưa rà chưa được
thống kê trong lượt cập nhật skill, không đưa tỷ lệ hoàn thành toàn ứng dụng.
Hiện hai dòng đã rà ban đầu đều MOT_PHAN; chưa có dòng DA_XAC_MINH theo toàn
bộ hợp đồng mới. Không làm lại các phần đã đánh dấu `[x]` chỉ vì còn mục mở.

Mỗi evidence mới cần phân biệt nguồn, preview, publication cuối và consumer
chịu ảnh hưởng; ghi kiểm chứng thực đã chạy và phần chưa chạy. Lỗi chung hiện
có như [guard `.om9` v5 BRep](../../crates/om9-archive/src/container.rs) được
ghi thành blocker khi tác động tới lệnh; không được tự đánh dấu Save/Open đạt.
