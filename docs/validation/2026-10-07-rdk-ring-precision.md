# OM9-FILE-012 — RDK3 and native ring precision repair

Actual Rhino5 acceptance of the new exports remains **in_progress**. This repair
addresses the diagnosed RDK4 layout, native revolution UV conversion and declared
solid orientation loss. It does not certify full openNURBS or renderer support.

## Implemented and verified

- A SHA-bound generated copy of SetRDKDocumentInformation writes genuine RDK3
  for archive5/50 without embedded files: int3, UTF-8 byte length and exact XML,
  without the RDK4 resource-count trailer. Archive80 and resource-bearing SDK
  models retain RDK4. The original SDK source is unchanged.
- A pristine SDK2013 reader decodes the exact Unicode/XML of new V5/V50 fixtures
  and six fresh host exports. The resource fixture remains RDK4 with its bytes;
  direct host export and single/multiple-source preservation reject it before
  writing. Preflight occurs before dependency pruning, protecting destination
  bytes and the unchanged source snapshot. Resources are not silently discarded.
- Native ON_RevSurface uses an analytic OCC revolution with an analytic circle
  for arc profiles. Native angular and profile domains map affinely to OCC UV;
  rational trim poles map with the same transform. Transposed UV reverses each
  complete boundary wire and changes face parity. This fixes an independently
  reproduced trimmed arc area of45.7975778528 instead of1.5765232724.
- Twelve real revolution faces and12 synthetic line/arc/scaled/transposed cases
  pass native point evaluation1e-10 and rational trim correspondence. Eight
  complete partial BReps pass independently computed area within1e-6, interior
  classification and physical exclusion by distance to the trimmed face.
  Their200 independent native physical samples lie within1e-6 of the final BRep;
  export/reimport area agrees within the same synthetic1e-6 model tolerance.
  Full-period seam/singular cases are not claimed by those eight BRep tests.
- Analytic revolution surfaces retain finite native angular/profile domains.
  Assembled faces and their mapped pcurves convert together to finite NURBS for
  consistent host/export extrema. Kernel bounds depended on representation and
  both under/overestimated compared with earlier Rhino source evidence; matching
  import/reimport bounds alone does not prove source fidelity. The four real
  revolution BReps retain host/export bounds within the existing1e-3 tolerance.
  Independent original surface/trim boundary samples lie within the source model
  tolerance of the final canonical BRep (largest measured distance4.12e-5 mm).
  These samples do not certify unsampled extrema or close the cross-kernel bounds
  findings. The raw native-boundary distances are retained in the diagnostics.
- Declared inward solids remain inward. Export classifies the OCC infinite point
  and writes the known sign. Native affine transforms retain an original +/-1
  multiplied by sign(det(A)); unknown SDK orientation stays unknown. Four box
  shear/nonuniform/reflection cases agree with signed volume24*det(A).
- All four placements of the original diamond UUID were matched against actual
  earlier Rhino5 source bounds and signed volumes: two negative and two positive.
  Fixture and source-report SHA are recorded in the small source evidence JSON.
  Export/reimport preserves each volume within1e-8. This regression is source
  evidence, not a new Rhino application run.
- Prior synthetic reflected-copy assertions assumed outward normalization.
  They now require -576/-2880 from the independently known determinant(-2,3,4),
  preserving magnitude, bounds and tolerance checks. Source/export acceptance
  thresholds for the user rings were not widened.

## Final verification

Fresh native35/35, every process0. Fresh final-runtime FreeCAD1901/1901 in57
suites, plus347 fixed-ring and547 unmeshed-ring checks (894/894), every process0.
Both original fixture SHA values are unchanged;101/135 geometry objects retain
metadata, topology, bounds, signed volume/area and FCStd reopen within the existing
host checks. Runtime source/install script equality, two binary and two earlier
independent-reader hashes are guarded. Rust/App behavior was not changed; their
historical tests are separate evidence. Independent review identified the preflight
guard gap and prompted the transposed full-BRep regression; both are fixed.

Build: MSVC2022 x64, Ninja RelWithDebInfo, source H:/FreeCAD-src/build/om9-dev,
module build H:/FreeCAD-src/build/openmatrix9-preservation, runtime
H:/FreeCAD-src/build/3dm-preservation-sdk; dependencies from .pixi/envs/default/Library.
Commands: rtk proxy cmd /c build/rdk-ring-full-native.cmd; rtk proxy cmd /c
build/3dm-isolated-module.cmd; rtk proxy Python build/run-rdk-ring-user-fixtures.py
and build/run-rdk-ring-regressions.py. All57 suites were rerun after the final
finite-domain/NURBS change. Independent-copy signed expectations were corrected;
the original undeclared ON_BrepBox keeps its previous positive fallback.

Native RED evidence: rdk-red-LastTest.log, ring-orientation-red-LastTest.log,
revolution-uv-red-LastTest.log, rdk-resource-transposed-red-LastTest.log,
revolution-transposed-brep-red-LastTest.log, ring-instance-orientation-red-LastTest.log,
revolution-domain-bounds-red-LastTest.log. The new synthetic canonical-BRep area
assertions use the1e-6 construction/conversion tolerance (earlier roundtrip
difference6.51e-7); native analytic parameter/trim evaluation stays1e-10.
Original user source/export acceptance thresholds remain unchanged.

Local fresh retest pack: H:\FreeCAD-src\build\rhino5-retest-20261007-135206.
Its manifest links every copied archive to its fresh GUI result and SHA.
The V5 RDK3 legacy decode report is in evidence/rdk-legacy-decode.json.
Source checkpoint: H:\FreeCAD-src\build\checkpoints\rdk-ring-precision-20261007-135414.zip.
Read-back SHA verification covers every archived file; public checkout untouched.

## Pending actual application acceptance

The provisional comparison of earlier actual Rhino ORIGINAL measurements against
the fresh FreeCAD IMPORT matches all recorded area/volume thresholds for135 leaves.
It still reports17 bounding-box mismatches (maximum0.3785423 mm), retained in
rdk-ring-host-vs-rhino-source.json. These include prior exported/reimported bounds,
so the final finite wrapper alone does not explain them. This diagnostic is not a
new Rhino application test. Bounds and source fidelity remain open until Rhino
measures the fresh exported geometry; thresholds were not widened.

Rhino5 runs elevated and the previous Computer Use input could not reach it.
The fresh acceptance script must be started through Rhino's command UI. It retains
the Unicode-safe atomic JSON writer and identity-sorted bounded metric comparison.
The earlier8-file run is retained unchanged; its13 unmeshed findings are historical.
Do not mark those findings closed until new Rhino mass properties and source/export
comparison succeed, no newer-RDK dialogs are observed, and Rhino SaveAs exports
pass FreeCAD reimport. RDK3 decode does not prove that Rhino5 understands every
modern XML/render field. Embedded resources, plugin/history, renderer/display/edit,
all128 classes/16 components/6 document categories remain incomplete.

The diagnostic raw source/export same-UV fields in ring-precision-diagnostics.json
are not accuracy assertions after analytic-to-NURBS reparameterization. Only the
mapped import_surface_uv_deviation and independent area/classification tests are
used as the new conversion evidence.
