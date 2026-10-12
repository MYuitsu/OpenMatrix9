# RCORE-01 — Owned derived curve evaluation, 2026-10-12T04:04:38+07:00

**Status: in_progress. RCORE-01 and Task2/3 remain open.**

Safe Rust now evaluates bounded Taylor coefficients Dk/k! from compact rational,
periodic NURBS, Bezier and all five analytic curve kinds. A generalized homogeneous
de Boor recurrence and series division retain absolute parameters and original
basis serialization. The internal order cap17 covers D1 through16 offset wrappers.
Derived trim/offset chains preserve every wrapper interval. Offset derivatives use
the retained tangent-cross-direction normal and bounded series normalization;
UV uses the clockwise normal. Singular normal, nonfinite arithmetic, excessive
order, outside trim and surface category requests return typed errors. No native
solver/API or caller transaction was rewritten.

Focused Rust **10 passed/1 failed**, full Rust **943 passed/4 failed/0 ignored**,
both exit101. Nine missing-method compile errors were reproduced before product
code; not a runtime assertion RED. All183 native queries on19 authored families
match Rust value/D1/D2 at component tolerance2e-9*max(abs(native),1). The14 original
non-offset families also reach D3 equality before the raw oracle fails at the first
OffsetBezier query. The raw test remains unchanged and enabled.

At OffsetBezier u0.1, Rust D3 is approximately[-13.5351,-0.557361,-1.86402];
native D3 is[-353.149,268.106,-1.86402]. A separate native diagnostic evaluates
central differences of native D2 at15 interior points. The discrepancy also exists
inside native D2/D3, so it is not attributed to Rust alone. An independent mpmath
oracle differentiates rational Bernstein/conic and normalized tangent definitions
at70 decimal digits. Rust D3 matches all45 offset queries, while native differs
on36; circular offset matches9. This independent oracle is sampled evidence,
not a rigorous global error-bound certificate or a native parity certificate.

The [primary OCCT8.0.1 offset evaluator source](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V8_0_1/src/ModelingData/TKG3d/Geom/Geom_OffsetCurveUtils.pxx)
was reviewed during diagnosis. Its published D3 expression reproduces all36
measured non-circular native derivatives using independently differentiated basis
values. SDK Standard_Version.hxx also records8.0.1. This identifies the discrepant
expression; exact native compiler-input identity remains unproven. No dependency
patch or FreeCAD/kernel rebuild was performed.
**Ruling:** retain mathematical derivatives and the conflicting native gate.
Copying a discrepant D3 to obtain green would not establish source correspondence.
Higher-order native parity remains open.

Literal tests additionally cover16 nested circular offsets, unchanged serialization,
trim rejection, clockwise UV line normal, category rejection and singular normal.
These literals do not certify16-deep native geometry or all UV/trim families.
Native OffsetTrim and nested-direction constructors flatten their captured bases;
the oracle checks those actual captured representations rather than inventing
an unobserved retained trim/nested chain.

Runtime86 standalone plugin build exits0, reusing the existing shared SDK/cache.
Three fresh hosts exit0: raw jet oracle58/58, History109/109 and cold reopen62/62,
total229/229. Raw oracle58 confirms capture/finiteness/source preservation; it
does not compare Rust derivatives or turn the Rust D3 failure green. Native cases
are identical to the original Runtime79 oracle. Circle History production remains
unchanged and independent source binding still has its original three failures.
Full candidate inventory match=True; changes=[]; all frozen
production inputs match, not a compiler dependency certificate. Shared SDK and
selected FreeCADApp/Gui/host/probe fingerprints remain unchanged.

Evaluator integration, complete edge/world/parameter/contact correspondence,
conversion/maps, complete budgets, all enabled consumers and native atomic/lifetime
remain open. The proposed native savepoint design still awaits review.

Next: Connect owned source value/D1 to selected edge/full-object/domain/frame/parameter admission across all supported families, preserving AroundCurve probe versus current anchor and Tangent probe versus final contact. Retain raw native D3 discrepancy test and investigate exact installed OCCT behavior separately; sampled high-precision derivatives are not a global bound. Keep proposed native savepoint interface review pending. Continue all original adapters1–7/C01–C12/T01–T11, all enabled GeometryResult consumers, conversion/maps, budgets, persistence and lifecycle.

[Machine evidence](2026-10-12-rhino-core-owned-derived-curve-evaluation.json).
