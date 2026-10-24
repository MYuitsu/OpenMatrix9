# M01 OM9-L2-03-01 — Jewelry viewport layout and cplanes

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Target:** standalone Rust OM9
**Owner:** `OM9-L2-03-01` (M01)

## Delivered

- Four fixed Matrix viewports: **Looking Down**, **Perspective**, **Through Finger**, **Side View**
  (`om9_scene::workspace::ViewId`, `Workspace::new`).
- Camera presets:
  - Looking Down: orthographic eye `(0,0,100)`, world XY cplane.
  - Perspective: perspective eye `(35,-35,35)`, 35° FOV, world XY cplane.
  - Through Finger: orthographic eye `(0,-100,0)`, XZ cplane.
  - Side View: orthographic eye `(100,0,0)`, YZ cplane.
- Active view state and view tabs; clicking a title/tab sets `workspace.active` and preserves the
  numeric command draft/focus.
- Viewport title bar: active tint, dropdown menu, and double-click maximize/restore
  (`Workspace::toggle_maximize`).
- F4 jewelry datum overlay: world `(0,0,0)` crosshair + ring centre in every visible viewport. F4
  toggles display only; it never changes document coordinates or Undo.
- Millimeter grid/axes generated from each view's cplane (red X, cyan Y, gray minor/major lines,
  40 mm span).
- Window/layout minimum and workbench-region clamping keep pane dimensions non-negative and labels
  legible.

## Verification — rechecked 2026-10-21

- `crates/om9-app/tests/viewport_layout.rs` — 3 passed: camera presets, per-cplane grid axes,
  maximize/restore visibility.
- `crates/om9-app/tests/chrome_actions.rs` — 13 passed: F4 datum toggle/paint without document/Undo change,
  view tab/menu active-view selection with numeric draft preservation, and tab switching while maximized.
- `crates/om9-app/tests/viewport_title_menu.rs` — 17 passed: captured/authorized maximize, menu
  lifecycle and no model mutation.
- `crates/om9-app/tests/ui_events.rs` — four-view placement test passed (one unrelated construction
  grid test is a known pre-existing CPU-painter failure).
- `crates/om9-app/tests/workbench_regions.rs` (8), `workbench_regions_sizes.rs` (10),
  `startup_contract.rs` (3), `window_minimum.rs` (5) passed.

## Milestone updates

`OM9-L2-03-01` and all seven L3 records carry `runtime_implementation` evidence. L4 planning records
keep their historical planning state; completion is recorded at L2/L3 as in the existing repository
convention.

## L4 artefacts 2026-10-22

Per-L4 source crosswalks, API contracts, positive/negative fixtures, implementation and acceptance
artefacts now exist under `implementation/{design,fixtures,evidence}/M01/OM9-L3-03-01-*`. Dashboard
counts: L3-001 **5/5**, other six L3 **4/4**. Tests re-ran on 2026-10-22: viewport_layout 3, chrome_actions
13, viewport_title_menu 17. The manual M01-UI gate remains unsigned.

## Remaining

- Manual M01-UI review of the real window/picker is still required; no human visual approval is
  claimed.
- F4 is a display affordance only; the overlay is not a pickable document object.
