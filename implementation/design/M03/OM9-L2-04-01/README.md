# OM9-L2-04-01 — 13-febe-matrix-transform-toolbar-bridges

- Owner record: `OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L2.json`
- Shared code: `crates/om9-app/src/session/transform.rs`
- Row catalog: `crates/om9-app/src/transform_menu.rs`
- Tests: `crates/om9-app/tests/transform_menu.rs` (`m03_04_01_*`), unit tests in `session/transform.rs`

## What this bridge is

The Transform menu already exists as authored artwork: `menu_catalog::TRANSFORM` holds the
40 icon names, `MainMenu.ini [Menu10] Name=Transform IconCount=40` lays the rows out, and
`crates/om9-app/src/ui/chrome.rs` maps every `OM9-TRANSFORM-*` id to the Transform
category, and `icons.rs` binds the authored artwork to feature ids.

The parity audit that came with this batch found the artwork table incomplete for the newer
rows, so the catalog now carries the authored key of every row (`command_key`, the same
identifier `MainMenu.ini IconN=` and `icons.rs` use) and the two defects it exposed were
fixed or documented:

- `TransformArrayAlongCurveonSurface` was bound to `OM9-TRANSFORM-021` (Array Along Curve);
  it is the artwork of Array Curve on Surface, so it is now bound to `OM9-TRANSFORM-026`.
- `OM9-TRANSFORM-017` (Orient on Curve) and `OM9-TRANSFORM-022` (ReleaseFromCage) have no
  authored icon and no SVG in `Resources/icons/transform-minimal`. Inventing artwork is not
  allowed, so both rows keep their catalog entry, stay disabled, and render the fallback
  glyph; the parity test pins that set so a future row cannot silently lose its icon.
- `TransformSmartPattern` is an authored menu row with no L3 in the milestone tree, so it
  stays uncatalogued instead of getting a command that does not exist.

What was missing is the **command layer**. Every Transform row was permanently disabled
because `Controller::availability` had no answer for it, so the menu rendered but nothing
ran. This bridge adds the command bodies and one availability answer per row:

- `transform::FEATURES` names the 16 rows that have a body.
- `transform_menu::BINDINGS` is the 41-row catalog: L3 id, Matrix label, authored command
  name, authored icon/command key, feature id, state and the reason a disabled row is
  disabled.
- `Controller::availability` answers from the catalog, so a row is either enabled with a
  body or visible and disabled with a recorded reason. There is no third state where a
  click silently does nothing.

## The two curve rows

`Orient Perpendicular to Curve` and `Array Along Curve` take their path from the selection:
exactly one selected curve is the path and every other selected object is transformed along
it, because this shell has no mid-command object pick. The first snaps the picked point to
the nearest station through the shared closest-point owner (analytic for a line) and aligns each object's
construction-plane normal with the curve tangent; the second measures equal arc-length
stations with `curve_ops::arc_length` and `length_between`. Both are refused before the first
prompt when the selection cannot be split that way.

The same batch fixed `Controller::commit_transform_mutations`: a row that hands in one step
per object (these two, and `Project To C-Plane` by height) now gets one step per target,
instead of the first object's step being applied to every object.

Clicking a menu row and typing its command name reach the same function: the typed name is
resolved by `transform::from_alias` (`by_command` for the authored script name, `by_label`
for the documented row label) and then dispatched to the same `Controller::begin_transform`.

## Transaction shape

One finished command is **one** transaction:

- a placement command composes one `Transform` onto each selected object and commits
  `Mutation::Placement` for every object;
- an array command keeps the source and commits `Mutation::Insert` copies whose placement
  is `step.compose(source.placement)`.

The transform arithmetic is never written here. `om9_core::gumball` owns
`scaling_along`, `rotation_about`, `rectangular_array` and `polar_array`, and
`om9_core::transform_session` owns the orient/remap solve
(`AlignRequest` + `align_transform`). The bridge only collects inputs, composes the kernel
steps and asks for one commit.

## Input contract

