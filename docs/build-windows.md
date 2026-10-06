# Build FreeCAD và OpenMatrix9 trên Windows

Cập nhật: 2026-10-06. Hướng dẫn theo source public hiện tại, **Windows x64,
MSVC và Qt6**. Standalone SDK CMake hiện dùng tên thư viện Windows;
Linux/macOS chưa được kiểm chứng cho module này.

## Chọn cách build

| Bạn đang có | Cách dùng |
|---|---|
| Chưa có FreeCAD SDK tương thích | **A. Build FreeCAD cùng OpenMatrix9 từ source**. |
| Có source, FreeCAD build hoàn chỉnh và dependencies tương ứng | **B. Build riêng OpenMatrix9**, không build lại FreeCAD mỗi lần sửa module. |
| Chỉ có FreeCAD installer/portable | **C. Dùng với bản cài đặt**: cần binary OM9 và SDK khớp chính bản FreeCAD đó. Clone source vào Mod chưa đủ. |

OpenMatrix9 có `OpenMatrix9Gui.pyd` native C++/Qt, Rust static core và Python
registration. Cùng số phiên bản FreeCAD chưa bảo đảm ABI tương thích: source/API,
kiến trúc, Python minor, Qt6, OCCT, Coin, MSVC/runtime và cấu hình build phải khớp.
Không mặc định dùng được với FreeCAD Qt5 hoặc với mọi bản release tải sẵn.

## Chuẩn bị

- Git; Visual Studio2022 hoặc Build Tools với **Desktop development with C++**,
  MSVC x64 và Windows SDK.
