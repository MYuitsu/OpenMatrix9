# Matrix9 menu — kết quả kiểm tra 2026-10-04

Sidebar đã chạy trong FreeCAD với Rust quản lý danh mục/trạng thái và C++/Qt quản lý giao diện cùng lệnh native. Có 7 khối, 18 nhóm, 11 nút nhanh, Reset, thu gọn/ẩn khối và lịch sử tối đa 20 lần thực thi thành công. Nhấp phải vùng sidebar → **Khôi phục các bảng** để lấy lại MAIN MENU sau khi đóng.

Workspace hiện có thêm hàng **All / None / Delete / Fit all / Fit sel / Views**. Chọn toàn bộ/bỏ chọn trong tài liệu đang mở, xóa qua FreeCAD, Undo/Redo và bảy hướng nhìn đã có phần triển khai native. Trạng thái nút tuân thủ tài liệu, selection, edit mode và bảng tác vụ. Xem `docs/validation/2026-10-04-workspace.md` để biết phạm vi và kết quả mới nhất.

![Nội dung sidebar thực tế](images/menu-full-sidebar.png)

Ảnh này lấy trực tiếp từ widget trong FreeCAD, gồm cả phần có thể cuộn. Theo yêu cầu mới nhất, workspace dùng bộ Matrix90 đã cộng RGB +5 trong `ref/matrix9/Matrix90/menu-images`. Bộ vẽ lại được giữ nguyên làm nguồn dự phòng; chính sách ưu tiên ảnh vẽ lại của lượt trước được thay thế cho workspace này.

## Mở bản đang kiểm tra

Chạy từ PowerShell, sau đó chọn **OpenMatrix9** trong danh sách workbench:

```powershell
$env:PATH='D:/FreeCAD-src/.pixi/envs/default/Library/bin;D:/FreeCAD-src/.pixi/envs/default;'+$env:PATH
& 'D:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe'
```

Module đã được copy vào `build/relWithDebInfo/bin/OpenMatrix9Gui.pyd`; InitGui và Resources nằm tại `build/relWithDebInfo/Mod/OpenMatrix9`. Không cần thư mục `ref` hoặc `OpenMatrix9_named_icon_crops` khi chạy bản phân phối.

## Tài nguyên icon

- **434/434 tên MAIN MENU dùng PNG RGB +5**, không còn fallback theo nhóm trong menu chính. Toàn tài nguyên510 tên gồm498 mapping từ497 ảnh đã đổi màu và12 biểu tượng tự viết cho chrome/cube Isometric. Các lệnh workspace mới có ảnh riêng trong ICON HISTORY; số tài nguyên không phải số chức năng đã triển khai.
- Ảnh được copy nguyên byte từ bộ đã đổi màu, không cộng thêm lần nữa. Manifest ghi record nguồn, hash hiện tại, hash PNG trước đổi màu, `rgb_delta=5` và mapping exact/alias. Đối chiếu toàn bộ PNG nguồn với tài nguyên đã phân phối.
- Bộ đọc từ chối sai delta/alpha/clamp, PNG hỏng, hash/kích thước không khớp, tên trùng và đường dẫn ra ngoài nguồn. Ảnh RGB +5 ưu tiên cao nhất; sau đó mới đến bộ vẽ lại và SVG. Không có PNG vẽ lại nào cần dùng trong danh mục hiện tại; thư mục người dùng không bị sửa.
- Bản phân phối chạy độc lập bằng `Resources/icons/rgb-plus5`, không cần ảnh tham chiếu, bản Matrix90 cài đặt hoặc ZIP backup. Bộ modern-first vẫn là mặc định khi không truyền `--shifted-root`; phải giữ flag dưới đây để xuất lại workspace hiện tại. Chỉ file cũ được INI trước đó khai báo mới bị dọn, có kiểm tra containment;214 ảnh managed cũ đã dọn khỏi SDK sau khi thay bộ.
- MAIN MENU có ba hàng chọn nhóm, nhãn trắng và chấm màu; DISPLAY có 6 toggle, 5 mode, 2 combo; SNAPS có hai hàng; INFO có 22 icon; LAYERS có 32 hàng và control màu/khóa/ẩn; PROJECTS có 3 ô và 5 nhóm. Icon disabled giữ nguyên màu, nhưng lệnh chưa port vẫn không thực thi được.

Xuất lại tài nguyên:

```powershell
python tools/export_menu_assets.py --ref-root ref --output-root Resources --named-root OpenMatrix9_named_icon_crops --named-bindings tools/named-icon-bindings.json --shifted-root ref/matrix9/Matrix90/menu-images
```

## Build và kiểm tra

