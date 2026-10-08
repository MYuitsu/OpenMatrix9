# Standalone PolyEdge probe numerical overflow — 2026-10-08

Actual Rhino5 runs091017 and091018 open the first fixture successfully but stop
while squaring sample-coordinate differences. Reports contain zero completed
cases. Both failed JSON files, original script and original hash preparation are
retained in `rhino5-standalone-reference-20261008/pre-overflow/`. This is a probe
failure, not proof that every standalone class is incompatible with V5. Raw
Rhino coordinates were not captured by that revision; an unresolved reference
returning UnsetPoint is a hypothesis until the revised diagnostic records it.

Numerical regression reproduces OverflowError on finite1e200 and SDK Unset
coordinates. Original comparison also ignores Point3d.IsValid and emits invalid
JSON for NaN/Infinity. Revised template scales coordinate differences before
the Euclidean norm; records Point3d validity, preserves finite Unset values,
represents nonfinite values explicitly, and rejects them. Per-case failures
retain partial measurements and continue later fixtures. Source checksum
preflight occurs before opening any document. Thresholds remain1e-9mm for17
curve samples and1e-6mm3 for BRep volume; no mesh/drop/tolerance relaxation.

Seven numerical/control-flow tests pass after observed RED. Installed Rhino5
IronPython2.7 engine separately verifies numerical helpers and strict JSON;
this executes no Rhino geometry/document API. Metadata verifies Point3d.IsValid
and XYZ against installed RhinoCommon; names/arity are not geometry evidence.
Full tools suite41/42 passes: existing unrelated UI routing test
`test_skill_reference_index.ReferenceIndexTests.test_ui_route_does_not_require_full_ghidra_export`
fails because the isolated development copy lacks `ref/MainMenu.ini`. Missing
reference is recorded separately; no reference is invented or test weakened.

Only diagnostic Python/template/tests/docs changed. Installed production binary
and all12 source fixture/oracle hashes remain unchanged. Exact retry script/hash,
preflight, RED/GREEN logs and numerical-engine output are frozen. Actual Rhino5
full retry remains pending; support must follow decoded native saved-class,
owner/graph and geometric evidence. Full exchange remains incomplete.
