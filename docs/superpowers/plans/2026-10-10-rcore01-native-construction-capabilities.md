# RCORE-01 native construction capability integration

## Native Builder conversion observations — 2026-10-11

Original authority: representation analytic→NURBS must report conversion and
measured deviation; preserve locus separately from parameterization; adapter5
validates before publication. Keep all original numbered requirements open.

- [x] Retain actual OCCT Copy/GTransform/Transform ModifiedShape lineage through
  the existing affine world/local pipeline, without an index-identity assumption
  or replacement solver. Include owned template input snapshot and subshape map.
- [x] Observe original edge parameters and native projected output parameters,
  native vertex values and trimmed-face surface samples. Safe Rust checks counts,
  identity/domain/frames and independently computes residuals; inconsistent
  measurements must not publish accepted output.
- [x] Publish measured sample deviation and recorded parameter-query evidence
  into the current native Builder GeometryResult, including History/FCStd.
  Sample evidence cannot certify a global bound, interpolation or general raw
  parameter reuse. Original conversion/all-family requirements remain required.
- [x] Native analytic circle under nonuniform scale, trimmed hole and open
  cylinder plus existing Builder/import regressions on one matching plugin;
  full Rust and scoped progress evidence synchronization.

Implementation ruling: FreeCAD TopoShape::transformGeometry delegates to an
extra BRepBuilderAPI_Copy followed by BRepBuilderAPI_GTransform; makeTransform
with copy=true delegates to BRepBuilderAPI_Transform. Use those same native
operators in the minimal OM9 bridge so their actual ModifiedShape histories can
be retained. Root Location changes update mapped locations by composition;
source/output enumerations are namespaces, not persistent identities. Native
geometry/projectors require C++; bounded independent mappings/samples and
residual policy remain owned by Rust. The existing source/slot/transaction and
input-tree/output validity contracts continue to apply. No FreeCAD root change.

Portable RED: prepared_result8/10 pass; current assembler ignores supplied native
observations and accepts a deliberately inconsistent measured output point.
Logs: build/builder-result-rust-conversion-red1-20261011.log and actual exit1.

Native conversion GREEN2: 16/16, actual host exit0 on plugin
de1945ab5e920021e7870a755ebf28a9b4abf9c189a9d7b001dc7ad0f0cebd5e;
fixture preserved in build/builder-conversion-green2-20261011. Initial native
RED13/15 on prior B120, then candidate GREEN1 15/16 exposed a fixture assumption:
OCCT's actual output is Geom_TrimmedCurve wrapping a degree2 rational NURBS.
GREEN2 verifies the saved basis/weights directly, keeping the actual native class.
Portable namespace RED11/12 -> GREEN12/12 rejects nonexistent native ID77,
missing/noncanonical namespace IDs. Full Rust conversion-full3: 756 tests/108
nonempty suites, actual exit0. Matching Builder/import regression batch is still
running; inspect it before completing the fourth scoped checkbox. No global
deviation bound or full-domain parameter mapping has been proved.

## Builder admission reproduction — 2026-10-11

`build/builder-capability-probe.FCMacro` ran on the Circle History plugin
`1b7eca8a2c80bb8bca4a0ff4676e2cb3244684da6b41f3fb92b8aa30758e06c5`:
9 checks,5 failures, no fixture exception, actual FreeCAD exit0. Reports and
launch are in `H:/FreeCAD-src/build/builder-capability-red1-20261011`.
The native affine Builder accepted a seed BSpline with a pole at1e151, created
record/output objects and consumed existing executable Redo. A Mesh seed was
rejected without writes but lacked typed Builder/seed context. Persisted `{}`
recipe cleared its dependent output and recovered after restoring valid raw
parameters, but its diagnostic lacked typed record context. These are input
admission/diagnostic failures, not evidence to replace the affine evaluator.

- [x] Add independent owned Builder request/options to the existing Rust
  operation/type/options registry, reusing feature identity, recipe parser,
  affine-plan dimension limits and aggregate replay budgets. Preserve the
  currently accepted Part representation set and registered affine evaluator.
- [x] Bound foreign string/dimension headers before access and retain no
  native pointers; prove concrete unsupported types/options and owned request
  independence with meaningful Rust RED/GREEN.
- [x] Before source/template copy, native extraction or transaction, admit
  the actual stored request and borrowed native geometry using existing
  InputTree validation. Re-admit final dimensions/options before solve.
- [x] Native GREEN of the reproduced cases must preserve source/Undo/Redo,
  durable recipe links and valid affine output; run existing Builder/group,
  Match/restore and fresh-process FCStd regressions on the changed plugin.
- [ ] Full GeometryResult/maps/attributes/source-lock-cache/lifecycle and
  original all-family review/acceptance remain separate required work.

Builder implementation ruling: a value-owned NativeBuilder request belongs
in the same operation registry as NativeCage/NativeEdit, using the existing
affine-template evaluator rather than feature-name inference. Capture admits
bounded raw recipe, exact syntactic feature ID, actual Part representation,
template/target counts and ScaleToGem before shape copies. Construct must also
carry the measured initial/current dimension triples and reuse affine_plan.
Accept the existing native Part Point/Curve/Polycurve/Face/Shell/Solid/Brep
set; Mesh/cloud/instance wrappers remain explicitly unsupported by this API.
The ID does not certify a setting/cutter implementation or enable a menu item.

The synchronous status FFI will receive byte pointers with explicit lengths
(existing1 MiB recipe bound and maximum24-byte feature ID), stage/type/count
scalars and an optional six-double dimension buffer. Null/header length,
alignment/address overflow and dimensions cardinality must precede reads.
Native callers retain live immutable buffers for one call; Rust retains only
owned values in the policy result. Cancellation/worker ownership is not implied
by this synchronous query. The permanent C++ exception is document/property
resolution, native topology/type observations, OCCT copy/transform, GUI-thread
transactions and recompute callbacks. Preserve existing frame composition,
source shape tags, slot identity, Match aggregate limits and output detach rules.

