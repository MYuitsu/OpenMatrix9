# Exact rational ellipsoid — 2026-10-23

Target: standalone Rust OM9 on Windows x64, OCCT 8.0.1. Owners:
`OM9-L3-01-09-001` (construction) and `OM9-L3-03-18-014` (Solid menu/CMD).
Logs and fingerprints: `implementation/evidence/M03/ellipsoid-exact-2026-10-23/`.

## Change and ownership

Previously, the ellipsoid constructor scaled an OCCT sphere with `GTransform` and the
menu regression allowed 0.2% volume error. It now constructs a rational biquadratic
9×5 net in safe Rust. The longitude circle uses four quadratic quarter arcs and the
meridian uses two; middle arc weights are `sqrt(1/2)`, with tensor-product weights.
Poles are scaled by the three semi axes before the native call. There is no sphere
conversion, fitting, or geometry fallback in the ellipsoid path.

For each quarter arc, tangent-intersection poles and rational weights describe a
circle. Their tensor product describes a sphere; affine scaling of its poles gives
`(x/a)^2 + (y/b)^2 + (z/c)^2 = 1`. “Exact” refers to this rational representation;
f64 arithmetic, OCCT topology tolerances and numerical integration remain finite.

The C++ exception remains limited to required OCCT API calls: create the rational
`Geom_BSplineSurface`, build its natural-bound face, recognize the duplicated seam
and degenerate pole edges, verify shell closure, and orient the solid. One face is
sufficient, so no sewing approximation is needed. It returns one bounded owned
snapshot through the existing synchronous ABI; no native handle is retained by Rust.
Document/session/transactions/history/publication remain in their existing Rust owners.

## Measurement correction

The old volume deficit does **not**, on its own, prove a geometric approximation.
The former `BRepGProp::VolumeProperties` overload uses non-adaptive Gauss quadrature.
With the new rational net it reports unit-sphere volume `4.227611066462`, although
independent curve/surface intersections verify the net. Both source-volume and
creation measurement now use adaptive quadrature (`Eps=1e-12`), including surface area.

A controlled diagnostic rebuilt the former `GTransform` path with adaptive measurement:
unit volume was `4.188790200151` (relative error `1.107e-9`), versus the old
`4.186958251793` (`4.373e-4`). It still failed the new `1e-10` volume criterion.
This diagnoses most of the reported deficit as integration error, without certifying
that converter's surface. The diagnostic path was removed; the final implementation
uses the authored net. `Convert_SphereToBSplineSurface` and `GeomEval_EllipsoidSurface`
were not rerun or certified by this batch.

## Geometry evidence

`ellipsoid_exact_volume_topology_and_surface` uses six axis triples:
`(1,1,1)`, `(2,3,4)`, `(3,4,5)`, `(0.1,0.2,0.3)`, `(20,0.5,3)`, `(4,3,2)`.
Every fixture has one solid, one shell, zero naked edges and analytic axis bounds.
The canonical B-Rep is serialized/read again with a different display deflection.
83 diameters per fixture exercise axes, seam, poles and a longitude/latitude grid;
498 diameters / 996 intersections agree with analytic fractions 1/4 and 3/4 to `1e-8`.
No display mesh or recipe-derived volume is used as the oracle.

| Semi axes | Measured volume (mm³) | Analytic volume (mm³) |
|---|---:|---:|
| 1 / 1 / 1 | 4.188790204747 | 4.188790204786 |
| 2 / 3 / 4 | 100.530964913938 | 100.530964914873 |
| 3 / 4 / 5 | 251.327412284846 | 251.327412287183 |
| 0.1 / 0.2 / 0.3 | 0.025132741228 | 0.025132741229 |
| 20 / 0.5 / 3 | 125.663706142423 | 125.663706143592 |
| 4 / 3 / 2 | 100.530964913938 | 100.530964914873 |

Maximum observed relative volume error is approximately `9.302e-12`.
The new regression threshold is `1e-10` relative; the existing L2 receipt now admits
`1e-8 mm³` instead of `0.2 mm³` for the 3/4/5 ellipsoid. The menu test's 2/3/4 oracle
also corrects the former mistyped constant to `4*pi*24/3`.

## Validation and remaining acceptance

Build/test commands use `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`, and the pinned SDK
DLL directory on PATH. The receipt records the dirty checkout fingerprints, commands,
test results and native executable. Existing historical records retain their dates.

Full kernel regression: 69 passed, 0 failed, 1 pre-existing ignored lifetime probe.
App library: 176 passed; Solid: 7 passed. Native build and kernel Clippy (`--tests`,
warnings denied) pass. A separate explicitly invoked fixture test cold-opens the
`.om9` saved through the real Windows Save dialog: one closed solid, no naked edges,
volume `100.530964913938 mm³`. That test is ignored in ordinary runs because it
requires an explicitly supplied native UI artifact; its measured invocation passed.

The native Windows run observed menu/CMD construction in all four GPU views,
Icon History Undo/Redo, rejection of zero semi axis, Esc cancellation and Save.
[Saved UI screenshot](../../implementation/evidence/M03/ellipsoid-exact-2026-10-23/ui-ellipsoid-saved.png)
and [machine-readable receipt](../../implementation/evidence/M03/ellipsoid-exact-2026-10-23/receipt.json).
The clean saved window was closed after inspection. This observation covers Ellipsoid;
it does not newly certify the other twenty Solid menu workflows.

Human M03-UI acceptance remains unsigned; an automated observation cannot sign that gate.
No reusable business rule changes: this repairs representation and measurement while
preserving command names, semi-axis inputs and the existing transaction workflow.
