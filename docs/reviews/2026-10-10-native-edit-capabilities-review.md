# Native Edit capability review — 2026-10-10

One fresh read-only source reviewer traced Rust policy/ABI, declaration gating,
native owner preflight, mesh copying and special Explode/Trim boundaries. No host,
build or source mutation was performed by the reviewer.

Important finding: special Explode returned before ordinary native preflight.
App::Link leaves reached getSubObject/transform and cage leaves Shape.copy
without basis/mesh-budget validation. Controller reproduction failed4/10 checks
on the previous plugin, including source/Undo mutation for the invalid block
basis. Shared native leaf preflight now precedes extraction/copy; the fixture
also proves scalar block curves retain their supported world placement.

Minor finding: Trim option callbacks split without the shared recheck used by
buildEdit. Existing Rust setters already reject invalid combinations. The
controller added the same common capability check before both Trim split paths;
existing real Trim option regressions validate behavior. No test-only bypass or
new production knob was added to fabricate invalid controller state.

Controller follow-up after the review: the preserved3DM regression exposed
an affine preview missing its actual native converter tolerance. Focused RED
failed2/2 without fixture errors. Native staging now carries the already resolved
expanded.tolerance; strict Rust binding validation remains unchanged. Five host
checks cover legitimate affine admission/result evidence and atomic Undo/Redo.
This later one-field correction was controller-inspected and runtime-tested.

The reviewer cleared a suspected Mesh getter-copy issue: PropertyMeshKernel
returns an immutable borrowed wrapper. Native property owner checks, counts,
segments and bytes precede actual copies. Corrections above were controller-
inspected and tested after the fresh review; no second review was requested.

Final matching evidence:679 Rust tests,4456 host checks,36 actual exits0.
All original remaining obligations stay open: aggregate budgets, complete native
Edit GeometryResult/conversion/maps, other consumers and uniform typed native
exceptions. Constraint tags are requirements, not proof of fulfilled reports.