Portable request checkpoint: `native_builder_capability::{Request,policy}` is
now integrated through `OperationId::NativeBuilder` and
`OperationOptions::NativeBuilder`. It reuses existing feature ID/recipe parsing,
affine dimension validation and replay budgets; all seven native Part kinds
remain admitted. Construct requires dimensions, actual non-Part inputs and
invalid counts/options are rejected with typed policy errors. The captured
request owns its strings and scalar values, with no native geometry/pointers.
Rust policy RED2:0/2 -> GREEN1:2/2; full741 tests/107 suites actual exit0.
Evidence: `build/native-cage-builder-policy-{red2,green1,full1}-20261011.log`
and their `.exit.json` files. The first RED attempt was a fixture compile error
from a misspelled Retained tag and is not behavioral RED evidence.
Public source audit:2416 files,511 bindings,510 SVGs,no errors.

Native Builder input-admission checkpoint now follows this portable slice:
bounded FFI RED0/3 -> GREEN3/3, full744/108 suites exit0; standalone Builder
plugin build exit0. Native RED4/9 -> GREEN9/9, expanded61, existing warm43/
groups10/cold3+4 all pass. Matching1484/32 actual host exits0 use one new plugin.
See docs/validation/2026-10-11-rhino-core-builder-policy.md and its receipt.
Circle1363/27 is historical; the new integration reruns the relevant suites.
This accepts input/solve admission only. Original GeometryResult/maps/source-
revision-lock-cache/ownership and all-family completion are still open.

Original authority: adapter items1/3/4/5/6 and C10/T07 in RCORE-01. The full
chapter remains the objective; completing this plan cannot close unrelated
conversion, ownership, worker, cache or persistence requirements.

Use executing-plans inline. Reuse existing Rust sessions, OCCT/FreeCAD solvers
and the standalone plugin/shared SDK. Safe Rust owns operation/type/options
decisions; native code only resolves host facts, copies and invokes native APIs.

## Current source evidence

- `CurveController::available/start` uses the existing command/session mapping,
  document/view/permission checks and `CoreRebuild` routing. Geometry creation
  remains in existing native adapters, with no common operation capability gate.
- `SurfaceController::available/start/preview/commit` invokes the existing
  Sweep1/Sweep2/Loft session and `buildSurface`. `surface::Kind::options_valid`
  already owns the real style/closed policy. Reuse it in the shared registry.
- `SurfaceGeometry::surfaceWire` copies the borrowed root Shape before its
  selected Edge/Wire extraction. The shared native input-tree gate is absent
  at this point. This is source evidence, not a reproduced runtime failure.
- `SolidController` supports existing Box/Sphere modes and native curve
  references. `SolidReferences::solidCurveReference` reuses `nativeShapeInput`,
  now with native preflight. Availability/construction still needs the common
  operation policy; other FitPoints/native conversion paths need separate audit.

## Required implementation

- [x] Inventory concrete currently enabled Curve/Rebuild/Surface/Solid families,
  actual input/output types, modes/options and existing feature IDs. Keep
  missing-input commands able to prompt; do not authorize Unknown concrete data.
- [ ] Extend the versioned shared Rust policy using actual existing session
  types/options. Reuse existing numeric/basis/cardinality validation; do not
  write new CAD solvers or a parallel arbitrary command-name registry.
- [ ] Demonstrate meaningful Rust policy RED, including unsupported types/modes
  and preservation of existing Inspect/exchange/native Edit behavior; GREEN.
- [ ] Before native source copy/extraction/evaluation, run the existing native
  input-tree validation on the borrowed owner. Resolve selected subelement
  representation accurately without breaking currently working selection/frame
  behavior or changing whole-object policy to hide unsupported cases.
- [ ] Consume shared capability at controller declaration, current request/input
  and solve boundaries. Existing native permissions, source stamps, placement
  and kernel validity checks still apply.
- [ ] Real host RED/GREEN for malformed native Surface input and current source/
  Undo/Redo preservation; valid Sweep/Loft, native curve creation and Box/Sphere
  regression must retain current semantics and actual placement.
- [ ] Run full Rust and matching relevant native host checks on the standalone
  plugin. Preserve already accepted native Edit and prepared CAD/mesh/cloud
  regression coverage; use longer measured timeouts for the large Edit suite.
- [ ] One fresh source review when this implementation is ready, then correct
  findings, synchronize original spec/README/ledger/evidence and retain all
  remaining chapter obligations. Do not equate registry tags with complete
  GeometryResult/conversion/parameter-map integration.

## Concrete source inventory — 2026-10-10

This inventory records current code, not runtime acceptance of a new shared
construction policy. The native Edit regression process uses the unchanged
plugin while these source facts are recorded. Existing fixtures below must be
replayed on the construction implementation before that implementation is
accepted. No new solver, command identifier or supported mode is proposed.

