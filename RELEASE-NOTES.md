# OpenMatrix9 0.3.5-202610121520 — Gumball chọn nhiều object

Sửa lỗi Gumball biến mất khi chọn nhiều object ở **Object Alignment**.
Giữ **Shift** rồi click thêm object, hoặc kéo khung chọn; cả selection dùng
một Gumball chung trong bốn view, không cần tạo Group.

Move dịch cả cụm; Rotate xoay cả cụm quanh tâm chung; Scale đổi kích thước
và khoảng cách tới tâm chung. Một Ctrl+Z hoàn tác cả batch.
Object Alignment nhiều whole object dùng trục CPlane active và tâm bounds
của cả selection. Frame của một object và các guard component vẫn được giữ.

## Cài đặt

1. Dùng [FreeCAD1.1.4 Windows x64](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4).
2. Lưu dự án và đóng FreeCAD. Tải **OpenMatrix9-0.3.5-202610121520-SetupLauncher-Windows-x64.zip** ở Assets, giải nén.
3. Chạy **Start-OM9.cmd** rồi dùng icon **OpenMatrix9** trên Desktop.

Giữ OpenMatrix9, launcher-manifest.json và các file Start-OM9 cùng nhau.
ZIP FreeCAD-1.1.4 dành cho cài thủ công; SHA256SUMS.txt kiểm tra cả hai ZIP.
[Hướng dẫn tiếng Việt](https://github.com/MYuitsu/OpenMatrix9/blob/main/INSTALL.vi.md).

## Kiểm tra

Tại máy: **848 Rust tests**, **31 tiến trình native/1.099 kiểm tra**, gồm
**143 kiểm tra mới** cho chọn nhiều object, MSR ở từng view, Undo/Redo,
Relocate, Esc và component rejection. [Bằng chứng và giới hạn](https://github.com/MYuitsu/OpenMatrix9/blob/main/docs/validation/2026-10-12-gumball-multi-selection.md).
GitHub Actions **chỉ build và đóng gói**; gói ghi commit/SHA256,
`validation_kind: build_only`, `tests_executed: false`.

Version tăng cc **4→5** cho task sửa lỗi; aa/bb giữ0/3, timestamp Asia/Saigon.
Đây là **Pre-release**. Giới hạn host/dependency đã ghi trước vẫn áp dụng;
full Rhino/Matrix/P0, Clayoo/SubD và SmartTargets ngoài phạm vi.
