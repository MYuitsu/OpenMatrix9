# STL authored mesh quad triangulation

P3a correctness fix: a validated authored mesh face may have three or four vertices. STL export now selects an interior diagonal for a concave quad using a dominant-axis projection instead of always using the first-to-third diagonal. A quad with no consistently oriented pair is refused, preserving the existing destination. Face winding is preserved; reflection handling remains separate.

Windows, default OCCT/openNURBS, `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`: all 27 STL integration tests passed. The concave fixture checks two encoded triangles have the original winding and total area of 1.5 square units; a crossed quad test checks refusal and preservation of existing file bytes. One additional unit test passed for the concave fixture in three axis orientations and reversed winding.

The projection test is not an exact-arithmetic predicate or global self-intersection certificate. Highly ill-conditioned or nonplanar quads, large coordinates and other polygon configurations require additional validation; this change does not certify all authored mesh surfaces. The 256 MiB aggregate limit, memory reservations, UI background export and other plan gates remain open. The live OM9 session was not modified.
