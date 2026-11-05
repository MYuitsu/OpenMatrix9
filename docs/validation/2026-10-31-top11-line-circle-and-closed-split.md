# Top11 line/circle intersections and closed-curve Split

The sampled curve intersection search missed two line/circle crossings near a tangent. The Rust solver now calculates candidates for line/circle pairs directly, checks the actual finite line and arc intervals and measured distance, and handles the tested crossing, tangency, skew and perpendicular cases. It scales the tested calculation to retain intersections at radii `1e-200` and `1e200`. Core tests also cover swapped argument order, a near miss, an oblique intersection, tolerance and an arc interval.

Split previously treated the parameter seam of a closed curve as an additional cut: a circle crossed twice produced three pieces. Closed targets now retain seam intersections and use the cyclic intervals between actual cuts. CMD and toolbar tests verify both crossings of a near-tangent chord and a chord crossing the seam, their endpoints, the total circumference and Undo/Redo. An existing periodic NURBS/BRep test expected five pieces for four crossings; it now expects four and still verifies length, points on the original curve, `.om9` persistence and Undo/Redo, with reversed and reseamed inputs.

The near-tangent Split/Trim test checks actual line-piece boundaries and Undo/Redo through CMD and toolbar. A typed Trim point at this proximity is ambiguous between target and cutter; the test explicitly selects the intended object. The initial core regression returned no crossings and the initial closed Split test returned three pieces; both now pass. The first full BRep run exposed the outdated five-piece expectation above, and the updated test and full suite pass. An intermediate test edit failed compilation before its loop scope was corrected.

The near-tangent line Split output was also exported to and imported from real 3DM; all three line pieces and both crossing positions survived. The admitted native adapter SHA-256 was `bddca46577747333f9377b379351a077b710158f47ce6e279d7b6e21e6439615`.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: 204 core library, 34 core Top11, 17 curve/surface, 120 app Top11 and 83 app Top11 BRep tests passed; 120 no-default app Top11 tests passed with existing warnings. One real-native 3DM roundtrip passed. These checks are not an openNURBS measurement batch.

General sampled curve intersections, overlapping curves, tolerance-only minima for tilted line/circle pairs, extreme ratios, cyclic deduplication near a closed seam, one-cut closed behavior and physical desktop input remain open. This evidence does not establish completion of Top11.