A point is resolved through the construction plane captured when the command started, with
`coords::parse_input` + `coords::resolve_with_base`, exactly like Line. A number is a plain
scalar; a trailing `mm`, `deg` or `°` is accepted because the authored manual prints angles
with a degree sign. Each scalar is validated **as it is typed** (`check_number`), so a bad
factor is refused at its own prompt instead of after three more questions. A refused value
keeps the run open on the same prompt and never touches the document; `Esc`/`Cancel` ends
the run through the normal cancel path.

## Row-by-row state (41 catalogued rows)

| state | rows |
| --- | --- |
| Implemented (39) | 001 Scale 3D, 002 Scale2D, 003 Scale1D, 004 Scale Non-Uniform, 005 Orient 2 Points, 006 Orient 3 Points, 007 Rotate3D, 008 Bend, 009 Array Polar, 010 Project To C-Plane, 015 Orient on Surface, 016 Orient Perpendicular to Curve, 017 Orient on Curve, 018 Symmetry, 022 Array Along Curve, 023 Array Linear, 024 Array Along Surface, 025 Array Curve on Surface, 026 Array, 027 Flow Along Curve, 028 Flow Along Surface, 029 Splop, 030 Set Points, 031 Scale By Plane, 032 Stretch, 033 Taper, 034 Shear, 035 Twist, 036 Maelstrom, 038 Remap to C-Plane, 039 MoveUVN, 040 Soft Move, 020 Cage Edit, 022 ReleaseFromCage, 041 Create Cage, 011 Smart Flow, 012 Smart Flow Rigid, 014 Smart Flow Break, 015 Smart Flow Pull Down Curves |
| Missing owner (2) | 019 Solid Points ON (its contract needs "editable subobject grips" and no sub-object selection service exists), 037 Make Down Facing (its L4-01 records `unresolved_owner_policy: BLOCKED_OWNER ... design minimal owner extension without enabling button`) |
| Deferred to M08 (0) | — every row whose owner the milestone tree assigns to M08 now has that owner implemented far enough to run; nothing is left deferred |

L3-042 (menu action availability) is the row that owns this catalog itself and is verified
by `m03_04_01_042`.

No row is deferred any more, and the two that stay disabled are blocked by their own contracts: their
`l3.json` records name M08 as the implementation milestone. The two missing-owner rows stay in
M03 and stay visible and disabled with the reason recorded in `transform_menu.rs`.

Six of the M08-deferred rows were enabled early because their owner services already existed
in `om9-core` and no M03 code called them: 015 Orient on Surface, 017 Orient on Curve, 018
Symmetry, 024 Array Along Surface, 025 Array Curve on Surface and 032 Stretch. They live in
`crates/om9-app/src/session/transform/rows.rs` and keep their M08 owner, planning state and
manual gate; see the section below.

## Explicit limits

- `Set Points` edits canonical point/control coordinates, preserving placement. SetX/Y/Z, World/CPlane and zero-based Indices select the coordinates. BRep solid grips and analytic-circle/annotation point editing remain explicitly unsupported.
- `Orient 2 Points` is a translation. The authored dialog also offers scaling between the
  two points; only the translate path is exposed here.
- Array copies are new objects. There is no "instanced array" (history-linked) form.
- `Project To C-Plane` moves whole objects by each object's placement origin height; it does
  not project individual curve control points or B-Rep vertices onto the plane.
- Read-only hover previews share the commit plan. Mesh/surface outlines reflect candidate geometry; BRep transforms show bounding outlines. Preview=No disables the overlay.
- The Transform toolbar icons are the authored catalog; no glyph was added or replaced.

## Verification

```
$env:CARGO_TARGET_DIR='L:\om9-target'
cargo test -p om9-app --test transform_menu -j 2   # 50 passed
cargo test -p om9-core --lib deform -j 2           # 7 passed
cargo test -p om9-app --lib -j 2                   # 188 passed (includes 7 unit tests)
cargo test -p om9-core --test gumball -j 2         # 15 passed
python tools/public_source_audit.py                # 0 errors
```

