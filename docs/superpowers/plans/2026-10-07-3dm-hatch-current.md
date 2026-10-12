# Current native Hatch fields and derived boundary display

Continue the approved full openNURBS specification inline. The preceding goal
turn made progress: verified native affine loops/patterns, GUI857/native15 and
SHA checkpoint. Full class/category and Rhino5 acceptance remain open.

1. Native RED tests for complete hatch_fields overlays (plane16, base2,
   rotation/positive scale, exact pattern UUID) and source-independent copies.
   Add tiny positive scale tests: the pinned setter ignores scale<=0.001, so
   normalization/current edits must preserve requested doubles rather than silently
   retaining an old value. Validate malformed/incomplete/ambiguous fields and
   atomic destination/source invariants.
2. Implement staged current-field application against the verified native model;
   resolve exact table/system pattern identity, materialize chosen built-in
   patterns when needed and retain loops/attributes/userdata. Inventory exposes
   normalized current native fields for a verifiable host baseline; final writer
   refreshes those facts before its existing exact reread gate.
3. Persist an explicit FreeCAD Hatch adapter with origin/X/Y vectors, base point,
   rotation (degrees), positive scale and named native pattern selection. Apply
   current fields before each object's own matrix, including canonical edits,
   independent/member/family copies and shared proxy routes. Ambiguous schemas or
   tampered baselines must fail, never silently substitute source fields.
4. Derive native rational boundary BReps/display from current fields; preview data
   stays separate from native export authority. Verify runtime Undo/Redo, parent,
   mm/cm, source deletion/FCStd and invalid fields; build and run regressions.
5. Update scoped README/spec/coverage/ledger and checkpoint. Full editable loops,
   pattern-line editing/rendering, legacy/gradient/child-loop userdata/repair and
   all remaining geometry/style/resource/component/document/history/version
   requirements and actual Rhino5 renderer acceptance remain required.


## Hatch current fields and boundary progress — 2026-10-07

- [x] Persistent native origin/axes/base/rotation/tiny positive scale and exact
  pattern reference selection; staged mm/cm current-field/copy/block/proxy routes.
- [x] Derived native rational boundary BRep/Coin outline, Undo/Redo, embedded
  FCStd/source deletion, schema/baseline/choice/invalid-field atomic rejection.
- [x] Gradient is explicitly retained without the ordinary adapter and this
  unverified Rhino5 export rejects before replacement; full gradient support open.
- [ ] Native loop/pattern-line editing, full pattern rendering, actual Rhino5
  renderer/roundtrip and remaining full class/category semantics and acceptance.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-current.md` — isolated
FreeCAD921/921 across43 suites; native16/16. Earlier dated pending current-field/
preview notes superseded within this scope only; full openNURBS in_progress.
