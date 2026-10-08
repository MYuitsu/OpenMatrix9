# Proposed UI skill update: viewport titles and display menus

Status: accepted 2026-10-06. User: "push code lên giúp tôi nha và lưu vào skill mới". Saved in the new workspace-group skill `skills/openmatrix9-workspace-contract/`; the historical proposal below records the pre-approval state.

Evidence: user's selected screenshot `C:/Users/Admin/AppData/Local/Temp/codex-clipboard-45030a5a-57ad-4bdf-9ab8-3627aabeded7.png` and the explicit request for title/dropdown in each viewport and title double-click to toggle single view/4V. Local specs OM9-VIEWPORT-001, OM9-IFACE-001, RH5-VIEW and RH5-DISPLAY were used; no PDFs reread.

Before: viewport names appeared in native FreeCAD MDI title chrome and the shared tab strip. There was no small in-canvas title/dropdown or Matrix title double-click action.

After: every native viewport has a small top-left title and dropdown. Double-clicking the title toggles that viewport alone maximized and the four-view layout. The dropdown includes Maximize/Restore 4V and the original Matrix display names. Wireframe/Shaded have native adapters; other display modes and submenu operations remain visibly disabled.

## Exact proposed additions

Target: `skills/openmatrix9-workspace-contract/references/grid-and-viewports.md`, with a pointer in `skills/openmatrix9-ui/SKILL.md`. The screenshot is packaged in the new skill; source and installed copies are synchronized.

- Each native viewport displays its original name (Looking Down, Perspective, Side View, Through Finger) in a small label at its top-left, with a separate small dropdown arrow. Active label uses Matrix green; inactive labels use gray. Keep the viewport's black canvas and construction grid.
- Clicking the label activates its own viewport. Double-clicking its label shows only that viewport; double-clicking again restores 4V. Dropdown Maximize/Restore 4V uses the same action. Preserve camera pose/zoom, geometry, selection, Undo and active point-command state. Native automatic clipping may update near/far planes on redraw.
- The dropdown retains the original screenshot's display-mode names and marks the current mode. Wireframe/Shaded change presentation for the selected viewport and preserve the neighboring view. Native FreeCAD global Wireframe/Shaded changes must reconcile all local marks. Unsupported Matrix rendering modes and submenu operations remain disabled until their own implementation and acceptance checks exist.
- Restore native title chrome and display settings when leaving OM9; rebuild one title/menu when a native view is replaced. Respect active-document and native task restrictions. Viewport tabs continue to switch the active single view while maximized.

Rendering scope: current Wireframe uses native Coin polygon lines with CAD edge topology retained, so tessellation lines can be visible. Shaded uses filled surfaces with CAD edges. This is a basic native adapter, not equivalence with all Matrix/Rhino display engines or presets. Provider normalization after insertion occurs on the next workspace update/activation, within the current 200 ms update interval.

This proposal extends the pending [grid/default-height/options](2026-10-05-grid-command-options-skill-proposal.md) and [pinned Command/suggestions](2026-10-06-command-completion-skill-proposal.md) proposals. A new implementation request does not approve saving those rules. Full 130-core completion and the middle-button mapping remain separate outstanding work.
