# Native viewport titles and display menus — 2026-10-06

User requested a small title/dropdown in each native viewport and title double-click to toggle the selected single viewport/4V. Reference: `codex-clipboard-45030a5a-57ad-4bdf-9ab8-3627aabeded7.png`. Local OM9-VIEWPORT-001, OM9-IFACE-001, RH5-VIEW and RH5-DISPLAY specs were read; no PDFs reread.

## Implementation

Rust owns the original 27 display-mode names, supported-adapter mapping and single-view toggle decision. C++ hosts small title labels, separate dropdown buttons and menus over the native views. Active label is Matrix green; others gray. Native MDI chrome is suppressed only while OM9 is active and restored on exit. Labels are siblings of the native viewer, so existing point-picking handlers do not treat title clicks as geometry input.

Label double-click and dropdown Maximize/Restore 4V share the toggle. Other quadrants are hidden during a single view. Native tab switching preserves maximization; resize, document changes, view replacement, task restrictions and reactivation are covered. `DontMaximizeSubWindowOnActivation` is temporarily enabled and restored, preventing Qt from maximizing a second quadrant while restoring 4V.

Wireframe/Shaded have per-view adapters. FreeCAD providers are shared between viewers, so direct native `setOverrideMode` alone changes every view. The adapter retains common Flat Lines topology for faces and pickable CAD edges, truthful Wireframe/Shaded viewer override names, per-view render-manager modes and a view-local SoDrawStyle for image capture. Point size, line width and line pattern fields are ignored on that node, preserving object/Polyline marker styling. Provider insertion/reattachment is normalized on the next workspace update or activation (current timer200 ms). Native global Wireframe/Shaded changes reconcile local mode marks. Original provider/view settings and presentation nodes are restored/removed on workbench exit.

Rendering limits: Wireframe can show tessellation lines; Shaded retains CAD edges/points alongside filled surfaces. This is a basic Coin/FreeCAD adapter. Other Matrix display engines/presets and submenu operations remain visibly disabled; no full Matrix/Rhino rendering-equivalence claim.

## Verification

Rust64 tests, fmt check, Clippy all-targets `-D warnings`, release and MSVC14.44/C++23 native compile/link passed. Incremental-cache hardlink warnings fell back to copying. `git diff --check` passed. Runtime `build/viewport-title-runtime/OpenMatrix9Gui.pyd`, SHA-256 `1ff0ac7c59c66d695057aea64c2bc8bbc3a994faa42554a88641200f69c12eaa`; matching host `build/host-camera-diagnostic/bin/FreeCAD.exe`.

All following native fixtures used the final runtime, isolated profiles, sequential processes and clean exit0:

| Fixture | Checks | Local report directory under build |
| --- | ---: | --- |
| viewport_title_smoke,100% | 45 | viewport-title-viewport_title_smoke-1/d4d5eec2f29a41828c575a4c1ef80d03 |
| viewport_title_smoke,200% | 45 | viewport-title-viewport_title_smoke-2/4bfccae8cbb343ad964cf50642ad1d0c |
| core_views_smoke | 36 | viewport-title-core_views_smoke-1/8d7520d996494aaeba4606f519b81890 |
| core_view_tabs_smoke | 28 | viewport-title-core_view_tabs_smoke-1/d04f1d9986254e1cb24862156c251865 |
| core_mouse_smoke | 46 | viewport-title-core_mouse_smoke-1/e24da32a188a43f99a6dccb95c27af17 |
| curve_smoke | 26 | viewport-title-curve_smoke-1/3375bcaf0407413fa22c21f48ed500df |
| core_command_theme_smoke | 38 | viewport-title-core_command_theme_smoke-1/82a40ef88c6547e0a8f7f96bc63004bb |
| command_completion_smoke | 29 | viewport-title-command_completion_smoke-1/21e6781c879d46ce8c141e26f2cf4d72 |
| polyline_console_smoke | 33 | viewport-title-polyline_console_smoke-1/1481b02552584b08aa967a92c7b2253d |
| core_keyboard_smoke | 28 | viewport-title-core_keyboard_smoke-1/0107519c60064e42bcb964cb804a370f |
| core_capture_smoke | 16 | viewport-title-core_capture_smoke-1/580ce77804204a92bcefb4daca6a9a9d |

Each directory contains results.json. Wrapper macros `build/viewport-title-<fixture>.FCMacro` import the exact runtime, then execute the checked-in test. Logs are `build/viewport-title-final-<fixture>-<scale>.log`. Rust logs: `viewport-title-rust-tests.log`, `viewport-title-clippy.log`, `viewport-title-rust-release.log`; native compile log: `CoreWorkspace-viewport-title-compile.log`.

Dedicated checks cover all four double-click/resize/restore cycles, inactive-view dropdown targeting, per-view images and unchanged neighboring capture, original mode names/disabled modes, native Wireframe/Shaded changes including slot0-only Wireframe reset, geometry creation in mixed modes, geometry/selection/Undo invariants, pending Polyline dropdown/maximize, restricted tasks, document switching, view replacement and reactivation. Native image captures were inspected. Qt-generated input is runtime evidence; physical desktop input is unverified.

RED evidence: old runtime has no labels; initial 4V restoration left a second view maximized; native Shaded synchronization failed; provider insertion inherited the last Wireframe override; shaded-only shared topology made native edge picking fail in Mouse. Final code fixes each. Independent source review also identified restoration snapshots and point-marker override fields, addressed before final checks. Review found no remaining ABI/lifecycle issue; Shaded's retained CAD edges/points are documented above.

Raw QOpenGLWidget framebuffer grabs in hidden fixture windows were blank or corrupt and did not reliably prove rendering. Native scene image capture exposed that render-manager mode alone was ignored by export; the local draw-style node fixes export as well. Camera diagnostics showed only Coin automatic near/far clipping-plane changes, with pose, projection and zoom unchanged. Capture/title fixtures compare all camera text except those two derived clipping fields, retaining before/after snapshots in reports.

## Preview and skill persistence

Visible preview is launched with Normal window style and a separate profile by `build/viewport-title-preview-final.FCMacro`, document `OpenMatrix9ViewportUI_Final`; verify the loaded module/four titles/dropdowns/compact Command in `build/viewport-title-preview-final-status.json`. Windows Computer Use window selection succeeded for the first preview, but activation failed twice with `failed to activate captured window`; no physical clicks were claimed successful. Native fixtures and preview initialization remain separate evidence.

Business skills remain unchanged pending consent. [Proposed viewport rules](2026-10-06-viewport-title-skill-proposal.md) link the earlier pending grid/Command proposals. Full130-core acceptance remains in progress, middle-button mapping deferred, and unrelated dirty work preserved. No commit/push was requested.

2026-10-06 follow-up: the user explicitly approved saving these rules into the new workspace-group skill and committing/pushing this work. The pending-consent statements above describe the state when this slice was first verified. See [approval and skill verification](2026-10-06-workspace-skill-push.md).
