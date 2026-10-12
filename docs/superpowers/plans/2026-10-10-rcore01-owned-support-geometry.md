# Owned supporting geometry for RCORE-01

Continue the original full RCORE-01 objective. The previous topology snapshot
only copied pcurve domains/UV endpoints. This increment adds portable owned
NURBS support data to every observed face/pcurve, preserving full basis domains
separately from trim restrictions and edge/face orientation. Reuse the existing
Rust basis validator and OCCT extraction; never convert analytic geometry to
NURBS merely to populate an inspection result.

Safe Rust owns curve/surface/axis snapshots, validates before cloning and emits
the versioned JSON value. C++ only reads OCCT and supplies immutable extracted
buffers. The synchronous ABI uses caller-owned byte output and a required-size
query; no opaque handle or cross-runtime free operation is needed. Failed calls
leave output unchanged. Each call validates/copies/serializes independently.
Python remains only the real FreeCAD test fixture.

Budgets: existing read-only companion 8192 poles and16384 knots/axis; bounded
logical and serialized payload plus an aggregate support-payload budget owned
by Rust. Check native counts before extraction. Over-budget support remains
explicitly uninspected rather than silently truncated. Analytic, Bezier, offset,
extrusion/revolution and other support types require faithful native type data
or later adapters; they cannot be labeled copied NURBS without conversion.

Files: `rust/src/support_snapshot.rs` owned NURBS model/serialization;
`rust/src/support_snapshot_ffi.rs` guarded existing basis ABI and output;
`Gui/CoreSupportSnapshot.cpp/.h` OCCT support extraction/Qt parsing;
`Gui/CoreTopologySnapshot.cpp` face/pcurve integration; focused Rust tests and
`tests/rhino_core_support_snapshot_smoke.FCMacro` real rational/periodic/trim data.

Ruling: expose the same Rust-owned payload for standalone 3D NURBS edges and
reuse the face payload for the legacy companion — the required common geometry
schema covers curves as well as pcurves. Existing `basis` fields remain compatible;
native readers no longer build a second pole/axis JSON implementation. This
adds no solver or implicit analytic conversion. The curve RED at
`build/support-snapshot-curve-red-20261010` fails at missing curve geometry after
the periodic-surface, restricted-domain, reverse and location controls pass.

Pre-flight: NativeCurve/NativeSurface reuse `nurbs_basis_ffi` repr(C) layouts;
borrow helpers are crate-private and used only after the shared validator accepts
the immutable buffers. Required output size excludes a NUL terminator; native
QByteArray owns exactly that many writable bytes. Native errors stay in C++.

## Task 1 — RED
- [x] Rust tests: rational and periodic curve basis, U-major rational surface,
  detached ownership, malformed counts/weights, output atomicity and budgets.
- [x] Host fixture: current common face/pcurve `geometry_data` is missing; verify
  the failure with a rational NURBS face and existing imported seam/hole data.

## Task 2 — Rust GREEN
- [x] Implement validated owned basis snapshots and fixed-key JSON serialization.
- [x] Reuse basis ABI guards, add caller-output JSON ABI and count/payload limits.
- [x] Run focused tests; preserve rational weights/periodic axes/full domains.

## Task 3 — Native GREEN
- [x] Read supports on GUI thread, gate counts, preserve trim restrictions,
  native locations and orientation separately; parse Rust JSON into owned Qt data.
- [x] Build only OpenMatrix9Gui with shared SDK/cache and run real host controls.

## Task 4 — Evidence
- [x] Fresh code review at the implementation boundary; RED/GREEN findings.
- [x] Full Rust plus topology/descriptor/import regressions on final plugin.
- [x] Update chapter/audit/status/ledger/lookup/MANIFEST with exact scope/hash.

Review focus: nonrational vs rational weights, periodic compact axes, rectangular
trim/reversed pcurve support, two-pass buffer size consistency, output rejection
atomicity, borrowed lifetime, aggregate budget, JSON numeric fidelity and source
immutability. Remaining original chapter requirements stay required and open.

Verification: two runtime REDs (missing common face data, then standalone 3D
curve data) and final 54 support +177 topology +151 descriptor +55 import =437
host checks, all exit0 on plugin `0fdcbbbaaf5d5db6fcbd87910225e0630c68a1500409711cd3cd66147ccbb40f`.
Full Rust594 passed, including nine owned model/ABI tests. The review's validation
gaps were covered by added periodic/trim/reversal/location/uninspected/budget
controls and direct surface ABI tests. Failed concurrent host startup attempts
are excluded; final regression controls ran sequentially. No full chapter
completion is implied by this plan's completed NURBS increment.
