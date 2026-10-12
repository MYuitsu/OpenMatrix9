# RCORE-01 — Circle History source roles and fixed constraints, 2026-10-12T03:29:00+07:00

**Status: in_progress. RCORE-01 and Task2/3 remain open.**

Safe Rust validates source roles against unique bounded role indices. AroundCurve
requires exactly one explicitly bound Curve/Polycurve path. Tangent requires its
complete recorded roles; a Point cannot stand in for a curve role, and fixed/curve
point spans cannot overlap. Every fixed point must lie in the solved plane and on
its circle. Curve input points remain solver probes, not asserted final contacts.
The existing native Tangent tolerances are shared with FromFirst validation;
accepted roundoff is retained. No native solver or FreeCAD API was rewritten.

Owned replay input now labels individual roles. AroundCurve retains its original
probe(5,0,0) and separately records the native resolved anchor(10,0,0). It no longer
calls the old probe a current resolved point. Radius pick semantics are preserved.
Actual Runtime70 saved Circle014 confirms these distinct values and labels.

TDD evidence: role/fixed tests RED4/15→12/15; native roundoff and curve-versus-point
checks RED12/17→14/17; provenance RED15/19→16/19; overlapping fixed/curve constraint
review finding RED16/20→17/20. Existing three geometric correspondence failures
remain unchanged. Twenty reconstructed actual native replay payloads all pass.
Those requests are reconstructed from authored Runtime59 results, not intercepted
producer packets. Numeric perturbation tests are explicitly synthetic mutations.

**Focused Rust17/20 and full Rust933 passed/3 failed/0 ignored, exit101.**
The full run finishes129 suite summaries with no-fail-fast. Remaining failures:

- `history_native_source_around_fraction_must_match_current_plan`
- `history_native_source_tangent_points_must_correspond_to_owned_curve`
- `history_native_source_tangent_plan_must_bind_selected_source_geometry`

Standalone OpenMatrix9Gui build exits0 with the single existing SDK/cache. Matching
Runtime70 unchanged all-mode fixture109/109 plus independent cold reload62/62
pass in two fresh isolated native hosts, both actual exits0. Twenty stored results
retain exact bytes and independently reconstructed CAD on cold reopen, then remain
paired after source edits. This171-check scope does not certify independent source
binding, native atomic savepoints/lifetime, all families or full RCORE-01.
All1460 frozen candidate source/resource/test hashes match after build and native
runs; the inventory is not a compiler dependency certificate. An earlier fixture
invocation failed before host launch because the plugin build was still running;
its startup failure is retained and excluded. No prior artifact is overwritten.

The two changed production files are Rust. Circle native adapters, selected
FreeCADApp/Gui/host/probe and shared SDK fingerprints remain unchanged. Other
concurrent checkout work is preserved. The proposed native savepoint design still
awaits review; this checkpoint does not implement it or imply approval.

Next: Complete independent source/domain/frame/parameter correspondence for AroundCurve and Tangent across every supported analytic, rational/periodic NURBS, Bezier and derived support and selected Edge/whole source convention. Reuse existing native solvers and Rust basis/frame validators; do not replace full scope with a line-only gate. Preserve numeric/curve probe versus solved contact semantics: Tangent probes need not equal final contacts. Keep all three unchanged failing tests and extend family witnesses. The native savepoint/lifetime design is still pending review; no approval is inferred and no new native interface implemented. Continue original RCORE-01 adapters1–7/C01–C12/T01–T11, every enabled GeometryResult consumer, conversion/maps, budgets and lifecycle.

[Machine evidence](2026-10-12-rhino-core-circle-history-source-role-fixes.json).
