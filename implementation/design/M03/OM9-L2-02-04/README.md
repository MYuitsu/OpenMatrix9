# OM9-L2-02-04 — 04-febe-freeform-curve-modeling

- **Milestone:** M03 (Geometry 3DM)
- **Owner:** `OM9-L2-02-04` — twelve L3 owners, 63 L4 records
- **Delivery date:** 2026-10-22
- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Gate:** `manual_test_gate: M03-UI` (no manual gate is promoted here)
- **Crate:** `crates/om9-core` (safe Rust, no native host dependency)

## What was delivered

| L3 | Capability | Rust entry point |
|---|---|---|
| `OM9-L3-02-04-001` | Line/polyline/point creation | `curve_build::{polyline,polygon}`, `curve_ops::{arc_length,sample_curve,join_curves}` |
| `OM9-L3-02-04-002` | Circle/arc/ellipse/polygon | `curve_build::{circle_through,circle_on_frame,arc_through,arc_by_center,ellipse,rectangle,polygon}` |
| `OM9-L3-02-04-003` | Interpolated/control-point NURBS | `curve_build::{parameters,basis,interpolate,insert_knot}`, `curve_ops::insert_control_point` |
| `OM9-L3-02-04-004` | Edit points and grips | `curve_ops::{curve_grips,move_grip,insert_control_point}` |
| `OM9-L3-02-04-005` | Join/explode/close/split | `curve_ops::{join_curves,explode_curve,close_curve,split_curve,trim_curve}` |
| `OM9-L3-02-04-006` | Offset/fillet/chamfer | `curve_modify::{offset_curve,fillet_lines,chamfer_lines,arc_blend_lines}` over `offset_curve::OffsetCurve` |
| `OM9-L3-02-04-007` | Blend/ArcBlend/match | `curve_modify::{blend_curves,match_curve}`, `Continuity::{G0,G1,G2}` |
| `OM9-L3-02-04-008` | Rebuild/refit/knots | `curve_modify::rebuild_curve`, `curve_ops::refit_curve`, `curve_build::{fit_points,insert_knot,elevate_degree}` |
| `OM9-L3-02-04-009` | Project/pullback/intersections | `curve_surface_ops::{closest_point_on_surface,pullback_curve,curve_curve_intersections,curve_surface_intersections}` |
| `OM9-L3-02-04-010` | Duplicate edge/border/isocurve | `curve_surface_ops::{duplicate_isocurve,boundary_edges,duplicate_brep_edge}` |
| `OM9-L3-02-04-011` | Seams/directions/divisions | `curve_ops::{arc_length,parameter_at_length,divide_by_count,divide_by_length,tangent_at,reverse_curve,set_seam}` |
| `OM9-L3-02-04-012` | Sketch on surface/mesh/two views | `curve_surface_ops::{sketch_on_surface,sketch_on_mesh,closest_point_on_mesh,point_from_two_views}` |

## Ownership and rules

- Safe Rust owns every rule: evaluation, length integration, parameter mapping, knot insertion,
  offset/refinement bounds, fillet/chamfer corner arithmetic, blend continuity, projection and
  intersection refinement.
- No OCCT, openNURBS or QymCAD handle crosses into this slice. `Curve`, `NurbsCurve`,
  `NurbsSurface`, `Mesh` and `Brep` are the portable records.
- A `Curve::Polycurve` parameter is the segment-uniform mapping already used by
  `surface_kernel::sample_curve`; split/divide agree with sampling. Trim is measured by arc
  length so a polyline trim does not depend on how its segments were authored.
- Every record is admitted by `Limits` before it is returned; a refusal is a typed `Error` and
  inputs are immutable values.
- Allocations use `try_reserve`, and no operation publishes a partial record.

## Measured acceptance (this checkout)

`cargo test -p om9-core --test l2_02_04` — 15 passed (12 L3 focused tests + 3 stress tests).
`cargo test -p om9-core -j 2` — 447 passed, 0 failed in 52 suites.

| L3 | Measured oracle |
|---|---|
| 001 | line length 5 mm; polyline 20 mm |
| 002 | circle 10π mm; polygon 40 mm |
| 003 | 4-pole cubic passes through its chord-length parameters; 25 stress curves |
| 004 | 4 grips/edit points; insertion raises to 5 |
| 005 | join 20 mm; trim [0.25,0.75] 10 mm; 9 polygon round trips |
| 006 | fillet arc π mm; chamfer 5 mm; 100-corner stress |
| 007 | G1 blend 14.862666324708 mm; G2 blend 15.243044351953 mm |
| 008 | rebuild radial deviation 1.6688046e-5 mm < 0.02 mm |
| 009 | projection distance 5 mm; pullback deviation 1 mm |
| 010 | 4 boundary edges; duplicated display edge 10 mm |
| 011 | square 40 mm; 5 divisions |
| 012 | surface sketch starts at (1,2,0); two-view point (0,0,0) |

Negative fixtures (degenerate line/polyline, zero-area polygon, malformed frame/radius/sweep,
even/oversized degree, out-of-range grip index, join gap, inverted trim, fillet radius above a
leg, negative circle offset, parallel fillet lines, coincident blend points, rational match,
fit control count out of range, knot outside domain, pullback/intersection/mesh deviation,
seam on an open curve, skew/parallel view rays) all return a typed error; no partial record is
published.

## Verification commands

```
cargo test -p om9-core --test l2_02_04
cargo test -p om9-core -j 2
cargo check --workspace --all-targets -j 2
cargo clippy -p om9-core --all-targets -j 2 -- -D warnings
cargo fmt --all -- --check
python tools/public_source_audit.py
python tools/om9_milestone_dashboard.py
```

## Evidence layout

- `implementation/design/M03/OM9-L2-02-04/{README.md,W2-CURVE-KERNEL.md}`
- `implementation/design/M03/OM9-L3-02-04-00X/{api.md,source_crosswalk.json}`
- `implementation/fixtures/M03/OM9-L3-02-04-00X/{positive.json,negative.json}`
- `implementation/evidence/M03/OM9-L3-02-04-00X/` — one receipt per L4 (focused test, regression,
  integration stress where planned, acceptance and the AUTO_ONLY review record)
- `implementation/evidence/M03/OM9-L2-02-04/om9-core-regression.log` — the full crate run

## Deliberate scope limits

- The menu/viewport command tier is not rewired; the portable services and records are the
  delivered layer (as with the other M03 owner slices).
- Fillet/chamfer are line-line; polyline corner chains and line-arc fillets are open.
- NURBS offset/rebuild are sampled and fitted, with the deviation measured by tests.
- B-Rep edge duplication reads the display snapshot; the exact analytic OCCT edge curve is a
  B-Rep-owner adapter.
- SDK v1 still exposes only the recorded partial curve surface (`add_arc`,
  `add_interpolated_curve`, `add_nurbs_curve`, `add_ellipse`, `add_rectangle`, `add_polygon`,
  `add_circle_on_frame`, `curve_segments`, `evaluate_curve_segment`); the new operators have an
  explicit SDK retrofit task.
- Manual `M03-UI` acceptance, macOS runtime and control-point deletion remain open.
