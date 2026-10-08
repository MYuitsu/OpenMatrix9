# Native reference copy/remap — 2026-10-08

Repeated imports of the same source archive allocated namespace UUID aliases
but left PolyEdge `m_object_id` pointing at the first namespace owner. Identical
geometry could hide this wrong graph in a point-only check. Native RED caught
the second root owner mismatch. FreeCAD RED separately showed that deleting a
current top-level referenced owner silently allowed restoration from snapshot.

The merge now rewrites only known owner UUID slots on detached deferred curve
trees after complete bounded preflight. Class, component index, all parameter
domains, reversal and native metadata remain exact. Missing/nil mappings, limits,
late invalid slots, live proxy pointers and ambiguous identity text fail before
mutation. Attribute identity text also refuses atomic export rather than retaining
a reference to a different namespace. Source bytes are immutable.

Current top-level owner deletion or geometry/placement edit is rejected before
staging. Owner metadata overlays follow reachable native dependencies; these
do not change the selected root set. Geometry edits still need a verified
topology/domain-preserving owner adapter. Selecting the canonical owner prevents
a duplicate metadata overlay. No full owner editing claim is made.

**18 independent frozen originals, 36 native namespace/copy cases: 36 passed.**
Seven negative controls and **2,611 native checks** passed. **18 FreeCAD lifecycle
cases /612 checks** passed repeated imports, copies, exact owner closure, current
metadata, delete/Undo/Redo, edited-owner refusal, FCStd reopen after source deletion
and protected public V5 output. Scope is internal SDK80 current-graph staging and
safe preservation. Original 12 standalone/6 mixed Rhino5 target failures remain
proven scoped incompatibilities; this repair does not convert them into V5 success.

Final installed regression: **45/45 native suites;34 actual FreeCAD reports /
5,343 checks**, including both ring files at unchanged0.001mm bounds and existing
area/volume limits. Whole tools42/42 passes. Sources, binary and reports are bound
in `native-reference-remap-20261008/binding.json` and the current-proof summary.
Previous44/33/4731 checkpoint and actual target failures remain immutable evidence.

Package2 remains open for complete seam/singular/correspondence applicability;
packages2–8 are unclosed. Total future batch count is unknown. Pinned SDK unchanged;
no primary-source integration or push. Full exchange remains incomplete.
