# Top11 nested closed curves and seam cutters

The scene previously rejected full circles and unseamed periodic NURBS as polycurve leaves. It now samples those leaves using the existing arc chord bound and each active NURBS knot span, in both direct and nested polycurves. Admission still rejects discontinuous knot structure and checks connectivity and display allocation before publication. Explicitly reseamed NURBS remain outside this display path.

Split's cutter-selection validation previously discarded hits at fractions zero and one even for a closed target. As a result, a cutter passing through the curve seam could be rejected before the later split calculation. Validation now uses the same closed-curve classification as the split operation. Open-curve endpoints continue to use the existing endpoint policy.

The scene fixture checks a full circle and a closed quadratic periodic NURBS nested twice, with both enclosing directions, a quarter-point versus the sampled polyline and an exact point-budget boundary. Application cases cover Split and Trim on direct and nested versions, CMD and toolbar entry, both enclosing directions, two tangent cutters including a full-circle seam, measured output endpoints and length, `.om9`, Undo and Redo. This is not physical-pointer or human acceptance evidence.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target-top11-nested`, Rust `-j 2`: complete scene suite, 131 app Top11, 84 app BRep, 10 options and three command snapshot tests passed. The no-default-feature Top11 suite passed 131 tests with existing warnings. An explicitly invoked real-native 3DM Split test exported and imported one nested circle and one nested periodic NURBS result and verified both endpoints on each of two pieces; adapter SHA-256 `bddca46577747333f9377b379351a077b710158f47ce6e279d7b6e21e6439615`. This interchange test is not an openNURBS measurement batch.

The evidence does not cover explicitly reseamed polycurve leaves, all periodic knot patterns, arbitrary geometric intersections, 3DM Trim, or full desktop/GPU/human M03-UI acceptance. The pinned native short-line validity limitation remains separate. Top11 is not complete.
