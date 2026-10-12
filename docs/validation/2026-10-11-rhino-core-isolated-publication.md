# RCORE-01 — Isolated native publication and rollback evidence

Updated: 2026-10-11T14:36:42+07:00. **RCORE-01 remains in progress; the full original scope remains required.**

The actual isolated API19 App DLL passes **28/28** global publication cases,
**46/46** root atomic transaction cases, and **602 passed /2 skipped /0 failed
of604 executed** App cases, with7 disabled. The skips are the two zero-file
backup policy cases listed in the structured report. Global fixtures are
additional to App604. The root executable combines fresh Document/bootstrap
units with cached other test units; it does not represent the whole FreeCAD suite.

## Changes and actual RED → GREEN witnesses

- API12 structural RED10:3/3 failures. New-object, deletion and removal-journal
  callbacks could overwrite an untouched object during checked restoration.
  Native lifecycle dispatch now suspends restoration permission for observers.
- API14 activation RED11:1/1 failure. Restored-object activation still changed
  an untouched Label while rollback returned success and restored prior Redo.
  Activation dispatch now uses the same observer scope. Exact object identity
  and executable prior Redo are checked.
- API15 move RED12: receiver-first fails, source-first passes. A nested property
  move abort during protected outer global close rejected the native reverse
  move on an already completed receiver. Checked native definition bypasses
  automatic global admission rejection while retaining the observer-depth
  barrier. Both document orders check restored property owner/value and replay
  both exact prior native Redo histories.
- API15 verdict RED12:1/1 failure. An independent failed rollback invoked during
  outer rollback poisoned the outer verdict and disposed its valid prior Redo.
  Verdict ownership now follows the native transaction ID.
- API17 interleaved verdict RED13:1/1 failure. A chain A(ID1) → B(ID2) → C(ID1)
  propagated C's failure only to the immediate parent and lost A's same-ID
  failure. Scoped native verdict frames mark every matching ancestor while
  preserving B's independent verdict. The direct two-journal same-ID witness
  remains covered. No transaction object layout or journal clone is introduced.
- API18 default verdict RED14:1/1 failure. An unchecked default abort invoked
  from outer checked rollback inherited its frame, enabled checked branches
  and lost the outer's prior Redo. The public default apply now masks that frame
  while both wrappers share the same private native driver. Its historical
  best-effort result remains unchanged; the outer checked verdict stays independent.

All eight previously failing named cases pass in the final28-case executable.
Additional cases prove that a completed independent inner cancellation retains
its executable Redo after outer before-close failure, and an inner close
exception does not silently borrow the outer group's verdict. Prior admission,
empty-commit Undo retention, global callback pairing, staged commit rollback
and source observer protections remain covered.

## Runtime and build identity

The selected native DLL SHA256 is `fa8492ba753eeb9298a9df3491a2f9af049bf3fb6a27b38046e0a9a2a5955176`. Each native
process reports its actual loaded App DLL path; runner receipts verify the
selection and hash. Build receipts cover all102 existing SDK App object targets
and every App source fingerprint. DLL/import-library/PDB/link outputs are
redirected into the test artifact; the shared SDK's API12 outputs remain
byte-for-byte unchanged. Only the selected App DLL is copied beside each test
exe when hardlinks are unavailable. No FreeCAD host or dependency runtime is cloned,
no root CMake configuration is run, and no user application is stopped.

The shared Gumball host PID75944 remains live at read-only inspection. Matching
OM9 GUI acceptance requires that host to close before updating the shared App
DLL, followed by the standalone OpenMatrix9Gui build and existing caller checks.
Runtime21/API11 caller462 checks and Rust808 are historical evidence and are
not relabeled as API19 GUI acceptance. These C++ changes are required native
journal/signal/ownership adapters; portable OM9 logic and existing solvers are reused.

## Remaining full chapter work

Worker/GUI callback boundaries, arbitrary callbacks, caller typed diagnostics
and Python result ownership, aggregate budgets, every enabled consumer/family,
world-once ownership/cache/persistence, full results/attributes, globally measured
conversion and complete parameter/topology maps remain unproven. Preserve all
eight error types, adapters1–7, C01–C12 and T01–T11. Consult the
[completion audit](../reviews/2026-10-10-rcore01-closure-audit.md) and
[enabled-consumer audit](../reviews/2026-10-10-rcore01-enabled-consumer-audit.md)
for the remaining requirement inventory. Passing these native cohorts does not
complete RCORE-01. Structured evidence records exact source/DLL/cohort hashes,
process exits, limitations and next actions.
