# Sphere viewport guides — 2026-10-30

The user's reference shows green spherical faces in both selection states,
with three green great-circle guides when unselected and magenta guides visible
through the sphere when selected. The viewport now derives those three circles
for a recognized full, single-face spherical display mesh; it retains the
object's face colour. The subsequent user-directed global line-colour rule
sets unselected guides green independently of stored line colour; see
`docs/validation/2026-10-30-object-line-colours.md`. The guides are presentation-only and
are neither document curves nor selectable edges.

Recognition fits a sphere from separated display vertices and checks all
vertices, three-axis extent and the triangles' solid angle. This rejects the
tested ellipsoid and trimmed fixture rather than drawing through its missing
surface. The selected guide pass uses the existing on-top GPU overlay and follows
the selected object's preview transform; it shares the box overlay's limitation
that guides can show through an unrelated object in front.

Validation on the OCCT-enabled target, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`:

- `cargo test -p om9-app --lib box_guide_tests`: two tests passed, including
  actual sphere BRep, 288 line segments in three great circles, translated
  input, ellipsoid rejection and incomplete spherical mesh rejection.
- `cargo test -p om9-app --test gpu_navigation_publication`: five tests passed;
  the focused sphere test was repeated with two spheres, one selected and one
  unselected, and passed without changing the document.
- The existing `om9-render` depth-overlay GPU test verifies on-top lines remain
  visible behind depth. No new actual desktop screenshot or image comparison was
  performed, and the material highlight in the supplied image was not matched.

Limitations: recognition currently requires a full single-face spherical mesh;
other representations and trimmed or multi-face spheres are not covered.
Solid-angle and radial checks are conservative display heuristics, not exact
classification of the canonical BRep. Original Matrix behavior is unverified.
