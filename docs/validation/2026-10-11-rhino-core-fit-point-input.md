# RCORE-01 bounded FitPoints input — 2026-10-11

**Scope:** Bounded builtin native point readers shared by Circle/Sphere FitPoints and Circle source/History reads, with Rust cumulative count and existing coordinate admission. Partial RCORE-01 integration. Full chapter and construction-plan status remain in progress.

Rust owns the shared 1024-point count admission with checked cumulative addition,
and reuses the current Circle/Sphere finite ±1e9 mm coordinate rules. Existing
session/batch/fit count checks reuse that admission/constant. New scalar-only ABI
functions retain no caller pointer and contain unwinding panics as status5.
Functional Rust RED1/4 passed → GREEN4/4; full Rust695/96 suites, actual exit0.

C++ is required for builtin FreeCAD property ownership, OCCT pole/topology access,
native placements and GUI/GIL checks. It reads native counts before private point
storage, uses borrowed surface/curve handles with their native location, and calls
the existing Rust-backed InputTree before pole extraction. Mesh/cloud readers take
only native vertices; they do not request Python Topology/facets or full Points
lists. Unique topology maps and traversal visits use existing Rust admission.
The reader preserves current parent/own placement and Link rejection. Existing
Sphere whole-cloud behavior for subelement labels is retained. No fit solver,
command ID, supported numeric range or tolerance is changed.

Native RED2: six missing or wrong-operation/source diagnostics among18 checks;
the twelve source/objects/Undo/executable Redo checks already passed. These runs
reproduce diagnostic defects, not a measured old native-allocation counter.
The original unbounded getter ordering and current bounded access are source
evidence. Additional typed-budget RED:44/45 passed; Sphere failed to include its
one accepted typed point before reading a1024-point selection. The adapter now
supplies its existing Rust point count, preserving failed-batch retry state.

Current native GREEN45 includes malformed curve poles, oversized curve/surface
poles, mesh/cloud counts, cumulative sources,1024-point boundary, own/parent world
placement, typed-plus-selected budget, source and Undo/Redo/retry behavior. The
boundary fixture initially used bounding-box centers; observed offsets were
approximate bounds. It now compares analytic curve/surface center and radius at
the original1e-6 bound, retaining bounding-box measurements as diagnostic data.
No production placement or solver change was needed for that fixture correction.

Matching unchanged regressions: circle-advanced=64, circle-history=53, circle-history-more=89, circle-input-context=16, solid-options=252, solid-commands=70.
Together: **589 checks/seven actual host exits0**, plugin SHA256 `d5744c8bbbceb78f04ec70a57504e25703b4e50d52cc743502d9d997b0dfed0e`.
Standalone build5 returned actual exit0 using the shared SDK/cache; no host rebuild.
Earlier partial GREEN18/39 and prior Surface checkpoints use previous plugins and
are excluded from this current-plugin total.

The first circle-history harness launch referenced a missing macro and timed
out without a results file; its launch exit remains unknown and is excluded
from acceptance. The owned PID was confirmed absent before rerunning. The
runner now checks every macro path before launching and uses the existing
circle_spatial_history_smoke fixture unchanged. The failed directory/log are
preserved separately from the successful fitinput2 runs.

Read exact Rhino5 Windows [Circle](https://docs.mcneel.com/rhino/5/help/en-us/commands/circle.htm)
and [Sphere](https://docs.mcneel.com/rhino/5/help/en-us/commands/sphere.htm) Help; the
pinned Guide object-type pages25–31 remain the chapter reference. Current OM9
specs/fixtures and approved Solid contract govern this technical migration;
this is not a new Matrix default or complete Rhino parity claim.

Remaining work:
- Finish shared operation/type/options policy for Curve/Solid/Rebuild and cage consumers; this point admission is not full C10 closure.
- Computed Link getters, selected getSubShape cache allocation and native explorer depth/stack bounds, constraint support and tangent-plane readers still require integration.
- Complete native GeometryResult/conversion/deviation/parameter/topology maps and original ownership, workers, cache, extrusion/annotation/preservation requirements.
- Full construction-plan regression acceptance and its final source review remain pending; current checks do not replace the earlier full native Edit/prepared-result suites.