The parity test (`m03_04_01_042_catalog_matches_the_authored_menu_features_and_icons`) walks
the real `Resources/menu/MainMenu.ini` block and the real `assets/features.json`, not a copy:
every catalogued key must appear in the authored Transform menu and resolve back to its own
feature id through `icons::feature_for`, every label must equal the authored feature name,
and the only authored row without a catalog entry must be `TransformSmartPattern`. A second
test (`m03_04_01_menu_click_and_typed_command_run_the_same_body`) clicks the row through
`InvocationOrigin::Menu` and types the command name, then compares the resulting placement
and the Undo journal, so the one-handler rule is enforced by a test rather than asserted.

`M03-UI` remains unsigned: the batch is `IMPLEMENTED_READY_FOR_REVIEW` /
`AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`. Nothing in this directory signs the manual
review.


## Verified corrections — 2026-10-27

`ArrayCrv` supports `Spacing=10mm` or Count, Basepoint and NoRotation/Freeform/Roadlike.
Roadlike follows the station tangent about captured CPlane up; a tangent parallel to up refuses.
NoRotation remains the prior OM9 default; no recovered Matrix default is claimed. Closed paths
omit the duplicated end; total count is bounded at 512. Preview is read-only; Enter/click commits one Undo.

019 stays disabled for the missing solid-grip owner. 025 stays disabled for its unverified command
adapter (the earlier forward-evaluation-only claim was incorrect). 037 stays disabled under
UNRESOLVED_BLOCK_ENABLE. No facing algorithm or whole-surface attachment guarantee is invented.

Evidence: `docs/validation/2026-10-26-measure-completion/README.md`, final-focused/core/scene logs,
receipt and runtime-launch metadata. M03-UI remains unsigned; the desktop helper returned Access denied/black capture.

## Reference-frame rows — fourth pass, 2026-10-27

Six rows that the tree still defers to M08 were enabled because their owner services exist in
`om9-core` today and nothing in M03 called them. The bodies live in
`crates/om9-app/src/session/transform/rows.rs`, behind the same `begin_transform` /
`submit_transform_text` / `finish_transform` path as every other row, so the menu click and the
typed name still reach one handler and each row is still one transaction.

| row | frame | owner service |
| --- | --- | --- |
| 015 Orient on Surface | selected surface, point projected onto it | `curve_surface_ops::closest_point_on_surface`, `surface_kernel::normal` |
| 017 Orient on Curve | selected path curve; local X follows the tangent | `curve_ops::point_at/tangent_at`, `transform_session::align_transform` |
| 018 Symmetry | mirror plane from two points; copy switch is 0/1 | affine reflection `x - 2 (n . (x - origin)) n` on the placement |
| 024 Array Along Surface | evaluated grid of the selected surface | `surface_kernel::evaluate/normal` |
| 025 Array Curve on Surface | equal arc-length stations of the path, each projected onto the surface | `curve_ops::arc_length/parameter_at_length`, `curve_surface_ops::closest_point_on_surface` |
| 032 Stretch | one-direction scale about the picked base along the picked direction | `gumball::scaling_along` |

A reference row takes its frame from the selection: exactly one surface (015/024/025) or one
path curve (017/025) plus the objects to transform, and it is refused **before** the first
prompt when that split is impossible, so a run never asks a question and then fails on the
answer. The three array/mirror switches are asked as documented `0`/`1` numbers because this
module's typed contract is points and scalars; `check_number` indexes the scalar, not the field.

`Symmetry` mirrors through the placement rather than a geometry-specific mirror routine, so a
copy shares the source geometry and every geometry kind is supported; this is also why it does
not depend on `top11::mirror_entities`, which refuses kinds it cannot bake.

Tests: `m03_04_01_015_orient_on_surface_stands_the_object_on_the_surface`,
`m03_04_01_017_orient_on_curve_follows_the_tangent`,
`m03_04_01_018_symmetry_mirrors_across_the_picked_plane`,
`m03_04_01_024_array_along_surface_places_copies_on_the_surface_grid`,
`m03_04_01_025_array_curve_on_surface_projects_the_stations_onto_the_surface`,
`m03_04_01_032_stretch_scales_along_the_picked_direction`, plus
`m03_04_01_reference_rows_refuse_a_selection_without_their_reference`.

