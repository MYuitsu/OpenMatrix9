# RCORE-01 — Owned curve evaluation, 2026-10-12T03:51:03+07:00

**Status: in_progress. The complete RCORE-01 objective and Task2/3 remain open.**

The existing validated owned basis now evaluates native-local position and D1 at
absolute OCCT parameters in safe Rust. Compact knots, weights, periodicity and
serialization are retained. Dual homogeneous de Boor evaluation uses degree+1
temporary control values, local weight scaling and the native left derivative at
the terminal periodic seam. Bezier delegates to the validated single-span basis.
Analytic Line/Circle/Ellipse/Hyperbola/Parabola evaluate using their retained axes,
origin and native parameters. Surface kinds are explicitly distinct.

Six focused tests pass. Actual OCCT valueAt/derivative1At expectations cover 138
queries across fourteen authored curve families: rational arc, unclamped knots,
repeated interior knots, periodic linear/quadratic/rational/repeated seam, native
rational circle, rational Bezier and all five analytic kinds. Each component is
compared at tolerance 2e-9*max(abs(native),1). Additional guards cover finite and
absolute/restricted parameters, equal 1e-307 weights, indirect UV axes and exact
preservation of serialized basis/analytic metadata. Expected values were produced
by OCCT on Runtime70 before the Rust evaluator, not by the implementation under test.

Clean missing-method compile RED73 and RED75 precede basis and analytic methods.
The initial integration-test parser access failure72 and syntax failure75-green
are retained and excluded. Full Rust finishes **939 passed/3 failed/0 ignored**,
exit101. The same three Circle History source geometry/parameter tests remain
failed; the evaluator is **not yet connected** to their admission path.

Runtime79 standalone OpenMatrix9Gui build exits0 using the existing SDK/cache.
Three distinct isolated hosts exit0: unchanged native analytic oracle43/43,
Circle History all-mode109/109 and independent cold reopen62/62. Total214/214.
Native oracle observations exactly match the prior fourteen-family fixture.
These host tests exercise capture and existing commands, not a new evaluator FFI.
Rust unit tests perform the independent owned evaluation comparisons.
The 1467-file frozen inventory differs only in the concurrent
Gumball fixture tests/gumball_associative_scale_smoke.FCMacro. All frozen production
inputs match after terminal build/native; full inventory match is false. This is
not a compiler dependency certificate.
Shared SDK and selected FreeCADApp/Gui/host/probe are unchanged.

No global deviation, complete parameter/topology mapping or exactness claim follows
from sampled queries. Trimmed/offset support chains and higher derivatives remain
to implement, along with selected-edge/world-frame and source/contact integration.
Extreme asymmetric weights, full aggregate allocation budgets and maximum-size
families remain unverified. The new prefix-multiplicity cache also consumes owned
memory and has not been certified against a complete aggregate budget. Native
atomic/lifetime failures remain; the proposed native interface awaits design review.

Next: Bind the owned evaluator to full selected-source/domain/frame/parameter correspondence, preserving existing native AroundCurve and Tangent probe/contact conventions. Complete trimmed/offset support chains and required derivatives, then verify unchanged source RED cases and native families. Keep native savepoint design review pending and continue all original RCORE-01 adapters1–7/C01–C12/T01–T11, enabled consumers, conversion/maps, budgets, persistence and lifecycle.

[Machine evidence](2026-10-12-rhino-core-owned-curve-evaluation.json).
