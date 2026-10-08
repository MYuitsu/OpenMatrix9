# Functional workspace — 2026-10-04

Base commit: `de354d2`. User-selected scope: workspace selection, deletion, Undo/Redo and camera controls. Existing RGB+5 artwork and Rust ownership remain. This is an extension of the existing native command flow; no new GUI framework or geometry subsystem is added.

## Resulting behavior

MAIN MENU has All / None / Delete / Fit all / Fit sel / Views beneath its quick row. The menu bar's OpenMatrix9 > Workspace exposes the same Rust catalog. All selects the active document's objects, including groups; None clears active-document selection. Delete retains FreeCAD's dependency dialogs and transactions. Undo/Redo restore/reapply that deletion. Views offers six axial orientations plus Isometric; Fit sel also enables the existing Zoom Selected icon.

Unsupported modeling and supplementary panel commands stay disabled. Empty/foreign/mixed selections and edit/task restrictions prevent invalid workspace actions. Native file/Undo/Redo availability retains FreeCAD's special task policy. Rust determines which observed effect counts as a successful history entry; no-op selection and canceled dependency deletion are excluded. Native commands own transactions; aliases use NoTransaction.

## Evidence

Final measurements:64 native workspace checks at each of100/150/200%,31 original-menu regression checks at100%,9 Rust tests,24 Python tests and11 Qt checks. All passed. Native widget captures were inspected at100% and200%.

- Observed initial RED: Rust missing SelectAll, Qt missing workspace row, native missing row. Semantic/catalog and GUI tests then passed.
- Fresh Rust9 tests, formatting, Clippy with denied warnings and release build passed. Incremental hard-link cache warnings are environmental and retained. Python24 tooling tests passed.
- Matching MSVC x64 SDK build compiled/linked `OpenMatrix9Gui.pyd`; no ABI errors. Four new exports have matching Rust/C declarations. Qt11 tests passed (nine cases plus init/cleanup), including disabled actions and dispatch refresh. Offscreen font/geometry warnings remain in logs.
- Native workspace results and scale factors are recorded in `workspace-runtime.json`. Checks use actual selections, grouped objects, a restricted task without getInEdit, sidebar/menu camera consistency, Undo/Redo inside that task, native file availability, canceled dependency deletion, foreign/mixed selection guards, close/reload/switch lifecycle and compact geometry. Camera assertions wait for animation completion; the initial intermediate-frame assertion was diagnosed against the host's500ms default animation.
- The solid fixture has volume192mm³. Delete removes only One, Undo restores that volume, Redo removes One again, then Undo/save/reload preserves both solids and the dependency. Fit selected uses a scene separated100mm and must reduce orthographic framing height to less than half of Fit all, preserving selection.
- Original menu regression results are in `workspace-menu-regression.json`; the previous four-scale artwork evidence remains in `menu-runtime.json`.
- All510 installed resource mappings match source bytes;498 RGB+5 mappings use497 unique PNGs, with12 authored mappings (11 chrome plus Isometric cube). Six new graphics use explicit semantic aliases, without claiming C-plane or selection algorithm recovery. Missing packaged files during the first incremental build were caught by the independent installed-byte audit; a fresh resource-glob reconfiguration packaged them. New runtime checks require every workspace image and verified derivative hash.
- Read-only review reproduced contextual tree All and task-menu/sidebar mismatches. Native group/task regressions observed RED before fixes. Follow-up caught file/Undo policy regression; native Undo-in-task RED then passed after preserving host policy and transaction ownership. Final review closed all findings.

See `workspace-assets.json` for current asset audit. Actual widget captures are in `docs/images/workspace-main.png`, `workspace-sidebar.png`, `workspace-full-sidebar.png`. The native whole-window grab is retained locally with each runtime run; it is not evidence of full OpenGL visual fidelity.

## Limits and next slice

Complete icon coverage is not complete Matrix9 functionality. Four simultaneous views, Matrix C-planes, green Command region, selection inversion/filters, geometry builders and jewelry functions remain separate work. Generic task permission behavior is verified; an actual Sketch edit session, point/subelement/link/group camera framing and camera persistence in FCStd are unverified. Geometry is tested with native whole-object solids. Undo follows the host's configuration. Win32 native file dialogs and failed SaveAs remain earlier documented gaps.

Next exact reference: `specs/01-core/om9-viewport-001-four-viewports-c-plane-f4.md`, then the native view APIs. Keep view layout independent from new geometry feature ports.
