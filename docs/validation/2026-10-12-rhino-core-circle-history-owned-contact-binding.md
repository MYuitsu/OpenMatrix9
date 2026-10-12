# RCORE-01 — Circle History owned contact binding, 2026-10-12T04:31:22+07:00

**Status: in_progress. RCORE-01 and Task2/3 remain open.**

The existing planar kernel and Rust spatial Tangent solvers are reused. The small
C++ adapter now retains their solved contact fractions after exact native checks;
it owns native handles only during synchronous evaluation. Circle History passes
plain fractions indexed by current constraint point, null for fixed points. Rust
independently evaluates each selected owned curve at that absolute edge parameter
with native_location applied once, then checks point on circle/plane, unit tangent
in plane, tangent perpendicular to radius and unchanged FromFirstPoint contact.
Recorded probe fractions remain distinct from solved contact fractions. Radius
and vertical constraints continue to use the existing replay contract.

TDD contact witness **1 passed/2 failed → 5 passed/0 failed**. The previously
accepted center(5,5),radius4 with correct probes on x=1/y=0 is now rejected.
Tests reject wrong/missing fractions, numeric Point contact metadata, wrong tangent
plane/radial direction and changed first contact. An additional183-query test on19
genuine native-local curve families constructs synthetic tangent circles from the
native value/D1 oracle. Expectations do not come from the source evaluator. This
tests generic admission, not live native root/family behavior. Existing source
cohort remains23/23. Full Rust **958 passed/1 failed/0 ignored**, cargo exit101;
only the unchanged raw native offset D3 discrepancy remains failed.

Runtime93 standalone plugin build exits0, sharing the existing SDK and cache.
Native all-mode replay109/109 and independent cold reopen62/62 pass. A new wrapper
reuses the existing nonplanar solver/History macro and independently checks real
native contact fractions/D0/tangent at source edits, placement Undo/Redo and save
reopen:65/65. Total **236/236**, three actual host exits0. The spatial archive is
reopened inside its host; independent cold spatial contact proof is still open.
Shared SDK/App/Gui/host/probe fingerprints and existing replay solver are unchanged.
No new FreeCAD journal/lifetime interface or root/dependency rebuild was performed.

Fresh reconstructed native fixtures retain actual fractions[.6,.625] for two
lines and[.6,null] for fixed-first/Point. Old Runtime59 fixtures remain untouched.
All20 recipes/current points/resolved plans/roles/probe/path parameters match
their original cases exactly. New request identities/stamps are genuine to the
fresh native run; replacement sources are captured in that same fixture. Packets
are reconstructed from stored owned results, not intercepted C++→Rust requests.
Missing current contact metadata is rejected; old stored results retain their
bytes until current recompute captures complete evidence.

Prebuild1475 candidate inputs match at build completion. Subsequent changes are
current tests and fresh native fixture data only; all production inputs match.
Full Rust captures1483 inputs; the later new spatial macro is its sole addition.
Neither inventory is a compiler dependency certificate.

Remaining requirements:

- Full original RCORE-01 remains open: stable GeometryRef identity/revision/representation/world, complete owned geometry/topology/attributes/units, adapters1–7, C01–C12, T01–T11, every enabled GeometryResult consumer, and independent read/edit/retain/export of all supported families.
- Current Circle History Tangent source fractions/contact/plane/radius/unit tangent/FromFirst admission is implemented for every decoded supported 3D curve category. Initial ConstructCurve Tangent command publication still needs the same complete independent contact/source contract. Other History modes need independent point/Fit/direction/radius-endpoint source-to-recipe correspondence; existing live native physical results are not a general ownership certificate.
- Raw native offset D3 discrepancy remains enabled RED: independent mathematical derivatives match45 queries, native differs at36. No dependency patch/rebuild. Higher-order parity, extreme asymmetric weights, complete UV/retained trim/nested native chains and periodic parameter continuation beyond captured natural basis still require native acceptance evidence.
- Nineteen captured families/183-query synthetic tangent circles pass owned contact admission; live native nonplanar checks exercise retained contacts on actual spatial NURBS History. This is named fixture coverage, not all native solver roots, curve families/options/backends, global conversion or rigorous deviation bounds. The spatial archive is reopened in the same host; independent cold spatial contact acceptance remains separate.
- Circle History atomic rejection/lifetime are unchanged: RED355/432/48 callbacks, caller journal pollution, changed metadata/source state and owner-close recompute hang witness. Native savepoint proposal remains pending human review; no new FreeCAD journal/lifetime interface implemented. All original owner/removal/lock/dependency/cache and Undo/Redo/cancellation/notification invariants remain required.
- Task2/3 remain open: generic snap/pick/mouse/render and parameter provenance; snapshot sharing, aggregate allocation/traversal/worker/cancellation bounds; Rebuild and every other enabled result consumer. Per-basis decoder bounds and sampled tests do not establish full aggregate budgets or lifecycle.
- Global conversion/deviation/locus exactness and complete parameter/topology maps remain unproven, with maps null. Full CAD-versus-mesh solid validity/orientation/manifold/cavity coverage, supported attributes and persistence families remain required.
- Fresh reconstructed Runtime93 packets retain genuine resolved fractions and all20 original recipes/points/plans/roles/probe/path parameters. Old Runtime59 fixtures remain immutable. Historical stored results may retain their old bytes; current recompute obtains new complete evidence. Raw legacy producer requests lacking contact metadata are rejected instead of bypassing verification. This does not replace all historical-format acceptance or production deployment requirements.
- Postbuild1475 candidate inputs match. Postnative adds current authored test fixtures and test-source changes only; all production inputs match. Full Rust frozen1483 candidate inputs match except the later new nonplanar test macro. These inventories are not compiler dependency certificates.

Next: Bind every Circle History source role to current replay inputs/configuration across Center/TwoPoint/ThreePoint/Vertical/Fit/Direction/radius endpoints, first reproducing stale source-to-recipe requests with actual owned fixtures. Reuse existing native solver and Rust owned values. Extend initial ConstructCurve Tangent contact provenance and independent cold spatial family coverage. Preserve the raw native D3 RED and pending native savepoint review; continue all original RCORE-01 adapters1–7/C01–C12/T01–T11, result consumers, conversions/maps, budgets, persistence and lifecycle.

[Machine evidence](2026-10-12-rhino-core-circle-history-owned-contact-binding.json).
