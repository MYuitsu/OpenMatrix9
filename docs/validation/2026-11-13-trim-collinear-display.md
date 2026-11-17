# Trim display with adjacent floating-point levels

**Later implementation:** the initial equivalent-boundary simplification
described below was replaced by the endpoint-based strip construction in
[adjacent trim levels](2026-11-13-trim-adjacent-levels.md). The current builder
retains every vertex during partitioning; the collinear fixture still passes.

A valid planar trim with two adjacent vertices one floating-point step apart
on the same vertical UV boundary passed source and loop-topology validation,
but the display scanline builder could not represent a midpoint between their
levels. The scene fell back to showing the untrimmed support surface, and
normal point selection refused the fallback even well inside the real trim.

Scanline decomposition and its admission planner now ignore a vertex only
when it lies strictly between its two neighbours on the exact same vertical
UV line. They traverse the resulting equivalent boundary without allocating a
separate polygon. The original trim metadata and boundary segments are not
changed. Non-collinear or backtracking points are not removed. This is a
display construction improvement; other unsupported fallback cases remain.

The reproduced surface now produces its trimmed display and supports a point
pick inside while rejecting a pick outside. Full and incremental scenes match,
and the candidate peak covers the retained cache in this fixture. A separate
test compares scanline output with an equivalent rectangle with a hole, in
both loop orientations, and checks bent/backtracking vertices are retained.
After `.om9` save/reopen, a simulated egui viewport click selected the source
without changing the document or losing a pre-existing Redo entry. The full
`om9-scene` suite passed; the `rubber_band` and `osnap_ui` batches passed 4 and
26 tests. Physical desktop clicks, arbitrary trim configurations, curved
surface precision and native/process peak remain unverified. The current
desktop binary was not replaced.
