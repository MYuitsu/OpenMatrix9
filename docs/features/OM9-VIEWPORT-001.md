# OM9-VIEWPORT-001 — Four Viewports / C-Plane / F4

Status: IMPLEMENTED BASE WORKSPACE; complete 01-core scope remains IN PROGRESS.

Reference: local `specs/01-core/om9-viewport-001-four-viewports-c-plane-f4.md`; no PDF reread.

Four native FreeCAD views share one document: Looking Down, Perspective, Side View, Through Finger. Views use a two-by-two MDI layout and independent cameras. Current document's quadrants are presented together; other documents are retained and their managed windows hidden. Workbench transitions preserve native MDI mode and each document's grid state. Deleted slots are recreated by Restore Viewports. Switching workbench does not create duplicate views or CMD docks.

Host defaults: Looking Down XY, Perspective XY, Side View YZ, Through Finger XZ; 50 mm orthographic height, 100 mm focal distance; finite 40x40 mm grid with 1 mm spacing. Grid is unpickable and excluded from native fit calculations through FreeCAD's SoSkipBoundingGroup action policy, while included in automatic clipping and native image capture. View state is not Part geometry. Each camera and grid visibility persist in document Meta keys `OpenMatrix9.ViewCamera.<slot>` and `OpenMatrix9.ViewGrid.<slot>`; unrelated metadata is preserved. Native view changes are not model Undo transactions.

Mouse points project onto the active C-plane. Typed coordinates and relative offsets use its axes; F4 supplies its origin to an active point command. F7 toggles only active grid; Display Show Grid and CMD ShowGrid toggle all four. F5/CenterViewport recenter without changing scale. Task view permissions apply to shortcuts and menu/CMD.

Remaining core dependencies: Gem View/Surface View plane alignment, configurable grid spacing, native viewport tab/maximize control, full snapping engine, other point commands and Matrix-specific viewport persistence details. These remain tracked by their own specs; no claim of full Matrix interface parity.

Native tests in `tests/core_views_smoke.FCMacro` cover four actual views, camera types, layout, visible render output, keyboard/CMD/mouse C-plane behavior, grid toggles, synchronization, camera/grid persistence, workbench and multi-document lifecycle, restricted tasks, owned background cleanup and Restore. Full verification reports are in `docs/validation/2026-10-05-core-workspace.md`.
