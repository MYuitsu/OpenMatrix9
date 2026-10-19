# M06/E11 final gate (checklist template — all unchecked)

## Preconditions

- [ ] G00 signed including licensing, native OCCT build and numerical tolerance.
- [ ] E01–E10 signed with working Rust application source, E10 gem/row fixtures and stable history IDs.
- [ ] 76 M06 L4 with precise local source SHA and own REUSE/ADAPT/ADD decisions.

## Exit criteria

- [ ] 19/19 L3 covered by tested implementations or documented, approved scope decision (not merely planned L4).
- [ ] Head basket, half-bezel, rail levels, full/under/scallop bezel generated with correct BRep validity/placement.
- [ ] Prong builder/editor/on-surface, metal piece, channel border, bead on curve/surface and pull-to-rail work in real app.
- [ ] No duplicate kernel/history/gem engine, no After dependency, no E12 cutter subsystem reowned.
- [ ] For every exact BRep output: OCCT checks, orientation, open edges/closedness, supported tolerance and independent geometry metrics PASS.
- [ ] For curve/instance outputs: correct output-type tests, stable reference and geometry/transform numeric checks PASS.
- [ ] Changing gem source produces validated update OR visible conflict, never silently loses history.
- [ ] Invalid inputs/cancel/OCCT failure leave original scene/file unchanged; Undo/Redo is exactly one user gesture.
- [ ] Save/Open persist full parameters, stable IDs, linked gems, material and source units.
- [ ] Mass excludes nonmetal / guide / cutter geometry; material density provenance and unit conversion verified; invalid solids do not yield trusted mass.
- [ ] Full E01–E10 regression and M06 targeted suites pass, logs attached with source/toolchain SHA and fixture provenance.
- [ ] Manual UI run on target platform (OS/GPU/UI interactions) documented and signed by reviewer.
- [ ] Legal/redistribution review complete; no proprietary UI assets/screenshots/code copied into public repo.

**Result:** `NOT_ASSESSED — PLANNING ONLY`.

**Human reviewer:** _not assigned_  
**Source SHA / toolchain:** _not available_  
**Evidence root:** `implementation/evidence/M06/` (planned; not created)  
**Blocking issue IDs:** _not assigned_