| Existing family / feature | Current input and output | Existing policy to reuse |
|---|---|---|
| Line / OM9-CURVE-002 | Typed world points -> native polygon edge | `CurveSession`, shared coordinates/frame/units and current Line input rules |
| Polyline / OM9-CURVE-001 | Ordered world points -> connected native wire | Existing point count, closure and session undo/cancel rules |
| InterpCrv / OM9-CURVE-003 | Ordered world points -> native BSpline edge | Existing `spline::interpolate`, odd Degree 1..11, Uniform/Chord/SqrtChord knot policy and publication basis validation |
| Rectangle / OM9-CURVE-004 | Points/dimensions/frame -> native polygon wire | `rectangle::Mode::{Corners,Center,ThreePoint,Vertical}` and existing dimension/frame validation |
| Circle / OM9-CURVE-005 | Points and optional native curve/fit-point references -> analytic wire or deformable spline wire | Existing Center/TwoPoint/ThreePoint/Vertical/AroundCurve/FitPoints/Tangent modes, Radius/Diameter/Circumference/Area, deformation basis and native reference guards |
| Ellipse / OM9-CURVE-006 | Points and optional native curve reference -> analytic wire or deformable spline; optional focus points | Existing Center/Diameter/Corner/Vertical/FromFoci/AroundCurve modes, deformation basis and `MarkFoci` output policy |
| Rebuild / OM9-CURVE-009 | Whole connected curve objects, up to 16 -> rebuilt spline edge | `CoreRebuild` is separate from `CurveSession::start`; current PointCount 2..256 / Degree 1..11, DeleteInput, source/result gates and deviation calculation |
| Sweep1 / OM9-SURFACE-001 | One rail + one or more curve sections -> native surface BRep | `surface::Kind::Sweep1`, existing section fitting/frame/closure and native validity checks |
| Sweep2 / OM9-SURFACE-003 | Two rails + one or more curve sections -> native surface BRep | `surface::Kind::Sweep2`, original transported-section/OCCT routes, MaintainHeight/slash/contact constraints |
| Loft / OM9-SURFACE-009 | Two or more curve sections -> native surface BRep | `surface::Kind::Loft::options_valid`: styles 0..5; closed Developable rejected; closed Loft requires at least three sections |
| Box / OM9-SOLID-012 | Points/dimensions/CPlane -> standalone BRep solid | Existing Corners/Diagonal/ThreePoint/Vertical/Center/Cube modes and Rust dimensions/frame policy |
| Sphere / OM9-SOLID-014 | Points/dimensions, optional curve constraints or FitPoints -> standalone BRep solid | Existing Center/Vertical/TwoPoint/ThreePoint/FourPoint/FitPoints/AroundCurve/Tangent modes and Rust fit/dimension policy |

Authoritative entry points are `rust/src/curve.rs::name/CurveSession::start`,
`rust/src/{rectangle,circle,ellipse,solid,surface}.rs`,
`Gui/CurveController.cpp::isCurveCommand/available/start/commit`,
`Gui/CoreRebuild.cpp::options`, `Gui/SurfaceController.cpp::handles/add`,
`Gui/SurfaceGeometry.{h,cpp}::SurfaceOptions/surfaceWire/buildSurface` and
`Gui/SolidController.cpp::handles/start/reference/submit`. Feature identity
comes from the existing Curve/Surface/Solid specs and current native properties;
the authored menu symbol manifest alone does not establish command support.

### Native input boundaries still requiring integration

1. `SurfaceGeometry.cpp::surfaceWire` borrows root `Shape`, then immediately
   calls `copy(True, False)` before chain traversal/subelement extraction.
   Gate the actual borrowed selected shape before its copy, and each referenced
   chain shape before copying. Resolve subelements before cloning the owner;
   whole-root basis gating would reject a valid edge because of an unselected
   malformed sibling. Retain current Edge/Wire/Compound handling and parent
   placement behavior; the Edit helper's Link rejection is not a drop-in
   replacement for this Surface reader.
2. `CoreRebuild.cpp::options` borrows `Shape`, calls `copy()` and transforms
   it before inspecting Faces/Edges. It does not use `nativeShapeInput` or
   the shared native input-tree gate. Rebuild's previously accepted delayed
   delivery/source gates do not prove this earlier copy boundary is protected.
3. `SolidReferences.cpp::solidSelectedFitPoints`, shared by current Circle
   and Sphere fitting, extracts CAD poles/vertices, Mesh.Topology or
   Points.Points before enforcing its 1024-result limit. The current reader
   rejects App::Link explicitly. Reuse bounded native Part/Mesh/cloud readers
   before these Python list-producing APIs; retain the current fitting and
   world-placement semantics. A per-result limit after extraction is not
   evidence of pre-extraction allocation bounds.
4. `solidCurveReference` already uses the now-gated `nativeShapeInput`;
   Circle/Ellipse/tangent paths reuse that reference reader. Keep its working
   owner/source guards and native copy behavior instead of implementing a
   parallel reference path. Remaining conversions/evaluations still need
   their operation-specific shared policy and result reports.

`SurfaceOptions` contains more than style/closed: Frenet, MaintainHeight,
sectionMode/PointCount/tolerance, continuityA/B, matchStart/End, History and
slash pairs. Integrating only `Kind::options_valid(style, closed)` would
leave actual enabled options outside the original C10 contract. Reuse or
migrate their existing decisions into the shared Rust policy; runtime source
facts and OCCT validity remain native observations.

Existing regression fixtures to reuse include `curve_smoke`, `curve_spline_smoke`,
`rectangle_smoke`, `circle_smoke`, `circle_advanced_smoke`, `circle_input_context_smoke`,
`ellipse_smoke`, `solid_commands_smoke`, `solid_options_smoke`,
`surface_commands_smoke`, `surface_options_smoke`, `surface_advanced_smoke`,
`surface_constraints_smoke` and `surface_history_smoke` (all `.FCMacro`). Their
presence is an inventory fact; acceptance requires current matching runtime
results. Follow the original RCORE-01 schema/ownership/typed-error contract
through every boundary above rather than treating registry tags as completion.

## Input-boundary execution — 2026-10-10

Ruling: implement the reproduced native source admission defect before extending
the construction registry. The existing Rust-backed InputTree already owns the
numeric/basis and traversal policy; the missing code is only its native consumer
adapter. This ordering fixes original T07 with real RED/GREEN while preserving
the required later operation/type/options, result and conversion work.

