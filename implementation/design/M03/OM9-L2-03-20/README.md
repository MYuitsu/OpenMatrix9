# OM9-L2-03-20 — 12-febe-matrix-measure-and-model-statistics

- **Owner:** `OM9-L2-03-20` — 5 L3 / 25 L4
- **Date:** 2026-10-27
- **Status:** The Measure chapter read-outs (Angle, Distance, Length, Radius), `Utilities > Bounding Box`
  with `Output=None` and the jewellery count/type summary run as read-only app commands. The Measure
  palette icon and the typed command line reach the same dispatch body, a read-out publishes its label in
  the Command Line and never opens a transaction. Acceptance records are READY_FOR_REVIEW (M03-UI).

## What the audit found

The arithmetic existed in `om9-core` (`measure::distance_mm`, `measure::angle_degrees`,
`curve_ops::arc_length`, `utilities::bounds`) and the plugin SDK already exported point/curve read-outs
(`Host::measure_distance|measure_angle|measure_curve_length|measure_curve_radius|object_bounds|
object_summary|report_measurement`), recorded in `sdk_implementation_progress`. The **application** had
nothing: no pending state, no `availability`/`dispatch_geometry`/`resolve_alias` arm, no label, no
`implementation/…/OM9-L3-03-20-*` evidence, and the dashboard row was `✅0 🟡0 ⬜5`. The SDK exports are a
separate surface and were not counted as app evidence.

## Command surface

| Feature id | Menu / typed name | Input | Label |
|---|---|---|---|
| `OM9-MEASURE-001` | `Angle`, `ang` | four typed/picked points; Objects: two straight lines; Continuous/Ortho | `Angle = <n> deg` |
| `OM9-MEASURE-002` | `Distance`, `dist` | two typed/picked points; Objects/Continuous/Ortho | `Distance = <n> mm` |
| `OM9-MEASURE-003` | `Length`, `len` | placed curves, canonical BRep edges, surface isocurve U/V | `Length = <n> mm` |
| `OM9-MEASURE-005` | `Radius`, `rad` | one circle/arc, or pointwise ellipse/spline station | `Radius = <n> mm` |
| `OM9-UTIL-010` | `BoundingBox`, `bbox` | selection | `Bounding Box = X x Y x Z mm, center (x, y, z)` |
| `ObjectSummary` | `ObjectCount`, `count` | whole document | `Objects: N total (<kinds>) · gems · heads · rails · cutters · unknown` |

`Angle`, `Distance`, `Length` and `Radius` are the four authored Measure palette icons
(`AnalyzeAngle/AnalyzeDistance/AnalyzeLength/AnalyzeRadius`). `BoundingBox` and `ObjectSummary` have no
Measure-palette row: the first reuses the documented `Utilities > Bounding Box` command id, the second is
also available through Analyze > Object Summary. Area and Volume have Analyze menu rows. A menu click and a typed name dispatch
the same feature id, so both doors run one implementation.

## Semantics

- **Units.** Every read-out is canonical millimetres; angles are degrees. The labels print three decimals
  and trim trailing zeros, so a 5 mm line prints `5` and never `5.000`.
- **Point input.** A `Distance`/`Angle` run latches the active construction plane and page unit when it
  starts, then resolves typed local coordinates through that plane (the Line path). The viewport pick path
  (`accept_world_point`) feeds the same `MeasurePending`, so a click and a typed coordinate are equal.
- **Hover/prompt state.** `prompt()` reports the next point (`First distance point: `, `Second distance
  point: `, the four angle prompts) and the input is refused while another command runs (`Busy`).
- **Read-only.** No measurement calls the coordinator: `report_measurement` only publishes a label and one
  `Started/Progress/Finished` run. Every golden test asserts the document revision, the entity count and
  `coordinator().undo_len()` are unchanged.
- **Label lifetime.** The label is tagged with the selection revision it measured. A new command drops it
  (`dispatch_effect`), a selection change invalidates it (`last_measurement` returns `None`) and
  `take_measurement_label` consumes it once, so the status line cannot repeat a stale value.
- **Bounding box.** The eight local bounds corners pass through the entity placement, so a moved object
  reports its moved box; `Output=None` means no geometry is created.
- **Object summary.** Counts come from `GeometryKind::tag()` plus the typed jewellery slots
  (`om9.gem`, `om9.head`, `om9.rail`, `om9.cutter`). An unknown slot increments `unknown_slots` and is
  never guessed as a gem. Block definition members are not objects: `definitions` is reported separately
  and never added to `total`.

## Refusals

- Length needs compatible curve/edge/isocurve operands; unsupported opaque geometry and annotation strokes refuse.
- Radius needs one curve. Ellipse/spline requests a point or explicit Station=0..1; lines/inflections and incompatible knot-side curvatures refuse a finite answer.
- `Angle` with a zero-length direction → `Error::Invalid` from `measure::angle_degrees`.
- `BoundingBox`/`ObjectSummary` with no finite vertices → `Error::Missing`/`Error::Unsupported`.
- A refused read-out publishes no label and leaves the document untouched.

## Evidence layout

- `implementation/design/M03/OM9-L3-03-20-00X/{README.md,api.md,source_crosswalk.json}`
- `implementation/fixtures/M03/OM9-L3-03-20-00X/{positive.json,negative.json}`
- `implementation/evidence/M03/OM9-L3-03-20-00X/OM9-L4-03-20-00X-0N.json` — one receipt per planned
  L4 role (Source Map, Contract Fixture, Implement, Regression, Acceptance), 25 files.
- `implementation/evidence/M03/2026-10-26-l2-03-20-measure.json` — the batch receipt.
- Code: `crates/om9-app/src/session/measure.rs`, the `session.rs` wiring, the `ui.rs` feedback branch.
- Tests: `crates/om9-app/tests/measure.rs` (`m03_03_20_001`…`005`) and `crates/om9-app/tests/measure_ui.rs`.

## Verification

```
cargo test -p om9-app --test measure
cargo test -p om9-app --test measure_ui
cargo test -p om9-app -j 2
cargo clippy -p om9-app --all-targets -j 2 -- -D warnings
rustfmt --edition 2024 --check crates/om9-app/src/session/measure.rs crates/om9-app/src/session.rs crates/om9-app/src/ui.rs crates/om9-app/tests/measure.rs crates/om9-app/tests/measure_ui.rs
python tools/public_source_audit.py
```

## Current scope and limits (2026-10-27)

The former curve-only, circle-only, point-only, missing-hover and typed-only-summary gaps are
superseded by [the verified correction report](../../../../docs/validation/2026-10-26-measure-completion/README.md).
Area/Volume are host commands; existing SDK evidence stays separate. Summary includes layer/placed-definition
groups and known/unknown gem carats, without adding unplaced definitions to root totals.

- Root BRep-edge and explicit U/V isocurve length; selected instance member edge paths remain unsupported.
- BRep area/closed-solid volume, numerical NURBS area; opaque NativeBrep payloads refuse exact measurement.
- Objects Angle takes straight lines; other angles use points. Mesh distance requires triangulation.
- Radius at a station is local curvature, not a whole-spline circle fit. Invalid/singular derivatives refuse.
- Thirteen model-space annotation commands now belong to **OM9-L2-03-21**, preserving this owner's five L3.
  Paper-space, source association and Rhino annotation/styles/layout roundtrip remain open.
- Headless UI frames and semantic tests pass; actual desktop replay was blocked by Access denied/black capture.
  M03-UI stays unsigned. See the receipt for exact test counts, build fingerprint and remaining boundaries.
