# Cap Planar: bounded registration mapping

The recovered VB6 registration procedure `Proc_198_35_1A2F840` in
`modMillC_Defaults.bas` associates the `SolidCapPlanarHoles` icon with the
external `_Cap` command near addresses `01A357DC` and `01A357C4` respectively.
The same-address Ghidra procedure is `FUN_01a2f840`. This is evidence of the
binding, not the external solver, selection policy, tolerance or defaults.
No recovered code or artwork is reproduced here.

The normalized M03 reference describes selecting surfaces or polysurfaces and
pressing Enter to fill planar closed boundaries. The subsequent kernel
checkpoint supports tested independent planar loops and a circular edge, with
closed-solid output. A later checkpoint adds one tested nested coplanar pair
forming an annular cap; general curved and nested topology remains unproved.
The later mixed-boundary checkpoint also retains nonplanar openings in a
partially capped shell for the tested fixture.
