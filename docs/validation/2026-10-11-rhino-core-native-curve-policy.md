# RCORE-01 native Curve capability — 2026-10-11

**Scope:** Shared Rust current-session operation/type/options policy for existing Line, Polyline, InterpCrv, Rectangle, Circle and Ellipse. Partial RCORE-01 integration. Full chapter and all-family plan remain in progress.

Rust snapshots the actual existing session options, input/output counts, plan
presence and captured units into an owned request. The shared registry reuses
existing interpolation degree, approximation basis, fit count and dimension
limits. Circle reference admission depends on AroundCurve/Tangent/FitPoints;
Ellipse curve admission depends on AroundCurve. No new CAD solver, command,
default, precision threshold or blanket authorization of unknown types is added.
An empty valid declaration/capture/preview can prompt; construction needs an
actual plan and valid output count. Ellipse uses effective MarkFoci, preserving
the existing FromFoci -> MarkFoci -> Center behavior and session cancellation.

Existing catalog identifiers drive command declaration. The new scalar ABI
queries the locked Rust session synchronously, catches unwinding panic, returns
typed status and stores no host pointer or caller buffer. Native C++ is necessary
for Qt interaction, FreeCAD owner/placement/transaction APIs and OCCT shapes.
The controller consumes policy at declaration, preview and commit. Live native
Circle/Ellipse curve capture gates borrowed native shapes before private copy;
Circle FitPoints source capture consumes policy before bounded point extraction.
Read-only History verification/replay retains independent existing behavior;
it does not depend on the currently interactive Curve session. Its complete
shared request/result integration remains a separate original obligation.

Native Mesh, PointCloud, Group, Link, wrong Shape owner and missing source are
rejected with operation/source context. Unsupported native owners retain
UnsupportedRepresentation; absent sources report StaleGeometry. Original source,
objects, Undo/Redo and executable Redo are retained for both Circle and Ellipse.
Existing Edit/Sphere calls without the new admission callback retain their
original adapter behavior; matching Edit and fit-input regressions are included.

Functional Rust RED: policy0/5, session5/7, scalar ABI0/2, and original feature
identity0/1. GREEN is10 focused tests; full Rust **712 tests/100 suites**, actual
exit0. The feature identity regression reads the authored specs: Polyline is
OM9-CURVE-001 and Line is OM9-CURVE-002. The initial plan inventory and new policy
had those two IDs reversed; both are corrected without renaming commands.
Native RED6/2 failures and early-owner RED36/10 failures both have actual host
exit0 without fixture errors; preservation checks already passed before repair.

Matching final native acceptance: policy=36, binding=59, curve=27, curve-spline=41, rectangle=37, circle=42, circle-advanced=64, circle-history=53, circle-history-more=89, circle-input=16, ellipse=75, fit-points=45, surface-binding=29, edit-capability=20, prepared-cad=47.
**680 checks/15 actual host exits0**, plugin SHA256 `e09a657ded7924d0b7c043d48f3b5ab98482a76801a86f720e838b7d622866ff`. Build3 actual
exit0 uses only OpenMatrix9Gui and the shared SDK/cache/runtime. Earlier Curve2
host checks and build1/build2 are historical, excluded from this final count.
The installed Python shim failed before audit; the configured project Python
then completed the public source audit with zero errors.

This reuses the supported feature slices specified by the six original Curve
specs. It does not establish all Rhino options or complete conversion/output,
ownership and worker contracts merely because a capability tag exists.

Reference review uses exact Rhino5 Help for
[Line](https://docs.mcneel.com/rhino/5/help/en-us/commands/line.htm),
[Polyline](https://docs.mcneel.com/rhino/5/help/en-us/commands/polyline.htm),
[InterpCrv](https://docs.mcneel.com/rhino/5/help/en-us/commands/interpcrv.htm),
[Rectangle](https://docs.mcneel.com/rhino/5/help/en-us/commands/rectangle.htm) and
[Ellipse](https://docs.mcneel.com/rhino/5/help/en-us/commands/ellipse.htm), alongside
the earlier Circle review and authored feature specs. Unimplemented Rhino
branches remain outside the currently enabled slices; no documentation name is
used as proof of runtime parity.

Remaining work:
- Complete common construction policies for Box/Sphere and cage consumers, and independent History replay requests.
- Complete original GeometryResult/conversion/deviation/parameter/topology maps and metadata/source/lock/cache integration for every enabled consumer.
- Computed Link getters, selected getSubShape cache allocation, native explorer depth/stack bounds and constraint/tangent-plane readers remain open.
- Retain original lifetime/worker/preservation/extrusion/annotation requirements; complete all-family plan acceptance and its final review.
