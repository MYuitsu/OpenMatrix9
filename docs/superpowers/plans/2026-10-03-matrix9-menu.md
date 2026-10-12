# Matrix9 Rust Menu Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Hiển thị sidebar Matrix9 trong FreeCAD, với MAIN MENU và ICON HISTORY hoạt động, dữ liệu và trạng thái do Rust quản lý.

**Architecture:** Rust đọc danh mục, ánh xạ lệnh và quản lý trạng thái qua ABI C. C++/Qt dựng sidebar, tải icon và gọi lệnh FreeCAD. Công cụ xuất tài nguyên chạy ngoại tuyến; bản cài không cần `ref`.

**Tech Stack:** Rust edition 2024, std, C++/Qt 6 hiện có trong FreeCAD, CMake/Ninja, Python stdlib cho công cụ chuyển đổi tài nguyên ngoại tuyến.

**Spec:** `docs/superpowers/specs/2026-10-03-matrix9-menu-design.md`

## Global Constraints

- Giữ crate `openmatrix9_rust`, edition 2024, loại `staticlib`.
- Không bổ sung framework giao diện Rust độc lập hoặc thay Qt của FreeCAD. Không chuyển logic menu sang Python.
- Một sidebar OpenMatrix9 bên trái, rộng mặc định 300 pixel logic Qt.
- Bảy khối theo thứ tự: ICON HISTORY, MAIN MENU, DISPLAY, SNAPS, INFO & SETTINGS, LAYERS, PROJECTS.
- MAIN MENU hiển thị 18 nhóm và 11 nút nhanh; tên và thứ tự icon khớp INI.
- ICON HISTORY hiển thị tối đa 20 lần thực thi lệnh thành công trong phiên, lần mới nhất trước.
- Nền sidebar `#333333`; ICON HISTORY `#646464`; thanh tiêu đề `#A19F94`; font ưu tiên Verdana 8.25 pt; icon 24 pixel trong nút khoảng 25 pixel logic.
- Không dùng tên lệnh Rhino như lệnh FreeCAD và không chạy chuỗi macro Rhino.
- Bản cài chạy được khi không có `ref`, không phụ thuộc đường dẫn tuyệt đối trên máy hiện tại.
- Giữ nguyên bản xuất của người dùng. Các thay đổi sẵn có trong `ref` và ba file danh sách bị di chuyển không được đưa vào commit triển khai.
- Viewport, vùng Command và thuật toán trang sức thuộc giai đoạn sau.

## Review Focus

1. Kích hoạt workbench lặp lại: một sidebar, không có signal trùng và khôi phục dock đúng (Task 4).
2. Chưa có tài liệu hoặc đóng tài liệu trong khi menu đang mở: availability cập nhật, không gọi con trỏ tài liệu hết hạn (Task 4).
3. Thiếu icon, macro không ánh xạ được hoặc icon lệch atlas: fallback rõ ràng và báo cáo đúng, không gán hình gần giống theo phỏng đoán (Task 2).
4. Chỉ số ABI không hợp lệ và chuỗi được dùng sau khi cập nhật trạng thái: trả về lỗi và sao chép chuỗi đúng vòng đời (Task 3).
5. Cửa sổ thấp và DPI cao: vẫn truy cập hết nhóm/nút, không ép dock che viewport (Task 5).

## Hiện trạng cần xử lý

- `rust/src/commands.rs` hiện chứa C++; `lib.rs` thiếu module/import và phần lớn symbol mà `Gui/RustBridge.h` khai báo.
- Module nằm trong `Mod/OpenMatrix9`, nhưng FreeCAD chỉ thêm module qua `src/Mod/CMakeLists.txt`; target OpenMatrix9Gui chưa có trong build hiện tại.
- `build/relWithDebInfo/CMakeCache.txt` còn đường dẫn `E:/FreeCAD-src`. Không sửa hàng loạt hoặc xóa bản build đó; dùng build mới `D:/FreeCAD-src/build/openmatrix9` với dependency prefix `D:/FreeCAD-src/.pixi/envs/default/Library` và môi trường MSVC x64.
- CMake/Ninja hiện có trong `.pixi/envs/default/Library/bin`. Đường dẫn tương đối trong các lệnh phía dưới tính từ repo OpenMatrix9 trừ khi ghi khác.