- Rust stable hỗ trợ edition2024, target `x86_64-pc-windows-msvc`.
- [Pixi](https://pixi.sh/latest/installation/) và dependencies theo `pixi.toml`/
  `pixi.lock` của đúng checkout FreeCAD.
- CMake>=3.24 và Ninja trong môi trường dependencies; compiler hỗ trợ C++23.
- Python>=3.10 cho tools; Python dùng để build/load module phải khớp SDK.

Mở **Developer PowerShell for VS2022** với compiler x64. Các đường dẫn `C:\CAD`
dưới đây là ví dụ, có thể thay. Chạy từng bước và dừng nếu trả lỗi. Đóng FreeCAD
trước khi link hoặc thay `.pyd`. Không checkout/clone đè workspace đang có thay đổi.

```powershell
git --version
pixi --version
rustup show
rustup target add x86_64-pc-windows-msvc
```

Tham khảo [FreeCAD Developers Handbook](https://freecad.github.io/DevelopersHandbook/gettingstarted/)
và [FreeCAD Pixi configuration](https://github.com/FreeCAD/FreeCAD/blob/main/pixi.toml).
Các lệnh dưới đây theo baseline OM9 đã sử dụng; main mới hơn cần kiểm tra lại API.

## A. Build FreeCAD cùng OpenMatrix9

### 1. Clone workspace

```powershell
New-Item -ItemType Directory -Force C:\CAD
Set-Location C:\CAD
git clone --recurse-submodules https://github.com/MYuitsu/FreeCAD.git FreeCAD-src
Set-Location C:\CAD\FreeCAD-src
git checkout 0205a6b3b5eb9546760a2aa39391354504f20194
git submodule update --init --recursive
git clone https://github.com/MYuitsu/OpenMatrix9.git Mod/OpenMatrix9
git -C Mod/OpenMatrix9 rev-parse HEAD
```

Commit FreeCAD trên là baseline đã dùng trong workspace phát triển. Ghi lại cả
commit OM9 để tái lập. Nếu dùng main FreeCAD mới hơn, cần build/test lại.
Nếu đã clone module thì dùng thư mục đó, không clone đè.

### 2. Tích hợp CMake

Từ `FreeCAD-src`, nếu chưa có `BUILD_OPENMATRIX9`:

```powershell
git apply --check Mod/OpenMatrix9/cmake/freecad-integration.patch
git apply Mod/OpenMatrix9/cmake/freecad-integration.patch
```

Chỉ apply khi `--check` pass; không apply lần thứ hai. Nếu source khác làm patch
không khớp, thêm đoạn sau **một lần** vào cuối `src/Mod/CMakeLists.txt`:

```cmake
option(BUILD_OPENMATRIX9 "Build the Rust/Qt OpenMatrix9 workbench" OFF)
if(BUILD_OPENMATRIX9)
    add_subdirectory("${CMAKE_SOURCE_DIR}/Mod/OpenMatrix9" "${CMAKE_BINARY_DIR}/Mod/OpenMatrix9")
endif()
```

Source OM9 nằm ở **`FreeCAD-src/Mod/OpenMatrix9`**, khác với các module có sẵn
trong `FreeCAD-src/src/Mod`. Patch nối hai vị trí này.

### 3. Dependencies và build

```powershell
pixi install
pixi run initialize
pixi run cmake --preset conda-windows-rel-with-deb-info -DCMAKE_GENERATOR_PLATFORM= -DCMAKE_GENERATOR_TOOLSET= -DBUILD_OPENMATRIX9=ON
pixi run cmake --build build/relWithDebInfo --parallel 8
```

Giảm parallelism nếu thiếu RAM. Target GUI kéo theo Rust core và copy Python/
resources. CMake tải openNURBS public ở commit cố định trong
[OpenNURBS.cmake](../cmake/OpenNURBS.cmake); lần đầu cần mạng. Không cần cài Rhino.

### 4. Chạy hoặc install

```powershell
pixi run build/relWithDebInfo/bin/FreeCAD.exe
```

Chọn workbench **OpenMatrix9**. Build tree có cấu trúc:

```text
build/relWithDebInfo/
  bin/FreeCAD.exe
  bin/OpenMatrix9Gui.pyd
  Mod/OpenMatrix9/
    Init.py
    InitGui.py
    ThreeDm.py
    ThreeDmArchiveState.py
    Resources/
```

Để install theo preset Pixi, đóng FreeCAD rồi:

```powershell
pixi run cmake --install build/relWithDebInfo
pixi run .pixi/envs/default/Library/bin/FreeCAD.exe
```

Preset install vào `.pixi/envs/default/Library`. Đây là install tree dùng cùng
môi trường Pixi, **không phải bộ cài `.exe` độc lập để gửi sang máy khác**.

## B. Chỉ build OpenMatrix9 với FreeCAD SDK sẵn có

SDK phải được build hoàn chỉnh trước. Không cần patch FreeCAD CMake cho cách B.
Standalone không reconfigure SDK nhưng **ghi binary/resources vào SDK build
tree**. Đóng FreeCAD và sao lưu module đang dùng trước khi thay.

Ví dụ source/SDK ở `C:\CAD\FreeCAD-src`, module ở `C:\CAD\OpenMatrix9`:

```powershell
Set-Location C:\CAD
git clone https://github.com/MYuitsu/OpenMatrix9.git OpenMatrix9
Set-Location C:\CAD\FreeCAD-src
pixi shell
```

Trong shell Pixi vừa mở, với compiler MSVC x64:

```powershell
cmake -S C:/CAD/OpenMatrix9 -B C:/CAD/OpenMatrix9/build/native -G Ninja -DCMAKE_C_COMPILER=cl -DCMAKE_CXX_COMPILER=cl -DCMAKE_BUILD_TYPE=RelWithDebInfo -DFREECAD_SOURCE_DIR=C:/CAD/FreeCAD-src -DFREECAD_SDK_BUILD=C:/CAD/FreeCAD-src/build/relWithDebInfo -DFREECAD_DEPENDENCY_PREFIX=C:/CAD/FreeCAD-src/.pixi/envs/default/Library -DCMAKE_PREFIX_PATH=C:/CAD/FreeCAD-src/.pixi/envs/default/Library
cmake --build C:/CAD/OpenMatrix9/build/native --target OpenMatrix9Gui --parallel 8
```

`FREECAD_DEPENDENCY_PREFIX` là thư mục **Library của đúng môi trường SDK**;
standalone tìm Python ở thư mục cha. Không trỏ nó vào thư mục chứa FreeCAD.exe
của một installer bất kỳ. CMake này sử dụng:

| Thành phần | Vị trí SDK/source |
|---|---|
| FreeCAD headers | Source `src`, SDK root/`src`, gồm generated headers như `FCConfig.h` |
| Third-party headers | PyCXX, FastSignals, Coin và headers dependencies tương ứng |
| Import libraries | SDK `src/Gui/FreeCADGui.lib`, `src/App/FreeCADApp.lib`, `src/Base/FreeCADBase.lib` |
| FastSignals | SDK `src/3rdParty/FastSignals/libfastsignals/libfastsignals.lib` |
| Coin | SDK `lib/Coin4.lib` |
| Python, Qt6, OCCT | Development files trong dependency prefix tương ứng |

Xem [StandaloneSDK.cmake](../cmake/StandaloneSDK.cmake) cho layout chính xác.
Kết quả build được đặt trực tiếp tại:

```text
<FREECAD_SDK_BUILD>/bin/OpenMatrix9Gui.pyd
<FREECAD_SDK_BUILD>/Mod/OpenMatrix9/{Python scripts, Resources}
```

Chạy `<FREECAD_SDK_BUILD>/bin/FreeCAD.exe` trong cùng shell dependencies để kiểm
tra. Khi chỉ sửa OM9, build lại target `OpenMatrix9Gui`; không cần build lại
FreeCAD nếu SDK/API/dependencies không đổi.

## C. Dùng với FreeCAD đã cài hoặc portable

### Điều kiện để dùng được

Installer/portable thông thường là **runtime**, không thay thế SDK layout ở B.
Source public hiện chưa cung cấp binary OM9 cho mọi release FreeCAD và chưa có
quy trình tạo installer FreeCAD+OM9 tự động.

Hai lựa chọn:

1. Dùng FreeCAD tự build theo A/B, cùng bộ dependencies đã kiểm tra.
2. Có SDK từ **đúng build tạo ra bản FreeCAD đã cài**, build OM9 theo B rồi kiểm
   thử với runtime đó. Nếu không lấy được SDK khớp, dùng cách A.

Không lấy SDK từ main để mặc định build cho một bản stable bất kỳ. Không copy
DLL Python/Qt/OCCT từ môi trường khác sang để chữa lỗi import.

### Xác định bản FreeCAD đích

Trong Python console của FreeCAD:

```python
import FreeCAD as App, sys
print(App.getHomePath())
print(App.Version())
print(sys.version)
from PySide6 import QtCore
print(QtCore.qVersion())
```

Đây là kiểm tra ban đầu, không thay thế kiểm tra source/API và toàn bộ ABI.
Đóng FreeCAD sau khi kiểm tra.

### Copy module đã build vào portable tương thích

Chỉ làm sau khi đã xác nhận SDK/runtime tương thích. Ưu tiên portable riêng.
Ví dụ root portable là `C:\CAD\FreeCAD-OM9`; module chưa tồn tại ở đích và thư
mục `Mod` của FreeCAD đích đã có sẵn:

```powershell
Copy-Item -LiteralPath C:\CAD\FreeCAD-src\build\relWithDebInfo\Mod\OpenMatrix9 -Destination C:\CAD\FreeCAD-OM9\Mod -Recurse
Copy-Item -LiteralPath C:\CAD\FreeCAD-src\build\relWithDebInfo\bin\OpenMatrix9Gui.pyd -Destination C:\CAD\FreeCAD-OM9\bin\OpenMatrix9Gui.pyd
```

Nếu đã có OM9, sao lưu toàn bộ module và `.pyd`, rồi thay đúng package đã build.
Tránh lồng `OpenMatrix9/OpenMatrix9` hoặc trộn resource cũ. Không copy toàn bộ
source, build, Rust target hoặc private models vào runtime.

Hiện menu/resources được đọc tại
**`<App.getHomePath()>/Mod/OpenMatrix9/Resources`**. Cài chỉ vào thư mục user
`%APPDATA%` chưa được bảo đảm hoạt động đầy đủ. Với Program Files, ghi vào root
có thể cần quyền quản trị; portable riêng dễ quản lý hơn.

Mở FreeCAD đích, chọn OpenMatrix9 và kiểm tra console:

```python
import OpenMatrix9Gui
print(OpenMatrix9Gui.__file__)
import FreeCADGui as Gui
Gui.activateWorkbench('OpenMatrix9Workbench')
```

Phải nạp `.pyd` đúng bản vừa build. Clone qua Addon Manager hoặc copy Python
đơn lẻ không tự biên dịch C++/Rust.

## Kiểm tra sau build

Từ repo OM9, trong môi trường có Cargo/Python:

```powershell
cargo test --manifest-path rust/Cargo.toml
python -m unittest discover -s tools/tests
python tools/export_menu_assets.py
python tools/public_source_audit.py
```

Kiểm tra GUI với đường dẫn thực tế:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tests/run_menu_smoke.ps1 -FreeCADExe C:/CAD/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe -DependencyPrefix C:/CAD/FreeCAD-src/.pixi/envs/default/Library -Macro public_icons_smoke.FCMacro -TimeoutSeconds 120
```

Harness dùng profile riêng, lưu `results.json` dưới build và kiểm tra process
exit. Icon smoke không chứng minh mọi lệnh CAD đã hoàn thành. Chạy thêm macro
cho chức năng đã sửa. Một số 3DM/native test cần fixture tự cung cấp; xem
[build and validation](build.md). Mẫu nhẫn riêng không nằm trong public source.

## Lỗi thường gặp

| Lỗi | Kiểm tra |
|---|---|
| `No module named OpenMatrix9Gui` | `.pyd` chưa build/copy hoặc sai search path/kiến trúc. |
| `DLL load failed` / missing procedure | SDK/API, Python minor, Qt/OCCT/Coin/runtime không khớp hoặc thiếu dependency. |
| Thiếu FreeCAD/FastSignals/Coin `.lib` | Đang dùng installer runtime hoặc SDK chưa build đủ; kiểm tra bảng ở B. |
| `LNK1168` | FreeCAD đang giữ `.pyd`; đóng tiến trình đang nạp module rồi build lại. |
| Không có icon/context menu | Kiểm tra resources tại FreeCAD home, resource cũ và source audit. |
| Patch không khớp/đã apply | Kiểm tra `BUILD_OPENMATRIX9`; không apply lần hai hoặc ép patch vào source khác. |
| Hết RAM | Giảm `--parallel`, giữ build type/toolchain nhất quán. |
| 3DM export từ chối retained data | Public chưa có full preservation writer; xem [README tiến độ](../README.md). |

Build source không tự tạo bộ cài Windows. Đóng gói bản phân phối phải theo
quy trình packaging của đúng FreeCAD checkout, kèm runtime dependencies,
source và [third-party notices](../THIRD_PARTY_NOTICES.md) cần thiết. Không coi
thư mục install Pixi là package độc lập đã được kiểm thử trên máy sạch.
