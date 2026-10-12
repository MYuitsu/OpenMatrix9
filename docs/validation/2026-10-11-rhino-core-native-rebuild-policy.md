# RCORE-01 native Rebuild capability — 2026-10-11

**Scope:** Shared Rust operation/type/options policy for the existing native curve Rebuild adapter, declaration/capture/construct gates and concrete curve-only topology checks before private copy. Partial RCORE-01 integration. Full chapter and all-family plan remain in progress.

Rust owns the existing PointCount/Degree policy through spline::rebuild_options,
the current 16-curve admission, concrete Curve/Polycurve/Brep decision and the
disabled current-layer option. The portable request retains DeleteInput, preview
and closed input context without claiming native observations or preservation.
Declaration permits zero sources to prompt; concrete capture/construct require
1..16 sources. Unknown/mesh/cloud/face/solid/etc. are never authorized as curves.
Brep authorization is conditional on the existing connected curve-only native
reader, not blanket permission for every Compound. No fitter or defaults change.

The scalar synchronous ABI has no pointers, caller buffers or native owners;
Rust checks stage/type/flags, catches unwinding panics as status103 and retains
no host lifetime. Native C++ is necessary for FreeCAD source/placement/topology,
Qt dialog and native copy/solve/transaction APIs. Actual declaration, capture
before copy and current options before sampling consume the shared registry.
Native basis admission remains in place. The admitted borrowed shape is checked
for faces/edge count/one connected wire before copying only its chosen edge/wire.
Source stamps, parent placement, dependent DeleteInput rejection, deferred result
claims, GUI thread ownership, sampled deviation and transaction behavior remain.

Functional Rust policy RED1/5 → GREEN5/5; scalar ABI RED0/2 → GREEN2/2.
Full Rust **702 tests/98 suites**, actual exit0, on current Rust fingerprints.
Native first policy RED has two missing typed-operation/source diagnostics among
6 checks; native Compound RED has two missing diagnostics among8 checks. Both
actual hosts exit0 without fixture exceptions; source/Undo/Redo and valid
connected Compound checks already pass. The initial missing-console fixture
exception in policy-red1 is retained and excluded from functional RED.

Current matched native acceptance: policy=6, binding=47, compound=8, curve-spline=41, delayed=173, construction-input=12, surface-binding=29, edit-capability=20, prepared-cad=47.
**383 checks/nine actual host exits0**, plugin SHA256 `aa7732dfa13b43c86e26545678e50aad13f58cff83b16b9a0c9f2b926579b957`. Build2 actual
exit0 uses only OpenMatrix9Gui and the shared SDK/cache. Build1 and rebuild1
regressions are historical and excluded from this final-plugin count.
No FreeCAD root rebuild, runtime clone or revised CAD precision threshold.

Read exact [Rhino5 Rebuild Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/rebuild.htm)
alongside the current Rebuild supported spec and native fixtures. Existing curve
Rebuild is reused; Rhino surface Rebuild, layers/subedges/master-curve workflow
and full Matrix compatibility are not added or claimed by this registry work.
Original chapter contracts still govern the remaining implementation.

Remaining work:
- Complete the common construction policies for Line/Polyline/InterpCrv/Rectangle/Circle/Ellipse, Box/Sphere and cage consumers.
- Complete original GeometryResult/conversion/deviation/parameter/topology map and metadata/source/lock/cache integration for all enabled consumers.
- Computed Link getters, selected getSubShape cache allocation, native explorer depth/stack bounds and constraint/tangent-plane readers remain open.
- Retain original lifetime/worker/preservation/extrusion/annotation requirements and full all-family plan acceptance/final review.
