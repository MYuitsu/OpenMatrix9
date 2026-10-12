# Native Surface GeometryResult review

Independent read-only reviewer: `/root/surface_result_review`.

P1: Part::Feature reinstates Placement during recompute, so serializing the world
solver result before Shape publication could persist a master different from
the actual output. Native RED4 reproduced two failures with separate source and
output groups and a nonidentity output Placement. Fix reuses FreeCAD's rigid copy
transform to convert world geometry into the feature frame, keeps the existing root
Placement, and serializes that exact private copy. The original four checks pass.

P2: preparing evidence using Loft/Sweep before native unique naming could retain
the wrong identity. Native command RED6 reproduced one failure with an existing
Loft and an actual new Loft001. Resolve getUniqueObjectName before prepare, then
create and validate that name in both publication paths. Original six checks pass.

Reviewer re-read both repairs and found no additional critical/important issue.
Reviewer performed no host launch/build; coordinator evidence is in the receipt.
An earlier identity fixture used unavailable Part.makeVertex; that fixture error
is preserved separately and is not counted as a product RED.

The initial frame repair reused Builder's generalized transform, which converted
the analytic cylinder into a BSplineSurface and failed the unchanged command area
regression. Native analytic RED8 (identity and nonidentity output frames) recorded
151.20111352201684 instead of 48*pi and the changed native surface type. The final
repair uses TopoShape::transformShape(...,true), the existing rigid copy path, and
retains original analytic type/area, snapshot pairing and source-world center.

Full original RCORE-01 conversion/maps/all-family lifecycle remains open.
