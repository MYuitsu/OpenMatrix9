# 01-core workspace checkpoint — 2026-10-05

Full goal remains ACTIVE: implement all 130 specs in 01-core. This checkpoint establishes the first native workspace implementation, not completion of 01-core.

## Verified behavior

- Four actual native views share the active document, with titles Looking Down, Perspective, Side View, Through Finger and a two-by-two layout.
- Three orthographic views, one Perspective camera; XY/YZ/XZ construction planes. F4 and CMD/mouse points respect the active plane.
- Grid toggles through Display mouse button, CMD ShowGrid and per-view F7. Grid renders in native exported images, is unpickable, and is excluded from fit-all while available to automatic clipping.
- CenterViewport/F5 and planar SynchronizeViews; Perspective camera is retained.
- Camera/grid state persists in FCStd metadata. Reload reconstructs four views.
- Workbench transitions restore native MDI mode; all document grid states are handled. Switching documents presents only that document's four managed quadrants. No extra views/CMD docks after reactivation.
- Restore resets cameras/grids, retains the focused view and clears owned background nodes. Task panel view restrictions apply to F5/F7.

## Evidence

- Rust: 22 tests passed (2 core view, 11 shared curve session, 4 ABI and 5 menu/state). Formatting and Clippy with denied code warnings passed. Incremental hard-link fallback warnings are filesystem-related.
- Matching SDK C++/Qt compile/link and Rust release build passed.
- Native core: 31 assertions passed at 100%, 150%, 200% DPI. The image assertion samples a grid band away from model geometry and navigation cube; the native image was visually inspected.
- Regressions after final code: workspace 64, Curve 26, menu 31 assertions passed.

Final reports:

- `build/core_views_smoke-1/16faffd0c4314fddad98585de6c0d44b/results.json`
- `build/core_views_smoke-1.5/683b4a4ed1c64b2792fa5a093f8799e6/results.json`
- `build/core_views_smoke-2/9a4d9213d40948a0a1756e3662dc4919/results.json`
- `build/workspace_smoke-1/bfdc43c13e6f4854b569783834140809/results.json`
- `build/curve_smoke-1/d5108ec818dd49e88849c7f0bbc934ee/results.json`
- `build/smoke-1/7156c8dfd38e45c1894d2bfa0f017981/results.json`

Initial RED: missing Rust core_views module and one-view runtime (45d29b1cb7484bf58154a83229b3795f). Review RED: multi-document windows remained visible (128461fc606c49658d0edcddbacfefa9). Render RED: scene nodes outside native capture root and clipping range did not produce visible grid (705d618d965e417487331a73f2d39b1b, e260c66bbd1049ed8e977044a1e6e783). Corrected root ownership and clipping; final rendered image contains grid and geometry.

## Remaining scope

The full requirement inventory is `docs/core-requirements.json`. Current feature coverage is recorded conservatively as partial until all relevant requirements are audited. Configurable spacing, Gem/Surface plane alignment, viewport tabs/maximize controls, complete background image import/restore, Perspective-source synchronization, view-close reconstruction tests and interactive manual desktop verification remain open. Most of 01-core (F6 customization, project metadata/database, layers, snapping, dimensions, transforms, utilities, builder/styles and dependency history) is not implemented yet. File and interface commands inherited from prior work require exact-spec audit. No PDF was reread. No unrelated changes were staged or committed.

Next dependency group: complete remaining view controls, then File Notes/Project Notes and other Info/File workflows; continue through the entire 130-spec inventory before extending Curve.
