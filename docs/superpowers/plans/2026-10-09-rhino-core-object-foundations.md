# Rhino core P0 continuation: shared object snapshots

The user requested continuation of `specs/00-rhino-core` after merging and
pushing the previous checkpoint. The existing P0 scope and written RCORE-01,
RCORE-04 and RCORE-06 contracts remain the design authority. This is a bounded
implementation increment of their object identity and stale-selection contract.

## Decisions and acceptance

- Work on `codex/rhino-core-object-foundations` in the existing nested checkout,
  preserving its matching SDK setup. Baseline: `f5dc335`, clean after merge;
  334 Rust tests and the recorded merge native checks already passed.
- Rust owns a shared registry of document sessions, native object IDs, monotonic
  generations/revisions and snapshot equality. It receives integers and owned
  values only. No host address crosses the ABI.
- C++ observes native document/object changes and lifetime. Capture creates no
  document object/property or Undo entry. A session token cannot outlive document
  close; removed/recreated objects cannot reuse an earlier generation. Undo/Redo
  invalidates previously captured work even when geometry returns to equal bytes.
- Geometry-property changes invalidate object revisions. Native world placement
  and explicit unit-context snapshots are also compared before native mutation.
  Display-only settings do not change source geometry identity/revision.
- Existing native geometry/type restrictions remain authoritative. This increment
  does not enable a new command or claim a universal type/backend capability table.
- Integrate the shared guard into Edit input capture (also consumed by solid and
  history adapters), Rebuild sessions and interactive Surface selection/chain
  capture. Keep existing geometry signature checks as independent protection.
- Native object IDs and document UUIDs are provenance. Runtime generations are
  session-scoped; cold restore gets a fresh session and cannot accept old work.
  Durable History/subelement remapping remains a separate uncompleted contract.
- Native fixtures must demonstrate same-name/same-geometry replacement rejection,
  edit-then-Undo rejection, nested placement and context changes, unchanged preview
  success, no extra Undo or modified flag on snapshot reads, display-only stability,
  and close/reopen rejection. Failed commit leaves source/output counts unchanged.

## Tasks

1. [x] Implement and test the Rust registry and small C ABI. Cover deletion/reuse,
   cross-document isolation, unknown/zero handles, revisions, overflow and cleanup.
2. [x] Add the native CoreObjects adapter and explicit snapshot diagnostics API;
   integrate shared capture/revalidation in existing command consumers.
3. [x] Reproduce missing native stale-state guards, then run the new fixture and
   affected Edit, Surface, Rebuild and reference-consumer regressions on the matching SDK.
4. [x] Review ownership, ABI failure paths and lifecycle; run Rust, Python tooling
   and source audit, then synchronize specs/README/ledger with the exact verified scope.

## Task/interface review

| Tasks | Shared boundary | Ruling |
| --- | --- | --- |
| 1 / 2 | fixed-width stamp and registry functions | One header contract; Rust retains no pointers; native reads/writes on GUI thread. |
| 2 / 3 | controller snapshots and test diagnostics | Exercise real command commits, not only the diagnostics API. |
| 3 / 4 | evidence and scope | Historical results remain separate; incomplete RCORE fixtures stay open. |
| 1 | registry lifetime vs equality | Session/generation counters prevent ABA; equal geometry after Undo is still stale. |
| 2 | native capture vs document mutation | Registry bookkeeping is out of document state; no hidden persisted metadata. |
| 3 | regressions vs new behavior | Preserve existing command permissions and supported representations. |
| 4 | checkpoint vs full P0 | Record a verified shared foundation; do not promote full chapter completion. |

Ruling: continue the previously authorized P0 design without another scope approval;
the current user explicitly asked to continue it. No skill/business-rule update is
required for this technical lifecycle implementation.
