# RCORE-01 native Box/Sphere capability — 2026-10-11

**Scope:** Shared Rust current-session operation/type/options policy for existing Box/Sphere, bounded owned output validation and live Sphere reference/FitPoints admission. Partial RCORE-01 integration. Full chapter and original all-family plan remain in progress.

Rust owns the actual existing kind/mode/stage, phase, accepted point count,
partial dimensions, radius, Point constraint option, bounded owned constraint
flags, frame, construction axes, optional circle plan and fixed15 scalar output.
No point buffer, history buffer or host owner is cloned into the request. Native
Box/Sphere continue to use their existing mm coordinate/dimension bounds and
existing Rust solvers. The frame validation is extracted from the existing
set_frame checks without changing its errors or tolerance. Output validation
uses the existing dimension/frame rules and bounds all Box extrema or the whole
Sphere radius; reflected axes and invalid Sphere auxiliary fields are rejected.
Signed Box input dimensions retain the existing positive-size shifted-origin
output. An empty valid declaration/capture/preview prompts; construct requires
phase6 with an actual validated output. Preview validates its own candidate
output and leaves the live session unchanged.

The scalar synchronous declaration/query ABI uses the existing two Solid kinds,
locks current state, catches unwinding panic and stores no host pointer/buffer.
The existing fixed15 geometry-output ABI is guarded before preview-session copy
and before buffer delivery, with its existing caller-size safety requirement.
This does not certify unsafe/native dependencies as memory-safe. Native C++ is
necessary for FreeCAD owner/property/placement/transaction APIs, Qt interaction
and OCCT shape creation. It only resolves native facts and selects the existing
Rust Curve or Solid query; it does not duplicate the operation/type policy.

Actual controller declaration, preview and pre-kernel commit consume policy.
Live Sphere curve references share the existing native owner/tree validation
and private-copy helper; unsupported Mesh is rejected before extraction/copy.
Sphere FitPoints consumes current policy before the existing bounded native
Part/Mesh/cloud point reader. Circle/Ellipse calls retain their Curve policy;
read-only independent History paths do not acquire a current Solid dependency.
Native shape validity, closed-solid count, positive volume, source guards,
placement, GUI ownership, transaction and original command behavior remain.

Functional Rust RED0/5 -> GREEN5, session RED5/7 -> GREEN7, scalar ABI RED0/2 ->
GREEN9. Full Rust **721 tests/102 suites**, actual exit0. Session-red1 initially
used an unsupported negative numeric Length; it is retained, excluded as proof
of the second failure, and corrected to the already supported signed corner
point input in session-red2. Native Sphere RED has six missing typed-operation/
source diagnostics among18 checks, actual host exit0 without a fixture error.
Source, object, Undo/Redo and executable Redo checks already pass before repair.

Matching final native acceptance: policy=18, binding=93, solid-commands=70, solid-options=252, fit-points=45, curve-policy=36, curve-binding=59, curve=27, curve-spline=41, rectangle=37, circle-advanced=64, circle-history=53, circle-history-more=89, circle-input=16, ellipse=75, surface-binding=29, edit-capability=20, prepared-cad=47.
**1071 checks/18 actual host exits0**, plugin SHA256 `4d60895523f86b658d8e9f4f94989083d6edf510c4a1e2038db7e829f1d49214`. Build1 actual exit0
uses only OpenMatrix9Gui and the shared SDK/cache/runtime. Previous Curve plugin
results are historical and are not included in this current-plugin count.

Reference review used exact Rhino5 [Box Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/box.htm)
and [Sphere Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/sphere.htm),
the original supported feature specs and native fixtures. No new solver, default,
command, unsupported branch, extrusion conversion or precision change is added.
The scalar output is a validated plan, not the complete native GeometryResult,
conversion report or topology/parameter map required by the original chapter.

Remaining work:
- Complete cage consumers and independent History replay requests; finish all-family acceptance and its final review.
- Complete original GeometryResult/conversion/deviation/parameter/topology maps and metadata/source/lock/cache integration for every enabled consumer.
- Computed Link getters, selected getSubShape cache allocation, native explorer depth/stack bounds and constraint/tangent-plane readers remain open.
- Retain original lifetime/worker/preservation/extrusion/annotation requirements; a policy tag or validated scalar solid plan does not close them.