The unchanged plugin AE403DF557E6C8DED0B24741A92067D455327979DD3A3A1518D27DAD7C6165BE
ran `build/rhino-core-native-construction-input-probe.FCMacro` through the real
GUI. Twelve assertions ran without a fixture exception; three failed: Sweep1,
Loft and Rebuild admitted a BSpline pole at1e151 without the required InvalidBasis
operation/object diagnostic. The other nine passed, including source geometry,
Undo/executable Redo and valid Edge2 Loft with an unselected malformed sibling.
Actual host exit0 is stored in `build/native-construction-input-red1-20261010`.

The native bridge uses borrowed live Part::TopoShapePy wrappers on the GUI
thread/GIL and calls the existing Rust-backed InputTree. No numeric policy,
solver, Rust retained pointer, worker or new command is added in C++. Surface
copies only the selected geometry after validation, preserving parent placement;
chain readers no longer clone an unused root. Rebuild validates before its copy.
Standalone build `native-construction-input-build1-20261010.log` finished with
actual exit0, using the shared SDK/cache and only OpenMatrix9Gui. The same
unchanged twelve assertions passed in `native-construction-input-green1-20261010`
with actual host exit0: the diagnostic comes from the existing Rust basis gate
(status12), and the source/Undo/Redo and valid selected Edge2 checks remain green.
The public fixture is `tests/rhino_core_native_construction_input_smoke.FCMacro`.
Public source audit passes with2356 files and no errors. Seven unchanged
Surface/options/advanced/constraints/History, Curve/Rebuild and native Edit
regressions finished sequentially under session48841 with actual runner exit0.
The same plugin passed Surface commands97, options180, advanced108,
constraints85, History65, Curve/Rebuild41 and native Edit capability20.
Together with input GREEN12 this is608 checks across eight actual host exits0.
The source/evidence verifier and package manifest retain original chapter status
in_progress. This is not full construction-plan acceptance or chapter closure.
Link computed getters, selected-topology extraction
budgets, constraint support and FitPoints readers remain explicit open work.

## Shared Surface policy execution — 2026-10-10

Read the pinned Guide PDF25–31 (object representations),56 (Loft),64–67
(Sweep1/2), then the exact Rhino5 Windows Sweep1, Sweep2 and Loft Help. The
current OM9 source/fixtures remain the behavioral baseline for this migration.
Rhino documents additional options and different eligibility (for example Loft
Match Tangents needs three curves); this policy preserves the existing OM9
two-to32-profile constrained slice. It does not establish Rhino parity or enable
Point endpoints, Roadlike modes, rail refit or other missing solver branches.

References read: https://docs.mcneel.com/rhino/5/help/en-us/commands/sweep1.htm,
https://docs.mcneel.com/rhino/5/help/en-us/commands/sweep2.htm,
https://docs.mcneel.com/rhino/5/help/en-us/commands/loft.htm.

Ruling: reuse the existing EditSurface operation and Surface Kind/feature IDs,
with one owned typed NativeSurface request variant in the common registry.
Options/capability values become Clone rather than Copy because slash pairs
are owned Vec data. This adds no native pointer or retained host owner and
does not change existing scalar exchange/Edit ABI layouts. Native source/type,
closedness, support-face and kernel validity checks remain required adapters.

The first test attempt identified missing API/types and is not counted as
meaningful RED. Compile-only request scaffolding returned UnsupportedOperation
for all Surface requests: eight tests ran, seven failed for absent supported
capabilities and one passed for unchanged inspection/exchange/native Edit.
`native-surface-rust-red1-20261010.log` retains the actual functional failure.
The implemented safe Rust policy passes all eight focused tests in
`native-surface-rust-green1-20261010.log`: styles/closed cardinality, section
mode/count/tolerance, MaintainHeight, bounded monotone slashes, continuity/
Match Tangents, actual input types and declaration-versus-construction stages.
Frenet/History/HistoryCommand/Preview context is retained in the owned request.
It reuses Kind::options_valid and surface::slash_parameter; no solver is added.

Full Rust verification before the ABI passed687 tests with actual exit0
(`native-surface-rust-full1-20261010.log`). A72-byte repr(C) native envelope
now carries initialized scalar kind/type/stage/options/context and an optional
immutable borrowed array of at most128 f64 values (64 slash pairs). Masks,
count, alignment/address overflow are checked before access; the caller must
still guarantee live initialized storage. Rust owns a bounded Vec copy with
try_reserve_exact, retains no caller pointer, frees it on the Rust side, and
catches unwinding panics as status103. Layout/offset tests establish the ABI.
Declaration uses zero count/null pointer and never authorizes Unknown input.

Compile-only ABI scaffolding produced meaningful RED0/4; actual decoding and
shared policy give GREEN4/4 (`native-surface-rust-ffi-green2-20261010.log`).
An intermediate syntax error is retained in ffi-green1 and excluded from GREEN.
Tests cover current native type codes, malformed masks/stages/numbers, null/
misaligned/oversized slash envelopes, monotone finite pairs and unchanged caller
storage. Fresh full Rust verification after this ABI implementation passed691
tests across95 suites with actual exit0 (`native-surface-rust-full2-20261010.log`).
Public source audit passed2361 files with no errors. These are Rust/source
checks, not evidence that the old native plugin consumes the new policy.

Native controller declaration/capture/build consumption and matching host
evidence for this new Surface policy are not implemented yet. The current
0E6D4AF7 plugin still proves
the earlier608-check native input checkpoint only. Remaining construction
families, complete output/results/conversion and all original obligations stay
open. Next: wire the bounded72-byte native request
bridge at real existing Surface boundaries (available, concrete capture and
buildSurface, including History/continuity routes), then standalone build and matching
host regressions before updating that policy's acceptance status.

## Native Surface integration — 2026-10-11

