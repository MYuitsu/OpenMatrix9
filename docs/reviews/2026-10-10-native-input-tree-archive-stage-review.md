# Independent review — archive/member input-tree preflight

**Verdict: approved for this bounded RCORE-01.T07 increment.** The gate
validates the original supported curve, BRep, or surface before transformed
block-member duplication, surface `BrepForm`, and the tagged signed-placement
solid-orientation query. Existing converters remain the implementation path.

The first review pass found a test gap: the archive tests checked only that the
read failed and a hook stayed at zero, so an earlier failure could satisfy the
assertion. The fixture was tightened to require the native NURBS rejection
reason. A tagged instance with an invalid BRep support surface now exercises the
signed-placement branch and asserts its orientation hook remains unentered. The
follow-up reviewer confirmed those assertions exercise the intended ordering.

Recorded validation is 67/67 native checks, six related archive/block CTests,
and 21/21 checks in a fresh shared FreeCAD host process with exit code 0. The
reviewer inspected the final source and result records but did not rebuild or
rerun tests. Exact fingerprints and outputs are in
[`2026-10-10-rhino-core-native-archive-stage.json`](../validation/2026-10-10-rhino-core-native-archive-stage.json).

This does not cover raw archive decoding or allocations inside
`ONX_Model::Read`, `ThreeDmExplode`, cage paths, process-wide allocation,
aggregate worker memory, or all P0 requirements. Existing Line/Polyline and
`Part.makePolygon` behavior is reused.
