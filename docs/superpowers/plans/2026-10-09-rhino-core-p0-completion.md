# Complete all Rhino core P0 requirements

> Execute the existing RCORE contracts continuously, with Rust-first implementation,
> test-driven development and independent review. The active user goal is all P0;
> an individual validated increment does not satisfy or replace that goal.

**Goal:** Complete and verify every P0 requirement in specs/00-rhino-core.
**Baseline:** `22fecb2`, clean branch `codex/rhino-core-object-foundations`.
**Architecture:** Rust owns portable policy, typed requests and immutable state;
native bridges own FreeCAD/Qt/kernel access and atomic GUI-thread transactions.
**Spec:** chapters 01–05, P0 portions of 06–07, chapters 09–10 and applicable
UI_DETAILS requirements. P1/P2 authoring/editor/render/layout tools are excluded
only where the specifications explicitly assign that priority.

## Completion requirements

- Every contract clause and numbered acceptance fixture must have current direct
  evidence, or an explicit priority mapping showing it belongs to P1/P2.
- Existing narrow tests, source presence and status labels are not completion.
- Preserve exact IDs, existing supported behavior and source license notices.
- Keep model units, presentation, topology tolerance, numerical resolution and
  pixel picking tolerances separate. No blanket replacement of kernel thresholds.
- A failed/cancelled/stale operation leaves no partial geometry/metadata or Undo.
- Every native change uses the matching SDK and fresh native runtime evidence.
- Keep older checkpoint evidence immutable. Record per-run source/module hashes.
- No main merge or publication is part of this continuation; work on the current
  dedicated branch to preserve the configured SDK. User authorization already
  covers implementation of all written P0 contracts; no repeated scope approval.

## Work packages (all remain required)

- [ ] Audit all P0 clauses/fixtures against the actual checkout and evidence;
  maintain a completion matrix with explicit missing/partial/verified states.
- [ ] RCORE-01 and P0 RCORE-07: operation/representation/options/backend capability,
  geometry/basis/topology validation, ownership, conversion/parameter reports.
- [ ] RCORE-02: effective per-operation tolerance with provenance; dimension-aware
  input across consumers, atomic unit-change plans, persistence and import/paste.
- [ ] RCORE-03: all required coordinate forms, frame policies, named/undo CPlanes
  and command adoption with numerical/placement verification.
- [ ] RCORE-04: shared request/session lifecycle, entrypoint normalization,
  cancellation, revision checks and late-result rejection with actual native replay.
- [ ] RCORE-05: common snap candidate provenance/resolver, constraints, lifecycle,
  locked/hidden source policy and marker/accepted-point consistency.
- [ ] P0 RCORE-06: selection snapshots/filter/cardinality/permission policies,
  inherited layer state and output layer identity throughout supported consumers.
- [ ] RCORE-09: durable typed replay/identity/schema/context, graph invalidation,
  dirty/error/update policy, cold persistence and stale-job integration.
- [ ] RCORE-10: current-state 3DM/paste semantics, typed provenance, supported
  preservation/conversion matrix, Rust-owned 60% worker budget, cancel and atomic IO.
- [ ] Run the requirement-by-requirement completion audit, fix all remaining gaps,
  obtain independent final review and synchronize the authoritative specs/ledger.

## Current execution

Independent read-only audits cover kernel, interaction and History/IO. The primary
worker handles RCORE-02 and integrates the shared interfaces. Existing merged
current-state export and coarse representation capabilities must be inspected and
extended, not reimplemented merely because an older report calls them missing.

The first detailed task is in
[the tolerance plan](2026-10-09-rhino-core-effective-tolerance.md). Later work packages
are expanded from the completion audit before their implementation, while the full
objective above stays active.