Standalone incremental confirmation actual exit0. Native declaration, capture and
construction now consume the bounded request ABI. Matching binding29/input12 and
seven unchanged regressions total637 checks/nine actual host exits0. The policy
uses current native rail closedness and retains existing solvers. Full691 Rust
evidence has unchanged source fingerprints. This is Surface-only progress;
plan checkboxes retain their all-family scope and remain open.

Evidence: [native Surface integration](../../validation/2026-10-11-rhino-core-native-surface-policy.md).

## Native Surface diagnostic correction — 2026-10-11

Ruling: retain the real source/subelement at native request rejection rather
than the generic selected-curves label; Rust policy is unchanged. RED2 isolated
one identity failure after finishing initial native feature execution before
the byte baseline. GREEN6 and binding29/input12/History65 passed on the new
plugin with four actual exits0. The earlier637 checks belong to their previous
plugin. Full chapter and all-family plan remain in progress.

## Native FitPoints admission — 2026-10-11

Ruling: share one builtin native reader across Circle/Sphere and Circle History;
Rust owns checked cumulative1024-point admission and reuses existing coordinate
limits. Preserve current native placements, fit solvers and selection behavior.
Functional Rust RED1/4 → GREEN4/4; full695 actual exit0. Native RED6/18 diagnostic
failures and typed-budget RED1/45 failure → GREEN45. Matched unchanged regressions
bring the final plugin to589 checks/seven actual host exits0. Bounding-box center
was an invalid analytic-precision metric; keep the1e-6 bound and compare actual
curve/surface centers. All-family plan checkboxes remain open.

## Native Rebuild policy — 2026-10-11

Ruling: continue existing family integration with Rebuild after Surface and
bounded FitPoints reads, retaining the original all-family scope. Reuse
spline::rebuild_options and the existing native source rules. Concrete Compound
contents are checked before private copy; generic Brep authorization alone is
insufficient. Scalar ABI retains no pointers/owners. Meaningful Rust policy
RED1/5 and ABI RED0/2 → GREEN7; native diagnostic RED2/6 and Compound RED2/8 →
GREEN on the final plugin. 383 matching checks/nine actual exits0; full702
Rust tests. All-family checkboxes and original chapter obligations remain open.

## Native Curve policy — 2026-10-11

Ruling: use the six existing typed sessions and original authored feature IDs;
correct the inventory's reversed Line/Polyline IDs. Cost if wrong: mismatched
feature evidence. Cross-spec RED0/1 -> GREEN proves the corrected mapping.
Rust owns portable request/options policy and session copies. Native callbacks
resolve borrowed host facts and never retain owners. Effective Ellipse MarkFoci
preserves the latent option behavior. Independent History replay remains separate.
Rust RED0/5 +5/7 +0/2 -> GREEN9, then feature identity RED0/1 -> GREEN10;
full712. Native diagnostic RED2/6 and owner RED10/36 -> GREEN on final plugin.
680 matching checks/15 actual exits0. Original all-family checklist stays open.

## Native Box/Sphere policy — 2026-10-11

Ruling: reuse existing Solid kind/mode and mm dimensions instead of adding a
parallel command registry or solver. Cost if wrong: rejected existing modes or
incorrect native output; mode/frame/signed-extent and runtime fixtures cover it.
Rust request snapshots bounded actual options and scalar plans. Native adapters
resolve builtin owners and select Curve/Solid queries only. Shared declaration,
preview/candidate-output and pre-kernel commit gates are active. Current Sphere
capture is explicit; independent History consumers keep their original behavior.
Meaningful Rust RED0/5 +5/7 +0/2 -> GREEN9, full721; native diagnostic RED6/18 ->
GREEN18. 1071 matching native checks/18 actual exits0. Remaining original
all-family acceptance, final review and full chapter obligations stay open.

## Existing Cage services — 2026-10-11

Ruling: extend this existing construction plan to the enabled Cage consumers
explicitly left open by RCORE-01. Reuse `cage_commands::Kind/Phase`, exact
OM9-TRANSFORM-041/020/022 identities, current box count/degree/budget/frame
validation and the existing rational deformation/native services. Cost if
wrong: lost API-only creation/restore/release behavior or rejected working Cage
options; portable and real-host fixtures below cover those distinctions.

The source inventory includes `CageController::handles/add/snapshot/commit`,
`CageFeature::{create,capture,prepare,restoreArchive,release}` and independent
`CageBinding::execute`. Current creation is World BoundingBox, counts2..128,
degree1..16, count>degree, total<=1,000,000, defaults2/1; Capture uses an existing
native3D control, Global/Local and finite nonnegative Falloff. Restore is explicit
and consumes the verified archive's own relationships. Direct creation accepts
explicit bounds without an interactive source selection. Direct Release also
accepts active binding storage, and must detach a broken captive without
evaluating its geometry. Do not add a cap on selected captive count or enable
the additional Rhino modes documented in the Help.

Rust owns a small options request copied from the actual session and an
independent136-byte scalar envelope for APIs/recompute. Native geometry owners,
point buffers and source signatures are never cloned into that request. Native
C++ resolves builtin Part/Mesh and invokes existing Rust-backed InputTree and
mesh owner/value/byte gates before clone/Topology extraction. Release validates
the binding's recorded representation because it consumes relationships rather
than solving source geometry. The remaining full rational-control/binding
allocation gates are not proved by this options request.

- [x] Portable policy RED0/5 -> GREEN5, session RED5/6 -> GREEN6, ABI/unknown
  role RED0/3 -> GREEN3; full Rust730 tests/104 suites actual exit0.
- [x] Native unchanged-source RED36/54 -> first GREEN54/54, actual host exits0;
  original API Basis and MeshBudget mutation/Redo failures are corrected.
- [x] Final matching Cage command/deformation/lifecycle/3DM restore and earlier
  Curve/Solid/native Edit/prepared regressions on the final plugin.
- [ ] Complete remaining rational-control/binding allocation gates, native
  results/conversion/maps, History requests and original all-family acceptance.

