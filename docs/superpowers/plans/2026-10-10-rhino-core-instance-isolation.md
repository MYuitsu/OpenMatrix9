# Rhino core instance isolation implementation plan

**Goal:** Complete the authored RCORE-01.T06 shared-definition affine fixture,
reusing existing native instance/transform APIs and filling only demonstrated gaps.
Baseline `661529f`; all P0 remains in progress.

**Architecture:** Existing FreeCAD App::Link/ScaleVector, OM9 native archive
expansion and atomic structural/working writers. Fixture-only first. Any missing
portable matrix validation/report policy must be safe Rust; C++ keeps native
ownership/kernel calls, Python supplies host acceptance/bootstrap only.

**Source:** RCORE-01.T06; reuse audit `H:/FreeCAD-src/build/p0-next-instance-reuse.md`.
Approved object overview PDF25–31/printed17–23 and explicit Rhino5 Insert/Block
reference scope in that audit. No new Mirror/Scale command or UI is proposed.

## Task 1: authored host baseline and isolation

- [x] Create box(2,3,4), member placement(1,2,3), definition(4,5,6).
- [x] Two links share that definition: A placement(100,200,300), unit scale;
  B placement(20,30,40), Z+90° rotation, scale(-2,3,.5).
- [x] Bijectively match all eight world corners to literal independent tables,
  within1e-7mm; keep definition/local-member/sibling exact fingerprints.
- [x] Change only B by(7,-11,13) in one transaction, verify A/definition unchanged,
  Undo/Redo and native structural/working output from both placements.
- [x] Invalid singular export keeps destination/document/Undo/source state.
  Fresh FCStd reopening only if explicitly claimed and checked.

## Task 2: exact native archive and invalid-transform boundaries

- [x] Native authored matrices A and B target one canonical local box definition;
  inspect actual import coordinates, unchanged native member/sibling payloads.
- [x] Exercise existing structural instance overlay/writer with projective and
  singular matrices; preserve input/output byte sentinels on rejection.
- [x] Native NaN/Infinity on detached geometry rejects before mutation; no JSON
  null/string masquerades as an IEEE native-number test.
- [x] Do not broaden to overflow/empty/all-family preflight without actual evidence.
  If required behavior fails, capture RED and implement only that missing gate.

## Task 3: review, verification and scoped synchronization

- [x] Matching native/FreeCAD proof, source/runtime/artifact hashes, independent
  review and appropriate existing block/exchange regressions.
- [x] Mark only T06's stated authored acceptance after every assertion passes;
  update live/spec/package records and commit locally, then continue all P0.

Review focus: Placement*Scale*definition composition, once-applied reflection,
exact local member/sibling identity/geometry, valid baseline cache observations,
raw versus derived definition bounds, named rejection before atomic replacement,
no global/History/reference-owner or command-UI parity claim.