## Task 1: Danh mục menu và trạng thái Rust

**Files:** tạo `rust/src/menu.rs`, `rust/src/state.rs`, `rust/tests/menu_state.rs`, `Resources/menu/MainMenu.ini`; sửa `rust/Cargo.toml`, `rust/src/lib.rs`, `rust/src/commands.rs`, `rust/src/workbench.rs`, `rust/src/toolbar.rs`.

**Interfaces:** `MenuCatalog::parse(text: &str) -> Result<MenuCatalog, MenuError>`; catalog có `groups: Vec<MenuGroup>` và `quick_icons: Vec<String>`; nhóm có `id: usize`, `title: String`, `kind: MenuKind`, `color: String`, `icons: Vec<String>`. `MenuKind` gồm IconList, Custom, Reset. `UiState::new()`, `select_group(index: usize, group_count: usize) -> bool`, `set_section(index: usize, visible: bool, collapsed: bool) -> bool`, `reset()` và `record_execution(command: usize, success: bool)`; state lưu nhóm được chọn, visibility/collapse của 7 khối, history tối đa 20.

- [x] Viết test `reference_catalog_has_18_groups_and_11_quick_icons`: assert nhóm đầu File, nhóm cuối Render, nhóm Custom và Reset không cần IconCount; các icon File theo đúng INI, quick icon đầu TopIconDuplicate và cuối TopIconRingRail. Viết test BOM/CRLF/UTF-8, khóa trùng, MenuCount/IconCount sai, icon thiếu và group không hợp lệ; lỗi chứa section và key.
- [x] Viết test state: chọn nhóm hợp lệ; từ chối index ngoài phạm vi; thu gọn/ẩn độc lập 7 khối; reset khôi phục nhóm 0 và hiện các khối; 21 lần thành công giữ 20 mới nhất; `success=false` không thêm history. Chạy `cargo test --manifest-path rust/Cargo.toml --test menu_state`; xác nhận lỗi baseline rồi lỗi test có ý nghĩa khi crate biên dịch được.
- [x] Thay mã C++ trong `.rs` bằng Rust; đọc danh mục nhúng bằng `include_str!` từ `Resources/menu/MainMenu.ini`. Giữ danh mục độc lập Qt. Bổ sung `rlib` cạnh `staticlib` để integration test import crate được, vẫn cung cấp static library cho C++. Sửa import/module và nhóm toolbar dùng command hợp lệ, không mô phỏng thuật toán trang sức bằng thông báo thành công.
- [x] Chạy test trên, `cargo fmt --manifest-path rust/Cargo.toml --check` và `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; commit riêng Task 1.

## Task 2: Tài nguyên menu và icon có truy nguồn

**Files:** tạo `tools/export_menu_assets.py`, `tools/tests/test_export_menu_assets.py`, `Resources/menu/icons.ini`, `Resources/menu/source-manifest.json`, `Resources/icons/`; bổ sung quy tắc tài nguyên vào `CMakeLists.txt` của module.

**Interfaces:** `export_assets(ref_root: Path, output_root: Path) -> dict`; `read_rui(path: Path) -> dict` trả về macro/bitmap mapping; `read_form(path: Path) -> dict` trả về control cha, thuộc tính và tham chiếu tài nguyên. `icons.ini` có group theo tên icon, trường `atlas`, `x`, `y`, `width`, `height`, `tooltip`, `source`, `status`; status là resolved hoặc missing. Manifest chứa SHA256 nguồn, danh sách resolved/missing và nguyên nhân. Đường dẫn atlas/source là tương đối, không ghi đường dẫn tuyệt đối của máy.

- [x] Viết unittest: ánh xạ macro qua GUID tới bitmap_item/index; dữ liệu PNG base64 hợp lệ; kích thước và rectangle nằm trong atlas; tên macro không trùng tên INI được đánh dấu missing; không có GUID/bitmap hoặc base64 hỏng có lỗi rõ. Test form `.123` như UserControl, giữ parent và tham chiếu `.ctx`; test exporter không thay đổi SHA256 nguồn. Chạy `python -m unittest discover -s tools/tests -v`, xác nhận FAIL trước implementation.
- [x] Implement exporter bằng stdlib `configparser`, `xml.etree.ElementTree`, `base64`, `hashlib`, `struct`. Lưu atlas PNG thay vì phụ thuộc Pillow để cắt; C++/Qt cắt bằng rectangle. Chỉ ánh xạ tên/script xác nhận được; OLE chưa hiểu giữ missing. Chuỗi macro chỉ là metadata và không được thực thi.
- [x] Xuất từ `ref` vào `Resources` rồi chạy test. Kiểm tra PNG bằng QImage trong Task 4, đối chiếu vài icon với ảnh; manifest phải liệt kê mọi fallback. Commit script và tài nguyên được tạo, không commit toàn bộ `ref`.

## Task 3: ABI Rust và ánh xạ lệnh FreeCAD

**Files:** tạo `rust/src/ffi.rs`, `rust/tests/ffi.rs`; sửa `rust/src/commands.rs`, `rust/src/lib.rs`, `Gui/RustBridge.h`, `Gui/Command.cpp`.

**Interfaces:** giữ đủ các symbol hiện có trong `RustBridge.h`. Thêm `om9_sidebar_group_count() -> usize`, `om9_sidebar_group_title(group: usize) -> *const c_char`, `om9_sidebar_group_color(group: usize) -> *const c_char`, `om9_sidebar_group_kind(group: usize) -> u32`, `om9_sidebar_group_item_count(group: usize) -> usize`, `om9_sidebar_group_item_command(group: usize, item: usize) -> usize`, `om9_sidebar_quick_count() -> usize`, `om9_sidebar_quick_command(item: usize) -> usize`, `om9_command_icon(index: usize) -> *const c_char`, `om9_command_native_id(index: usize) -> *const c_char`.

State API: `om9_sidebar_selected_group() -> usize`, `om9_sidebar_select_group(group: usize) -> bool`, `om9_sidebar_section_flags(section: usize) -> u32`, `om9_sidebar_set_section(section: usize, visible: bool, collapsed: bool) -> bool`, `om9_sidebar_reset()`, `om9_sidebar_record_execution(command: usize, success: bool)`, `om9_sidebar_history_count() -> usize`, `om9_sidebar_history_command(item: usize) -> usize`. Sentinel cho command index không hợp lệ là `usize::MAX`/`SIZE_MAX`; kind 0=IconList, 1=Custom, 2=Reset, invalid=UINT32_MAX; section flags bit 0=visible, bit 1=collapsed, invalid=0.

Catalog/CString bất biến được giữ tới khi module unload. State dùng Mutex và mọi API trả về lỗi có kiểm tra, không unwrap dữ liệu người dùng. C++ sao chép UTF-8 sang QString, không giữ pointer động. `Command.cpp` cung cấp `bool om9ExecuteNativeCommand(std::size_t command)` và `bool om9NativeCommandAvailable(std::size_t command)` trên GUI thread; không gọi alias đệ quy.

- [x] Viết test ABI cho null/0/sentinel ở mọi accessor index ngoài phạm vi, CString NUL hợp lệ và pointer catalog ổn định sau sự kiện state. Test các group/quick command index trỏ đến catalog; history failure và reset. Các test thay đổi global state giữ một khóa test chung để không tranh chấp khi cargo chạy song song. Chạy `cargo test --manifest-path rust/Cargo.toml --test ffi`, xác nhận FAIL.
- [x] Implement tất cả symbol. Ánh xạ xác nhận từ source FreeCAD: FileNew→Std_New, FileOpen→Std_Open, FileSave→Std_Save, FileSaveAs→Std_SaveAs, ViewZoomZoomExtents→Std_ViewFitAll; bổ sung Undo/Redo và góc nhìn từ ID command có thật. Icon chưa có binding trả null native ID, availability false. Các command chỉ log hiện tại bị vô hiệu hóa thay vì báo thành công giả.
- [x] Host kiểm tra `getCommandByName` và `isActive` trước dispatch, bắt lỗi tại host. Với lệnh mở/lưu có dialog, xác nhận kết quả qua trạng thái tài liệu; cancel không được xác nhận thành công. Host chỉ gọi record_execution sau kết quả đã xác nhận. Thêm test host fake dispatch success/failure/cancel trong Task 4.
- [x] Chạy test, fmt, clippy và `cargo build --manifest-path rust/Cargo.toml --release`; kiểm tra symbol đủ bằng link C++ ở Task 4; commit Task 3.

## Task 4: Sidebar Qt và tích hợp build

**Files:** tạo `Gui/MatrixSidebar.h`, `Gui/MatrixSidebar.cpp`, `Gui/tests/MatrixSidebarTest.cpp`; sửa `Gui/Workbench.h`, `Gui/Workbench.cpp`, `Gui/CMakeLists.txt`, module `CMakeLists.txt`; thêm tích hợp có điều kiện tại `D:/FreeCAD-src/src/Mod/CMakeLists.txt`.

**Interfaces:** `MatrixSidebar : QDockWidget`, constructor nhận catalog qua ABI; `activate()`, `deactivate()`, `refreshState()`, `refreshAvailability()`. `iconForCommand(std::size_t) -> QIcon` đọc icons.ini và cắt QImage đã validate; fallback có tooltip. Command availability/execution dùng interface Task 3. Tiêm callback host vào constructor dùng `std::function<bool(std::size_t)>` để kiểm thử không cần GUI command thật.

- [x] Viết QtTest kiểm tra 7 title theo thứ tự, 18 selector và 11 quick button; chọn nhóm đổi grid; Reset khôi phục khối đã ẩn; command không hỗ trợ disabled; fake success có history, fake failure/cancel không có history. Test gọi activate/deactivate 3 lần không tăng số widget/signal và khôi phục dock trước đó. Test empty/no document và chuyển availability không dereference document cũ. Chạy test target với `QT_QPA_PLATFORM=offscreen`, xác nhận FAIL trước implementation.
- [x] Dựng sidebar rộng mặc định 300, QScrollArea và lưới responsive; palette/font/kích thước theo spec. MAIN MENU lấy toàn bộ từ Rust. Các khối còn lại dùng mẫu control và hình tham chiếu, chỉ bật binding đã kiểm chứng; tránh nội dung kỹ thuật dài trong giao diện. Undo/Redo dùng availability host; Refresh sau đổi tài liệu/hoàn tất command và một timer nhẹ cho trạng thái không có signal công khai.
- [x] Workbench tạo sidebar một lần, hiện/ẩn theo lifecycle. Ghi visibility dock ngay trước activation, chỉ khôi phục dock mình đã thay đổi; không thay thiết lập mặc định của workbench khác. Không tự tạo tài liệu cho phần menu.
- [x] Tích hợp root bằng option `BUILD_OPENMATRIX9` mặc định OFF; khi ON add_subdirectory `${CMAKE_SOURCE_DIR}/Mod/OpenMatrix9` tới binary dir `Mod/OpenMatrix9`. Link Qt Widgets/Test đúng phiên bản, copy/install Resources và InitGui.py. Native target phụ thuộc cargo release lib và script/resources, khai báo đủ DEPENDS để build lại khi Rust/assets thay đổi.
- [x] Cấu hình build mới từ môi trường MSVC x64 với CMake/Ninja của `.pixi`, `BUILD_OPENMATRIX9=ON`, `BUILD_GUI=ON`, `CMAKE_BUILD_TYPE=RelWithDebInfo` và prefix ổ D. Không sửa/xóa cache cũ. Build OpenMatrix9Gui và test target, chạy QtTest. Ghi patch tích hợp root trong repo OpenMatrix9 để có thể tái áp dụng; commit module riêng, báo rõ thay đổi root chưa thuộc repo private.

## Task 5: Kiểm tra FreeCAD thật và bàn giao

**Files:** tạo `tests/menu_smoke.FCMacro`, `tests/run_menu_smoke.ps1`, `docs/menu-validation.md`, `docs/images/menu-sidebar.png`; sửa tài nguyên/style chỉ khi kiểm tra phát hiện sai.

**Interfaces:** macro kích hoạt OpenMatrix9, lấy dock theo objectName `OpenMatrix9Sidebar`, kiểm tra trạng thái bằng API Qt/FreeCAD, ghi JSON kết quả và quit sau chế độ test. Runner dùng môi trường cấu hình riêng cho test với `FREECAD_USER_HOME`, không ghi cấu hình cá nhân của người dùng. Không có logic menu trong macro.

- [x] Viết smoke check cho 7 title, selector/quick count, switching 3 lần, New tạo document và unsupported disabled. Bản test có đường dẫn tài nguyên phân phối, không dùng `ref`. Chạy bằng FreeCAD mới build, FAIL có chỉ rõ assertion trước khi sửa implementation nếu có lỗi.
- [x] Chạy cargo test/fmt/clippy/release và native build/test sau thay đổi cuối. Chạy smoke no-document và document-open; thử lệnh Save/Open bằng UI, kiểm tra cancel không ghi history và lỗi command không gây crash.
- [ ] Dùng computer-use để xem sidebar thật, chụp ảnh bàn giao; đối chiếu Matrix9. Kiểm tra cửa sổ thấp và các scale 100%, 125%, 150%, 200% trong instance test; báo những scale chưa thực sự kiểm tra được. Không đổi Windows display settings của người dùng.
- [x] Kiểm tra Resources phân phối không tham chiếu `ref` hoặc đường dẫn máy hiện tại; báo số icon resolved/missing. Lưu kết quả kiểm tra, lỗi/fallback còn lại và cách mở OpenMatrix9 vào docs/menu-validation.md.
- [x] Review toàn bộ diff, chạy git diff --check, commit những file được triển khai. Chỉ nhận phần menu đạt tiêu chí khi Rust/native build, lifecycle và smoke đều có bằng chứng; không nhận đã hoàn thành viewport hay chức năng trang sức. Bàn giao ảnh và hướng dẫn mở bản FreeCAD tương ứng.

## Handoff

Đề xuất Native execution: implement lần lượt trong phiên hiện tại vì các task phụ thuộc chặt vào catalog và ABI; dùng `superpowers:executing-plans` sau khi người dùng duyệt kế hoạch. Review độc lập cuối nhánh theo workflow của skill nếu có công cụ phù hợp. Không tự dispatch subagent trước khi chọn phương thức thực hiện.

## Execution status 2026-10-03

Menu slice validated through the actual native module. SDK build/C++23 rulings, independent review fixes and user-supplied named crops are recorded in docs/menu-execution.md. Task 5 live FreeCAD computer-use remains unchecked: Qt dock grabs and all four scale tests passed, but Sky did not expose a targetable FreeCAD window. Full parent rebuild and Win32 dialog interaction are not asserted.

2026-10-04 icon follow-up: installed ButtonIcons/SliderIcons archives now provide 434/434 menu names plus 26 auxiliary mappings. No menu fallback icons remain; unsupported controls stay disabled. Python 19 tests, QtTest 8 totals and actual FreeCAD 27 checks per scale passed. Every menu icon and six Display presentations were compared with original BMP pixels. Evidence and remaining scope are in docs/validation/2026-10-04-original-icons.md.
