# OpenMatrix9 0.3.6-202610211407 — Standalone Windows x64

Phát hành ứng dụng standalone Rust với viewport GPU, điều hướng orbit/pan/zoom
và adapter OCCT/openNURBS đã pin. Gói portable chứa runtime DLL, adapter 3DM,
manifest SHA256, giấy phép và nguồn/bản vá OCCT TKG3d đi kèm.

Tải `OpenMatrix9-0.3.6-202610211407-Standalone-Windows-x64.zip` trong Assets,
giải nén toàn bộ thư mục rồi chạy `Start-OM9.cmd` hoặc `om9.exe`. Giữ các DLL,
adapter 3DM và tệp digest cạnh executable. Ứng dụng cần GPU tương thích wgpu.
Kiểm tra gói bằng `SHA256SUMS.txt` của cùng release.

Run main `39543107947` đã build thành công cả sản phẩm và native adapter.
Release mới được dựng lại từ commit/tag tương ứng; không đổi tên artifact cũ
để gán cho version mới. CI giữ `validation_kind: build_only`,
`tests_executed: false`; các gate UI, geometry và performance vẫn theo bằng chứng
riêng. Đây là pre-release. Giữ aa/bb ở 0/3, tăng patch 5→6; timestamp Asia/Saigon.

---

# OpenMatrix9 0.3.5-202610121520 — Gumball chọn nhiều object (lịch sử)

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
