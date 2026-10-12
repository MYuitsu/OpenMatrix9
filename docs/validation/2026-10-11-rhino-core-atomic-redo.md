# RCORE-01 — Native Redo retention and Edit integration

Updated: 2026-10-11T12:55:37+07:00. **RCORE-01 remains in progress.**

FreeCAD owns the actual retained Redo transactions. The explicit additive
`Document::openTransactionPreservingRedo` API books a fresh exact ID, keeps
the prior native Redo records while the selected transaction runs, restores
them only after every enlisted document completes checked rollback, and
invalidates them on successful commit or failed group rollback. Empty bookings,
detached owners and reentrant document/history operations have targeted tests.
Default transaction behavior remains separately tested.

Checked native property moves temporarily suppress their internal journals.
Scoped guards at the reviewed same-thread property/dynamic-property/move signals
prevent callbacks from inheriting that internal permission. Cross-document move
ownership is restored in both group orders. C++ and the Python binding are
required because FreeCAD owns these native history records and document lifetimes;
Rust business validation and the existing CAD solver are reused unchanged.

The rebuilt shared SDK has **43/43 targeted native tests** and **599 passed,
2 skipped, zero failed of 601 executed App tests**, actual exit0. Seven disabled
cases are not executed. The two skipped cases are the zero-file backup policy
fixtures. This executable uses a fresh Document unit/SDK bootstrap and cached
other App test units; these results do not describe the entire FreeCAD suite.

Matching Runtime18/API9: **401 checks across 13 serial host processes**, each
with actual exit0 and the selected plugin verified. This includes the original
publication callback **5/5**, Python API **23/23**, CAD results **110/110**, Edit
commands **115/115**, options **57/57**, recompute, mutation/output/layer/budget
guards, cold FCStd **37/37**, and Dimension finite/publication **22/22**.
The original callback on Runtime15/API5 produced 3/5 despite host exit0; the
wrapper correctly rejected it. `EditGeometry` now uses the explicit opt-in API.
Full Rust: **808 passed**, exit0. Producer54/Rust218 and native source/DLL input
fingerprints remain equal to their selected build/test captures.

The additive App API/source change required rebuilding only `FreeCADApp` in the
shared SDK; OM9 used the standalone `OpenMatrix9Gui` plugin build and existing
cache. No independent FreeCAD build or full runtime copy was created.
Source audit: 2529 files, 511
bindings, 510 SVGs, no errors.

The structured report preserves staged RED/GREEN native receipts, source/DLL
fingerprints and harness failures. Old checkpoints remain historical; their
pending opt-in API/callback statements are superseded only within this verified
scope. Source-only review found no new actionable defect in reviewed same-thread
boundaries and explicitly left worker/custom virtual callbacks open.

## Remaining original objective

- Worker-to-MainThreadSignal dispatch does not inherit the emitting thread-local observer guard. Cross-thread callbacks and arbitrary custom virtual callbacks remain unproven.
- Generic aggregate rollback postconditions, nested verdict integration, and aggregate allocation/traversal/worker/cancellation budgets still require complete coverage.
- Complete GeometryResult, conversion provenance, parameter/topology maps and source attributes for every enabled consumer/family, including non-CAD Edit results, remain required.
- Global conversion deviation bounds and real dependency parameter mapping must be proved; sampled conversion checks do not establish these bounds.
- All original read/edit/preserve/export representation contracts, persistence, ownership/cache/lifecycle and eight typed diagnostics remain required, including adapters 1–7, C01–C12 and T01–T11.

[Structured receipts and fingerprints](2026-10-11-rhino-core-atomic-redo.json).
