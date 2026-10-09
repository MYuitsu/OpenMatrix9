# Native Cage command contract — 2026-10-09

Applies to OM9-TRANSFORM-041 Create Cage, OM9-TRANSFORM-020 Cage Edit and
OM9-TRANSFORM-022 ReleaseFromCage. This records the implemented command slice;
native command acceptance passed against the matching private SDK build.

## Sources and command routes

Engineering sources are Spec v1 `specs/05-transform/om9-transform-041-create-cage.md`,
`om9-transform-020-cage-edit.md` and `om9-transform-022-releasefromcage.md`.
These are requirements and request-planning examples, not solver acceptance.
The authored menu uses TransformCageEditingCreateCage and TransformCageEditingCageEdit.
Their existing FreeCAD identifiers remain `OM9_TransformCageEditingCreateCage`
and `OM9_TransformCageEditingCageEdit`. The added release identifier is
`OM9_ReleaseFromCage`. CMD aliases Cage, CageEdit and ReleaseFromCage, their
feature IDs, and the native menu identifiers resolve through the same Rust
identity mapping and native controller.

The controller calls the public native service directly with owned Python
objects: `OpenMatrix9Gui.createCage`, `captureCage`, `releaseFromCage` and
`restore3dmCage`. It never evaluates command text or Python expressions.

## Selection and options

Cage accepts whole native geometry from preselection, viewport selection or
typed object names. Enter freezes its ordered source snapshots and derives the
union of their current world bounding boxes. The nonmodal panel explicitly
shows World coordinates and BoundingBox. All three extents must be positive
and finite. A flat source rejects instead of gaining an invented thickness.
U/V/WCount start at 2 and U/V/WDegree at 1. These are explicit OM9 defaults,
not reconstructed Matrix defaults. Counts are integer 2..128, degrees 1..16,
each count exceeds its degree, and their product cannot exceed 1,000,000.
The panel validates all six final values together in Rust. Typed individual
options use the same Rust validation. OK creates one independent native
`OpenMatrix9Gui::CageControl` with editable ControlPoints and feature ID 041;
source geometry is retained and no binding is created by Cage.

CageEdit first selects whole captives. Enter advances to a distinct control
role. The user names or preselects one existing native 3D cage, or chooses it
in a separate nonmodal list. Multiple preselected cages require an explicit
choice. Captives cannot include the control. The options panel shows both
roles and Global/Local, with finite nonnegative Falloff in mm, initially 0.
Local uses the current cage world bounding box. OK calls captureCage with
the selected captives and current options. The service rechecks the current
control and captives and stores persistent native bindings. Those bindings
remain active when global History Record or Update is Off.

In the control phase, `Restore=retainedObjectName` explicitly invokes retained
3DM restoration. The list also identifies retained ON_NurbsCage and
ON_MorphControl records as Restore choices. The panel states that restoration
uses the archive's own explicit captive relationships; the preceding selection
is not added to that archive record. OK calls restore3dmCage alone. Archive
integrity, import namespace, source capability, original captive UUIDs and
supported morph conditions belong to that service. Its precise errors are
preserved in the native panel and transcript. No unsupported retained control
is promoted into an editable control merely because it appears in the list.

ReleaseFromCage accepts whole captive preselection or typed object names.
Enter/OK calls releaseFromCage in the service's Undo transaction. It preserves
current geometry, the control and unselected captive bindings. No separate
options are invented for release.

## Ownership, lifecycle and native exceptions

Rust owns identity, permissions gates, phase order, deduplicated input order,
independent snapshot strings, frozen selection and option validation. The C++
adapter reads exact native BRep or triangular mesh topology, object identity,
global placement and world bounds. Cage snapshots also include exact control
points, counts, degrees, full knots, rational weights and reference frame.
Before any service mutation the controller recaptures those values and asks
Rust to reject stale input. Deleted or replaced objects, changed geometry,
changed parent frames and changed control parameters therefore require cancel
and reselect. Native services additionally validate current geometry and
document ownership before committing.

Qt widgets, selection, FreeCAD documents/transactions and Part/Mesh objects are
permanent native integration exceptions: their host APIs and GUI thread cannot
be replaced by portable Rust state. No Qt/document raw pointer is retained by
Rust. FFI text and bounds are copied into owned Rust data; its opaque session is
allocated/freed in Rust and called serially from the GUI thread. C++ local
Python references are RAII-owned under the GIL. This boundary does not certify
the native libraries or the entire application as memory-safe.

The controller creates no document preview objects. Cancel/Esc, document close,
document switch and workbench deactivation discard the uncommitted Rust session
and close native widgets. API failures retain their error and leave the options
available. Successful create/capture/release/restore uses the service's native
transaction and persistent document properties for Undo and FCStd storage.

## Limits and verification

This command slice exposes World BoundingBox creation and existing native 3D
cage capture. CPlane/3Point and base-point modes, automatically created cage
controls during CageEdit, line/surface controls, Accurate/Fast, general refit,
PreserveStructure switches and Other sphere/cylinder regions remain unsupported.
They are not enabled or displayed as working alternatives. Nonlinear general
BRep topology remains subject to the cage service's explicit geometry limits;
no promise of arbitrary polysurface refit or seam preservation follows from
the existence of a CageEdit command.

Rust fixture `rust/tests/cage_commands.rs` covers identity, phase order,
selection freezing, options/defaults, cardinality, finite values, count limits,
missing box extents, selected order, permissions and stale snapshots. The
five command tests cover the portable controller independently. Native fixture
`tests/cage_commands_smoke.FCMacro` covers menu/CMD creation, typed mesh capture,
CV deformation with global History Off, release/Undo, cancellation, document
and workbench switch, stale topology/control data and shared Rust UI validation.
The current matching native command report is
`build/cage_commands_smoke-1/94c78083a5a54e09aad9de3ff1935e54/results.json`
with 27/27 passing checks, loading
`H:/FreeCAD-src/build/reusable-history-runtime/bin/OpenMatrix9Gui.pyd`.
The editable control check uses FreeCAD's returned editor-status list and
requires neither ReadOnly nor Hidden. This verifies the exposed command slice;
the broader native geometry/lifecycle service has separate acceptance evidence.
The final Rust run passed 256 tests across 41 suites; optimized cage/recipe/manifest
suites passed 49 tests. The installed SDK command fixture also passed 26 checks
(the private-path assertion is omitted without an override). See the shared
[validation record](../validation/2026-10-09-reusable-builder-cage.md) for all native
geometry/lifecycle/3DM reports and the runtime checksum.
No full Matrix/Rhino compatibility is claimed by this record.
