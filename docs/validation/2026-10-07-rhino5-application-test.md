# OM9-FILE-012 — actual Rhino 5 application evidence, 2026-10-07

Overall acceptance remains **in_progress**. Actual Rhino 5.14.522.8390 opened,
saved as V5 and reopened all eight copied test archives. Native geometry validity
reported zero invalid top-level objects and zero invalid expanded leaves in every
case. This does not establish full surface accuracy or rendering compatibility.

## Verified application and host lifecycle

- The two fixed archives contain101 expanded geometry leaves each; the two
  unmeshed ring archives contain135 each. Four small exported samples contain
  one object each: box, polyline, mesh and edited hidden green box.
- Rhino open/save/reopen inventories agree after sorting by identity. The
  unmeshed original has two computed volume deltas of4.44e-16, within64 scaled
  double epsilon. No topology, attributes or layer differences were waived.
- Six exported archives were imported into the unchanged FreeCAD SDK runtime
  after Rhino SaveAs and saved/reopened as FCStd:25/25 checks, process0. Counts,
  CAD validity/topology, measured geometry, mesh counts, placement, labels,
  layer/color/visibility/OM9Locked metadata and input SHA match their exported
  baselines. This compares the FreeCAD exports before/after Rhino; it does not
  certify equivalence to every original source field.
- The fixed original vs its geometry export passes the bounded comparison.
  Both rings match expanded counts/types/bounds within0.001mm and recorded
  faces/solid/naked/nonmanifold edge counts. Blocks are flattened in these
  geometry exports; preservation-mode native graphs have not been tested here.

## Open RDK/render compatibility issue

The user observed repeated Rhino dialogs: “This document was created with a
more recent version of RDK”. The exact per-file warning count was not captured.
The modern SDK's ONX_ModelPrivate::SetRDKDocumentInformation writes version4
unconditionally, even for archive_3dm_version5. Its comment states that4 adds
embedded files to the version3 UTF-8 XML layout. The SDK2013 reader accepts
document RDK versions1 and3, not4. This is strong causal evidence for the warning,
not an actual application proof that changing one integer fixes all RDK content.

RDK controls rendering content such as materials, environments and textures.
Geometry validity and reopening do not prove that Rhino5 understands every
render setting or resource. RDK warning removal, XML/content compatibility,
embedded resources and visual/material acceptance remain unverified. No SDK
source was modified, table stripped, warning suppressed or payload relabeled.

## Open unmeshed ring conversion findings

Original-source vs exported-file comparison passes40/41 assertions but the
area/volume assertion fails for13 of135 matched leaves. Six exported five-face
Breps return no area or volume from Rhino mass-property computation despite
passing IsValidWithLog and being solid. Five other five-face Breps exceed the
declared area threshold max(0.001mm²,0.01% source area). Two101-face diamond
instances change signed volume from-1.257183756390893 to+1.2571837563848005;
magnitude agrees but orientation is not identical. Full conversion acceptance
is withheld. These are findings to diagnose, not proof of a particular kernel
bug or grounds to loosen the area/volume comparison.

The original analyzer also had two inappropriate sample expectations. Rhino
mesh bounding box XMin is129.99998474121094 rather than130; the corrected
bounding-box check uses the already documented0.001mm threshold. Independent
FreeCAD reimport confirms mesh XMin130 before/after Rhino. Native hidden/locked
attributes have Mode=Hidden; OpenMatrix9.Locked user-string metadata retains
the lock on FreeCAD reimport. The test now distinguishes native Hidden mode
from host lock metadata. No unobserved Rhino UI selectability claim is made.
The original36/39 failed analysis and analyzer are preserved with hashes.

## Evidence and scope

Local pack: H:/FreeCAD-src/build/rhino5-manual-20261007-120314.
Actual application run: application-test-20261007-122618/results.json,
command-history.txt, eight resaved3dm archives and screenshots. Raw report is
unchanged: completed=true, ok=false due to enumeration/exact-float comparison.
validated-lifecycle.json derives identity-sorted bounded metric comparison from
that real run; it is not another application run. analysis.json remains ok=false.
freecad-reimport-summary.json links the fresh25-check host result and runtime
SHA guards. evidence-index.json fingerprints these reports and source paths.

The failed first application's truncated JSON, user Check log and IronPython
Unicode red/green proof remain separately retained. The Check log has no exact
file identity; IsValidWithLog does not replace every advanced mesh Check test.
Historical native30/30 and GUI1901/1901 evidence remains historical; no new full
regression was run for these test/report changes. Full128 native classes,
16 component and6 document categories, preserved graphs, Hatch/CurveOnSurface/
PolyEdge rendering/editing, plugin/history/resources and final public integration
remain incomplete. Next: isolate RDK serialization from resource/XML migration,
then diagnose the13 unmeshed mass-property/orientation findings using retained
native source vs converted geometry and independent surface/trim comparisons.

## Additional user-provided command history

The pasted history from attachment4ec1b53b-cc5d-47d2-8a50-bc05292442c4 confirms
one successful write and one successful reopen for each of the eight outputs
in application-test-20261007-122618; their hashes still match the raw report.
The earlier121526 attempt stopped at the strict inventory comparison. The final
"file lifecycle ok: False" matches the unchanged v2 raw report, whose enumeration
and two computed-volume roundoff differences were diagnosed separately.

"Unable to zoom - no objects are visible" occurs for the deliberately hidden
green sample; recorded native visibility is false. This is expected test state,
not evidence of missing geometry. SaveAs prompts show Version=5,
GeometryOnly=No, SaveTextures=No and SavePlugInData=Yes. The texture option makes
this run unsuitable as proof of embedded texture/resource retention. No RDK
dialog text appears in this command history; the separately supplied warning
image remains the evidence for that warning. Clayoo/V-Ray/Rhino Render startup
lines do not, by themselves, identify the cause of the RDK warning.

Exact user log and machine-readable comparison:
pack/evidence/user-rhino-command-history.txt and
pack/evidence/user-rhino-command-history-summary.json.
