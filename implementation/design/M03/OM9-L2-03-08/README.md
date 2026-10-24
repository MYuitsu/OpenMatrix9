# OM9-L2-03-08 — 07-febe-matrix-top11-and-shortcut-actions

- **Milestone:** M03 (Geometry 3DM)
- **Owner:** `OM9-L2-03-08` — five M03 L3 owners and 26 L4 records; `OM9-L3-03-08-005` (Ring Rail) is deferred to M04 with no M03 L4
- **Delivery date:** 2026-10-23
- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Gate:** `manual_test_gate: M03-UI` (no manual gate promoted)
- **Crates:** `om9-core::{top11,curve_ops,curve_surface_ops}` and `om9-app` (toolbar, controller adapters, grip drag, OCCT BRep dispatch)

## What was delivered

| L3 | Capability | Entry point |
|---|---|---|
| `OM9-L3-03-08-001` | Duplicate (Copy) | `top11::{duplicate_entities,duplicate_entities_offset}`, `Controller::{top11_invoke,top11_duplicate_in_place,top11_finish,top11_set_duplicate_option}` |
| `OM9-L3-03-08-002` | Edit/Control Points On | `top11::{GripMode,grip_handles,move_grip_geometry}`, `curve_ops::{curve_grips,move_grip,move_edit_point}`, `MatrixApp::top11_grip_overlay/top11_grip_drag_ui`, F10 |
| `OM9-L3-03-08-003` | Mirror/Move/Rotate | `top11::{translate_placements,rotate_placements,rotation,mirror_entities}`, point prompts, `Mirror 3Point`, BRep/non-rigid baking |
| `OM9-L3-03-08-004` | Explode/Join/Split/Trim | `top11::{explode_entities,join_entities,split_at_fractions,trim_by_fractions,split_by_cutter,trim_by_cutter}`, `curve_surface_ops::curve_brep_fractions` |
| `OM9-L3-03-08-005` | Ring Rail entry point | disabled Top 11 button; deferred to M04 |
| `OM9-L3-03-08-006` | Toolbar availability | `top11::{Top11Context,availability,toolbar}`, Top 11 tab and icon gating |

## Interaction contract

- The Top 11 tab lists all eleven original commands with their authored icons.
- A button is enabled only when the current selection/kind and command state admit it; the tooltip carries the disabled reason. Ring Rail always reports the M04 deferral.
- Duplicate: pick a base point, then each destination places one copy. While the prompt runs, `Vertical`, `FromLastPoint`, `UseLastDistance` and `UseLastDirection` (or `=Yes/No`) change the placement rules; Enter finishes. `Copy InPlace` copies immediately.
- Control points / edit points: F10 toggles the display. Control grips cover lines, NURBS, circles (centre + quadrants), every polycurve segment and NURBS surface poles; edit grips cover line/NURBS/circle/polycurve points plus an OM9-specific surface Greville grid. The surface edit grip moves its pole by the drag divided by that pole's rational basis influence, so the point reaches the target while knots, weights, degree and the periodic/reversed record stay unchanged. Drag projects onto the CPlane through the grip and commits one geometry transaction; Esc cancels.
- Move/Rotate collect base/centre/reference points. Mirror copies by default, `Mirror Delete` reflects in place, and typing `3Point` while the Mirror prompt runs (or `Mirror 3Point`) switches to an arbitrary three-point plane; the toolbar default is the two-point vertical plane.
- Mirror bakes rigid and non-rigid placements: affine circles become rational NURBS ellipses, mesh/cloud normals and winding are transformed, and BRep bodies use the OCCT adapter.
- Split/Trim accumulate cutting objects until Enter: curve, NURBS surface or BRep. The product default asks OCCT for exact curve/face intersections; the no-default developer build refuses a BRep cutter because it has no exact kernel path. Split cuts at every intersection; Trim then asks for the portion to remove.
- Every action emits the standard command events and leaves its Undo entry.

## Measured acceptance (this checkout)

```
cargo test -p om9-app --no-default-features --lib --test top11
cargo test -p om9-core --test top11
cargo test -p om9-app --test top11_brep   # with OCCT DLLs on PATH
```

| L3 | Measured |
|---|---|
| 001 | In Place 2 objects; base flow 2 copies; Vertical (3,4,5)→(0,0,5); Last Distance 10; Last Direction 5 along x |
| 002 | 2 line grips; circle radius 5 from a quadrant drag; 4 surface control and 4 edit grips; periodic/reversed cylinder edit keeps its record; drag/Undo passes |
| 003 | Move/Rotate; Mirror copy/Delete; 3Point start x=−1; non-rigid end x=−2; BRep volume 24, scaled BRep bounds −4…0 |
| 004 | join 1/explode 2; one cutter 3 objects; two cutters split 5 and trim 4; surface cutter 3; exact BRep sphere cut fractions 0.25/0.75; 50 stress cycles |
| 006 | 11 buttons; Ring Rail disabled; running command disables every button |

Surface edit-point performance is part of the acceptance: a 96x96 grid is evaluated by tensor contraction in ~93 ms (debug), the adapter handle list in ~101 ms, and a committed single-pole move in ~50 ms including transaction/scene refresh; the overlay caches handles until document revision, selection or mode changes, and grids above 10,000 edit points are refused with a budget error. These are debug-build measurements from `surface_edit_grid_is_fast_on_a_large_surface`.\n\nNegative fixtures (empty selection, running command, unsupported geometry for grips, one curve for Join, native curve for Explode, gap/different placement for Join, invalid fractions, degenerate mirror plane, unsupported mirror geometry, no cutting intersection, cutter without shared placement) return a typed error and leave document/selection/Undo unchanged.

## Verification commands

```
cargo test -p om9-app --no-default-features --lib --test top11 -j 2
cargo test -p om9-core --test top11 -j 2
cargo test -p om9-app --test top11_brep -j 2   # exact OCCT intersections need the DLLs on PATH
cargo check --workspace --all-targets -j 2
cargo clippy -p om9-core -p om9-app --no-default-features --lib --test top11 -j 2 -- -D warnings
cargo fmt --all -- --check
python tools/public_source_audit.py
python tools/om9_milestone_dashboard.py
```

## Evidence layout

- `implementation/design/M03/OM9-L3-03-08-00X/{api.md,source_crosswalk.json}`
- `implementation/fixtures/M03/OM9-L3-03-08-00X/{positive.json,negative.json}`
- `implementation/evidence/M03/OM9-L3-03-08-00X/` — focused, regression, stress (L3-004), BRep logs (L3-003/004), acceptance and AUTO_ONLY review
- `implementation/evidence/M03/OM9-L2-03-08/om9-app-regression.log`

## Deliberate limits

- Duplicate has no fifth placement option beyond Vertical/From Last Point/Use Last Distance/Use Last Direction.
- Surface edit grips are an OM9 extension, not Rhino/Matrix parity; they move a pole and do not reparameterise the surface. Circle quadrant drags resize without rotating the frame.
- The default build uses OCCT exact curve/face intersections for BRep cutters; the no-default developer build refuses the action rather than approximating display triangles. Cutters must share the target placement.
- The toolbar Mirror starts with the two-point vertical plane; typing `3Point` resets the picks and switches to an arbitrary plane.
- Non-rigid circles become NURBS ellipses; mesh winding is corrected by the reflection swap.
- Ring Rail is deferred to M04 (no M03 L4).
- Manual M03-UI acceptance and macOS runtime remain open.
