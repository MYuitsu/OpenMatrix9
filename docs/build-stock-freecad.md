# Building the official-host native adapter

The published 0.0.2 binary targets Windows x64 FreeCAD 1.1.4. Installing it does not require building anything; see [installation](../INSTALL.en.md).

For a source rebuild, use MSVC 2022 x64, CMake/Ninja and Rust. Provide the exact FreeCAD 1.1.4 source headers, generated Python binding headers, FreeCADGui/App/Base and Part import libraries matching the official installed binaries, and matching development dependencies: Python 3.11.14, Qt 6.8.3, OCCT 7.8.1, Coin 4.0.3, Boost 1.86, Eigen 3.4, Xerces 3.3 and fmt 12. The stock installer does not include this development SDK. SDK generation and dependency acquisition are not supplied as a one-click public build tool in this release.

From an initialized MSVC environment, configure this **plugin source directory**, never the FreeCAD root:

```powershell
cmake -S . -B build-stock -G Ninja `
  -DCMAKE_BUILD_TYPE=RelWithDebInfo `
  -DFREECAD_SOURCE_DIR="D:/sdk/freecad-1.1.4-source" `
  -DFREECAD_SDK_BUILD="D:/sdk/freecad-1.1.4-generated" `
  -DFREECAD_DEPENDENCY_PREFIX="D:/sdk/matching-deps/Library" `
  -DCMAKE_PREFIX_PATH="D:/sdk/matching-deps/Library" `
  -DOPENMATRIX9_STOCK_HOST="C:/Program Files/FreeCAD 1.1" `
  -DOPENMATRIX9_RUNTIME_ROOT="D:/om9-runtime"
$om9Workers = [Math]::Max(1, [int][Math]::Floor([Environment]::ProcessorCount * 0.6))
cmake --build build-stock --target OpenMatrix9Gui --parallel $om9Workers
```

The plugin target builds Rust, openNURBS, worker and resources. Place the resulting `OpenMatrix9Gui.pyd` and `OM9ThreeDmImportWorker.exe` in `Mod/OpenMatrix9/bin` beside the generated scripts/resources; retain licenses. Do not copy a complete host runtime or replace official FreeCAD DLLs. The independent `ThreeDmModelingBrepTests` target is excluded from default plugin builds and can be built explicitly for verification.

The 0.0.2 source adapter is stock-specific; development-host source rebuilds of this tag are unsupported. Use the separate v0.0.1 source and matching 27.1 host for that development build. Never mix native binaries between host ABIs. openNURBS source is fetched at the revision defined in `cmake/OpenNURBS.cmake`; its compatibility source transformations and license notices are included in this repository.
