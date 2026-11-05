# Intersecting closed STL components

Two authored closed tetrahedra are exported as separate selected sources. The second tetrahedron has one vertex strictly inside the first and an adjacent vertex strictly outside, so that edge crosses the first tetrahedron's face. The binary STL contains eight triangles and the receipt reports two source-scoped edge-connected components. Indexed and exact-coordinate topology checks report closed, while the receipt explicitly states that self-intersection was not checked.

The focused integration fixture passed on E: with `-j 2`. It verifies an important limit of the existing receipt, not self-intersection detection or printability. No production behavior was changed. More generally, closed incidence, winding and connected vertex fans cannot establish that a mesh is free of geometric intersections or has correct material containment.