The first unchanged Cage deformation regression ran27 assertions without a
fixture exception and found one wording incompatibility: it requires the
existing `Empty captive shape` diagnostic. Keep that text within typed
InvalidTopology operation/object context; rebuild2 and rerun acceptance.
Source audit2404 files had no errors before this wording correction. Neither
this section nor a green slice marks the full chapter or plan complete.

Next native allocation boundaries: `CageFeature::cageHandle` calls `flat` on
ControlPoints before its Rust descriptor count/basis checks; controller control
snapshot serialization traverses the same basis without that admission.
`CageBinding::execute` flattens SourceVertices/BoundParameters before checking
their lengths, and `cage::om9_cage_create_placed` clones borrowed knots/points/
weights before value validation. Preserve the imported-basis count rules
(different from generated box128), independent recompute/release and existing
rational solver while moving borrowed admission ahead of those allocations.

Final Cage acceptance: 1268 checks/23 matching actual host exits0 on build2; full Rust730. docs/validation/2026-10-11-rhino-core-native-cage-policy.md. Original all-family and chapter obligations remain open.

## Borrowed Cage descriptors and binding values — 2026-10-11

Ruling: borrow native standard-layout XYZ values synchronously, proved by
compile-time size/alignment/offset assertions, instead of flattening before
Rust admission. Cost if wrong: invalid foreign slice layout/lifetime; build
assertions and GUI-thread immutable-call ownership are mandatory, not a claim
of native memory safety. Rust validates before owned knot/point/weight copies
and preserves imported clamped-basis budgets separately from box128 creation.

- [x] Rust allocator RED5 copies for malformed weight ->0, borrowed descriptor
  and binding gates GREEN4; full734/105 suites actual exit0.
- [x] Native control selection/API and saved binding diagnostics GREEN54,
  including empty mesh/cached-null BRep and damaged-binding Release.
- [x] Final matching1322 checks/24 actual host exits0, standalone plugin build2.
- [ ] Original results/maps, remaining native allocation and lifecycle, History
  consumers and full all-family/chapter acceptance.

See docs/validation/2026-10-11-rhino-core-cage-borrowed-admission.md. Resume at
independent History replay requests and full prepared GeometryResult consumers;
do not repeat the completed Cage borrowed-basis/binding admission work.

Next-consumer inventory (source inspected, not new acceptance): CircleHistory
copies Recipe/Points then calls solidCurveReference and circle_history::replay;
its native calls must use an independent request rather than the interactive
curve_ffi session. The existing Recipe already owns Circle options and the
original numeric radius; reuse these instead of inventing another recipe.
SurfaceHistory already passes explicit SurfaceOptions to buildSurface, which
queries surfaceCapabilityStatus before private source processing; preserve
that path and audit the native JSON parsing separately. BuilderRecord still
copies source gems/templates before common operation/type/options admission;
it is an enabled consumer in the original chapter, beyond this Cage checkpoint.

## Independent Circle History requests — 2026-10-11

Ruling: use persisted Recipe/Circle options and the existing Capture admission
stage to select the common native backend before point copies/source readers;
do not derive replay permission from the interactive curve_ffi session. Stored
points/radii are already mm. Cost if wrong: unit double-scaling or rejection of
valid persisted options; native valid/legacy/FitPoints/tangent/FCStd regressions
and active-Line independence cover this scope. Complete output GeometryResult
and maps remain separate; Capture policy is not a completed-output certificate.

- [x] Rust existing-replay RED0/2 -> GREEN5, full739/106 suites actual exit0.
- [x] Native valid baseline RED16/22 -> final GREEN33; source BRep/recipe/Undo/
  Redo preserved, independent Line and legacy/Mesh/cloud requests verified.
- [x] Matching1363 checks/27 actual exits0 including two fresh-file cold runs.
- [ ] Builder/other independent requests, native metadata/reader budgets and
  full result/conversion/lifecycle/all-family original acceptance.

See docs/validation/2026-10-11-rhino-core-circle-history-policy.md. Resume at
BuilderRecord::gemFeature/localGem/templates/validate/evaluate before source
and template copy, reusing builder_history's existing recipe/feature/budget
policy. Surface build already receives explicit options; do not duplicate it.

## Builder native admission accepted scope — 2026-10-11

- [x] Explicit bounded UTF-8/dimension headers, no retained native pointers;
  policy2/2 and FFI3/3 meaningful RED/GREEN; full744/108 suites actual exit0.
- [x] Borrowed source/template InputTree before copies, independently stored
  request/options and final dimensions; GUI-thread API admission retained.
- [x] Native RED4/9 -> GREEN61; rejection preserves executable Redo; repaired
  recipe/source/template/dimensions retain links and original evaluator.
- [x] Existing Builder warm43/groups10 and two fresh-file cold3+4; matching
 1484 checks/32 actual host exits0 on one new standalone plugin.
- [ ] Full original output results/conversion/maps/attributes, stale/lock/cache
  publication, other consumers/native budgets and all-family final review.

Next: verify that Builder solved-output rejection precedes transaction and
preserves executable Redo; create/restore currently evaluate outputs after
opening the transaction. The prepared GeometryResult services already exist
and should be reused for the original output contract across consumers.
Surface History raw JSON parsing also needs independent bounded admission;
reuse existing shared Rust Surface options and the existing Surface solver.

## Builder solved-output publication — 2026-10-11

Previous goal turn (base-priority answer): no progress on implementation; source
and current RED artifacts were revalidated before this continuation.

Ruling: execute the original snapshot/solve/validate/transaction contract at the
Builder API boundary, including every target and template before the first
write. Reuse the existing affine evaluator, world-to-local Location composition,
CoreObjects revision tokens, InputTree and CoreTopologySnapshot/ShapeValidation
Rust admission. C++ owns OCCT copies and synchronous GUI publication because
FreeCAD/OCCT objects cannot be transferred into safe Rust. Cost if wrong:
double-applied placements, frozen History replay or changed detached slots;
existing warm/groups/cold fixtures and the new native output cases are required.

