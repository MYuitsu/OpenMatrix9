# RCORE-01 common topology snapshot

Objective remains full RCORE-01. This increment closes the missing common
face/loop/edge-use observation; it does not certify the chapter or Rhino parity.
Reuse OCCT and the existing input-tree/basis gates, not a new geometry solver.

Design: a bounded flat native observation records faces, ordered loops and edge
occurrences. Snapshot-local edge/vertex identities include native locations;
seams retain both occurrences and separate pcurves. Safe Rust owns and checks
the independent rows, ranges, flags, finite UV observations, closed loop vertex
adjacency and seam pairs, then computes edge incidence and boundary components.
Edge-incidence manifoldness is explicitly distinct from a full vertex-link or
valid-solid certificate. Surface periodicity never establishes solidness.

Permanent native exception: C++ must traverse OCCT topology/pcurves on the GUI
thread. It holds live immutable shapes only for the synchronous call, copies
observations, serializes the checked result and verifies the source stamp before
and after. Rust never retains an OCCT/document pointer. Python is only the real
FreeCAD fixture. Reject unsupported/incomplete wire traversal rather than return
a misleading partial topology. Limits are Rust-owned, checked before row growth.

Files: `rust/src/topology_snapshot.rs` portable owned snapshot and validation;
`rust/src/topology_snapshot_ffi.rs` small borrowed-buffer ABI;
`Gui/CoreTopologySnapshot.h/.cpp` OCCT extraction and JSON adapter;
`Gui/CoreGeometryContract.cpp` integrates the common snapshot;
Rust tests and `tests/rhino_core_topology_snapshot_smoke.FCMacro`.

Pre-flight interfaces: native flat rows -> Rust typed validation -> JSON all use
one-based snapshot identities and zero-based contiguous spans. Boundary output
is a component number per use, zero for non-boundary. No mutation authorization
or persistent subelement naming is implied by these local identities.

## Task 1: RED host and Rust contracts
- [x] Add real host square-with-hole, open periodic cylinder, sphere, hollow solid,
  reversed face, same geometry/different native identity and detached JSON cases.
  Expected RED: common descriptor `faces` is null.
- [x] Add Rust behavioral tests for ordered seam pairs, incidence/components,
  malformed ranges/IDs/flags/UV, disconnected loops and count/overflow limits.
  Expected RED: module/API does not yet exist.

## Task 2: Rust model and ABI, GREEN
- [x] Implement owned validated snapshots and topology graph decisions.
- [x] Add layout/null/alignment/output atomicity tests; run focused Rust tests.

## Task 3: Native extraction and real host, GREEN
- [x] Gate native input before topology queries; preserve face orientation while
  traversing loops on a forward face, retaining oriented edge uses/pcurves.
- [x] Integrate topology JSON without replacing unknown full manifold/orientation
  facts with incidence-based assumptions. Build only `OpenMatrix9Gui` using the
  shared SDK/cache. Host test must load the rebuilt selected plugin and exit0.

## Task 4: Review and evidence
- [x] Fresh review of ownership, incomplete traversal, seams and malformed ABI.
- [x] Run full Rust and relevant host descriptor/import regression, public audit.
- [x] Record fingerprints, native exceptions and remaining original requirements;
  synchronize chapter/status/lookup/ledger/MANIFEST. RCORE-01 stays in progress.

Review focus: repeated faces in compounds, located shared edges, degenerate
pole edges, internal/external orientations, wire traversal that omits occurrences,
seam pcurve selection, malicious spans/IDs, writes on rejected ABI input, source
cache mutation, graph cost and the distinction between incidence and manifoldness.

## Execution record

Task 1 RED: old selected plugin returns null faces on the authored hole face;
Rust topology module and ABI APIs initially absent. Task 2 GREEN:13 focused tests.
Task 3 GREEN:177 real-host checks. Task 4:585 full Rust,177 topology +55 import
+151 previous descriptor =383 host checks; selected plugin/host paths and hashes
match and all processes exit0. Public audit zero errors;664 manifest rows checked.

Final fixes: repeated face incidence RED -> canonical unique native face+location
rows plus all `face_occurrences`, GREEN; inconsistent reversed-edge endpoint
direction RED -> Rust directional identity check, GREEN. Details in
`docs/reviews/2026-10-10-topology-snapshot-review.md`.

Ruling: preserve repeated compound occurrence metadata while computing the
recorded-face graph from unique native identities, matching FreeCAD Faces.
Downstream quantity consumers must account for occurrence multiplicity; ignoring
it could produce incorrect quantities. Supporting pcurve geometry and complete
manifold/solid/world/attribute contracts remain original required work. No goal
completion or Rhino parity is claimed by these checked task boxes.
