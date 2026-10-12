# RCORE-01 — Native checked rollback prerequisite

Updated: 2026-10-11T11:47:17+07:00. **RCORE-01 remains in progress.**

`Transaction::applyChecked` supplies a scoped native completion verdict while
preserving default best-effort `apply` behavior and public object layouts.
It detects caught property restore exceptions, missing rename/static/recreated
properties, refused removal, missing moved owner-side properties, occupied
restoration names and restored objects removed by native observers. Receiving
move registrations are distinguished through saved owner identity without
dereferencing a removed property; default attachment-first ordering is retained.
This host-owned transaction prerequisite requires C++; it does not move native
history pointers into Rust or replace existing Rust admission.

Nine targeted checked-rollback tests pass against rebuilt shared FreeCADApp.
The combined targeted run is **13/18**, exit1: the five original missing
preserving-Redo API/lifecycle failures are retained. Full App executable:
**569 passed, 2 skipped, 5 failed of 576**, exit1. This executable uses a freshly
compiled Document test unit, cached other App test units and a selected SDK-DLL
bootstrap; it is not the complete FreeCAD project suite. The two skips are
`BackupPolicyTest.StandardWithZeroFilesDeletesExisting` and
`BackupPolicyTest.TimestampWithZeroFilesDeletesExisting`.

The expected failing tests remain:

- `DocumentTest.AtomicAbortPreservesDetachedCreationOwner`
- `DocumentTest.AtomicAbortPreservesExactRedoIDsAndExecutableOrder`
- `DocumentTest.AtomicDetachedRedoOwnerRejectsPropertyWritesBeforeRecording`
- `DocumentTest.AtomicSharedIDAbortRestoresBothDocumentsRedo`
- `DocumentTest.AtomicTransactionAPIIsExplicit`

The actual native transaction pointer is observed via host signals in the
restoration fixtures. The observer fixture removes through the public API and
disconnects before cleanup Undo to keep native ownership valid. RED2/3/4/5/6b
retain the failing behavior; checked1/2/3 retain staged fixes, exact source/DLL
fingerprints, XML and terminal exit receipts. The failed red6 compilation is a
fixture error, not a failing native requirement.

The first broad run exposed harness errors: wrong module home for the isolated
executable and rejected temporary-file renames. The corrected bootstrap locates
modules through the selected shared SDK DLL and uses a workspace temp/profile.
All additional failures disappear in the corrected broad run. Original reports
are retained rather than overwritten. Historical OM9 Runtime15 proof used the
previous App DLL; the newly changed host/plugin pair has not been certified.

Shared SDK `build/relWithDebInfo`, target `FreeCADApp`, workers14; actual core
API/source changes justify this build. No complete runtime copy, independent
host build or plugin target rebuild was created. Source-only final review found
no additional actionable finding in this prerequisite.

## Remaining original objective

- Implement explicit Document/Python preserving-Redo API, real native history ownership, detached-owner prewrite rejection, and group-wide checked rollback gate.
- Protect native document/owner lifetime during rollback observers and close/replacement; validate original five callback assertions and Python lifecycle fixture on matching rebuilt host/plugin.
- Broader checked rollback edge cases, including nested verdicts and aggregate postconditions, remain subject to host integration proof.
- All original RCORE-01 families/consumers, parameter/topology maps, measured global conversion bounds, attributes, export/persistence, budgets/workers/ownership/cancellation/cache, eight typed diagnostics, adapters1-7, C01-C12 and T01-T11 remain required.

[Structured receipts and fingerprints](2026-10-11-rhino-core-checked-rollback.json).
