# Encoded STL with disconnected vertex fans

An authored mesh contains two tetrahedra sharing exactly one indexed vertex and no edges. Its encoded binary STL is read independently: it contains eight triangles, seven unique exact-coordinate vertices and every encoded edge has incidence two. The app receipt nevertheless reports two edge-connected triangle components, one disconnected vertex fan, and `encoded_closed_by_topology_checks = false`; no encoded vertex identity ambiguity is reported. The application still writes the file with its existing topology diagnostics; this fixture does not add automatic repair or a new rejection policy.

On E: with `-j 2`, the focused document-to-STL fixture passed, as did all 10 exchange STL policy tests including the existing strict-writer disconnected-fan refusal. This checks one singular-vertex case, not self-intersection, general manifold certification, or printability. No production behavior changed.
