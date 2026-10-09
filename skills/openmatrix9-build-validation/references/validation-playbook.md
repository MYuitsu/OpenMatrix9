# Environment and verification

Read the accepted plan's current-state section and Task 4–5, then inspect:

- Module `CMakeLists.txt`, `Gui/CMakeLists.txt`, `rust/Cargo.toml`, `InitGui.py`.
- `Gui/RustBridge.h`, matching Rust exports and native callers.
- FreeCAD root `src/Mod/CMakeLists.txt` and applicable top-level build options. OpenMatrix9 lives under `Mod/OpenMatrix9` on this machine; do not assume the parent automatically includes it.
- The chosen build's `CMakeCache.txt` and target files. Resolve the intended checkout explicitly.

## Detect before configuring

Check executable paths, Rust host target, CMake generator/build type, MSVC x64 environment, Qt/FreeCAD development dependencies and prefix paths. Rust target and C++ ABI must agree. Read cache variables such as `CMAKE_HOME_DIRECTORY`, `CMAKE_GENERATOR`, `CMAKE_BUILD_TYPE` and `CMAKE_PREFIX_PATH`; compare with actual directories.

Observed local baseline, not a permanent fact: `D:/FreeCAD-src/build/relWithDebInfo` has an E: source/prefix cache and no OpenMatrix9 target. Local CMake/Ninja exist under `D:/FreeCAD-src/.pixi/envs/default/Library/bin`. Recheck before use. The plan suggests a fresh `build/openmatrix9` after parent registration and crate repair; configure with options supported by the current source instead of inventing a target or option.

## Rust checks

From the module checkout:

```powershell
cargo test --manifest-path rust/Cargo.toml
cargo fmt --manifest-path rust/Cargo.toml --check
cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings
cargo build --manifest-path rust/Cargo.toml --release
```

Run checks appropriate to the slice. For the menu plan, verify real INI/BOM/CRLF/error handling, bounded indices and successful-only history/state semantics. A test that merely mirrors implementation text is insufficient.

## Native integration

Confirm the configured target exists before building `OpenMatrix9Gui`. For a valid chosen build, use CMake's build interface with its actual configuration/generator. Verify:

- Every ABI declaration has a correctly typed exported Rust symbol and matching caller.
- UTF-8/NUL string lifetimes, ownership, bounds/error returns and no unwinding across C ABI.
- Rust static library rebuild dependencies and C++/Qt/FreeCAD linking.
- GUI-thread ownership and absence of Rust-owned Qt widget pointers.
- Correct `InitGui.py`, native module location and packaged resource paths.
- Workbench starts from the built installation and does not require `ref`, original manuals or development-only absolute paths.

Compiler/linker errors are investigated from diagnostics and source/build configuration. Retest after relevant changes; do not broaden/repeat passed checks without a new reason.

## UI/runtime cases for the current menu slice

| Case | Evidence required |
|---|---|
| First activation, no document | One sidebar, seven ordered sections; no invalid document operation |
| MAIN MENU | 18 groups/11 quick icons from current INI, correct order/tooltips/selection/Reset |
| ICON HISTORY | Most recent successful command first, capacity20 per accepted design; failures excluded; host Undo/Redo availability |
| Command execution | Proven FreeCAD ID and availability; unsupported buttons disabled |
| Three workbench switches | No duplicate widgets/signals/commands; restore altered dock state |
| Open document and cancel/error | Preserve document edits; check the selected feature's actual transaction/lifecycle contract |
| Small window and DPI | Wrapping/scrolling, no clipped controls; record tested100/150/200% and remaining cases |
| Assets/distribution | Icons match mapped commands; fallback list; installed module works without `ref` |

Four-view camera behavior and the green command region are separate later UI slices; menu validation cannot mark the entire Matrix9 startup screen complete. For a geometry feature, add its exact-ID input/output/tolerance, applicable preview/commit/cancel/history, undo/redo and save/reload tests.

## Validation record

Record slice/feature ID, source commit and local changes, build directory, compiler/runtime environment, commands with exit results, relevant logs, actual screenshot paths, visual differences, tested interactions, untested cases and next action. Missing evidence remains explicit. Do not claim success from expected behavior or file presence alone.
