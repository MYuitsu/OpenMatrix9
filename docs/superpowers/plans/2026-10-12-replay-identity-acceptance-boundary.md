# Replay Identity Acceptance Boundary Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement this
> approved continuation inline, with test evidence and final independent review.

**Goal:** Preserve exact identity for reversible rejection, separate accepted
native publication from disposal diagnostics, and correct OCCT offset D3.

**Architecture:** Reuse FreeCAD's existing journals, native ownership and observer
barrier. Acceptance precedes physical destruction. Reuse the existing OCCT
8.0.1 offset helper; correct its derivative coefficients and build TKG3d alone
against the existing SDK dependencies.

**Tech Stack:** FreeCAD C++ private APIs, existing Rust geometry validation,
OCCT8.0.1, Python native fixtures/build tooling, standalone plugin cache.

**Spec:** ../specs/2026-10-12-replay-identity-acceptance-boundary-design.md

## Global constraints

- Keep exact original identities for every rejection before acceptance.
- Keep the old contradictory witnesses and their recorded failures.
- No manual finalizer invocation, address reconstruction or wrapper retargeting.
- Block owner/source mutation during disposal; report disposal independently.
- Reuse H:/FreeCAD-src/build/relWithDebInfo; no FreeCAD root reconfiguration,
  shared binary replacement or full runtime copy.
- Portable policy/data stays Rust-owned; C++ changes cover native APIs only.
- Full RCORE-01 stays open until all original requirements have evidence.

## Review focus

1. Early native open callbacks: exact booking/Redo/native/Python identity remains.
2. Actual finalizers after acceptance: once-only destruction and blocked mutation.
3. Error cleanup: late disposal cannot falsely claim lifetime rollback.
4. Offset D3: direct curve, adaptor, bundled D2 and alternate helper branch agree.
5. Runtime selection: actual loaded native/kernel/plugin hashes match the build.

## Task 1: Correct native offset D3

Files: tests/native/occt_offset_d3_probe.cpp; tools/build_occt_offset_d3.py;
tools/patches/occt-8.0.1-offset-d3.patch; native oracle fixture/provenance.

- [x] Reproduce shared helper/direct curve/adaptor failure: probe2384/12.
- [x] Correct the shared helper's normalized-normal D3 and bundled D2 terms.
- [x] Build only TKG3d against unchanged SDK headers/imports/dependencies.
- [x] Verify helper probe, native19-family oracle, independent45 high-precision
      derivatives, actual loaded kernel and Rust's full suite.
- [x] Preserve the old raw oracle and record only verified native output changes.

## Task 2: Native acceptance and disposal result

Files: src/App/Document.h; src/App/private/PropertyReplay.h;
tests/native/replay_retirement_probe.cpp; new boundary fixture.

Interfaces: keep bool commit()/rollback()/completeErrorState(); add noexcept
bool disposalVerified() const and const char* disposalDiagnostic() const.
Disposal verification distinguishes completion from blocked post-acceptance
mutation. These accessors do not allocate or trigger callbacks.

- [x] Test the new boundary against App227 before implementation: accepted
      finalizer disposal currently rejects; early native/Python identities remain.
- [x] Set the acceptance/completion phase before physical disposal, retain the
      barrier and record new disposal attempts separately from early vetoes.
- [x] Verify both Undo modes, regular/preserving/active caller states, finalizer
      controls, completion retries and executable original Redo after early veto.

## Task 3: Consumer integration and progress

Files: Gui/CircleHistory.cpp; relevant native regression fixtures;
docs/features/RCORE-01.md; specs00/01; README; catalog, ledger and manifest.

- [x] Circle History reports disposal diagnostics without requesting rollback
      after accepted publication or after completed error restoration.
- [x] Review source and run matched native/plugin/kernel warm/cold regressions.
- [x] Record superseded strict-finalizer witnesses, fresh Rust results, exact
      runtime hashes and all remaining original RCORE-01 requirements.

Ruling: continue the user-authorized work in the current dirty checkout; preserve
other work and isolate binaries, profiles and test outputs. Do not move or reset
the existing source work merely to satisfy a generic worktree suggestion.

Verified slice: [acceptance/offset D3 evidence](../../validation/2026-10-12-rhino-core-acceptance-offset-d3.md)
records App247/plugin254/kernel240, 3313 assertions through 28 native hosts and
1037 Rust checks. Independent review found no further acceptance/disposal defect.
These checked tasks complete this approved design slice; they do not complete
the original native-savepoint Task2/3 or the full RCORE-01 acceptance contract.

## Continuing original RCORE-01 work

- [x] Reproduce constructor history/name budget holes with actual App247:
      32/38; aggregate history and property admission and two long-name cases fail.
- [x] Preflight bounded native snapshots before copying; reuse Application's
      transaction descriptions without intermediate owned string copies.
- [x] Verify snapshot admission App264/plugin266: 38/38, exact state and Redo.
- [x] Reproduce ordinary-close history finalizer reentrancy separately:
      host267 exits -1073740940; actual finalizer reenters clearUndos.
- [x] Protect actual native history destruction using existing FlagToggler and
      mutation controls; verify close, regular clear, detached objects and Redo.
- [x] Run matching regressions and update all progress views for this new slice.
- [ ] Audit remaining peak/opaque allocation budgets, source/global participants,
      normal object disposal, and original RCORE-01 C01-C12/T01-T11 requirements.

Native history admission/disposal evidence:
[3372 behavior checks, startup gate33/34](../../validation/2026-10-12-rhino-core-snapshot-history-disposal.md).
The startup failure was then traced to NotificationArea's popup entering its
logger while holding the same storage mutex. Reusing unique_lock and releasing
it before the UI call gives [3375/3375 through35 clean hosts](../../validation/2026-10-12-rhino-core-notification-lock.md).
Only the changed GUI compilation unit was rebuilt; shared binaries remain intact.

Reconciled acceptance audit: original T01-T11 and several C contracts have
accepted slices in the accumulated evidence. Their old unchecked rows are not
blanket new failures. Complete conversion/correspondence, consumers/results,
ownership/attributes, representation preservation matrices, peak bounds and
publication/lifetime/cache coverage still require work.

## Native object/member lifetime continuation

- [x] Actual derived Python member finalizer exposes an expired sibling property:
      native App273 returns None then TypeError (RED3/4), rather than a lifetime
      error. No crash is claimed for this particular witness.
- [x] Reuse FreeCAD's cached-wrapper setInvalid mechanism before physical delete
      in native owners, without invalidating reversible Undo/Redo detach.
- [x] Matching App294/plugin296 verifies51 checks across12 read/get/write and
      document mutation cases for close, direct remove and detached Redo disposal.
- [x] Reproduce close lookup leak with two objects: native ID map still contains
      the first freed owner while the second finalizer runs. Python currently
      returns None for that freed entry; the native nonnull pointer is the defect.
- [ ] Retire public name/ID/active lookup before callback-bearing object deletes,
      retaining the existing map nodes locally to keep native name keys alive.
- [ ] Verify lookup, member lifetime and the full matching warm/cold regressions;
      update all progress views with the accepted scope and remaining contracts.
