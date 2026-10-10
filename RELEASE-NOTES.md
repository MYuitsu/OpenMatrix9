## English

Experimental **Windows x64 / official FreeCAD 1.1.4** release.

- **SetupLauncher ZIP (recommended):** extract everything, run `Start-OM9.cmd`.
  Installs/updates OM9 in AppData, creates Desktop and Start Menu icons, opens
  OM9 and checks for updates in the background.
- **FreeCAD plugin ZIP:** manual installation in
  `%APPDATA%\FreeCAD\v1-1\Mod\OpenMatrix9`.
- Latest launcher includes startup UI cleanup; installer preserves user settings
  and previous plugin backups. Save and close FreeCAD before replacing a loaded plugin.
- Both ZIPs are compiled and packaged by GitHub Actions from this release's public
  source, with native/Rust/launcher tests, shared **SHA256SUMS.txt** and signed build
  provenance. [Build runs](https://github.com/MYuitsu/OpenMatrix9/actions/workflows/release-windows.yml)
  · [Verify origin](https://github.com/MYuitsu/OpenMatrix9/blob/v0.0.3/tools/ci/README.md).

[Installation](https://github.com/MYuitsu/OpenMatrix9/blob/v0.0.3/README.md)
· [Official FreeCAD download](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4).
Experimental: unfinished commands remain disabled. Back up models; provenance
does not guarantee antivirus acceptance or complete Matrix/Rhino compatibility.

## Tiếng Việt

Bản thử nghiệm **Windows x64 / FreeCAD 1.1.4 chính thức**.

- **ZIP SetupLauncher (khuyến nghị):** giải nén toàn bộ, chạy `Start-OM9.cmd`.
  Tự cài/cập nhật OM9 vào AppData, tạo icon Desktop và Start Menu, mở OM9 và
  kiểm tra phiên bản mới ở tiến trình nền.
- **ZIP plugin FreeCAD:** cài thủ công vào
  `%APPDATA%\FreeCAD\v1-1\Mod\OpenMatrix9`.
- Launcher mới dọn UI lúc khởi động; giữ cấu hình và bản sao plugin cũ khi cập nhật.
  Lưu bản vẽ và đóng FreeCAD trước khi thay plugin đang được nạp.
- Hai ZIP được GitHub Actions build và đóng gói từ source công khai của tag này,
  có kiểm tra native/Rust/launcher, **SHA256SUMS.txt** dùng chung và chứng thực
  nguồn build. [Xem lần build](https://github.com/MYuitsu/OpenMatrix9/actions/workflows/release-windows.yml)
  · [Kiểm tra nguồn gốc](https://github.com/MYuitsu/OpenMatrix9/blob/v0.0.3/tools/ci/README.md).

[Hướng dẫn cài](https://github.com/MYuitsu/OpenMatrix9/blob/v0.0.3/README.md)
· [Tải FreeCAD chính thức](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4).
Bản thử nghiệm: lệnh chưa hoàn thiện vẫn bị vô hiệu hóa. Hãy sao lưu bản vẽ;
chứng thực không bảo đảm antivirus chấp nhận hoặc toàn bộ tương thích Matrix/Rhino.
