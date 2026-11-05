# Sphere preview and completed display — 2026-11-01

The user asked for six pole-to-pole lines and three horizontal lines while
constructing a sphere, and four pole-to-pole lines and one horizontal line once
it is complete. The supplied screenshot shows an unfilled spherical wire
preview. OM9 interprets the first request as three full meridian circles (six
pole-to-pole arcs) and three latitude circles; the completed sphere retains two
full meridians (four arcs) and one equator. The screenshot does not fix exact
latitude values, so the two non-equatorial circles use z = center ± radius/2
as an OM9 interpretation. Guides are display-only, do not create model curves,
and retain the existing green/unselected and magenta/selected display policy.

Validation with default OCCT and `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`:

- A focused pre-fix test failed with 192 segments where the six preview circles
  require 384. The corrected `sphere_preview_grid_tests` passed: three distinct
  meridian planes meet both poles, all points lie on the sphere, and the three
  latitude circles are at the stated heights. Commit and document Undo passed.
- 52 existing Sphere tests passed via a temporary lib-test harness, removed
  afterward. They cover Center, two/three/four-point, around-curve and other
  supported workflows; counts in their preview assertions were updated.
- The existing completed-sphere GPU guide test passed with 288 segments in the
  three coordinate planes, and rejects an ellipsoid and an incomplete mesh.
- `cargo build -j 2 --release -p om9-app --bin om9` succeeded. The release
  executable SHA-256 is
  `8646a08df06c4d47818e850ed54ab2d756dd1022afb5b33c94366e9c7914dbb9`.

An actual desktop screenshot comparison has not been performed. Completed
display recognition still covers the full single-face sphere representation
described in `docs/validation/2026-10-30-sphere-viewport-guides.md`; do not
infer support for every imported or trimmed sphere from this test. The user's
currently open OM9 document was not closed.
