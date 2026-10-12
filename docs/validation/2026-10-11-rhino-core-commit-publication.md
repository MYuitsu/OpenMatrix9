# RCORE-01 — actual caller commit failures, 2026-10-11T15:12:00+07:00

**Status: in_progress. The complete original RCORE-01 scope remains required.**

Builder Create/Restore/Match now own CPython result references through native
commit. A failed commit destroys the local return owner; successful commit
transfers its new reference to the caller. Partially filled result lists also
own their entries. Commit errors carry KernelFailure, the actual transaction
operation and source names frozen before publication. Sweep1/Sweep2/Loft plain
and History paths similarly translate Base/std/unknown native commit exceptions
into the standard exception handled by the Surface command, retaining every
selected source/subelement/chain name. Existing Rust policy and solvers are reused.

## Executed RED → GREEN

| Witness | Actual RED | Matching Runtime24/API19 GREEN |
|---|---:|---:|
| Builder Create/Restore/Match, early/before/close | 57/69 | 69/69 |
| Sweep1/Sweep2/Loft, plain/History, before/close | 60/72 | 72/72 |

Both cohorts use actual native C++ before-close/close signal exceptions. The
Builder early source-mutation witness measures the pre-return reference baseline;
six failed-close paths previously leaked one reference and reported generic API
errors. All now match that baseline. Surface's twelve paths previously allowed a
Base::Exception to escape into Qt notify, leaving no typed command diagnostic.
The fixture schedules verification as a separate event before the commit click,
so that original escape path can be measured. Initial Surface attempts red1/red2
produced no report; their owned launcher process paths/PIDs were checked before
termination. They are retained infrastructure evidence, not numeric RED cohorts.
Surface red3 wrote all72 checks, then terminated with actual exit -529697949
and fatal Python thread-state diagnostics after the twelve native exceptions
escaped through Qt. It is failed-runtime evidence, never a successful host run.
Both corrected fault cohorts terminate normally with actual exit0.

Assertions verify original native input-wrapper identity, geometry/metadata/links
and Undo restoration, invalidated output wrappers and returned-reference counts.
Early stale-source rejection retains executable prior Redo; a failed native
close disposes uncertified Redo under the existing checked-group contract.
This report does not claim preservation of that discarded failed-close history.

The matching16-case existing caller suite has 462 checks;
five independent cold hosts add 46 checks, including deleted
Builder output recreation and all three saved Surface families. Together with
fault witnesses: **649 checks / 23 actual host exits0**. Full Rust:
**808 passed, 0 failed, 0 ignored**, with218 source fingerprints unchanged.
Native global28/root46/App602pass2skip remain the earlier API19 checkpoint,
reverified against its source receipts; they are not counted as new executions.

## Selected runtime and reuse

Plugin SHA256: `a459cd7637c0d1ee2b246ab55b1297e0dba6e3787bf2bbc407f09e208e304942`.
Actual App DLL SHA256: `fa8492ba753eeb9298a9df3491a2f9af049bf3fb6a27b38046e0a9a2a5955176`.
Each GREEN process writes its loaded App path/hash and SDK home; runner receipts
check the actual plugin path/hash and terminal exit. A fresh test GUI entry point
is compiled from the shared SDK's cached MainGui unit/flags/resources and uses
its loaded Base DLL to locate the shared SDK home. Only the selected App DLL is
copied alongside that test executable. Shared FreeCAD executable/App/Gui/Base
files remain byte-for-byte unchanged. No root configure, complete host clone or
user-host termination was needed. The production build target was standalone
OpenMatrix9Gui; test-only native probe/launcher units were compiled separately.
This supersedes the earlier assumption that isolated matching GUI checks had
to await closure of the user demo; shared production deployment remains separate.

The native C++ exception is necessary for CPython references and FreeCAD commit;
the test probe is never linked into the production plugin. No portable business
policy or solver was added to Python/C++.

## Remaining requirements

- RCORE-01 remains in_progress. Preserve every original geometry family, enabled consumer, eight typed errors, adapters1–7, C01–C12 and T01–T11; these caller witnesses do not certify the whole chapter.
- Global conversion deviation and complete parameter/topology maps, remaining source attributes/full GeometryResult consumers, rational/periodic basis/trims/seam occurrences, solid validity and truthful representation/read/edit/retain/export remain required.
- Worker-to-GUI dispatch, arbitrary/custom callbacks, full source/lock/owner gates, aggregate allocation/traversal/cancellation budgets, previews/cache cleanup and independent persistence/lifetime contracts require complete evidence.
- Caller commit diagnostics/reference release are proven only for the exercised Builder Create/Restore/Match and Sweep1/Sweep2/Loft plain/History paths; other producers, partial Python allocation failures and additional native exception classes are not certified.
- The selected App19 GUI test launcher reuses the shared SDK and resources; shared SDK App remains API12. Production deployment/ABI acceptance on the shared host is distinct from isolated GUI acceptance and remains pending.

Continue the [full closure audit](../reviews/2026-10-10-rcore01-closure-audit.md)
and [enabled-consumer audit](../reviews/2026-10-10-rcore01-enabled-consumer-audit.md).
Review found no production issue in these slices; its missing fixture input
identity assertion was added and executed. Structured JSON retains source,
binary, fixture, log and process identities. No full chapter completion is claimed.
