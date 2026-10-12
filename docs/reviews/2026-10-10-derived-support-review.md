# Derived supporting geometry review — 2026-10-10

Fresh read-only reviewer examined the new Rust chain/ABI, native reader,
derived form integration and test plan. Existing uncommitted increments were
outside this focused review. Reviewer performed no builds or runtime tests.

P2: surface trim nodes rejected infinity flags even though native rectangular
trim can restrict only one axis. The shared SDK and FreeCAD Python constructor
confirm this case. `inner_surface_trim_can_keep_the_other_native_axis_unbounded`
failed with Error::Parameters before the fix; after the fix the whole derived
Rust test target passes. Domain/sign/canonical masked-slot checks remain in place.
The final real-host fixture also verifies an offset with inner U-only extrusion
trim retaining finite U and unbounded V, plus independent native samples.

Two coverage uncertainties were raised and addressed: malformed category/unit
direction chains now reject with4002 before a selected NURBS leaf's unusable
pointer envelope is touched, with unchanged output; weighted native NURBS
extrusion and NURBS/Bezier offset surfaces now verify original weights and sampled
geometry. Rust ABI tests cover every analytic/NURBS/Bezier leaf category.

No other actionable issue was reported. Fix evidence and the full Rust/host
regression outcomes belong to the final
[validation record](../validation/2026-10-10-rhino-core-derived-support.md).
This review does not establish full RCORE-01 completion or global native safety.
