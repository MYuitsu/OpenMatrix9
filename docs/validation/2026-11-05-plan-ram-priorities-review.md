# Rút gọn plan và ưu tiên RAM — 2026-11-05

Theo yêu cầu người dùng: bỏ phần đã xong khỏi việc cần triển khai, bổ sung phần RAM chưa hợp lý. Đây là thay đổi tài liệu, không triển khai thêm runtime, không chạy Rust/GPU/release/desktop mới và không thao tác phiên OM9 đang mở. Không đổi nghiệp vụ hoặc skill; không nâng trạng thái hoàn tất milestone.

## Đối chiếu và phần đã bỏ khỏi checklist

Đọc plan/alias, live ledger và các báo cáo Undo, tương tác >256 MiB, phase Save/STL, picking pressure/recency, account roots, diagnostics/refusal và presentation Weak. Các kết quả trước giữ nguyên phạm vi; bảng hoàn tất còn chín nhóm kèm liên kết, thay các dòng và đoạn lặp. Bằng chứng gốc không bị xóa. Gumball giữ xác nhận đã hết lỗi của người dùng.

Các phần không giao lại: cấu hình Undo 1024 MiB/50%/kiểm kê 100 MiB, đường tương tác đã chứng minh >256 MiB, buffer/metadata ownership đã kiểm thử, pressure picking, bỏ toàn-headroom ở Save/STL, diagnostics và final external-refusal. Không suy ra cache display đã độc lập, v5/GPU batching/UI export/checkpoint đã xong hoặc native đã có trần cứng.

## RAM: thiếu sót xác định trực tiếp từ code

- `session/stl.rs::capture_stl_selected_with_sample` giữ `scene_auxiliary_bytes` tại thời điểm capture; detached export cộng với `scene_high_water`. Đây không phải inventory auxiliary sống khi UI tiếp tục chỉnh sửa. Plan giao R1a kiểm owner thật, tăng/giảm auxiliary và old/new overlap; chưa gọi mọi trường hợp là lỗi đã tái hiện.
- Cũng tại capture, coordinator rỗng được dựng rồi `adopt_external_captures`; `transaction/cache_pressure.rs` đã có `memory_admission_peer`. Plan yêu cầu đối chiếu/tái dùng khi cùng hợp đồng, không tự refactor hoặc khẳng định peer hiện tại sai.
- `session/layers.rs::auxiliary_usage` liệt kê selection/order/subselection, pending solid, workspace và strings; có dữ liệu thay đổi sau capture. Không quét history/scene mỗi frame để bù cho owner thiếu.
- Báo cáo STL phase ghi rõ callback Rust nhận lengths sau khi native tạo kết quả; bootstrap một byte không là native envelope. Giữ đo sphere đã có, thêm đo conversion/mesher/result/copy overlap và tác vụ đại diện. Estimate không thành upper bound; hủy không trả lease trước native return.
- Undo 1024 MiB là hạn mức lịch sử bên trong managed 60%, không là ngân sách toàn app hoặc RAM đã cấp. Physical Available và Windows commit là hai điều kiện độc lập. Allocation sống khi trần thay đổi vẫn phải giữ charge; chỉ từ chối tăng trưởng mới không vừa và thu hồi cache được phép.

## Thứ tự và cổng kết thúc

R1a capture/auxiliary/peer → R2a-STL native/scratch → P3b worker UI; giữ các guard chính xác hiện có. Việc chuẩn bị UI không phải đợi kiểm toán mọi metadata/codec hoặc đo hết Boolean/fillet/loft. P1 và gate chính xác P3a tiếp tục theo phụ thuộc; P3a vẫn phải đạt trước tuyên bố chứng nhận sai số toàn miền. R1b/R2b chia theo caller/tác vụ với gate riêng; chưa xong thì toàn P0 chưa được công nhận hoàn tất. Không thu hẹp phạm vi P1–P5, 3DM hoặc AFTER Mesh Repair.

Mỗi bàn giao phải có: đường production cụ thể, thiếu sót đã chứng minh, thay đổi cần thiết, oracle kết thúc và giới hạn. Dùng tests liên quan; chỉ mở rộng khi có phụ thuộc/lỗi mới hoặc đến cổng tích hợp. Không dùng tổng số tests hay thời gian trước để dự báo hoàn thành.

## Kiểm tra lần sửa tài liệu

Kiểm tra đạt: 35 local Markdown links trong plan/alias/biên bản; JSON ledger parse được, entry lần sửa duy nhất, các ràng buộc và mục còn lại được giữ. `tools/public_source_audit.py` exit 0: 15.188 source files, 594 icon bindings, 593 SVG, errors rỗng. `git diff --check` trong scope đạt; các tài liệu mới cũng được kiểm tra whitespace trực tiếp. Không tạo batch runtime hoặc batch đo openNURBS/3DM mới; không có batch build mới cần prune. Packages exchange chưa đóng, tổng batch tương lai chưa xác định.

[Plan chính thức](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md), [alias](../../implementation/design/M01/viewport-export-separation-20261102/PLAN-REVISED.md). Bước triển khai tiếp theo có phạm vi là R1a; báo cáo này tự nó không triển khai R1a.
