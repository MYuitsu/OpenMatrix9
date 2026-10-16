# Gumball ring interaction performance diagnosis — 2026-10-13

Status: reproduced and diagnosed; no performance implementation was changed.

## Runtime and fixture

- Reused the installed stock `C:/Program Files/FreeCAD 1.1/bin/FreeCAD.exe`, version 1.1.4, and installed OM9 plugin at `C:/Users/nguye/AppData/Roaming/FreeCAD/v1-1/Mod/OpenMatrix9/bin/OpenMatrix9Gui.pyd`. This specifically reproduces the user's installed host rather than the different development SDK. No host was copied, configured or rebuilt.
- Installed plugin SHA256: `180b8a193be9ac89a9e3f24fced1860a6c98bb29ccce8a6d00a9518667b8d33c`.
- User-specified immutable fixture: `C:/Users/nguye/Downloads/nhan oval 11.91x8.37x4.97.3dm`, 6,553,713 bytes; SHA256 `5cdd84533c2dca20ef441fcbe2f1edd0226cc3ab02805caa4af60ef3dbba9997`.
- Modeling import produced 29 CAD objects, 439 faces and 963 edges. All 29 passed the separately measured native shape validity checks.
- Ryzen 9 5900X, 12 cores / 24 logical processors. Windows reports an RTX 3060 and a Remote Display Adapter; actual OpenGL renderer was not established.
- Diagnostic processes used separate user/system configuration and profiles under `H:/FreeCAD-src/build/om9-ring-perf-20261013/`. The user's unsaved live document was not edited or closed.

## Measurements

Native Qt pointer events exercised the installed Gumball event handler. Three mouse movements were measured in each selection case. Times are per event, not GPU FPS or alternative-host benchmarks.

| Selection | Pointer press | Mouse move range | Median mouse move | Event processing after move |
|---|---:|---:|---:|---:|
| 14 visible, editable fixture objects | 3.618 s | 5.590–5.613 s | 5.611 s | 0.037–0.047 s |
| Three curved fixture details: RhinoObject004/005/006 | 1.243 s | 2.060–2.150 s | 2.084 s | 0.038–0.051 s |

CPU time was approximately equal to wall time during pointer handling. The three-detail gesture reached dragging phase 3. Escape cancelled it without geometry or Undo changes.

For the three details, one independent native validity pass totaled **0.952 s**, optimal bounds totaled **0.134 s**, and geometry copies totaled **0.0017 s**. Across all 29 objects, one validity pass totaled **2.536 s**, versus **0.0148 s** for copies.

An isolated idle A/B did **not** establish an idle Gumball regression: selected Gumball-on and -off samples overlapped at roughly 10–18% of one CPU core. A separate five-second sample of the live user process measured 51.8% of one core, but without a live A/B this cannot be attributed to Gumball alone.

## Root-cause evidence

- `Gui/GumballController.cpp:281`: each accepted pointer motion calls `updatePreview`, which stages native geometry before producing preview lines.
- `Gui/GumballGeometry.cpp:550`: staging runs on the GUI thread; it transforms native BRep, validates the input tree and calls `BRepCheck_Analyzer`.
- `Gui/GumballGeometry.cpp:567`: staging then calls `geometryFacts` for every item.
- `Gui/GumballGeometry.cpp:414`: facts recompute optimal bounds, call `BRepCheck_Analyzer` again and calculate volume.
- `Gui/GumballGeometry.cpp:589`: preview traverses every edge and evaluates 33 points per edge, then updates all view overlays.

The measured validity costs, duplicated in the staging/facts path, account for most of the observed motion latency. The installed native pointer-handler timings reproduce the symptom; source inspection identifies an expensive exact-geometry preview path. No function-level native sampling profiler was run, so the precise split among individual C++ calls remains unmeasured.

Secondary source observation: a 100 ms timer refreshes Gumball, and overlay rendering rebuilds widget scene nodes and requests redraws. This was not established as the main cause by the idle A/B.

## Receipts and limits

- `build/om9-ring-perf-20261013/installed-stock-visible/results.json`: completed, 8 integrity/fixture checks passed.
- `build/om9-ring-perf-20261013/installed-stock-three/results.json`: completed, 9 integrity/fixture checks passed.
- Both cases verified private staging/cancel left shapes unchanged, created no Undo entry, and left the original 3DM hash unchanged. These checks do not certify all modeling commands.
- `installed-stock/results.json` retains the initial failed diagnostic: selecting all objects included hidden/uneditable objects and was rejected. Its idle samples are not accepted as a valid selected-Gumball benchmark.
- Computer Use captured the live four-view ring window. Subsequent activation/input failed with `GetCursorPos failed: Access is denied. (0x80070005)`, including after the user unlocked the desktop and the JS session was reinitialized. Direct live-window dragging was therefore not tested.
- The three-detail selection is an explicitly identified subset of the supplied fixture, not proof that it matches the three selected objects in the user's modified live document.
- QymCAD and SolidCraft were researched through their primary repositories; neither was benchmarked on this model. QymCAD uses OCCT; SolidCraft uses a truck-based Rust kernel with egui/wgpu and documents remaining freeform/STEP limitations. There is no measured ranking of their performance here.

## Next action

Separate cached display preview and affine transforms from exact native publication for Move/Rotate/Scale. Retain cheap source-generation/permission checks during interaction and complete geometry validation at publication. Keep extrusion and other geometry-changing previews explicitly scoped. Measure the same selections before/after and verify Cancel, Undo/Redo, source ownership and native geometry. This is a proposed implementation direction, not a completed optimization or a changed business rule.
