# Rhino core subobject mutation ownership implementation plan

> **For agentic workers:** Use superpowers:executing-plans; independent native/
> fixture review is required before scoped acceptance.

**Goal:** Complete authored RCORE-01.T11 owner/isolation and standalone-versus-
subobject lifecycle evidence, continuing all P0 from `422098b`.
**Architecture:** Reuse native Part shape/deep copy, copied Curve/Surface Python
wrappers, App::Link shared definition and existing Rust-backed object snapshots.
Add a host fixture first; change product code only if an actual contract fails.
**Tech Stack:** FreeCAD/OCCT, OM9 object guards, Python native host fixture.
**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`,
T11; native reuse audit `H:/FreeCAD-src/build/p0-next-kernel-reuse.md`.

## Global constraints and review focus

State the mutable owner before each edit. A strong native shallow shape alias is
not a borrowed raw pointer or a document/subelement request. Native Curve/Surface
getters copy support; mutation of that detached wrapper must not mutate source,
shared instances or prior native snapshots. Publish only through an explicit
new output owner and one transaction. Test source-owner mutation separately:
shared-definition instances then change as intended. No trimmed-face editor,
worker release/sanitizer or all-family ownership completion is inferred.

Create `tests/rhino_core_subobject_ownership_smoke.FCMacro`. Identity-frame source
Wire has line(0,0,0)→(1,0,0) and quadratic curve poles[(1,0,0),(2,2,0),(3,0,0)],
degree2, knots[0,1],multiplicities[3,3],weights1. Source Face is untrimmed degree2×2
BSpline on grid(x,y)∈{0,1,2}²,z0, clamped[0,1] both axes, weights1.
App::Part definition owns both; two App::Link instances share that exact owner.

- [x] Capture exact source/definition/sibling CAD and placement fingerprints,
  native shallow shape alias and explicit deep geometry copy after fixture setup.
- [x] Edit the detached subcurve middle pole to(2,2,2) and detached surface center
  pole to(1,1,4). Independent values at0.5 and(0.5,0.5) must be(2,1,1) and(1,1,1).
  Source/sibling/shallow/deep data and document/Undo stay unchanged; guards valid.
- [x] Publish two standalone outputs in one transaction using existing
  `toShape().copy(copyGeom=True,copyMesh=False)` to separate native support owners.
  Re-edit retained wrapper poles to z6/z12 (midpoints z3); output raw bytes/basis
  remain at z1, source/sibling exact, one Undo removes both and Redo restores.
- [x] Explicit source-Wire owner replacement changes shared instances and stales
  the old Edge request; independent Face request and detached owners stay valid.
  Undo restores source geometry and invalidates previous request epoch.
- [x] Close source document, reopen same name; old document/subelement guards
  reject, detached native Curve/Surface and shallow/deep snapshots remain usable.
  Validate exact values, no output/object/Undo in replacement document.
- [x] Review native getter/copy/guard ownership sources and actual matching host
  artifacts independently; scope report/spec/ledger, verify hashes/manifest and
  commit locally. Full P0 and T07/T08/T10 remain open.
