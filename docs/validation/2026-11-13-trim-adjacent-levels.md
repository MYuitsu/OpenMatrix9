# Trim display with adjacent non-collinear levels

A valid planar trim with two adjacent floating-point UV levels forming a real
notch previously fell back to the untrimmed support surface and could not be
point-selected, even well inside the trim. Unlike a redundant collinear
vertex, the notch cannot be removed without changing the boundary.

The scanline builder and its planner now determine the crossing edges from
both strip endpoints. The ordering coordinate comes from those endpoint
intersections, so no representable UV midpoint is required. All source
vertices are retained, including for the earlier collinear fixture; this
replaces the earlier display-only vertex simplification. Existing topology and
crossing ambiguity checks remain in place.

The notched fixture passes source topology validation, produces a trimmed
scene containing both notch vertices, selects a point inside and refuses a
point outside. Full and incremental scenes match and the planned peak covers
the retained scene in this fixture. After `.om9` save/reopen, a simulated
egui click selects the surface without changing the document or a pre-existing
Redo entry. The earlier collinear fixture and hole/reversed-loop controls also
pass with vertices retained. The full `om9-scene` suite, five `rubber_band`
tests and 26 `osnap_ui` tests passed after the final implementation.

This is sampled/headless evidence for planar fixtures. It does not prove
arbitrary numerical configurations, curved trim accuracy, native peak or
physical desktop interaction. Unsupported and ambiguous trims continue to
use the existing safe fallback. The running desktop binary was not replaced.
