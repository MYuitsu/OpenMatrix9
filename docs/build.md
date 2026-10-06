# Build and validation

Hướng dẫn từng bước bằng tiếng Việt: [build FreeCAD cùng OM9, build riêng OM9
với SDK, và dùng với FreeCAD installer/portable](build-windows.md). Standalone
native module cần SDK/ABI khớp; installer runtime đơn lẻ chưa đủ để build.

The runtime does not require commercial CAD installations, proprietary resource
archives, private manuals or this developer's filesystem paths.

## Rust and source checks

Requires Rust with edition 2024 support and Python 3.10 or newer:

```text
cargo test --manifest-path rust/Cargo.toml
python -m unittest discover -s tools/tests
python tools/export_menu_assets.py
python tools/public_source_audit.py
```

The exporter uses `Resources/menu/MainMenu.ini` and authored icon catalogs.
Its legacy flags cannot enable bitmap import. Unknown commands fail instead of
receiving a misleading generic symbol.

## FreeCAD integration

Build this module inside a matching FreeCAD source/build checkout using
`cmake/freecad-integration.patch`, or configure the standalone SDK mode on
Windows. Requirements include Cargo, CMake, a supported C++23 compiler, a matching
FreeCAD source/build SDK, Qt6, Python and Open CASCADE dependencies.

Standalone SDK configuration accepts these caller-provided paths:

```text
cmake -S . -B build/native -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo -DFREECAD_SOURCE_DIR=<FreeCAD-source> -DFREECAD_SDK_BUILD=<matching-FreeCAD-build> -DFREECAD_DEPENDENCY_PREFIX=<dependency-prefix>
cmake --build build/native --target OpenMatrix9Gui
```

Use the compiler/toolchain environment that built the SDK. The standalone
configuration currently assumes Windows SDK library naming. openNURBS is fetched
from its public upstream at a pinned commit; retain its distribution notices.

The resource copy step does not remove stale installed files from a previous
version. Install into a clean module resource directory or remove old resources
after keeping any needed private backups.

## Native icon checks

Pass explicit executable and dependency paths:

```text
powershell -NoProfile -ExecutionPolicy Bypass -File tests/run_menu_smoke.ps1 -FreeCADExe <FreeCAD-executable> -DependencyPrefix <dependency-prefix> -Macro public_icons_smoke.FCMacro -TimeoutSeconds 60
```

This checks all 510 bindings, manifest hashes, Qt loading at 32px/523px, actual
menu button images, disabled palettes, auxiliary controls and the authored F6
configuration. Runtime artifacts are created under ignored `build/`.

Some 3DM integration tests require locally supplied models or native test
generators. Private ring samples have been excluded from this public source;
their tests are not evidence from a clean public checkout without such inputs.
The native test CMake project enables ring/unmeshed sample tests only when the
caller supplies `OM9_RING_FIXTURE`/`OM9_UNMESHED_FIXTURE`. The inventory stability
macro requires `OM9_3DM_FIXTURE`; no private sample path is embedded.

Before distributing binaries, review the complete dependency licenses and
provide all required source and notices. The source audit does not do that work.