This section supersedes the earlier note that 025 stays disabled: the closest-point/pullback
services were already present, and the adapter and its golden cases are now in place. 019 and
037 remain visible and disabled for the reasons recorded in the table above, and M03-UI is
still unsigned for all 22 enabled rows.

## Deformer rows — fifth pass, 2026-10-27

The kernel had no deformer at all, so this pass added one before enabling a single row:
`crates/om9-core/src/deform.rs` owns `Deform::{Bend, Taper, Twist, Shear, Maelstrom,
SoftMove}`, `Deform::validate`, `Deform::span`, `Deform::map`, `deform::apply` and
`deform::apply_placed`, with seven unit tests of its own.

The maps are documented geometry, not invented behaviour:

| deform | map |
| --- | --- |
| Bend | the span along the axis becomes an arc of radius `span / theta` in the axis/bend plane; the section keeps its size |
| Taper | the section scales by `1 + (factor - 1) * share` over the span |
| Twist | the section turns about the axis by `angle * share` (right-hand rule) |
| Shear | affine `p + factor * ((p - base) . measure) * axis` |
| Maelstrom | twist plus `1 - pull * share` radial shrink |
| Soft Move | displacement times the smoothstep `(1 - (d/r)^2)^2`, so the weight and its slope vanish at the radius |

Two consequences are stated in every record rather than hidden:

- a nonlinear map cannot live in a placement, so a deform writes geometry. A straight segment
  that a bend, twist or maelstrom curves is written as a polycurve of 17 mapped samples
  (`deform::LINE_SAMPLES`); a shear is affine and keeps the line a line.
- a **solid is refused** with `Error::Unsupported("deformation of a solid")`. A B-Rep needs a
  kernel deformer and mapping only the display mesh would look deformed while the solid stayed
  wrong.

The map runs in world space and the result is written back into the object's own frame
(`top11::bake_geometry` and `top11::inverse_affine`, both now public), so a deform on a moved
object bends about the world points the user typed and the object still takes exactly one
`Mutation::Geometry` and one Undo entry.

Rows: 008 Bend, 033 Taper, 034 Shear, 035 Twist, 036 Maelstrom, 040 Soft Move. Tests:
`m03_04_01_008_bend_turns_the_curve_into_an_arc`,
`m03_04_01_033_taper_scales_the_section_over_the_span`,
`m03_04_01_034_shear_slides_the_section_along_the_picked_direction`,
`m03_04_01_035_twist_turns_the_section_over_the_span`,
`m03_04_01_036_maelstrom_twists_and_pulls_the_section_in`,
`m03_04_01_040_soft_move_moves_only_inside_the_falloff` and
`m03_04_01_deform_rows_refuse_a_solid`.

Eleven rows still stay visible and disabled: the Smart Flow family (011-014), the cage family
(020 Create Cage, 021 ReleaseFromCage, 041 Cage Edit), Flow Along Curve/Surface (027, 028),
Splop (029) and MoveUVN (039). They need owners that do not exist yet — a cage entity with
stored weights and a flow/parameterisation service — and no row is enabled on a body that does
not exist.

## Flow rows — sixth pass, 2026-10-27

`Flow Along Curve`, `Flow Along Surface` and `Splop` are the three remaining rows whose owner
services already existed. They are written as world-space maps in the same kernel module:

- **Flow Along Curve** — a point keeps its station along the source path
  (`curve_surface_ops::closest_point_on_curve`), the share of the arc length
  (`curve_ops::length_between`) picks the target station
  (`curve_ops::parameter_at_length`), and `transform_session::align_rotation` turns the point
  from the source tangent onto the target tangent about the source station.
- **Flow Along Surface** — a point keeps its `(u, v)` from
  `curve_surface_ops::closest_point_on_surface` and lands on `surface_kernel::evaluate` of the
  target at the matching parameters, turned by `align_rotation` from the source normal onto the
  target normal.
