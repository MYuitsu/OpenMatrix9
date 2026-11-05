# Matrix reference options vs the OM9 command bar (derived, 2026-11-02)

Comparison of the option words recorded in the per-L3 reference texts with the options the OM9 commands publish (start / after one point / after two points). Automated extraction can include sub-options from descriptions, so rows need a per-command read before changes.

Commands compared: 26; with differences: 26.

| Command | Reference options OM9 does not offer | OM9 options not in the reference | Reference |
|---|---|---|---|
| `ExtrudeCrv` | alongcurve, autochainyes, bothsides, corners, deleteinput, direction, draftangle, flipangle, no, round, smooth, solid, splitattangents, subcurve, toboundary, topoint | - | p229 |
| `FilletEdge` | addhandle, all, angletolerance, autochainyes, chaincontinuity, copy, copyhandle, direction, fromcurve, fromtwopoints, gaptolerance, next, no, previousedgeselection, trimandjoin | cancel | p227 |
| `Torus` | corner, diameter, fitpoints, fromfoci, markfoci, no, none, point, radius, solidyes, tangent | 2point, 3point, cancel, undo | p234 |
| `InterpCrv` | center, chord, close, endtangent, persistentclose, sharp, sqrtchrd, starttangent, uniform, vertical | cancel | p135 |
| `Arc` | deformable, degree, extension, fromfirstpoint, point, pointcount, radius, tangent, throughpoint | cancel, startpoint, undo | p144 |
| `Fillet` | direction, extractall, join, no, radius, toggle, trim, yes | cancel, nextradius, showradius, undo | p156 |
| `Line` | bisector, fourpoint, fromfirstpoint, perpendicular, point, tangent, vertical | length | p133 |
| `Box` | aroundcurve, fitrailno, global, local, tangent, yes | 3point, cancel, height, undo, width | p233 |
| `Explode` | cagecontrols, mesh, polycurve, polysurface, text, undo | - | p85 |
| `History` | brokenhistorywarning, lock, no, print, record, update | - | p44 |
| `Polygon` | aroundcurve, edge, extensionlength, numsides, type, vertical | cancel, sides | p160 |
| `Rectangle` | arc, area, arebuilt, conic, corner, deformable | 3point, cancel, center, length, square, undo, vertical | p138 |
| `Spiral` | aroundcurve, flat, numpointsperturn, reversetwist, vertical | axis, cancel, endradius, height, startradius, turns, undo | p160 |
| `EditPtOn` | inplace, no, vertical, yes | - | p82 |
| `PointsOn` | copy, no, xaxis, yaxis | - | p83 |
| `Polyline` | center, close, no, normal | length, undo | p133 |
| `Move` | copy, no | - | p84 |
| `Shell` | directionconstraint, toboundary | cancel, undo | p246 |
| `Angle` | twoobjects | cancel, continuous, objects, ortho | p121 |
| `Distance` | unit | cancel, continuous, objects, ortho | p121 |
| `Ellipse` | pointcount | cancel, radiusx, radiusy, undo | p140 |
| `Ellipsoid` | rightclickcommand | aroundcurve, cancel, corner, diameter, fromfoci, undo | p233 |
| `Helix` | vertical | axis, cancel, height, radius, turns, undo | p161 |
| `Mirror` | vertical | - | p84 |
| `BooleanUnion` | - | cancel, undo | p223 |
| `Pipe` | - | cancel, cap, chainedges, diameter, shapeblending, thick, undo | p231 |
