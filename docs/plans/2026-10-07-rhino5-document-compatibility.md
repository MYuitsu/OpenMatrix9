# Rhino 5 document compatibility follow-up

Approved scope: continue OM9-FILE-012 import/export repairs; retain original user
fixtures, metadata, signed placements and the existing 0.001 mm geometry threshold.

Fresh application run `rhino5-retest-20261007-135206/application-test-20261007-135520`
completed all eight lifecycles without invalid objects or Unicode exceptions.
The user explicitly confirmed no newer-RDK dialogs. Six earlier missing mass
properties and five area failures are resolved. Two inward diamond placements
still become outward in the opened flat export. Two revolution bounds differ by
0.00161462 mm. FreeCAD reimport independently reproduces the signed-volume change.

1. Preserve this actual RED run, command history and direct RDK confirmation.
2. Use raw Rhino File3dm.Read before document insertion to distinguish archive
   orientation from Rhino document normalization. Measure trim-aware physical
   samples bidirectionally for the two bounding discrepancies, without relaxing
   the threshold or replacing raw evidence.
3. Regress document normalization on archive export. If flat inward BReps are
   normalized by Rhino, encode their physical placement with outward block members
   and a negative-determinant instance transform, preserving user labels, layer,
   visibility, lock metadata and color. Keep ordinary blocks unchanged.
4. Build/test the specific archive path, then both unchanged ring fixtures.
   Run native and GUI checks sequentially. Recheck broader affected export paths
   only after targeted tests pass.
5. Prepare a fresh application retest with immutable exports and module hashes.
   Close remaining findings only from new Rhino measurements and FreeCAD reimport.
6. Record scoped completion in README/spec/ledger, update the specification
   manifest and retain a verified source checkpoint. Full128/16/6 remains partial.

Rhino document normalization evidence:
https://discourse.mcneel.com/t/what-magic-does-baking-do-to-make-breps-more-realer/147810/8
The user's read-only diagnostic completed successfully. Raw File3dm.Read retains
both inward solids; Rhino document SaveAs flips their faces and sign. The
archive/document boundary is confirmed. Native35/35 passes with an outward
reflected member and negative-determinant root. Empty labels and unlocking retain
logical metadata after a second reflection. Preservation of flat inward solids
now refuses before opening the target; native definition members remain allowed
but require a separate actual Rhino lifecycle case. This is an explicit remaining
  preservation limitation, not full openNURBS completion.

Follow-up actual nine-file run application-test-20261007-145132 completed True;
the user explicitly confirmed no warnings/errors. Both ring negative placements
now retain their sign in Rhino and FreeCAD reimport. The added preserved inward
member is a real RED: -2880 becomes +2880 mm³ even inside a definition. Aggregate
analysis43/46 and reimport28/29 remain immutable failure evidence.

The bounded remedy rejects every raw inward BRep before public Rhino5 writes.
Geometry-only placed blocks assemble their verified current graph in a distinct
internal SDK80 stage, retaining existing class/UUID/attribute/payload checks, then
flatten through the signed geometry codec into Rhino5. No JSON flag bypasses the
public preservation guard. Test atomic member refusal, SDK80 staging, final
negative volumes, physical bounds, native source immutability and FCStd Undo/Redo;
then native/GUI/ring regression suites serially and fresh actual Rhino acceptance.

The two strict bounds findings remain open: 0.00161462 mm exceeds 0.001 mm.
4,227 bidirectional surface/edge samples have maximum physical distance
0.0000090731 mm. Sampling is separate evidence and does not convert strict bounds
failures into passes or prove unsampled extrema.

Follow-up: resolve standalone SDK +2 from native winding on a copied BRep. The
canonical OCCT shell must keep its inward/outward winding, with infinity
classification refusing an indeterminate result. Derive the sign before an
affine transform, then apply determinant parity. Identity import keeps winding;
public Rhino5 preservation also resolves +2 on a copy and refuses inward raw
members before touching the target. Declared +/-1 behavior and native payload
identity remain intact. Test unknown inward/outward, shear/nonuniform scale,
odd/even reflections, native source immutability, public atomic refusal, nested
Geometry-only export with the formerly incorrect +576 corrected to -576.
Use pinned McNeel GitHub and official OCCT sources, then serial native/GUI/ring
regressions and a fresh actual Rhino5 lifecycle pack.
