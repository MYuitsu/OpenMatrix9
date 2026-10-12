# RCORE-01 — Common Curve GeometryResult RED, 2026-10-11T22:31:24+07:00

**Status: in_progress. Task3 and all original requirements remain required.**

The real six existing commands Line, Polyline, Rectangle, Circle, Ellipse and
InterpCrv were executed on preserved runtime50 in an isolated offscreen host.
The fixture completed normally with actual host exit0, no fixture exception:
**12 passed/42 failed in54 checks**. The wrapper returned failure because the
expected result contract is missing, not because startup or kernel failed.

All six independent physical-dimension checks and all six one-Undo checks passed.
Each command already has OM9CurveConstruction, but OM9GeometryResult is absent.
The missing checks cover actual ConstructCurve identity and construction pairing,
original numeric input snapshots before normalization, a full independent CAD
output master and topology, truthful conversion gaps, units and absence of invented
sources, paired Undo/Redo, and exact FCStd result bytes. Six FCStd files were saved.

Expected quantities are literal/hand-derived: Line BothSides length20; closed
Polyline3–4–5 length12; Rectangle4×3 perimeter14; Circle radius1 circumference2π;
Ellipse center(1,2,3), axes5/3; degree1 InterpCrv3+4 length7. Numeric input must
retain the original points, including the Line center/second point before
BothSides generates two new endpoints. Reconstructing output from its master
must agree independently with these dimensions and edge/vertex counts.

The RED conversion checks demand honest uninspected/null fields; they are an
intermediate result-publication gate, not a replacement for the original global
deviation/parameter/topology mapping certification requirements.

The preserved native bridge517/517 and full Rust874/0/0 checkpoint remains
separate. The new fixture was added after build and is identical to its executed
copy. Current Curve production sources match runtime50; concurrent Gumball
changes mean full current-checkout/plugin matching remains false. No Rust or
production code was changed for this RED step; no root build was run.

## Next action

Implement Rust ConstructCurve admission/result using existing native_curve_capability and original accepted-input events, with numeric input at0 and native source refs1..N; reuse prepared CAD output capture and existing solvers. Pair OM9GeometryResult/Shape/construction, then implement checked GUI publication and auxiliary foci/History/Rebuild paths. Replay unchanged native RED54 expectations, cold witnesses and matching regressions; preserve every original RCORE-01 requirement.

## Remaining required work

- All original RCORE-01 requirements remain required: adapters1–7, C01–C12, T01–T11, every enabled consumer, GeometryRef/Snapshot/Result, rational/periodic basis, trims/pcurves/seam occurrences, conversion and complete mapping, independent family read/edit/retain/export.
- Task2 remains open: generic native pick/snap sources for Line/Polyline/Rectangle/InterpCrv and original inputs across every path; full native subset indices, shared full parent snapshots across different selection observations, aggregate allocation/traversal/worker/cancellation bounds.
- Curve OM9CurveConstruction is immutable construction evidence, not a common GeometryResult or current-validity certificate. Exact output CAD master/topology/basis/units, numeric/native input snapshots, auxiliary Ellipse foci, Circle History and Rebuild results remain required.
- Curve checked GUI publication must still revalidate generation/revision/context, complete source content/units/placement/locks/ownership, output and metadata at callback barriers, with real native Undo/Redo preservation. Current source digest guards do not establish every publication barrier.
- Global analytic-to-NURBS deviation/locus and parameter mappings plus full topology maps remain unproven. Native validity and independent physical dimensions do not certify these contracts.
- Offscreen native evidence covers the named command/key/kernel/Undo/FCStd paths only. Actual mouse, preview lifetime in every viewport, OpenGL/render acceptance and all original lifecycle/cache requirements remain open.
- Matching producer inventory includes source/resources/tests selected by extensions, not an exact compiler dependency certificate. Production deployment/ABI acceptance remains pending; the isolated accepted App/GUI host reuses the one SDK.

[Machine checkpoint](2026-10-11-rhino-core-curve-result-red.json).
