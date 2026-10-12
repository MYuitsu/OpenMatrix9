# Build OpenMatrix9 trên Windows — dùng SDK chung

Cập nhật: **2026-10-12**. Tài liệu này dành cho phát triển plugin.
Để dùng gói đã phát hành, xem [cài đặt trong README](../README.vi.md#installation):
FreeCAD 1.1.4 chính thức và SetupLauncher tự cài plugin vào AppData.

## 1. Chuẩn bị SDK và toolchain

Workspace này dùng **một FreeCAD build/SDK** tại
`H:/FreeCAD-src/build/relWithDebInfo`. Khi chỉ sửa OM9, build riêng
`OpenMatrix9Gui` và dùng lại cache của checkout.

| Thành phần | Yêu cầu |
|---|---|
| Host và SDK | Source, generated headers, import libraries và runtime khớp nhau. |
| Compiler | MSVC x64 hỗ trợ C++23; cùng toolchain/CRT với SDK. |
| Rust | Edition 2024; target `x86_64-pc-windows-msvc`. |
| Dependencies | Python, Qt6, OCCT và Coin của đúng SDK. |
| Build tools | CMake >= 3.24, Ninja và Cargo trong môi trường build. |

FreeCAD installer cung cấp runtime; build plugin cần thêm SDK tương ứng.
SDK development không mặc định dùng để build cho FreeCAD 1.1.4 stock.
Ghi lại source commit, SDK và toolchain khi tự build.

Các file cần có trong SDK:

```text
build/relWithDebInfo/
├── bin/FreeCAD.exe
├── src/App/FreeCADApp.lib
├── src/Base/FreeCADBase.lib
├── src/Gui/FreeCADGui.lib
├── src/Mod/Part/App/Part.lib
├── src/3rdParty/FastSignals/libfastsignals/libfastsignals.lib
└── lib/Coin4.lib
```

Source và generated headers cũng cần hiện diện. Layout CMake sử dụng:
[StandaloneSDK.cmake](../cmake/StandaloneSDK.cmake).

## 2. Build trong workspace H:/FreeCAD-src

Hai helper dưới đây thuộc workspace này, nằm tại `FreeCAD-src/build`;
chúng không đi kèm ZIP plugin. Chạy từ `H:/FreeCAD-src`.

Kiểm tra SDK/cache trước; lệnh này không build:

```powershell
rtk proxy pwsh -NoProfile -ExecutionPolicy Bypass -File build/build-om9-plugin.ps1 -CheckOnly
```

Nếu cache plugin đã hợp lệ, build lại bằng:

```powershell
rtk proxy pwsh -NoProfile -ExecutionPolicy Bypass -File build/build-om9-plugin.ps1
```

Helper dùng cache `build/openmatrix9-history-cage`, nhập FreeCAD libraries,
build Rust, openNURBS, import worker, resources và target `OpenMatrix9Gui`.
Worker mặc định: **60% logical CPU, làm tròn xuống, tối thiểu 1**.
Nó chờ build khác dùng cùng cache thay vì xóa/tạo lại cache.

Khi cấu hình cache plugin lần đầu hoặc chọn runtime/dependencies khác:

```powershell
rtk proxy pwsh -NoProfile -ExecutionPolicy Bypass -File build/build-om9-plugin.ps1 -Configure -RuntimeDirectory H:/FreeCAD-src/build/om9-plugin-runtime
```

Giữ cùng `BuildDirectory` cho checkout. Khi cần chọn runtime thử nghiệm,
truyền đúng `RuntimeDirectory`; phối hợp với phiên đang dùng chung cache.
Plugin output tách khỏi FreeCAD host.

## 3. Standalone trong workspace khác

Nếu không có helper trên, dùng standalone CMake của OM9. Mở
**Developer PowerShell for VS2022 x64**, rồi môi trường dependencies của SDK.
Ví dụ dưới giả định source, SDK và dependencies đã sẵn tại `C:/CAD`;
thay paths theo máy của bạn.

```powershell
Set-Location C:/CAD/FreeCAD-src
pixi shell
```

Nếu chưa clone OM9, trong shell dependencies:

```powershell
git clone --branch main https://github.com/MYuitsu/OpenMatrix9.git C:/CAD/OpenMatrix9
```

Cấu hình cache **plugin** một lần, tách runtime output khỏi SDK:

```powershell
cmake -S C:/CAD/OpenMatrix9 -B C:/CAD/OpenMatrix9/build/native -G Ninja `
  -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_CXX_COMPILER=cl `
  -DFREECAD_SOURCE_DIR=C:/CAD/FreeCAD-src `
  -DFREECAD_SDK_BUILD=C:/CAD/FreeCAD-src/build/relWithDebInfo `
  -DFREECAD_DEPENDENCY_PREFIX=C:/CAD/FreeCAD-src/.pixi/envs/default/Library `
  -DCMAKE_PREFIX_PATH=C:/CAD/FreeCAD-src/.pixi/envs/default/Library `
  -DOPENMATRIX9_RUNTIME_ROOT=C:/CAD/OpenMatrix9/build/plugin-runtime `
  -DOPENMATRIX9_RUNTIME_OUTPUT_DIR=C:/CAD/OpenMatrix9/build/plugin-runtime/bin
```

Các lần build sau giữ cache và chỉ build target plugin:

```powershell
$om9Workers = [Math]::Max(1, [int][Math]::Floor([Environment]::ProcessorCount * 0.6))
cmake --build C:/CAD/OpenMatrix9/build/native --target OpenMatrix9Gui --parallel $om9Workers
```

Lần đầu CMake tải openNURBS ở commit được pin trong
[OpenNURBS.cmake](../cmake/OpenNURBS.cmake), nên cần mạng.
Runtime plugin chứa `bin/OpenMatrix9Gui.pyd`, `bin/OM9ThreeDmImportWorker.exe`
và `Mod/OpenMatrix9` với Python/resources. Giữ cùng một bản build.

## 4. Chạy với host dùng chung

Trong workspace H:/FreeCAD-src, chọn plugin và profile riêng:

```powershell
rtk proxy pwsh -NoProfile -ExecutionPolicy Bypass -File build/run-om9-plugin.ps1 -PluginRuntime H:/FreeCAD-src/build/om9-plugin-runtime -KeepOpen
```

Chọn runtime thực sự đã build. Helper mặc định nhận accepted runtime
`build/om9-phase3-sdk`; khi thử runtime khác, truyền `-PluginRuntime`.
Nó dùng host `build/relWithDebInfo`, profile riêng và ghi đường dẫn/hash
của plugin đã nạp.

`build/3dm-preservation-sdk` là junction tới SDK chung. `build/om9-phase3-sdk`
chứa plugin đã được chấp nhận và các link tới host, không phải một FreeCAD build khác.
Không cần tạo bản sao toàn bộ runtime cho từng giai đoạn.

Ở workspace khác, chọn plugin trước khi kích hoạt workbench, kiểm tra
`OpenMatrix9Gui.__file__` và resource lookup của adapter/host tương ứng.
Thêm `-P` không bảo đảm đã thay mọi binary/resource của một bản cũ.
Giữ binary development riêng với binary stock của gói Release.

## 5. Kiểm tra sau thay đổi

Tại thư mục OM9:

```powershell
cargo test --manifest-path rust/Cargo.toml
python -m unittest discover -s tools/tests
python tools/public_source_audit.py
```

Chạy macro phù hợp với phần đã sửa trong workspace chung:

```powershell
rtk proxy pwsh -NoProfile -ExecutionPolicy Bypass -File H:/FreeCAD-src/build/run-om9-plugin.ps1 -PluginRuntime H:/FreeCAD-src/build/om9-plugin-runtime -Macro H:/FreeCAD-src/Mod/OpenMatrix9/tests/public_icons_smoke.FCMacro
```

Helper tạo profile, log và `results.json` riêng; kiểm tra exit code và plugin.
Icon smoke xác nhận giao diện/icon; chạy thêm fixture hình học cho chức năng đã sửa.
[Báo cáo và giới hạn](validation/) nằm ngoài README tổng quan.

## 6. Khi nào build lại FreeCAD?

Chỉ khi core source, generated headers, ABI hoặc matching dependency/toolchain
thay đổi. Giải thích lý do cụ thể trước khi làm và giữ SDK/host khớp nhau.
Thay đổi OM9 riêng lẻ dùng lại SDK và target plugin ở trên.

Thiếu `.lib`: kiểm tra SDK đã build đủ và paths. `LNK1168`: đóng tiến trình
đang nạp plugin cần thay. Lỗi import/DLL: kiểm tra host, output và dependencies.
