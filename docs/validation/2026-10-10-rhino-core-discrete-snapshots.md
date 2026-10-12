# RCORE-01 owned mesh/cloud snapshots — 2026-10-10

The shared geometry descriptor now copies native Mesh::Feature and Points::Feature
geometry into Rust-owned raw-local XYZ arrays, triangle indices and inspected
local bounds. Direct objects and ordered group/link definition leaves use the
existing guarded source attributes, occurrence graph and checked world composition.
Mesh/cloud owners retain their representation even with an attached Group extension.
No source kernel, solver, editor, display cache or Line/Polyline constructor is
recreated. Empty native data is known empty; unsupported selectors reject.

Rust validates counts, retained bytes, finite coordinates and triangle indices
before cloning. MAX_POINTS/MAX_FACES100000 each and8MiB retained/serialized budgets
apply to this model. The40-byte ABI gates envelopes/counts before immutable slices;
live initialized storage, nonoverlapping writable output and native ownership remain
caller obligations. Short, invalid-value and oversized-serialization output is
atomic. Serialized overflow rejects after count-bounded native staging/Rust cloning;
the bound does not certify process/global allocations or peak staging.

C++ reads existing module-owned storage synchronously, after exact builtin property
checks. Mesh additionally checks registered MeshObject type. Points does not register
PointKernel in AppPoints.cpp; its exact PropertyPointKernel owns the concrete kernel
returned by getComplexData(), an explicit native cast/ownership obligation.
Raw float32 kernel coordinates are widened to float64 without claiming archive
float64 preservation. The independent native kernel transform is composed exactly
once with parent/link placement. Unknown channels/manifold facts stay unknown;
solid=false means this representation is not a CAD solid, with representation
evidence rather than a BRep shape-type assertion.

MSVC exports Mesh STL vector readers. The plugin imports the shared matching
Mesh.lib and delay-loads Mesh.pyd via system delayimp. No Points import library,
FreeCAD root reconfiguration/build or second host runtime was needed. The cold
fixture observes operating-system DLL handles: neither Mesh nor Points is loaded
before/after OM9 import and a CAD descriptor; both become usable after their
native imports. This establishes deferred loading on this shared SDK.

**628 full Rust tests and 1016 FreeCAD checks passed** on one
matching plugin, all11 actual host process exits0:99 discrete geometry,6 cold
loading and911 prior instance/world/support/topology/descriptor/import checks.
Plugin SHA256:`9ca1733885d85b8102f647c6f06e5d58bb8a3743752f00a9a63a50dd564cd434`. [Machine evidence](2026-10-10-rhino-core-discrete-snapshots.json)
records exact source/native dependency/report/log/runtime fingerprints.

Native tests compare raw points/facets with native storage and world points with
independent native matrix paths, covering nested rotations, mirror/nonuniform scale,
both LinkTransform settings, shared group definitions and grouped native owners.
Exact read-only source/history comparisons, empty data, NaN/Inf, count overflow,
count-valid serialized overflow and a literal float32 rounding example pass.
Replacement and Undo publish fresh stamps/payloads; Redo/close preserve returned
snapshots. Native Points reconstructs Placement from its stored matrix on Undo,
changing placed getters by one double ULP; only that restoration comparison uses
1e-12. Raw owned geometry and read-only preservation remain exact.

Retained RED results expose missing payload, mistaken solid evidence, unavailable
PointKernel registration and native-owner/group routing; corresponding runtime
checks now pass. [Read-only review](../reviews/2026-10-10-discrete-snapshots-review.md)
found no production defect and recommended the now-passing ABI value/serialized
canary tests. Existing SDK OCCT deprecation warnings are recorded in build logs.

**Full original RCORE-01 remains in progress.** Native channels/flags/segments,
archive float64 attributes, mesh/cloud subelement maps, arrays/reference contexts,
full topology/solid/parameter certificates, reports, uniform typed capability/error
integration and original allocation/worker/mutation/lifetime requirements remain
tracked in the [completion audit](../reviews/2026-10-10-rcore01-closure-audit.md).
These snapshots do not close C05/C06 or establish Rhino parity for every operation.