- **Splop** — every control point moves to the nearest point of the selected surface.

Both flow rows take their two frames from the selection. Because a selection is a set with no
pick order, the row asks for **one point on the source frame**: the curve or surface nearest
that point is the source and the other is the target. Splop asks nothing, because its single
surface is unambiguous.

Limits stated in every record: the flow carries sections **rigidly** with the frame (no
stretching along the path, no twist control around the tangent), the surface flow matches the
two parameter domains proportionally, and splop attaches at the nearest point with no distance
limit. A solid is refused as with every deformer row.

Tests: `m03_04_01_027_flow_along_curve_carries_geometry_onto_the_target_path`,
`m03_04_01_028_flow_along_surface_maps_the_source_parameters_onto_the_target`,
`m03_04_01_029_splop_wraps_the_selection_onto_the_surface` and
`m03_04_01_flow_rows_need_two_frames_and_one_object`.

What stays disabled and why: the cage family (020 Cage Edit, 021 ReleaseFromCage, 041 Create
Cage) needs a cage entity with stored weights, the Smart Flow family (011-014) needs the
history-backed flow that M08 owns, and MoveUVN (039) needs the surface UV editing contract.
Those are new owners, not new call sites, so none of them was enabled on a body that does not
exist.

## Surface edit row — seventh pass, 2026-10-27

`MoveUVN` was the one deferred row whose owner services already existed: `surface_kernel` has
`greville_parameter` and the exact `derivatives`, so the row needed a control-net offset, not a
new subsystem.

- every pole of every selected surface moves along that pole's own frame: the U tangent `S_u`,
  the V tangent `S_v` and the normal `S_u x S_v`, evaluated at the pole's **Greville**
  parameter (`deform::move_uvn`);
- the frame is frozen before the edit, so the three offsets are independent of each other and
  one command is one step;
- a zero is meaningful for an individual offset, and the row refuses only the step where all
  three are zero;
- the row edits surfaces only: a solid or a curve in the selection is refused
  (`Invalid("MoveUVN edits surfaces; select surfaces only")`) rather than silently skipped.

Limits recorded with the row: the offset is measured at the Greville parameter, so the
directions differ from pole to pole on a curved surface (it is not a rigid translation of the
control net), and the frame is not re-solved during the command.

What still stays disabled: the cage family (020 Cage Edit, 021 ReleaseFromCage, 041 Create
Cage) needs a cage entity with stored weights, and the Smart Flow family (011-014) needs the
history-backed flow that M08 owns. Both are new owners, not new call sites.

## Why these nine are not coded here

The nine remaining rows are not unfinished work in this module; their own planning records decide
it, and both decisions are checkable in the tree:

- **Seven rows are owned by M08.** `l3.json` for each says
  `target_implementation_milestone: "M08"` and `planning_handoff.implementation_owner_milestone:
  "M08"` with the reason **"Original implementation schedule is authoritative; do not implement
  twice in M03"** and scope **"No M03 L4 assigned"**. Their dispatch input points at the owning
  M08 module (`OM9-L2-04-05` features 002/003/005/006 for the Smart Flow family,
  `OM9-L2-04-08` feature 005 for the cage family). Implementing a body here would duplicate that
  owner, which is exactly what the records forbid — so the rows stay visible, disabled, and print
  the reason.
- **Two rows are blocked by their owner policy.** `L4-01` of Make Down Facing carries
  `unresolved_owner_policy: "BLOCKED_OWNER until an existing backend service is verified; never
  implement a duplicate geometry kernel in GUI"` and asks for the missing owner to be reported
  "without enabling button". Solid Points ON depends on editable **sub-object grips**, and the
  selection model is object-level only, so there is no service to bind to.

What that leaves testable in this module is the 32 enabled rows above.

## Cage rows — eighth pass, 2026-10-27

