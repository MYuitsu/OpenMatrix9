# Complete 01-core implementation

User objective: implement every function in `specs/01-core` before continuing Curve. Existing interface shells and enabled commands do not establish feature completion. The requirement inventory is `docs/core-requirements.json`, containing every source behavior, workflow, option cue and acceptance checklist from this group.

## Architecture and constraints

Retain Rust ownership of command/state/validation; C++/Qt integrates native FreeCAD geometry, widgets and views. Retain the original public command names where supplied. Menu, mouse and CMD must route to the same implementation. Preserve unrelated work and the existing Curve changes. Use the local specs, with no PDF reread. Missing source defaults are documented host decisions, never recovered Matrix defaults. External service workflows require available service integration; a link alone cannot prove completion.

## Dependency order

1. Four real native document views, per-view construction planes, visible grid and origin input; restore/synchronize/center and view controls.
2. Interface/F6, file and project metadata, layers, display and info controls.
3. Snap engine and transforms, shared interactive input and editing widgets.
4. Measurement and annotation editing/persistence.
5. Utilities, builder/style infrastructure and dependency history.
6. Project database, materials/report and service integrations; audit every remaining requirement across the full inventory.

## View workspace design

Use four native `Gui::View3DInventor` views of the same document in the MDI workspace. Reuse existing views and create missing slots. Titles: Looking Down, Perspective, Side View, Through Finger. Layout two by two without closing views from other documents. Planar views have orthographic cameras and XY/YZ/XZ construction planes; Perspective uses a perspective camera and XY C-plane. These orientation choices are documented host choices. Each grid is a nonselectable Coin scene overlay owned by its view. F7 affects only the active grid; Show Grid affects all four. F4 supplies the construction-plane origin to the current point command. Restore resets titles/cameras, centers on origin, clears workspace background images, and returns inactive views to wireframe. Synchronize propagates planar scale and center without replacing orientations or changing Perspective. Save/reload reconstruction must be verified.

## Verification

Rust tests prove slot descriptors and state rules. Native macros test actual view count/scene nodes, projection in each construction plane, commands through menu and CMD, keyboard shortcuts, resizing, document/workbench switches, Undo/Redo where applicable and persistence. Run existing menu/workspace/Curve regressions. Visual viewport evidence supplements model assertions. Completion requires requirement-by-requirement evidence in the inventory; leave unverified requirements open.
