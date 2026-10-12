# Rhino core periodic FCStd persistence implementation plan

> **For agentic workers:** Use superpowers:executing-plans in this session;
> independent fixture review follows matching host/cold evidence.

**Goal:** Complete the authored RCORE-01.T09 periodic-curve and seam/hole face
FCStd acceptance fixture, continuing the approved all-P0 plan from `83a806c`.

**Architecture:** Reuse FreeCAD native geometry construction, FCStd serializer,
world shape extraction and existing identity guards. Add only test fixture and
evidence; no new command, production geometry algorithm or serializer.

**Tech Stack:** FreeCAD/OCCT, existing OM9 native snapshot API, Python FCMacro.
**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`,
T09. Reuse audit: `H:/FreeCAD-src/build/p0-next-kernel-reuse.md`.

## Global constraints

Four intended objects: R10 rational circle, square20 with holeR3, open cylinder
R4/H7 seam face, and true periodic degree2 curve. Periodic curve poles
[(0,0,0),(1,2,0),(2,0,0)], knots[0,1,2,3], multiplicities[1,1,1,1], weights1,
translation(7,-5,2). Sampled geometric bound1e-7mm, numerical persistence field
comparison1e-12; bbox kernel numerical padding stated separately. No 3DM
periodic-preservation, global continuous deviation or all-family claim.

## Review focus

True periodicity versus visual closure; exact compact native basis before save;
once-applied world placement; ordered seam uses versus unique edges; retained
hole trim; cold module/file hashes and old-session rejection; no read-created
objects/properties/Undo; sampled oracle versus a global bound.

### Task 1: authored persistence fixture

Create `tests/rhino_core_periodic_persistence_smoke.FCMacro`.
Reuse the existing kernel fixture's basis/topology predicates and supported
native constructors without modifying that historical macro.

- [x] Create four fixtures; inspect native periodic basis before saving.
- [x] Check three cyclic quadratic spans at257 samples each plus independent
  extrema against world native shape. Bounds local x[1/3,5/3],y[0,3/2],z0;
  translated x[22/3,26/3],y[-5,-7/2],z2. Do not assume a parameter-span ordinal.
- [x] Record raw BRep, basis, topology, property/object/Undo inventory and world
  bounds before assertions. Save FCStd and compare after warm close/reopen;
  reject old snapshot tokens and accept fresh ones.
- [x] Reopen exact saved file in a fresh process using explicit baseline path;
  compare type/topology/basis/placement/bounds and repeat geometric witnesses.

### Task 2: verify reuse and scoped acceptance

- [x] Run existing kernel warm/cold macro unchanged on the same module as control;
  diagnose any fixture or native discrepancy before changing predicates.
- [x] Independently review fixture, literal T09 coverage and persisted artifacts.
- [x] Synchronize scoped report/evidence/spec/ledger and664-entry manifest;
  retain full P0 in progress and every other open fixture. Verify hashes and
  commit locally. Existing serializer working is a reuse proof, not a new
  product RED→GREEN behavior or new production test requirement.
