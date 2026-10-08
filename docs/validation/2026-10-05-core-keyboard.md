# Matrix9 keyboard integration — 2026-10-05

This slice registers the 11 defaults explicitly documented in the local `01-core` specifications. Menu actions, CMD and keyboard entry use the same handlers. Rust owns the shortcut table and persistent Ortho state; Qt handles focus, native shortcuts, panels and the original F6 menu export. This does not complete all 130 core specifications.

## Sources and implementation

See `docs/matrix9-shortcuts.md` for each key, original command and feature ID. No PDFs were reread. F6 uses a packaged byte-identical copy of `ref/ContextMenu.xml` at `Resources/menu/ContextMenu.xml`; SHA-256: `50EDA7D08B260AFBD33CFEBB490045A976B19FF5E80D0239096DB13B1A3A122D`.

F2 displays typed CMD inputs and the native Report view. Ctrl+T raises the selected object's native property inspector. F7 works with CMD focus and toggles only the active viewport. F8 saves its state in OpenMatrix9/Keyboard/Ortho preferences. Mouse Line/Polyline and PictureFrame preview/commit use Main O-Snap AND (saved Ortho XOR Shift); typed coordinates stay exact. F4 input excludes modifiers, autorepeat, dialogs and popups in Curve, Distance/Angle and PictureFrame.

F6 preserves eight mode titles and original action labels/order. It classifies Empty, one Curve, Surface, Polysurface and Default, then chooses the source group or its Default fallback. Unsupported and parameterized actions stay disabled; enabled actions use the shared native handler.

## Verification

The module was compiled with MSVC14.44, Qt6.11.2, Python3.13 and matching FreeCAD/Coin SDK, linked at `build/keyboard-runtime/OpenMatrix9Gui.pyd`. Native tests use the clean patched host `build/host-camera-diagnostic/bin/FreeCAD.exe`, isolated profiles and sequential runs. Directory names do not imply a diagnostic binary. A result requires both `ok=true` and process exit 0.

- Rust: all 55 tests passed; format check, Clippy all-targets with `-D warnings` and release build passed. Test log: `build/keyboard-rust-tests.log`.
- Initial missing F2 regression: `build/keyboard-isolated-1/8c367e49eb5642839b76cce48aae2a8c/results.json`.
- Review correction RED: modal F4 advanced Distance before the guard (`build/keyboard-isolated-1/68a2820617a74b2a83c9f3293ffa2582/results.json`). Guarded build passes this case and Alt+F4, alongside existing shortcut/geometry tests.
- Review correction RED: F8 did not constrain PictureFrame preview (`build/keyboard-picture-1/519801f222e7483c8ddcdfba1e3f7a3b/results.json`). Preview and commit now consume the shared Rust calculation. The dedicated fixture also tests Shift release and Main O-Snap off.

Final keyboard: 28 checks at 100% with normal exit (`build/keyboard-isolated-1/306555cab19144a8a52a2e8a8b3f8230/results.json`) and the same 28 at 200% with normal exit (`build/keyboard-isolated-2/b97bf5406fa645aa917c7cc213dadc07/results.json`). PictureFrame: 43 checks including the four new Ortho cases and full previous geometry/lifecycle/Undo/reload coverage, normal exit (`build/keyboard-picture-1/27bb7d25871d488497eaf40a0c787a5d/results.json`).

Final module regressions also exit normally: workspace 36 checks (`build/keyboard-core-views-1/d2b0bb4f4b604484a3468d46608cddbc/results.json`) and Curve 26 checks (`build/keyboard-curve-1/822db08069d2475c9979804732016831/results.json`). The interactive preview is launched from `build/workspace-keyboard-preview.FCMacro`; its status file reports the actual loaded module and visible native views. Older previews are retained with their older modules.

Checks use Qt-generated key and mouse input; they do not establish physical desktop input delivery. A read-only review found the two issues above, and reviewed both source repairs without further actionable findings.

## Remaining scope

F10/PointsOn, Ctrl+Q/Group, Ctrl+W/UnGroup and Ctrl+Alt+C/gvCenterObjects have their original names and bindings reserved; their CAD handlers remain unavailable. Ctrl+Q/W no longer invoke host quit/close while OpenMatrix9 input is active. This is binding coverage, not implementation of the four tools.

Ortho still requires native verification of persisted state in a separate process. Distance/Angle mouse constraints and other drawing tools are not wired to it. F2 does not implement unlimited/chronological Rhino history or all original options. Ctrl+T uses native object properties; complete Matrix material/custom property workflows remain missing. F6 does not yet cover gem/grip/specialized selection predicates, customization, builders, reports or materials. No reliable complete Rhino/custom keyboard map was recovered for the unspecified function keys.

The prior End/Mid fixture's intercepted Mid-sidebar click remains unresolved and is recorded in `2026-10-05-core-point-snap.md`. The shortcut evidence does not supersede that failing regression. The reported stationary grid under the user's own zoom gesture also remains to be reproduced; see `2026-10-05-core-grid-zoom.md`.
