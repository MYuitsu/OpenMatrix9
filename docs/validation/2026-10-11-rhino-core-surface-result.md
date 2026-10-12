# RCORE-01 — native Surface GeometryResult

Status: scoped native validation; **full RCORE-01 remains in progress**.
Recorded 2026-10-11T07:49:52+07:00. [Evidence receipt](2026-10-11-rhino-core-surface-result.json).

Rust admits an actual EditSurface result using the saved Surface request and
shared native capability policy. It validates logical/flattened cardinality,
concrete feature ID, current rail closure, source stamps/frames/units, owned wire
snapshots and matching native source attributes. Output admission requires a
valid surface with faces and no solid. Import and Builder provenance are kept
separate. Input snapshots keep native masters before seam/reverse/refit options;
the recipe records those options. No native pointer survives in result JSON.

The native adapter reuses FreeCAD/OCCT Loft/Sweep, captureObject/sourceAttributes,
topology/validation and the existing prepared CAD snapshot factory. It captures
and verifies source guards around the solver and publication. The command prepares
both a deep CAD copy and accepted metadata before opening its GUI transaction;
output recompute is nonrecursive. History prepares both privately, publishes the
pair, clears both after a replay failure, and retains immutable evidence while
frozen or detached. Recorded and unrecorded FCStd outputs retain the evidence.

World-to-feature conversion reuses FreeCAD's existing native rigid copy transform,
so a recomputing Part feature's Placement does not change the serialized master
after acceptance. The result records output parent frame and the actual unique
output name. C++ is limited to native ownership, host properties and kernel
serialization; Rust owns portable admission and independent result composition.
Python macros are host verification only. Shared FreeCAD SDK was reused; no root
build or complete runtime clone was made.

Construction correspondence remains explicitly uninspected: deviation, global
parameter/topology maps are null. Constructed-output attribute availability is
explicit; source attributes belong to input snapshots and are not invented as
output attributes. Historical evidence is not current validity after direct edits.

## Verified evidence

- Original result RED12/9 failures: all three existing surfaces valid, missing
  output result/operation/snapshot. Same unchanged probe now passes12.
- New native result fixture81 covers independent BRep restore, real recipe/IDs,
  source attributes/revisions, paired replay, freeze/detach, Undo/Redo and FCStd.
  Three fresh cold processes pass13 each, load native History without preimport,
  and replay a source edit into both Shape and result.
- Review findings reproduced before repairs: frame RED4/2 failures and unique
  name RED6/1 failure, both actual host0; final frame4 and command identity6 pass.
- The first frame repair caused an analytic-area regression by using a generalized
  transform. Native RED8 recorded the cylinder becoming BSplineSurface, area
  151.20111352201684 instead of48*pi. The rigid copy repair keeps analytic geometry
  in identity and nonidentity frames; original command thresholds remain unchanged.
- One final analytic host launch exited -1073741816 before writing any stdout or
  report. Its terminal launch artifact is preserved; no live process was restarted.
  A fresh diagnostic run reached the fixture and exposed a generic Part.Shape API
  mismatch (CenterOfMass belongs to the restored face here). The fixture now reads
  the single face center. Fresh acceptance uses the same plugin and preserves
  original type/area/frame assertions. These two runs are not counted as passes.
- During progress recording, Gui/CMakeLists.txt differed from its source-capture
  hash; the current target still includes CoreSurfaceResult.cpp exactly once.
  All other scoped producer/native/Rust/test source fingerprints match. The
  accepted binary remains the exact plugin used by all final21 runs. The changed
  build configuration is recorded separately and is not claimed to match that
  binary; full chapter integration remains open.
- Final matching 848 checks/21 actual host exits0. Full Rust 791
  tests/112 nonempty suites, actual exit0. Existing path/cache and OCCT
  deprecated-header warnings remain diagnostics; tests report no failure.
- Standalone plugin build4 actual exit0. Plugin SHA256 `cfe4229f3102d2df07c8f8fa3b9f293aa2bd780c22814fa0a307b5881b7a0fea`;
  shared host SHA256 `98eef899354a8aa84cfa2934e6e1a3c8558c7e6fff3b89c347c06731f06018a8`. Final scoped sources are fingerprinted.
- [Independent review](../reviews/2026-10-11-surface-result-review.md).

| Native fixture | Checks | Actual host exit |
|---|---:|---:|
| frame1 | 4 | 0 |
| identity1 | 6 | 0 |
| analytic1 | 8 | 0 |
| contract1 | 12 | 0 |
| result1 | 81 | 0 |
| cold-result-sweep1 | 13 | 0 |
| cold-result-sweep2 | 13 | 0 |
| cold-result-loft1 | 13 | 0 |
| admission1 | 55 | 0 |
| commands1 | 97 | 0 |
| options1 | 180 | 0 |
| advanced1 | 108 | 0 |
| constraints1 | 85 | 0 |
| history1 | 65 | 0 |
| cold-sweep1 | 3 | 0 |
| cold-sweep2 | 3 | 0 |
| cold-loft1 | 3 | 0 |
| policy1 | 29 | 0 |
| diagnostics1 | 6 | 0 |
| builder1 | 17 | 0 |
| prepared1 | 47 | 0 |

## Remaining original scope

- Original global conversion/deviation bounds, full-domain parameter/topology correspondence and remaining native GeometryResult consumers.
- Uniform source/output attributes, preservation/export, lock/revision/generation, cancellation/cache/transaction contracts across all original families.
- Original adapters1–7, C01–C12, T01–T11 final acceptance and aggregate native/worker budgets; no all-family completion claim.
