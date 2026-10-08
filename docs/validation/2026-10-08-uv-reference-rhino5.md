# Actual Rhino5 C2 UV-reference incompatibility — 2026-10-08

Rhino5.14 GUI read all8 original fixtures:0 passed,8 failed. All8 standalone
owner curves were valid, with17 independent line/quadratic samples matching at
1e-12mm. Each CurveOnSurface root was invalid and returned ON_UNSET_VALUE in
all3 coordinates at all17 points. SelBadObjects selected1 curve per file.
All8 SaveAs commands returned failure; the attempt directory contains0 saved
3DM files. This is not a completed write/read roundtrip, and there is no saved
native decode or FreeCAD reimport to claim.

Actual raw validity/coordinates, per-case SaveAs traceback, UUID/domain, probe
and original-oracle hashes are frozen. Inputs remain byte-identical. The probe
finished the8-case matrix without numeric overflow; failed commands were
reported separately. Negative target evidence is not a successful exchange.

Pinned native acceptance **507 checks passed**: exact source class/UV/surface/
metadata graph, independent physical composition after binding each owner,
actual owner samples and UnsetPoint failures, and atomic selectedV5 refusal.
The previous new implementation verifies namespace/copy/lifetime serialization
with895 native and272 actual FreeCAD checks. PublicV5 remains refused; native
source and dependencies remain in the project. No conversion to an owning
curve, mesh, metadata omission or tolerance increase was performed.

Final regression **49/49 native suites;37 installed FreeCAD reports/5,696
checks**. Production source/module unchanged since the48-suite binary check;
the additional test only verifies the frozen actual target report. The two
rings remain green at0.001mm bounds and existing area/volume thresholds.
Tools42/42 passed. SDK pin eb92af3ba1806b0a34a99aba0d3bda83e3d46083 unchanged.

The scoped8-case batch is closed as verified target incompatibility with
verified source preservation/refusal. This does not prove every possible C2
reference unsupported.0 prepared batches waiting;7 packages2–8 unclosed,
total future batch count unknown. Next native BRep seam/singular applicability
and remaining correspondence/owner-edit coverage. Full exchange still incomplete;
no primary integration or push. See uv-reference-remap-20261008/actual-source-binding.json.
