# RCORE-01 topology validation review — 2026-10-10

Scope: owned surface-complex observations and native CAD inspection reports in
`2026-10-10-rcore01-topology-validation.md`. This does not close the original
chapter or certify all operation/commit consumers.

Task1 read-only review found an important correctness defect: a wholly
degenerate loop disappeared when its vertex already had a valid link from
another face. A normal triangle plus a collapsed face reproduced the false
`vertex_manifold=true` result. The fix marks the collapsed loop's incident
vertices invalid, retaining mixed-loop sphere/cone poles through seam germs.
The regression also covers a closed tetrahedron with an attached collapsed
face: edge closure/manifoldness remain true while vertex manifoldness is false.

Final: fixed collapsed-loop false positive —
`collapsed_face_at_an_existing_valid_vertex_is_not_silently_discarded`
RED→GREEN; full Rust suite655/655. No additional Task1 algorithm, ABI or native
mapping finding was reported. No repeat review was requested after the fix.

Task2 fresh read-only review found no actionable issue in the44-byte Input,
32-byte Summary, bounded native preflight, owned scalar facts, tri-state policy
or immutable native→world orientation framing. Native Solid roots require
independent validity, closed/manifold/oriented boundaries and known orientation.
Closed shells and compound wrappers retain distinct representation provenance.

The first native solid fixture used `outer.cut(inner)` as a Solid, but FreeCAD
returned a Compound containing one cavity Solid. Its rejection was correct for
the declared root policy. The fixture now tests `.Solids[0]` positively and the
compound wrapper separately. No production rule was weakened to pass it.

Both reviews were read-only: no edits, builds, native host launches or Git
mutations. There are no deferred findings or rulings from these reviews.

Uniform conversion reports and operation/commit consumers, other original
representations/attributes and lifecycle obligations remain in the closure
audit. Native BRepCheck validity is explicitly separate from a global
self-intersection or mathematical embedding certificate.