The cage family is the first group whose owner is an **M08** L3 (`OM9-L3-04-08-005`,
"Cage Edit And Release Command", whose purpose line is "deform using 1D/2D/3D cage controls"
with inputs "GeometryRef, cage BRep/control grid, vertex edits, release mode"). Instead of
duplicating the owner inside M03, the behaviour now lives where the milestone tree puts it:

- **Kernel** `crates/om9-core/src/cage.rs` gains the box cage (`box_corners`, `box_cage`,
  `cage_from_corners`), the exact parameter inversion `Cage::invert` (Newton on the trilinear
  map with an analytic Jacobian, box cages only), the displacement field `cage::displacement`,
  the visible box `cage::box_mesh` / `cage::mesh_corners` and a real `capture_parameters`.
- **Document model** gets one namespaced extension slot `om9.cage`, registered in
  `model::extension_slot_known` so a transaction may write it. A cage is an ordinary **mesh box
  object** whose eight vertices are the corners, and its extension keeps the corners it was
  created with; a caged object carries the **back pointer** to that cage. No new geometry kind
  was added, so a cage persists, moves, selects and undoes like any other object.
- **Rows** in `crates/om9-app/src/session/transform/cage.rs`: `Create Cage`
  (`OM9-TRANSFORM-041`, script `! _Cage`) hugs the world bounds of the selection and links every
  object; `Cage Edit` (`OM9-TRANSFORM-020`) asks for one corner (`1-8`) and one offset and
  carries the attached geometry with the cage field; `ReleaseFromCage` (`OM9-TRANSFORM-022`,
  the catalogued feature of `L3-04-01-021`) drops the back pointer and keeps the shape.

Why Cage Edit is exact on repeated edits: every point keeps its parameter in the cage **as it
was before this edit** (`Cage::invert`) and lands on the same parameter of the cage after it, so
each command applies one incremental displacement and errors never compose.

Stated limits: box cages only (a non-box cage is refused by name, not approximated); one corner
per command, hence one Undo entry per corner move; points outside the cage follow its nearest
boundary; the cage hugs the selection bounds with no margin.

Remaining: the four Smart Flow rows (011-014) stay disabled until their owner
`OM9-L2-04-05` (Smart Flow history: base/target mapping, rigid placement, break/reconnect,
pulldown curves) is built. Their records forbid implementing them twice in M03, so the next
slice of this work is that M08 owner, not another M03 body.

## Smart Flow family — ninth pass, 2026-10-27

The last four rows were owned by `OM9-L2-04-05` (Smart Flow history). Their behaviour now lives
in the kernel plus one persisted link instead of a second M03 implementation:

- **`deform::rigid_flow`** — takes an object frame from the base surface to the target surface at
  the same proportional `(u, v)` and turns it from the base normal onto the target normal, so a
  gemstone keeps its shape and size; the row writes `Mutation::Placement` and never touches
  geometry.
- **`deform::pulldown_from_target`** — the inverse of `flow_along_surface`: a curve that sits on
  the target surface is rebuilt on the base surface at the same parameters.
- **`om9.flow` extension slot** — the flow link: base, target, `rigid`, `revision` and the
  `ACTIVE`/`BROKEN` state. It is document state, so it saves, undoes and moves with the object.
- Rows in `crates/om9-app/src/session/transform/flow.rs`: `SmartFlow` (011, flow + record),
  `SmartFlowRigid` (012, placement only), `SmartFlowBreak` (014, mark `BROKEN`, keep geometry,
  refuse an already-broken link) and `SmartFlowPullDownCurves` (015, target → base).

Stated limit, recorded with the rows: the link is **not** a history node yet. A later change of
the target surface is recomputed by running the row again (which bumps the revision); automatic
recompute is the M08 feature `OM9-L3-04-05-004` and stays that module's work. The three flow
rows name the base surface by a **picked point**, because the selection is a set with no pick
order.

Final state of the module: **39 rows run a body, 2 are disabled with their contract's own
reason** — Solid Points ON (needs editable sub-object grips) and Make Down Facing (`L4-01`
records `unresolved_owner_policy: BLOCKED_OWNER`).
