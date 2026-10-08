# Original icon recovery validation

**Historical evidence, superseded:** the measurements below describe the earlier original-icon experiment. Current distribution uses the user's redraws and authored SVGs; see `2026-10-04-modern-sidebar.md`. Counts and screenshots from this stage are retained in Git history, not current product coverage.

User report: too many missing icons. Previous distribution used 178 named redraws + 10 RUI bindings and had 246 fallback names. Root cause: the exporter did not read the installed Matrix90 `ButtonIcons.bin`/`SliderIcons.bin` archives.

Scope: resources and Qt presentation for OM9-IFACE-001 / OM9-MAIN-001. Rust catalog/state/dispatch remain unchanged. MAIN MENU now resolves 434/434 names; 433 exact keys and one explicit SolidPtOn alias. There are 26 additional sidebar/host mappings, totaling 460 original mappings. Resource source hashes/offsets/dimensions are in `Resources/menu/source-manifest.json`; structural evidence is in `analysis/matrix90/icon-archives.md`.

Checkout: `D:/FreeCAD-src/Mod/OpenMatrix9`, branch `codex/matrix9-menu`, changes from `8c103c0`. Native SDK: `D:/FreeCAD-src/build/relWithDebInfo`; module build: `D:/FreeCAD-src/build/openmatrix9`; Qt tests: `D:/FreeCAD-src/build/openmatrix9-tests`. MSVC x64 / Qt 6.11.2 / Python 3.13.15 and existing matching FreeCAD SDK. Existing FreeCAD cache was not rewritten.

Verification:

- `python -m unittest discover -s tools/tests -v`: 19 passed. Observed RED before archive support, before slider export, and before separating auxiliary misses from menu misses; all now GREEN. Covers exact original lookup/precedence, PNG integrity/path handling, archive truncation/corrupt compression/duplicate/unsafe keys/terminator, indexed BMP and names with spaces.
- `cmake --build D:/FreeCAD-src/build/openmatrix9-tests`: exit 0. `MatrixSidebarTest.exe -o build/icon-final-qt.txt,txt`, with dependency prefix PATH and `QT_QPA_PLATFORM=offscreen`: 8 passed (six cases plus init/cleanup). Snap/Display button regressions failed before implementation, then passed; pixel equality and disabled state checked. Font/offscreen warnings are retained in the log.
- `cmake --build D:/FreeCAD-src/build/openmatrix9 --target OpenMatrix9Gui` under VS `vcvars64.bat`: exit 0. Native binary and complete Resources copied into SDK runtime. Cargo release core rebuilt by the native target; no Rust source changes. Existing seven Rust semantic/fmt/clippy checks remain the menu baseline.
- Actual FreeCAD runner `tests/run_menu_smoke.ps1 -Scale <scale>`: all 27 checks passed at 1 / 1.25 / 1.5 / 2. Each run uses a fresh GUID/profile. All 434 menu assets load from installed relative paths, every visible menu/quick icon matches original pixels, Snap and all six Display icons remain disabled, including mode keys with spaces. New/Open/Save/cancel/history/workbench switching/section restore/small-window Projects reachability still pass.
- Independent fresh-context read-only review: no important defects. Verified every exported original against archive bytes, source provenance, hash, dimensions, offsets and paths; confirmed CMake distribution does not require installed Matrix90 or `ref`.

Exact runtime reports and checks are retained in `docs/validation/menu-runtime.json`. Actual Qt dock captures from the final scale-1 run are `docs/images/menu-sidebar.png`, `menu-curve.png`, `menu-builder.png`, `menu-gems.png`, and `menu-render.png`. Images were visually inspected. This evidence validates native Qt/FreeCAD loading and rendering; it does not claim live Sky interaction or four viewport rendering.

Remaining scope: INFO still has some text labels; LAYERS/PROJECTS controls and many modeling functions remain presentation-only. Mode images use exact named records, not a recovered original Display button-index association. Unsupported actions are gray/disabled. Four viewports and green Command area remain the next UI slice. Source archives and raw reference directories are excluded from the commit; exported resources and analysis are included.
