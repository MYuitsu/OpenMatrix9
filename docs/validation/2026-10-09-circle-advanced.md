# Circle advanced native validation — 2026-10-09

Feature `OM9-CURVE-005` remains `partially_implemented` /
`validated_supported_slice`. [Scope and host decisions](../features/OM9-CURVE-005.md).
Source: Matrix8 Book1 PDF138–140 (printed128–130), Circle continuation read
before Ellipse. The original engineering contract and Rust guide are preserved.

The later [spatial/History validation](2026-10-09-circle-spatial-history.md)
supersedes this report's CPlane-only Tangent and unsupported AroundCurve History scope.

## Native evidence

Matching MSVC2022/Qt6/OpenCascade8 native build passed. Macros ran through
`tests/run_menu_smoke.ps1` with `H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe`
and the isolated newly built `OpenMatrix9Gui.pyd`. Each macro explicitly asserts
the selected native module file path. All listed runs exited 0; reports have
`ok=true` and every check passed. Paths are relative to the module root.

| Suite | Checks | Report |
|---|---:|---|
| circle_advanced_smoke | 64 | `build/circle_advanced_smoke-1/40958fcf3fbe413bac04533d944e4e93/results.json` |
| circle_smoke | 42 | `build/circle_smoke-1/5667e7573fe9477d96c02599bb25fac2/results.json` |
| rectangle_smoke | 37 | `build/rectangle_smoke-1/a8c29c8dbb8d4934a31804ffdec80ee2/results.json` |
| curve_smoke | 27 | `build/curve_smoke-1/c2696cb2a63b437c9fa7f1c78027052c/results.json` |
| curve_spline_smoke | 41 | `build/curve_spline_smoke-1/c7a4c15c7c7e4ec9ba0711c86f93eba3/results.json` |

Advanced checks cover 3Point Radius value/location/opposite sides/half-chord and
failure recovery; spatial FitPoints and placed point-cloud, mesh, curve poles
and surface pole grids; exact Degree/PointCount periodic nonrational output,
tiny-radius normalization and stored deviation; AroundCurve straight/curved
edge center/tangent, OnCurve and stale-reference cancellation; Tangent lines,
circles and bounded NURBS, Point, Radius, FromFirstPoint, ambiguous Solution,
Undo/reselect; actual Qt native edge picking and transient contact cue; live
Area measurements and invalid-hover reset; sidebar right-click 2Point;
one-transaction native Undo/Redo and FCStd geometry/metadata persistence.
The four basic Curve regression suites total 147 checks.

The shared native tangent/selection helper also passed 252 Box/Sphere checks
before the final Circle-only hover UI change: `build/solid_options_smoke-1/d53eab4c359d430f8fe1487983b8af01/results.json`. This verifies
the retained Sphere wrapper and original default selection behavior.

## Rust and source checks

- Circle suites: 10 basic + 11 advanced tests pass; complete Rust run: 166 tests
  in 32 nonempty suites, including concurrent feature work.
- Python tools: 23 tests pass.
- `cargo clippy --lib`: exit 0, 23 warnings in other code, no Circle source
  warning. Report: `build/circle-advanced-clippy.txt`.
- Selected Rust files pass `rustfmt --check`; Git whitespace checking uses
  `cr-at-eol` for the workspace's existing CRLF files.
- Source audit retains 510 icon bindings; 290 pre-existing private/reference
  PDF and binary artifacts are flagged (exit 1), recorded in
  `build/circle-advanced-public-source-audit.json`. No publication performed.

## Corrections and ownership

Independent review found stale-reference checks blocked typed Cancel/Esc;
those recovery commands now bypass the guard and clear Rust/native references.
It also found tiny Deformable circles failed because internal samples used the
user-pick distinct-distance tolerance. A red Rust regression reproduced this;
fitting normalized unit samples before scaling fixes it, with Rust/native
tiny-circle checks passing. Invalid hover now restores the session prompt
instead of retaining a previous successful measurement. Its native check
reuses the exact center-pick pixel to avoid screen/world rounding ambiguity.

Safe Rust owns state, numeric constructions, best-plane fitting, spline rebuild
and deviation. C++ is required for FreeCAD/Qt selection/transactions and OCCT
curve/GCC APIs; native objects stay GUI-owned with revision checks. Test macros
use the FreeCAD Python API. Native FFI/dependencies are not certified memory-safe.

## Remaining acceptance scope

General nonplanar/different-plane Tangent and tangent Vertical combinations,
exhaustive/certified NURBS roots, individual surface edit-control-point selection,
associative History, active-layer mapping and full Matrix compatibility remain
unaccepted. Deviation is a sampled estimate; host tolerances/defaults are recorded
separately from source facts. The full feature is not marked complete.
