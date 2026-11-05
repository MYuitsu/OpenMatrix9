# Side View grid axis colours

The user's Side View screenshot showed a cyan vertical grid axis and red
horizontal grid axis, while the Gumball identifies world Y as blue and world Z
as yellow. The default Side View construction plane's local X is world Y and
local Y is world Z. Grid generation had assigned red to local X and cyan to
local Y without accounting for that preset.

For the exact Right construction-plane orientation, grid generation now colours
the world Y line cyan/blue `[0, 200, 220, 255]` and the world Z line yellow
`[235, 180, 50, 255]`. The finite geometry, camera, CPlane, document and other
plane orientations are unchanged. An arbitrarily rotated or reversed CPlane is
not covered by this preset-specific change.

The new scene test failed before the fix because the horizontal Y line was red;
it passes after the fix, checking screen direction, colours and unchanged
colours in the other three default views. An offscreen GPU test reads pixels
near both axes of the published Side View target and verifies blue and yellow
respectively; it passed without an adapter skip. On the E: default-feature
build with `-j 2`, seven camera/grid, 13 GPU/egui, 11 gesture and 26 GPU
navigation/publication tests passed. These tests do not establish pixel parity
with Matrix or a physical desktop display.

An isolated `-j 2` release build was copied to
`E:/om9-surface-snap-validation/side-view-release-20261103/om9.exe` with 81
runtime DLLs. Its SHA-256 is
`C4949B4CB0F76EFC1FB37851684DB76C8EDDAAA19369B1F75859E4E167CA4E63`;
the hidden `--create-fixture` startup check returned exit code 0 and created
its fixture. This verifies that startup path and runtime loading, not the
appearance of a physical desktop window. Existing OM9 windows were not closed.
