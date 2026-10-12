# Independent legacy5 Hatch semantic acceptance

Continue the approved full openNURBS specification inline. Native20 cases, raw
ownership/gradient probes and GUI24 legacy checks are now implemented; final
native17/17 and GUI945/945 evidence is recorded in
`docs/validation/2026-10-07-3dm-hatch-legacy5.md`. This plan is not evidence by
itself. Actual application rendering in item6 and broader class cases stay open.
Use the unchanged official McNeel SDK20130711 archive and existing isolated
tests/native/rhino5_reader project. Do not replace the pinned modern SDK or alter
the SDK sources. The existing independent PointCloud reader remains required.

1. Add a separate bounded legacy Hatch reader/writer executable using actual
   SDK2013 ON_Hatch and ON_HatchPattern APIs. Read native version5/50 input;
   report exact plane16/base2/rotation/scale/pattern UUID, loop types/native curve
   classes, rational NURBS dimension/order/CV weights/knots and line pattern
   angles/base/offset/signed dashes. Resolve table identity without modern
   ON_ModelComponent APIs. Preserve attributes/native object identity.
2. RED tests compare modern outputs with the independent legacy report: native
   custom lines, solid/Grid60 selection, current tiny scale/tilted plane edits,
   affine rational loops/derived patterns, source mm/cm normalized outputs and
   canonical/copied/shared block closure. No object-count-only acceptance.
3. Legacy writes a fresh version5 file; the modern reader verifies equivalent
   complete Hatch/pattern fields, loop representation and dependency graph.
   Distinguish modern same-SDK reread from legacy decode/re-encode evidence.
4. Probe gradient data separately using modern version80 input and a deliberately
   staged version5 serialization. Document whether real legacy SDK retains or
   loses its native data. Keep the current explicit incompatibility gate until
   there is full source/target-version semantic evidence; no newer userdata
   wrapper may be mistaken for Rhino5-native support.
5. Retain snapshot/source bytes and sentinel destinations through failed cases.
   Run final native and isolated FreeCAD regressions, synchronize scoped coverage,
   spec/README/ledger and save a verified checkpoint. Keep class-wide completion
   false while fields/legacy/userdata cases remain unverified.
6. Actual Rhino5 display remains a separate application oracle: custom nonzero
   bases/offsets, Plus/Grid60 patterns, reflected/anisotropic placements and signed
   dashes. The published line-frame contract and SDK reread are insufficient to
   establish actual visual rendering. Native loop/pattern-line editing and full
   boundary clipping/fill/dash rendering remain required after this acceptance
   work; other full class/category packages and final comprehensive review remain
   open. No slice subagent review or public integration is performed here.


## Hatch legacy5 semantic progress — 2026-10-07

- [x] Independent unchanged SDK201307115 native20 mm/cm exact Hatch/loop/pattern/
  block decode-reencode cases, including current tiny scale and named builtins.
- [x] Typed legacy movable-base representation and public attribute ownership
  repair; separate raw-loss and repaired evidence retained without SDK edits.
- [x] Enabled gradient type/colors/repeat are lost at the modern version5 boundary;
  target-version incompatibility and atomic production rejection are established.
- [ ] Actual Rhino5 rendering, native loop/pattern-line editing, full pattern
  rendering, broader legacy/userdata/repair and remaining full class/category work.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-legacy5.md` — isolated
FreeCAD945/945 across44 suites; native17/17. Full openNURBS remains in_progress.
