# OpenMatrix9 0.3.4-202610120606 — Gumball

Hoàn tất ba specs Gumball lõi **OM9-INFO-020/021/022** trong phạm vi đã duyệt:
Scale object tham số, Object Alignment và Relocate Gumball. Scale giữ kích thước,
expression và dependency được hỗ trợ. Bật bằng lệnh **Gumball** trong Command;
bốn viewport dùng chung frame.

## Cài đặt trong 3 bước

1. Cài [FreeCAD 1.1.4 chính thức](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4) trên Windows x64.
2. Tải **OpenMatrix9-0.3.4-202610120606-SetupLauncher-Windows-x64.zip** ở **Assets** bên dưới, rồi giải nén.
3. Chạy **Start-OM9.cmd**. Launcher cài OM9, tạo icon Desktop và mở workbench.

Giữ thư mục OpenMatrix9, launcher-manifest.json và các file Start-OM9 cùng nhau
trong thư mục đã giải nén. ZIP FreeCAD-1.1.4 dành cho cài thủ công;
**SHA256SUMS.txt** kiểm tra cả hai ZIP.

[Hướng dẫn tiếng Việt](https://github.com/MYuitsu/OpenMatrix9/blob/main/INSTALL.vi.md) · [English installation](https://github.com/MYuitsu/OpenMatrix9/blob/main/INSTALL.en.md).

## Kiểm tra và build

Bản đã hợp vào **main** giữ các thay đổi Layers, clipboard và native geometry
của hai nhánh. Kiểm thử cục bộ: **30 ca Gumball / 955 kiểm tra**,
**46 tiến trình native / 1564 kiểm tra tổng cộng**,
**846 Rust tests**. [Phạm vi, hash và giới hạn host](https://github.com/MYuitsu/OpenMatrix9/blob/main/docs/validation/2026-10-12-gumball-main-release.md).

Theo yêu cầu, GitHub Actions **chỉ build và đóng gói**. Gói ghi đúng commit,
SHA256 và `validation_kind: build_only`, `tests_executed: false`; kết quả test
cục bộ không được gán cho DLL CI. Workflow kiểm tag/version và commit trên main
trước khi tạo GitHub Release.

Version bắt đầu từ 0.0.3: **aa giữ 0**, **bb tăng 3 specs**, **cc tăng 1 task**;
suffix dùng Asia/Saigon. Giữ cùng version cho các sửa trong tác vụ phát hành này.
Đây là bản thử **Pre-release**. Full P0/core catalog, Clayoo/SubD, SmartTargets
và tương thích tuyệt đối với Rhino/Matrix nằm ngoài phạm vi nghiệm thu này.