The prepared local native shapes are scoped to the API's initial recompute and
cleared on success/abort. They are not durable validity certificates or a worker
cache. Existing detached slots remain unchanged. The new native fixture proves
Create/restore/Match solved-output and multi-slot ownership failures before
mutation, including actual executable Redo. It does not establish full
GeometryResult, conversion/parameter/topology maps or source attribute coverage;
those original all-family requirements remain open and cannot be closed by this
publication repair.

- [x] Meaningful native RED on original plugin54FA: Create, restore, Match last
  target and missing-first/occupied-last restore consume Redo after rejection.
  RED4 fixture fixes setup transactions; earlier RED2 restore setup was weak.
- [x] Rebuild standalone plugin and prove native output/slot cases GREEN28.
- [x] Verify unchanged Builder input admission, warm/groups and fresh cold files
  on the new plugin; inspect placement, detach and future recompute behavior.
- [ ] Full original results/conversion/maps/attributes and all-family acceptance.

Separate observed boundary: Surface History malformed/over-byte/deep JSON probe
completed with actual host exit0 on plugin54FA. Typed child context and bounded
parsing fail; existing repair, sources, links and Redo witnesses pass. This probe
is evidence for the next independent bounded Rust parser change, not acceptance.

Ruling after first build: recompute only the prepared Builder records/outputs
with FreeCAD's nonrecursive recomputeFeature. Full document recompute invokes
Part::Feature::execute, whose Shape.touch advances captured source revisions
for dirty targets even when geometry is unchanged. GREEN1 reached all three
rejection checks but exposed this false stale failure in the repaired Match
case. Avoid mutating unrelated dirty inputs; keep revision checks before and
after publication. The same repaired-case assertion must pass unchanged.

After GREEN28 and Builder warm/groups/cold, refactor the already tested scalar
CAD validity into om9_native_cad_result_admit. Import keeps its own backend gate;
Builder uses only the common native validity, after its actual NativeBuilder
request admission. No import conversion/lifetime evidence is relabeled. Run the
existing validity/solid truth tables against both entry points and rerun native
acceptance on the final linked plugin. Generic output invalidity uses InvalidTrim.

Verification ruling: the shared plugin output changed from62FF to6CE between
otherwise successful native runs; the actor/cause is unproved. A forced relink
then failed LNK1168 on that output. Exclude the mixed-binary runs, preserve them,
and change only the existing standalone plugin cache's runtime output directory
to build/builder-output-pinned1-20261011. No shared FreeCAD configuration/build,
SDK binary copy or host termination. Build the current sources there and rerun
all seven selected acceptance cases serially before writing the checkpoint.

Pinned-output configuration evidence: build6 revealed an existing cached
OPENMATRIX9_RUNTIME_OUTPUT_DIR overriding the requested RUNTIME_ROOT. The
requested pinned bin contained neither pyd nor worker (observed RED). Correct
build/build-om9-plugin.ps1's plugin-only Configure arguments to set both output
variables together. Build7 now links both plugin and import worker into the
requested pinned bin using the same plugin cache and shared SDK. This is a
build-output selection repair, not a FreeCAD root reconfiguration or new host.

Builder output checkpoint: final matching196/7, full Rust744/108; docs/validation/2026-10-11-rhino-core-builder-output.md. Original full results/maps/attributes/all-family requirements stay open. Resume at truthful native GeometryResult context/publication and bounded Surface History JSON.

Native Builder result checkpoint: matching297/9, full Rust747/107; docs/validation/2026-10-11-rhino-core-builder-result.md. Truthful operation/context and captured native seed attributes are integrated into Create/restore/Match/History/FCStd. Original full measured conversion/maps and all-family scope remain open.

Final Builder conversion checkpoint: matching315/10, full Rust758/108; docs/validation/2026-10-11-rhino-core-builder-conversion.md. Recorded native queries and explicit unavailable diagnostics are integrated. Global bounds/full-domain maps and original all-family scope remain open.

## Saved Surface History admission — 2026-10-11

The previous question-only turn supplied no implementation progress. Revalidated
the next concrete defect in build/surface-history-admission-red3-20261011:
actual host exit0, 11/16 checks pass; bytes/depth are accepted and malformed
settings lack typed child context. Its observed plugin hash is26E606, different
from the historical Builder receipt; preserve that receipt as historical only.

- [x] Decode independent saved options/group metadata in Rust before Qt/native
  source capture. Reuse bounded owned JSON and native Surface capability policy.
  Bound bytes1MiB/nodes16384/depth32, inputs256/count4096, slashes64; preserve
  bounded unknown fields and the existing numeric/schema constraints.
- [x] Pass only a fixed caller-owned scalar result through synchronous ABI;
  retain no native pointers and resolve sources solely in the native adapter.
- [x] Preserve solver behavior, sources, raw recipe, links, Undo/Redo and repair;
  clear stale output and identify the child in malformed metadata diagnostics.
- [x] Verify matching plugin with admission and Loft/Sweep/History/persistence
  regressions, full Rust suite, public audit and independent source review.

This fixes saved-request admission only. Full original RCORE-01 results/maps,
all-family policies and ownership/lifecycle acceptance remain required.

Saved Surface History checkpoint: matching698/13, full Rust767/109; docs/validation/2026-10-11-rhino-core-surface-history-admission.md. Original all-family results/maps/attributes/lifecycle requirements remain open. Next concrete consumer: Surface GeometryResult publication before GUI transaction.

## Native Surface GeometryResult — original consumer contract

