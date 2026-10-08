# Working geometry public integration — 2026-10-08

Independent modeling import/export is integrated with the public Curve/Spline,
Surface/Edit/Solid commands and plugin-owned archive/Hatch storage. FreeCAD core
remains at the original source tree; matching SDK/ABI is required.

Import3dm offers **Working geometry (continue modeling)**. It creates independent
CAD, mesh and point-cloud objects, resolves nested placement/units and rejects
unsupported model-space geometry before document mutation. Page-space data,
history, render, materials, textures, lights, layouts and userdata are omitted
only in this explicitly selected workflow. Preserve mode remains the default.

Working export uses current selected geometry, including edits/new objects and
placed links. Ordinary App::Part containers expand every member; a
PartDesign::Body exports its final shape. Invalid/retained geometry and ambiguous
selection refuse before replacing the destination. Native V5 staging is
validated before atomic replacement.

Review found that App::Part could silently omit mesh/cloud children. The actual
FreeCAD regression failed before the fix and passed all eight checks afterward:
three mixed records, parent transforms, retained-child refusal with unchanged
destination, and Body final-shape behavior. A second review found no remaining
Critical/Important implementation blockers.

Fresh integration checks: **15 FreeCAD suites / 464 checks**, **8 native suites**
linked to the public module's actual OM9ThreeDm/openNURBS libraries,
**110 Rust tests / 27 suites**, and **42 Python tests (one optional skip)**.
Tests include working import/edit/export, mm/cm, nested workers/failure cleanup,
actual dialogs, source-deleted FCStd, preservation workers, legacy migration,
both private ring regressions, wireframe and the existing CAD command groups.
Private model bytes and commercial reference assets are not published.

Rhino5 application acceptance for this merged binary is pending: four generated
files/seven objects prepared for actual Open/SaveAs. Native/FreeCAD checks do not
replace that evidence. Existing lint/format debt is outside this integration;
the test results do not claim a clean full Clippy/format gate.

Machine-readable evidence and local artifact paths:
[modeling-public-checks-2026-10-08.json](../modeling-public-checks-2026-10-08.json).
There are still **seven open full-exchange packages**. The total future test
batch count is unknown; this checkpoint does not claim full openNURBS support.

To run the portable Phase1 test gate after a matching build, provide absolute
FreeCADExe, DependencyPrefix and NativeTestDirectory to
`tests/run_modeling_phase.ps1`. The runtime's `modeling-build.json` must bind
the actual source/runtime hashes. Run native ModelingExchange first to generate
`modeling-fixtures`, then ThreeDmImportWorkerFixture with a destination
`<NativeTestDirectory>/import-worker-fixture.3dm`; the failure worker executable
must also be present there. Tests write only isolated reports/generated models.
Optional performance tests require the local baseline and supplied ring; their
historical speed measurements are not rerun or claimed for this merged build.
