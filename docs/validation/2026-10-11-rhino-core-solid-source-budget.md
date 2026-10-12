# RCORE-01 — Solid source budget, 2026-10-11T19:51:28+07:00

**Status: in_progress. The complete original chapter remains required.**

Solid reuses the FreeCAD solver. Before accepting another FitPoints source batch,
C++ now serializes exactly the same owned, deduplicated source envelope used in
the final capture; Rust checks its byte, depth and node limits. Resource-limit
errors are InvalidBasis with the actual Sphere/source identity. Each selection
keeps its own reference, stamp, frame and point range. Complete source geometry
is retained; limits were not raised. The physical retained-byte check remains.

## Actual failure and verification

The temporary Runtime40 diagnostic hook captured the actual serialized C++→Rust
request:16,308,922 bytes and1,630,773 JSON nodes, with54 unique source snapshots.
The request was under16MiB but above the1million-node parser limit. This is an
intercepted packet, not a reconstructed model. Runtime40's unchanged native
fixture was RED3/5, actual host exit0. The exact diagnostic source was recovered
and matched to its pre-build SHA256; the hook was removed before Runtime41.
The diagnostic Runtime40 binary is no longer present; its hash is supported by
the preserved launch receipt only. It is not freshly verified binary evidence.

Runtime41 with that same fixture is GREEN5/5. It accepts33 unique batches and
rejects Source.Vertex34 before modifying the document or accepting the batch.
Input Undo leaves32 refs/34 numeric points, which create a valid sphere through
one native Undo transaction. Repeated identical source admission still accepts
55 batches, then rejects the retained-byte overflow; its original5 checks pass.

- Full Rust:829 passed,0 failed,0 ignored, including3 new source-budget tests.
- Matching Runtime41:553/553 native checks,13 distinct actual host exits0,
  serially:unique5,repeated5,warm61,guards13,CADfit5,cold26,indexed12/12,
  binding93,policy18,callbacks150,publication86,construction67.
- Two distinct cold processes open five recipe FCStd files and one indexed
  54-reference/two-source result. Exact result bytes and full source masters
  survive; the archive hashes are recorded. Bootstrap already imports OM9.
- Standalone plugin retry build exit0; shared SDK hashes unchanged. The initial
  cache-contention build exit1 is preserved separately and excluded.
- All437 frozen producer-input files still match. Runtime39's earlier Gumball
  source drift remains historical; Runtime41 was built from the new fingerprints.
- Independent narrow source review found no actionable findings.

## Scope and remaining work

This is a necessary source-envelope gate, not proof that complete request/output
overhead always fits. The final common request/result parser retains its own
bounded checks. Source snapshots were already removed from public result
operation_context; duplicate output snapshots were not the observed cause.
Offscreen exercises actual native kernel/commands/callbacks but has no valid
OpenGL context. Mouse/render/viewport coverage is still required.

- Every original RCORE-01 requirement remains required: GeometryRef/Snapshot/Result, rational/periodic basis, UV trims/pcurves/seam occurrences, complete attributes, every enabled consumer, independent family read/edit/retain/export, adapters1–7, C01–C12 and T01–T11.
- The source-envelope node/depth/byte gate is necessary admission only. Complete request/output overhead and aggregate allocation/traversal/worker/cancellation bounds remain required; accepted source envelopes do not certify every future solved result.
- Global conversion/deviation bounds and complete parameter/topology maps remain unproven. Valid closed native Solid is not a certificate of these contracts.
- Existing CurveController/CurveGeometry commands still require the common GeometryResult path using their current FreeCAD/Line/Polyline/curve solvers.
- Offscreen evidence covers native kernel, command, Qt keys, callbacks and scoped FCStd persistence only. Actual mouse, OpenGL rendering and viewport acceptance remain required; historical graphics cohorts are not current reruns.
- The two cold hosts verify five named recipes and one indexed54-reference result, with OM9 imported by bootstrap. Unsignalled native root-frame mutations, actual Solid preview leases and complete original lifetime/cache invariants still require evidence.
- Production deployment/ABI acceptance remains pending. An isolated App19/camera-guard GUI host reuses the single shared SDK; no root FreeCAD build or full runtime copy was made.

The [machine checkpoint](2026-10-11-rhino-core-solid-source-budget.json) records
exact source, binary, fixture, launch, log, archive and captured-packet hashes.
Plugin SHA256:`1e15ba9f24abb2787a6a8692ab34bb2124f8abc44cd35c2812bcfe9b89d20a2a`.

Next: complete full aggregate request/result budgets and the common result path
for existing curve commands, then continue the original requirement audit.
