# Unmeshed ring runtime validation — 2026-10-06

Input: `tests/fixtures/3dm/Oval-twist-ring.3dm`, immutable SHA-256 `2ca463b90bf449e57bd53356ce1f357be77e56cdffbdf87d7a312dcc35d879fb`.

Command: `rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File build/test-unmeshed-ring.ps1`, using the installed FreeCAD SDK and `OM9_RING_FIXTURE=Oval-twist-ring.3dm` with `three_dm_ring_smoke.FCMacro`.

Result: FAILED at import after 3.11 seconds. Fixture hash check passed. Native reader reported an unsupported object; export/reimport and geometric comparisons did not run. Runner exit 1.

Independent openNURBS inventory (`build/inspect-ring.exe`, exit 0) identified 138 model geometry entries: 107 BReps, 9 block instances (`ON_InstanceRef`), 1 line curve, 17 NURBS curves and 4 polycurves; no mesh entries. Unsupported block indices: 19, 20, 22, 26, 38, 43, 47, 49, 62. Counts include definition geometry and should not be interpreted as visible expanded object counts.

Cause: current importer intentionally rejects block instances. This fixture cannot yet complete import/export. Next implementation must resolve instance definitions and transforms (including nested blocks) and validate CAD round-trip geometry, without silently dropping instances.

Reports: `H:/FreeCAD-src/build/validation/rhino5-3dm-20261006/unmeshed/results.json` and `inventory.json`. The original fixed fixture's passing evidence remains separate.