Build module bằng MSVC x64, CMake/Ninja, Qt 6.11.2, Python 3.13.15 và thư viện SDK FreeCAD hiện có. Đây là build mới của module, sử dụng FreeCAD.exe/DLL hiện có; không phải build lại toàn bộ FreeCAD. Cache cũ trỏ ổ E được giữ nguyên. Header FreeCAD yêu cầu C++23; phần catalog/state vẫn là Rust edition 2024.

Trong Developer Command Prompt x64:

```bat
D:\FreeCAD-src\.pixi\envs\default\Library\bin\cmake.exe -S D:\FreeCAD-src\Mod\OpenMatrix9 -B D:\FreeCAD-src\build\openmatrix9 -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_MAKE_PROGRAM=D:/FreeCAD-src/.pixi/envs/default/Library/bin/ninja.exe -DCMAKE_PREFIX_PATH=D:/FreeCAD-src/.pixi/envs/default/Library -DFREECAD_SOURCE_DIR=D:/FreeCAD-src -DFREECAD_SDK_BUILD=D:/FreeCAD-src/build/relWithDebInfo -DFREECAD_DEPENDENCY_PREFIX=D:/FreeCAD-src/.pixi/envs/default/Library
D:\FreeCAD-src\.pixi\envs\default\Library\bin\cmake.exe --build D:\FreeCAD-src\build\openmatrix9 --target OpenMatrix9Gui
```

Build cùng parent FreeCAD dùng option `BUILD_OPENMATRIX9=ON`. Patch `src/Mod/CMakeLists.txt` đã áp dụng trên checkout cha và lưu trong `cmake/freecad-integration.patch`; repo private chỉ lưu bản patch. Chế độ build parent đầy đủ chưa được chạy.

Đã kiểm tra:

- Rust: 7 tests; fmt, clippy `-D warnings`, release build.
- Python:24 tests, thêm kiểm tra RGB +5 ưu tiên trước ảnh vẽ lại/kho gốc, copy nguyên byte và từ chối delta/hash/path sai. Hai test mới đã thất bại trước khi thêm importer, rồi passed.
- QtTest: 8 test cases + init/cleanup, tổng 10 passed; đối chiếu pixel SNAPS/DISPLAY với tài nguyên được chọn, kiểm tra full color ở Disabled và ngăn thực thi/history. Có cảnh báo font/plugin trong chế độ offscreen; không có test thất bại.
- FreeCAD thật: New, Open, Save As, Save với đối tượng chưa recompute; cancel Open/Save As không tăng lịch sử; alias native cập nhật lịch sử; chuyển workbench 3 lần không nhân sidebar; khôi phục MAIN MENU; cuộn đến Projects trong cửa sổ 1024×600.
- Chạy các scale `QT_SCALE_FACTOR` 1 / 1.25 / 1.5 / 2 với profile test riêng,31 checks mỗi lượt. Xác nhận mọi mục MAIN MENU/nút nhanh dùng PNG RGB +5, pixel render khớp ảnh đã phân phối, hash nguồn đúng và disabled giữ màu; số control/no-wrap ở chiều rộng startup vẫn đúng. Không thay Windows DPI hoặc cấu hình cá nhân. Bằng chứng nằm trong `docs/validation/menu-runtime.json`.

Runner: `./tests/run_menu_smoke.ps1 -Scale 1.25`. Mỗi lượt có thư mục GUID riêng nên không dùng nhầm kết quả cũ. Hộp thoại test dùng backend Qt của FreeCAD qua preference trong profile riêng. Backend Win32 native và trường hợp Save As vào file không ghi được chưa có kiểm tra tương tác riêng; xác nhận lưu nay dựa trên `signalFinishSaveDocument`, không suy đoán từ FileName/isTouched.

## Phần tiếp theo

Bốn viewport, vùng Command màu xanh và thuật toán trang sức chưa thuộc lần triển khai menu này. DISPLAY/SNAPS/INFO/LAYERS/PROJECTS hiện là control trình bày disabled; binding FreeCAD đã xác minh trong menu vẫn hoạt động. Đủ ảnh menu không đồng nghĩa chức năng đã được port.11 biểu tượng chrome tự viết vẫn được giữ vì không có record ảnh tương ứng. Cần tiếp tục bố cục còn lại và port từng chức năng.

Ảnh sidebar được chụp bằng `QDockWidget.grab()` trong FreeCAD thật và đã kiểm tra trực quan. Computer-use đọc được cửa sổ Matrix9; lần mở FreeCAD qua Sky chưa trả về cửa sổ có thể điều khiển, nên chưa nhận đã kiểm tra UI tương tác qua Sky. Ảnh toàn cửa sổ trong lượt test ẩn có lỗi vùng OpenGL; chỉ ảnh sidebar được bàn giao, viewport chưa được xác nhận hiển thị đúng.
