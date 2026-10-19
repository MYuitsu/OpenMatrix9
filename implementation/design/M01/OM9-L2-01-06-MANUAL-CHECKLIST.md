# Checklist test tay — M01-L2-01-06 (9 L3)

Mỗi mục tương ứng một L3 khai `requires_manual_test = true`. Chỉ cần nhìn và ghi "đạt/không".

| L3 | Nhìn gì | Đạt khi |
| --- | --- | --- |
| 001 GPU device and render loop | mở app, khung hình đầu tiên | không nhấp nháy, không màn hình đen |
| 002 Shared graphics resources | 4 viewport cùng nội dung | cùng mesh, cùng màu, không lệch |
| 003 Tessellation cache | đổi tham số sphere | mesh dựng lại, hình giống hệt, không giật |
| 004 Wireframe | bật wireframe | cạnh sắc, không z-fight, không nhấp nháy |
| 005 Shaded | nhìn sphere | tròn, có sáng tối, không phẳng một màu |
| 006 Grid + clipping | xoay camera, phóng to | lưới nằm trên mặt đất, không rung khi orbit |
| 007 Highlight | chọn một object | xanh lá rõ, không lẫn màu nền |
| 008 Invalidation | để yên cửa sổ | không redraw liên tục (CPU ~0) |
| 009 Device loss | (tuỳ chọn) reset driver | app không đóng, khung hình tự vẽ lại |

Ghi chú: phần tự động của cả 9 L3 đã xanh (`cargo test -p om9-render`, 11 test).

## Điều khiển chuột (đã nối)

| Thao tác | Kết quả |
| --- | --- |
| Kéo chuột **phải** trong **Perspective** | quay camera quanh tâm (orbit), giữ khoảng cách |
| Kéo chuột **phải** trong **view trực giao** | pan (orbit ở view trực giao không có bậc tự do hữu ích vì elevation đã ở cực) |
| Kéo chuột **phải** + **Shift**, hoặc kéo **chuột giữa** | dịch (pan) theo đúng tỉ lệ pixel→world của view |
| **Lăn chuột** trong viewport | zoom (dolly); view trực giao zoom theo `vertical_span_mm` |
| **Click trái** | chọn / nhập điểm như trước |

Camera mới được ghi vào workspace nên giữ nguyên khi đổi view hoặc mở lại panel; khung hình GPU của
chính frame đó dùng camera mới (cập nhật trước khi vẽ).

## Vì sao phải chạy release

`tools\run-gpu-sphere.cmd` build **release**. Bản debug của egui + wgpu chậm hơn nhiều lần và làm thao tác
kéo camera giật; bản release trên RTX 3060 vẽ 4 viewport mượt. Lần build release đầu mất khoảng 5 phút,
các lần sau gần như tức thì.

Khi camera không đổi (không kéo, không lăn chuột), `GpuViewport` **bỏ qua** pass GPU và dùng lại texture
đã render — nên app đứng yên không tốn GPU.
