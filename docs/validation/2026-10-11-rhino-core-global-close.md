# RCORE-01 — Global native close and remaining admission gaps

Updated: 2026-10-11T13:59:11+07:00. **RCORE-01 remains in progress. API13 runtime acceptance is pending.**

API12 keeps exact native commit journals reversible until the global close
verdict, guards captured sources during before/close callbacks and catches
standard/unknown callback exceptions without a throwing notification destructor.
Eight isolated global cases pass (actual exit0): default abort lifecycle,
two-document abort ordering and real prior Redo replay, protected before-commit
mutation, committed state during close, and before/close exception rollback for
commit and abort. The earlier API11 close-commit witness terminates with actual
process exit3 and no XML; this is fatal process evidence, not an ordinary test
failure. Its wrapper result must not replace the recorded process exit.

API12 root transaction tests: **46/46**. App executable: **602 passed,2 skipped,
zero failures of604 executed**, with seven disabled cases. The two skips are
zero-file backup policy tests. The isolated global unit is additional evidence;
it is not included in the root App count.

## Runtime-proven open regressions

The corrected admission/history RED6 cohort has **four failures of four**, exit1:
before-commit and before-abort observers admit an uncaptured document, leaving
its booking/journal and losing executable prior Redo. Empty-only and mixed
materialized/empty commit trim old Undo despite no new journal in the empty
participant. The mixed fixture uses the materialized document as initiator:
FreeCAD would otherwise automatically create a journal in an empty initiator.

The refined RED8 cohort has **four failures of four**, exit1: global sticky ID
admission bypasses explicit booking; an inner commit observer can join the
still-closing outer ID; nested checked abort suppresses its own notification
pair and inherits the outer observer depth, blocking native rollback; the
nested before-exception observer is never invoked and its verdict is lost.
RED7 additionally caught a native SEH after calling unavailable Redo; RED8 checks
availability first. These are distinct executable/source cohorts, not a single
combined16-case run.

## API13 candidate — compiled objects, not verified runtime

The source correction seals every closing ID through nested scopes; rejects
explicit/automatic admission before booking, materialization or source mutation;
and trims Undo only after a journal was staged. A private per-document observer
baseline permits native nested rollback while each new observer remains guarded.
Checked groups dispatch independent before/close pairs and own independent
verdicts; separate explicit IDs do not silently borrow an outer notification.
Legacy signaller constructor/destructor and exported stack-object layout stay
unchanged. These are native FreeCAD journal/lifetime adapters; no portable
business logic is added to C++ and Rust/solver code is reused.

The three native implementation objects compile successfully using the existing
SDK cache. **The linked DLL remains API12.** A visible Gumball demo (PID75944 at
inspection) uses that shared host, so DLL replacement is deferred until it closes.
No application process was stopped and no independent FreeCAD runtime was copied.
After it closes, build only `FreeCADApp`, then the standalone `OpenMatrix9Gui`
against the same SDK. Run the current16-case global unit, root regressions and
matching plugin callers; object compilation alone cannot close any new RED.

Runtime21/API11 caller462 checks/16 hosts and Rust808 are historical checkpoints.
They are not repeated or relabeled as matching evidence for this candidate.
Every original representation/consumer/conversion/maps/attributes/export,
persistence/ownership/cache/budget/typed-error requirement remains open until
its own full-scope evidence exists. See the structured report for hashes,
exact cohorts, exits and next actions.
