# Rhino 5 ring bounds diagnosis — 2026-10-07

Status: verified bbox containment defect; supplementary numerical verification passed. Full openNURBS remains partial.

## Independent actual Rhino evidence

Rhino5.14.522.8390 readonly File3dm diagnostic completed True. All24 points
are Interior according to pristine Rhino faces and PointAt matches each native
point exactly. The two mirrored original boxes exclude confirmed points by
**0.002185702223 mm in X** and **0.001043789242 mm in Z**. This disproves
containment by the source GetBoundingBox(True); it is not evidence of a
conversion displacement of that size. The exported box is also approximate.

Native Newton/OCCT candidates alone were not independent: SameParameter may
repair candidate pcurves. The actual Rhino IsPointOnFace/PointAt checks provide
the independent contained-point evidence. Candidates and meshes do not prove
exhaustive global extrema. No geometry cropping or tolerance relaxation was made.

## Bounded measurement correction

Supplementary empirical geometry analysis **50/50 process0**. Raw bbox analysis remains **49/50 process1**, retained verbatim.

Both fine-mesh source/export bounds are identical in the actual supplementary
run. Each coarse/fine bound converges within0.000003606081 mm. The maximum
cross-projection distance is0.000000272893 mm; the combined empirical screening
allowance is0.000011627767 mm. These values characterize the measured samples.

Only raw-failed BRep pairs are selected from the existing acceptance report.
There is no UUID exception or automatic waiver. Compare diagnostic mesh bounds
at tolerances1e-5/1e-6, require convergence<=1e-5 mm, project six mesh extrema
onto trimmed BReps, check cross distances in both directions and retain all
independent Interior points. An omitted known point refuses. The 0.001 mm
threshold stays unchanged and includes an empirical screening allowance.

Rhino MeshingParameters.Tolerance constrains edge-center deviation; that
allowance is not a certified global extrema error bound. This supplementary
result is practical numerical verification for these fixtures, not a proof
for arbitrary surfaces. The original raw49/50 result remains accessible.

Twelve unit tests pass, including rejection of a fully shifted0.002 mm target,
Exterior/missing/nonfinite points, missing levels, nonconvergence, changed or
invalid geometry, projection/cross-distance failures and omitted known points.
Actual installed Rhino5 IronPython verifies20 checksum reads, CLR Single mesh
coordinate normalization/JSON roundtrip and generated-script syntax.

## Script defects and preserved evidence

The first diagnostic measured geometry but failed at a temporary file stream
in a checksum generator. Explicit stream lifetime repaired this and the actual
Rhino rerun passed. The supplementary script then measured the first pair but
failed JSON serialization of CLR Single mesh coordinates. That exact exception
was reproduced in installed IronPython; coordinates are now Python floats and
the installed-engine regression passes. Neither script defect implies invalid
CAD geometry. Partial RED reports are retained separately.

Evidence directory: `H:/FreeCAD-src/build/ring-bounds-diagnostic`.
Pristine witness report: `results-witness-green.json`; raw failed diagnostic:
`results-hash-red.json`; Single/JSON partial report:
`measurement-results-json-single-red.json`; supplementary report:
`measurement-results.json`; separate analysis: `corrected-analysis.json`.
Raw acceptance: `H:/FreeCAD-src/build/rhino5-retest-20261007-160440/application-test-20261007-160529/analysis.json`.

Production converter/binary and original fixtures were not changed by this
bounds diagnosis. Earlier native35/35, GUI1910/1910, rings894/894 and eight
saved-output reimports33/33 remain the existing baseline, not newly rerun tests.

Official API references:
- [Brep.ClosestPoint since5.0](https://mcneel.github.io/rhinocommon-api-docs/api/RhinoCommon/html/M_Rhino_Geometry_Brep_ClosestPoint.htm)
- [Trim-aware closest-point behavior](https://mcneel.github.io/rhinocommon-api-docs/api/RhinoCommon/html/M_Rhino_Geometry_Brep_ClosestPoint_1.htm)
- [Meshing tolerance scope since5.0](https://mcneel.github.io/rhinocommon-api-docs/api/RhinoCommon/html/P_Rhino_Geometry_MeshingParameters_Tolerance.htm)
