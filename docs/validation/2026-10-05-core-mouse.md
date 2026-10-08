# Native Matrix mouse integration — 2026-10-05

This slice implements viewport navigation, native object/subelement selection, recent commands and shared CMD confirmation. It does not complete all 130 core specs or all Matrix tool-specific mouse actions. The behavior table and source distinctions are in `docs/matrix9-mouse.md`.

## Implementation and source decisions

Local `OM9-VIEW-004` takes priority for Ctrl-left drag zoom. General navigation and selection follow the official Rhino 5 HTML shortcuts and Mouse Options linked in the behavior document. No PDFs were reread. Middle-button defaults remain an explicit configurable OM9 choice pending the user's own Matrix configuration; Recent is mode 0, Pan 1, Orbit 2, Host 3.

Rust `core_mouse` owns button/modifier mapping, finite movement thresholds and rectangle containment, exposed through three C ABI functions. `Gui/CoreMouse` handles native camera, picks and geometry crossing. Window selection filters native center-based results with projected bounding boxes per selected subelement, while retaining previous Shift selection. Bounding-box containment can be conservative for complex geometry.

Point tools keep their existing shared handlers. Navigation does not add points or end their sessions. Right click accepts pending CMD text before attempting to repeat a successful recent command. Held-right uses the same F6 handler; middle Recent entries use the same native dispatch. Focus/dialog/task-policy guards and native-edit pass-through protect other UI contexts.

Press/release ownership uses a release tombstone after Esc, held-right popup or capture cancellation. Gesture state clears before releasing the mouse grab because UngrabMouse may be synchronous. Application deactivation, lost capture and no-button movement cancel without continuing navigation; Esc additionally restores the camera pose.

## Build and verification

Built against matching MSVC14.44 / C++23, Qt6.11.2, Python3.13, Coin4 and FreeCAD SDK. Current module: `build/mouse-runtime/OpenMatrix9Gui.pyd`. Runtime host: clean patched `build/host-camera-diagnostic/bin/FreeCAD.exe`. Each native fixture uses an isolated GUID profile, sequential execution, and requires both `ok=true` and exit code 0.

Local manual incremental builds used `build/core-mouse-compile.cmd`, `build/mouse-view-controls-compile.cmd`, `build/view-tabs-compile.cmd`, `build/core-distance-compile.cmd`, `build/picture-frame-compile.cmd`, and `build/mouse-link.cmd`. These ignored scripts and ABI-specific binaries are local evidence, not portable build instructions. The normal CMake source list includes CoreMouse. Set `CARGO_TARGET_DIR=D:/FreeCAD-src/build/openmatrix9/Gui/rust-target` on this machine to link the newly built Rust archive.

All 57 Rust tests, formatting, Clippy all-targets with `-D warnings`, and release build passed; test log `build/mouse-rust-tests.log`. The two mouse tests observed a missing-module failure before implementation. Native compilation and final relink passed after the capture-loss repair.

Final mouse fixture `tests/core_mouse_smoke.FCMacro`: **46 checks per scale**, normal exit:

| Scale | Native result |
| --- | --- |
| 100% | `build/mouse-isolated-1/a70ba324ebe24271be8a23bab2f32c98/results.json` |
| 200% | `build/mouse-isolated-2/f3e6cbf0970947519d301d47d9d31b7c/results.json` |

Coverage includes four-view pan/orbit/Ctrl-right zoom/wheel; forced parallel orbit, dolly, tilt and look; whole-object and Ctrl deselection with physical modifier key events; both selection directions; subedge containment with preserved prior edges; Polyline completion, active Curve/Distance navigation and pending CMD confirmation; idle repeat; Esc rollback/release suppression; recent menu and configured middle pan; held-right F6 and restored hover; task locks; capture loss, missing held button and application deactivation.

Final-module native regressions at 100%, all checks passed with exit 0:

| Fixture | Checks | Result |
| --- | --- | --- |
| Keyboard | 28 | `build/mouse-core_keyboard_smoke-1/8796391fd685406abe4aa6ce32231d8c/results.json` |
| Workspace | 36 | `build/mouse-core_views_smoke-1/20f345bf0ee046c8ae475292849cc5d0/results.json` |
| View controls | 20 | `build/mouse-core_view_controls_smoke-1/123f6660205345a48c95dc752612baa9/results.json` |
| Curve | 26 | `build/mouse-curve_smoke-1/0addf3980e3144669ba342f56d57cb10/results.json` |
| PictureFrame | 43 | `build/mouse-core_picture_frame_smoke-1/33964fc25d664db8a5890e86a0460894/results.json` |

Interactive launcher `build/workspace-mouse-preview.FCMacro` opens `OpenMatrix9MousePreview` with two sample boxes. `build/workspace-mouse-preview-status.json` records the actual module, PID and four visible native views; `build/workspace-mouse-preview.png` captures the native workspace. Older previews keep their previously loaded modules.

Desktop verification found the initial KeepOpen launcher still used Hidden window style. The local runner now uses Normal for KeepOpen and Hidden for automated fixtures. The new visible preview loaded mouse-runtime (PID22228); computer-use window selection and capture confirmed four rendered native views. Injected desktop wheel zoom visibly enlarged the LookingDown grid and objects. Selection-drag verification was interrupted by user input and is not counted as passing desktop coverage. On the user's latest instruction, the proposed middle-default Pan change is deferred for direct Matrix9 comparison; the visible preview remains on mouse-runtime. An ignored experimental mouse-pan-runtime build was tested but is not the active deliverable.

## Observed failures and repairs

- Original keyboard runtime lacked Shift-right pan: `build/mouse-red-1/d22ff51a17a54a61b9323edac9b1bb38/results.json`.
- Real Ctrl keypress promoted the older CoreViewControls filter, stealing Ctrl-click and Ctrl+Shift selection: `build/mouse-isolated-1/0ca1c4fdd6494835ba6640de5d7312d3/results.json`. Removed implicit legacy activation; explicit menu/CMD view tools remain available.
- Review found whole-object window pruning could remove contained subedges or preexisting Shift selection. Native fixture now verifies Edge1 inside the box while retaining Edge2 outside it.
- Held-right originally left a stale target after the F6 menu closed. Cleanup now precedes opening the menu, and hover is verified afterward.
- Esc leaked the remaining right release to the host, opening its context menu and preventing the following recent menu: `build/mouse-isolated-1/8ff507d55d184a19a49db23709753de1/results.json`. Release tombstone fixes this sequence.
- Capture loss allowed no-button hover to keep changing the camera: `build/mouse-isolated-1/69b0a326f3714a8a84515df6d854a843/results.json`. Clear-before-release cleanup plus capture/application/no-button guards pass all three new checks. Read-only reviewer confirmed the final repair without further actionable findings.

Fixture corrections are separate from product fixes: LookingDown may use Wireframe after RestoreViewports, so selection picks a visible box edge rather than a nonexistent filled face. Native projection coordinates are physical, bottom-up pixels and are converted to logical top-down pixels using devicePixelRatio. Coin updates near/far clipping during rendering; camera rollback/lock assertions compare position, orientation, focal distance and zoom, excluding automatic clip values.

## Remaining scope

Qt-generated native events establish handler behavior, geometry and normal shutdown; physical desktop input delivery remains unverified. Favorites/full Rhino command history, configurable F6 predicates, builder handles, gumball/object drag transforms and alternate right-click icon commands remain incomplete. The prior Mid-sidebar intercepted press and the user's stationary-grid report are not resolved by this slice. See the earlier Point Snap and grid-zoom reports. No all-core completion claim is made.