Previous turn made verified implementation progress. Native RED in
build/surface-result-contract-red1-20261011 proves three valid30mm2 native
Sweep1/Sweep2/Loft outputs but nine missing result/schema/operation checks;
actual host0, no fixture exception, matching6534BF plugin.

- [x] Rust owns an EditSurface result producer using the shared Surface policy,
  saved recipe, complete flattened source refs, native wire snapshots/attributes,
  native rail closure and output parent frame. Require exact group/ref cardinality
  and units/stamps; use the existing native snapshot/validity producer.
- [x] Keep construction evidence independent of import and Builder provenance.
  Report new geometry and uninspected correspondence honestly, no invented
  conversion deviation/global maps or output attributes before publication.
- [x] Native bridge captures refs/wires/attributes and validates output privately,
  then prepares Shape/result before opening the GUI transaction. Use existing
  Surface solver and prepared snapshot factory; verify captured refs at publication.
- [x] History recompute stages Shape/result together, clears both after failure,
  keeps freeze/detach behavior and preserves immutable evidence through FCStd.
- [x] Matching native result, options/constraints/History/UndoRedo/FCStd and
  import/Builder regressions, full Rust, source audit and independent review.

Preserve the full original adapters1–7, C01–C12, T01–T11 and all-family scope.

Continuation audit: the intervening chapter-priority answer made no implementation
progress. Revalidated the portable Surface producer RED (1/2 fails because the
actual EditSurface context is rejected), then implemented owned Rust admission and
the native producer. Initial matching contract12, result81 and three cold-result13
runs pass on B148219B; scoped regression is still running. Read-only review found
output Placement/group replay pairing and unique output-name defects. Add actual
native RED fixtures before repairing these; do not close this checkpoint yet.

Native Surface result checkpoint: matching848/21, full Rust791/112; docs/validation/2026-10-11-rhino-core-surface-result.md. Native frame4 and unique-name6 RED→GREEN verified. Original full all-family conversion/maps/attributes/lifecycle remains open; next actual consumers are native Edit Curve/Surface/Boolean results.

## Native Edit GeometryResult — 2026-10-11 continuation

The intervening chapter-priority answer changed no implementation. Re-inspection
finds the existing seven CAD Edit solvers and capability policy, but commitEdit
does not publish GeometryResult. Keep the full original RCORE-01 requirements.

- [x] Prove the missing CAD results with actual GUI Join/Explode/Trim/four Boolean
  commands; require independent native masters, sources after deletion, exact
  output identity, group frames, Undo/Redo and FCStd evidence.
- [x] Reuse prepared native snapshot factory and shared native Edit policy in
  safe Rust. Validate operation/options, cardinality, stamped refs, source
  attributes/units, output ownership/frame and native validation observations.
- [x] Stage privately owned CAD shapes/results and unique output names before
  transaction; preserve existing solver, DeleteInput, layers and group semantics.
- [x] Validate matching CAD result/command/options/guard/cold regressions, full
  Rust, audit and source review. Runtime14 has terminal per-plugin proof;
  Runtime15 changed-consumer evidence is recorded separately.
- [ ] Preserve prior native executable Redo on rejected publication. The original
  callback fixture remains3/5; scoped passes above do not close this lifecycle gate.
- [ ] Extend the result contract to mesh and special Explode families and obtain
  truthful conversion/maps, native aggregate budgets and original all-family
  lifecycle/acceptance. CAD progress does not discharge these requirements.

Native Edit continuation 2026-10-11T08:55:10+07:00: Rust804/114 passes; native runtime1 producer baseline is historical. Current Union digest false rejection, NaN rollback and abort/Redo fail. Review layer/group/finite repairs have actual RED evidence. Keep all native lifecycle/acceptance checkboxes open; docs/validation/2026-10-11-rhino-core-edit-result.md.

Native Edit continuation 2026-10-11T09:24:06+07:00: Rust805/114 passes; native runtime1 producer baseline is historical. Union/NaN repairs have actual runtime8 proof; native cache refresh, matching regressions and abort/Redo remain under audit. Review layer/group/finite repairs have actual RED evidence. Keep all native lifecycle/acceptance checkboxes open; docs/validation/2026-10-11-rhino-core-edit-result.md.

Native Edit continuation 2026-10-11T09:30:19+07:00: Rust805/114 passes; native runtime1 producer baseline is historical. Union/NaN repairs have actual runtime8 proof; native cache refresh, matching regressions and abort/Redo remain under audit. Review layer/group/finite repairs have actual RED evidence. Keep all native lifecycle/acceptance checkboxes open; docs/validation/2026-10-11-rhino-core-edit-result.md.

Native Edit continuation 2026-10-11T11:00:03+07:00: Runtime14 original CAD110/commands115/options57/special25, guard and cold37 suites have matching terminal proof. Callback abort remains3/5; do not waive either Redo assertion. Runtime14 diagnostic10/12 RED led to operation-specific guard diagnostics; Runtime15 finite10/10 and strengthened diagnostic12/12 pass. Full Rust808/114 has fresh unchanged218-file fingerprints. Source proofs are scoped per runtime. Controlled stops and fixture API failures remain separate; every all-family requirement stays open; docs/validation/2026-10-11-rhino-core-edit-result.md.

Native Edit continuation 2026-10-11T11:23:51+07:00: Runtime14 original CAD110/commands115/options57/special25, guard and cold37 suites have matching terminal proof. Callback abort remains3/5; do not waive either Redo assertion. Runtime14 diagnostic10/12 RED led to operation-specific guard diagnostics; Runtime15 finite10/10 and strengthened diagnostic12/12 pass. Full Rust808/114 has fresh unchanged218-file fingerprints. Source proofs are scoped per runtime. Controlled stops and fixture API failures remain separate; every all-family requirement stays open; docs/validation/2026-10-11-rhino-core-edit-result.md.
