# RCORE-01 owned supporting NURBS geometry — 2026-10-10

The common geometry descriptor now carries complete owned NURBS support for
each inspected face and UV pcurve, plus standalone 3D NURBS curves. Safe Rust
validates before cloning and emits versioned JSON; native C++ only reads OCCT
and parses the caller-owned byte result. No native handle survives the call.
The old curve/surface `basis` fields remain compatible and reuse that payload.

The selected plugin passed **54 support +177 topology +151 descriptor +55
import gate =437 FreeCAD checks**, all with process exit0 and the same host/plugin
hashes. Full Rust passed **594 tests**, including nine owned support/ABI
tests. Plugin SHA256: `0fdcbbbaaf5d5db6fcbd87910225e0630c68a1500409711cd3cd66147ccbb40f`.
[Machine evidence](2026-10-10-rhino-core-owned-support.json) records source and
runtime fingerprints, launch reports, REDs, reviewed scope and excluded attempts.

Authored controls exercise U-major rational surface poles/weights, a native
periodic surface with compact knots, a restricted periodic curve, a rational
circle, separate full-vs-trim domains, reversed face/edge uses, nonidentity native
location, imported hole/seam pcurves and an 8194-pole over-budget surface. Native
local poles stay unchanged; location and orientation remain separate observations.
Analytic supports/pcurves stay explicitly uninspected without conversion.
Exact source BRep and Undo/Redo survive inspection; returned data stays usable
after source replacement and document close. Regression controls retain existing
topology, source guards, CAD/display independence and import rejection behavior.

The original host RED fails at missing face support; the second fails at missing
standalone curve support after the new periodic/trim/location controls pass.
Rust tests verify exact sized output, malformed surface/curve envelopes, weights,
short-buffer atomicity and actual serialized-limit rejection of valid data with
long subnormal decimals. No arbitrary unreadable pointers are manufactured.
[Fresh review](../reviews/2026-10-10-support-snapshot-review.md) found no actionable
defect; its initial validation gaps were addressed in the final tests.

Budgets apply before native companion extraction and output allocation: 8192
poles, 16384 knots/axis, 2MiB logical basis and 8MiB serialized/aggregate copied
support payload. They do not bound the fully expanded Qt JSON descriptor,
repeated boundary views, retained aliases or global/native allocator overhead.
The ABI requires live initialized aligned immutable input and writable
nonoverlapping output; it cannot certify pointer accessibility or process OOM.
Qt normalizes signed zero, so preservation is numeric, not IEEE bit preservation.

**Full RCORE-01 remains in progress.** Analytic/Bezier and other support types,
attributes, world composition, full manifold/solid/cavity certificates, reports
and remaining mutation/worker/input requirements stay open in the
[full audit](../reviews/2026-10-10-rcore01-closure-audit.md). Existing kernels,
solvers and Line/Polyline constructors are reused.
