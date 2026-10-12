# RCORE-01 existing native Cage policy — 2026-10-11

**Scope:** Common Rust policy for existing Cage/Create/Capture/Release, owned session options and independent API/recompute requests; borrowed Part/Mesh input admission before copy or Topology extraction. Partial RCORE-01 integration. Original chapter and all-family plan remain in progress.

Rust reuses existing cage_commands Kind/Phase, exact feature IDs041/020/022,
box_dimensions and box_reference validation. The request copies only scalar
kind/stage/phase/role, input count, control/retained/explicit-bounds flags,
counts/degrees, bounds, Global/Local and Falloff. It never copies the session's
source/signature buffers or retains Qt/FreeCAD owners. The count policy remains
2..128, degree1..16, count>degree and at most1,000,000 generated controls;
defaults2/1 and Global/Falloff0 are unchanged. Existing native/archived cage
basis remains separately validated by the rational deformation service; the
128 creation limit is not imposed on imported/deformed control bases.

The common policy distinguishes declaration/input capture/construction and
independent recompute. A command with no selection can prompt. Interactive
construction needs completed selection; direct createCage with explicit bounds
needs no selected objects. Explicit retained Restore consumes archived
relationships and needs no preceding interactive captives. Geometry, native
control, retained control and Release binding-storage roles stay distinct.
Unsupported cloud/group/Link/annotation/unknown input is not authorized as CAD.
Pure safe requests and the136-byte repr(C) initialized scalar envelope have
layout/offset tests. The by-value ABI retains no storage; the opaque session
query borrows a live aligned Rust-owned handle synchronously under GUI-serialized
access/free. Invalid kind/stage/role/flags/reserved fields fail closed, unknown
session-add roles no longer become native controls, and unwinding panic becomes
status103. Release builds still panic=abort; native/unsafe code is not certified
memory-safe by these checks.

C++ is a permanent native adapter exception: FreeCAD properties/GUI/document
transactions and OCCT storage require native APIs. CageCapability resolves
concrete builtin shape/mesh owners, calls the Rust decision and existing
Rust-backed native InputTree/mesh budget/value readers before copying Shape,
Mesh or extracting Mesh.Topology. Controllers consume declaration, actual
capture and precommit checks. Independent create/capture/restore and binding
recompute use explicit value requests and never depend on an interactive
session. Restore captives are admitted before its transaction. Release consumes
active relationships' recorded representation and remains able to detach a
broken captive without evaluating source geometry. Existing solvers, global
placement, signatures, transaction rollback, Undo/Redo and FCStd remain.

Meaningful Rust policy RED0/5 -> GREEN5, session RED5/6 -> GREEN6, ABI/unknown
role RED0/3 -> GREEN3; full730 tests/104 suites, actual exit0. The initial
policy-red1 was a compile-only missing Backend match and is excluded from
functional RED. Native old-plugin RED36/54 contains18 failures without a
fixture exception, including API malformed-basis/MeshBudget mutation and
executable-Redo failures. First plugin GREEN54 corrected them. An unchanged
deformation regression then exposed one diagnostic wording mismatch among27
assertions; preserve `Empty captive shape` inside typed InvalidTopology with
operation/source. Build2 addresses the wording; the unchanged regression is
included in final matching acceptance.

Matching final acceptance: policy=54, cage-commands=28, cage-deformation=27, cage-lifecycle=33, cage-restore=55, solid-policy=18, solid-binding=93, solid-commands=70, solid-options=252, fit-points=45, curve-policy=36, curve-binding=59, curve=27, curve-spline=41, rectangle=37, circle-advanced=64, circle-history=53, circle-history-more=89, circle-input=16, ellipse=75, surface-binding=29, edit-capability=20, prepared-cad=47.
**1268 checks/23 actual host exits0**, plugin SHA256 `0f14aade26d69958751ad2f09dc201003f5b9c8ec3730e37dba63bc152151601`. Both standalone
builds exit0 and reuse the shared SDK/cache/runtime, with14 workers (60% logical
CPUs); no FreeCAD root rebuild or independent host copy is made. Earlier Solid
and first Cage plugin evidence is historical and excluded from this total.

Reference review: exact Rhino5 [Cage Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/cage.htm),
[CageEdit and Release Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/cageedit.htm)
through its ReleaseFromCage section, original feature specs and existing native
fixtures. Current OM9 supports World BoundingBox/native3D cages and its existing
Global/Local slice. This migration enables no missing Rhino modes or new solver.
Its options/basis-input admission is not the full GeometryResult, conversion,
deviation, parameter/topology-map or worker-ownership contract.

Remaining work:
- Finish CageFeature::cageHandle/flat, controller control serialization, CageBinding::execute buffers and Rust cage descriptor borrowed-basis admission before allocation; options policy does not prove full NURBS input/result ownership.
- Complete independent History replay requests for other enabled families, all-family acceptance and one final review.
- Complete original GeometryResult/conversion/deviation/parameter/topology maps and metadata/source/lock/cache integration for every enabled consumer.
- Computed Link getters, selected getSubShape cache allocation, native explorer depth/stack bounds and constraint/tangent-plane readers remain open.
- Retain original lifetime/worker/preservation/extrusion/annotation obligations; no chapter completion follows from a registry tag or green slice.
