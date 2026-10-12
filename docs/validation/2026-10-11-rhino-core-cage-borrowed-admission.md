# RCORE-01 — borrowed Cage admission — 2026-10-11

Borrowed rational Cage descriptors and saved binding values are validated by Rust before owned input copies or result buffers; native control selection and independent recompute use typed operation/object errors. Existing deformation/restore/release services are reused.

**RCORE-01 and the complete construction plan remain in progress.**

## Verified change

Safe Rust owns borrowed basis validation and saved binding admission. The
existing constructor now validates all basis/pose/reference/region values
before reserving owned knot, XYZ and weight vectors. Owned copies use fallible
reservation; allocation/free stays in Rust. Invalid weights previously caused
five actual Rust allocations before rejection; the focused allocator fixture
now proves zero allocations for the four invalid basis cases. The valid
descriptor query and binding query also require zero allocations.

Native C++ remains the permanent FreeCAD adapter. It borrows immutable native
property vectors synchronously under GUI-thread/GIL ownership, with compile
assertions for standard-layout/trivially-copyable XYZ, size24, alignment8 and
offsets0/8/16. Neither query retains host storage. It no longer flattens Cage
control or binding input vectors into temporary double vectors. Control
selection checks the actual basis before displayed-lattice copy and signature
serialization. Independent binding recompute checks source/parameter lengths,
finite values, mesh budgets and signed native triangle indices before creating
the owned Cage handle or output buffer. Cached BRep sources use the existing
Rust-backed native InputTree before native deformation/copy.

Generated box counts2..128 and degree1..16 retain their original rules. Imported
clamped bases keep their separate count/product limit of1,000,000 points; the
valid129-axis basis fixture is admitted. Empty affine CAD parameter buffers
remain valid; empty saved mesh bindings are rejected. Release still reads the
relationship and can detach a damaged captive. No solver/default/repair or
additional Rhino mode is introduced.

## Evidence

- Rust functional RED0/4 (including the five-copy constructor failure), then
  GREEN4/4. Triangle diagnostic RED3/4 then GREEN4/4; actual cargo exits101/0.
- Full Rust:734 tests,105 suites, actual exit0.
- Native initial RED34/51, then GREEN51/51. Cached-null BRep RED53/54, then
  final GREEN54/54. Each application process exited0 without fixture errors.
- Final matching integration: 1322 checks across24 actual host exits0,
  including unchanged Cage commands/deformation/lifecycle/3DM restore and
  prior Curve/Solid/Surface/native Edit/prepared-CAD regressions.
- Standalone plugin build2 exit0; shared FreeCAD host unchanged. Plugin SHA256:
  `5747452954cf9741e5abb74a31d2fc69b216b96fb05e6027647368269e4ce47d`. Build output contains the existing OCCT deprecated-header warning;
  Rust emits path-canonicalization warnings on this Windows checkout.
- Exact artifacts, source/runtime hashes and synchronized views are in the
  companion [receipt](2026-10-11-rhino-core-cage-borrowed-admission.json).

The existing accepted Cage policy checkpoint remains historical. Its source
fingerprints and runtime hash must not be used to certify this newer source.
Rust tests prove portable behavior; the real-host fixtures prove the stated
integration, rather than global native memory safety or complete Rhino parity.

## Remaining original scope

- Complete independent History replay requests, all-family integration acceptance and one final review.
- Complete original GeometryResult/conversion/deviation/parameter/topology maps and metadata/source/lock/cache integration for every enabled consumer, including Cage outputs.
- Cage native control-point extraction, aggregate captive budgets, native kernel/output allocations and all-family lifecycle remain separate from the verified borrowed basis/binding gates.
- Native Cage missing/invalid control dependencies and FreeCAD skipped-recompute error propagation still need typed-context runtime evidence.
- Computed Link getters, selected getSubShape cache allocation, native explorer depth/stack bounds and constraint/tangent-plane readers remain open.
- Original lifetime/worker/preservation/extrusion/annotation requirements remain open; this checkpoint does not close RCORE-01.
