# Remaining packages2/3 applicability — 2026-10-08

Current verified checkpoint:51 native suites,39 installed FreeCAD reports,
5806 checks. Latest BRep seam/pole+edit actual Rhino batch4/4 closed;
0 prepared pending batches,7 plan packages open,total future count unknown.
This is a gap inventory,not a prepared fixture batch or a completion percentage.

| Remaining slice | Current evidence | Work still required |
|---|---|---|
| PolyEdge component owners on periodic seams/poles | Native sphere/cylinder physical seam mapping verified; standalone12/mixed6/UV8 have scoped actual V5 incompatibility | Explicit component-reference fixtures binding both seam trim sides, edge proxy directions and domains; distinguish singular trims without3D edges; source/copy/FCStd graph evidence |
| Current owner geometric edits | Current metadata/copy namespaces verified; deleted/changed owners have safe export guards | Define a topology/domain-preserving correspondence adapter or verify preservation-only applicability; never reuse stale component indices after CAD replacement |
| General native edge/trim correspondence |15 bounded interior projections plus endpoints; diagnostics explicitly say no certified global correspondence | Enumerate supported bases/domains and adversarial interior/ambiguity/failure controls; do not promote sampled evidence to a global claim |
| Native CurveOnSurface children/nesting/transforms | Scoped ordinary and nested owning UV profiles have actual target evidence | Remaining child-class/parameter maps currently rejected by safeGeometry/verifiedNestedUVProfile; preservation and target applicability require individual evidence |
| BRep graph identity | Full topology slot/domain checks,exact3D/surface payloads and common C2 basis checks for the latest4 files | Audit use of BRep DataCRC as identity: pinned ON_Brep::DataCRC covers vertices/edges/faces and omits C2. Require separate decoded child/property checks where complete native graph equality is claimed |
| BRep multi-shell/reflection/unknown winding | Existing planar/curved shell,reflection and+2 reports remain valid; latest analytic sphere/cylinder positive winding verified | Close the full applicability matrix,including non-CAD-equivalent complements and current native graph/source preservation; a positive solid volume does not prove all shell forests |

Source gates reviewed:Gui/ThreeDmNativeReferences.cpp(trimDomainAnalysis and
component binding),Gui/ThreeDmCurveOnSurface.cpp(safeGeometry and nested-profile
preflight),ThreeDmArchiveState.py(current-owner policy),Gui/ThreeDmBrep.cpp
(seam/singular conversion and bounded cap synchronization). The pinned SDK
opennurbs_brep.cpp:1464 defines the restricted BRep DataCRC implementation.
These limits remain explicit; the latest successful batch does not close either
package. Opaque actual GUI plugin-table export remains a package7 dependency
gap while complete source bytes are retained in FCStd.
