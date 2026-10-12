# RCORE-01 — Circle History owned source binding, 2026-10-12T04:20:25+07:00

**Status: in_progress. RCORE-01 and Task2/3 remain open.**

Rust now decodes owned analytic, rational/periodic NURBS, Bezier and trim/offset
curve values through the existing typed captures. It checks integer counts before
basis allocation, retains compact knots/weights and limits wrapper depth. Selected
EdgeN resolves by canonical native topology index; a whole curve must contain one
edge, matching the existing native source policy. Finite edge restriction, geometry
category and affine placement are checked. The document_world snapshot embeds
source placement in native_location, which is applied once. Translation affects
points; the linear frame part transforms D1, followed by bounded normalization.

AroundCurve path_fraction is converted to its absolute edge parameter and binds
both the replay center and oriented normal to owned point/unit tangent. Tangent
recorded source fractions bind current curve probes, independently from final
contacts. Fixed Point roles bind the selected source point with the existing native
roundoff tolerance. Original recorded Around probe and current anchor remain distinct.
No existing solver, C++ adapter or native FreeCAD interface was changed.

Source tests **17 passed/6 failed → 23 passed/0 failed**. The original three source
failures are now green, unchanged. Added RED cases cover reversed path normal,
altered curve fraction and a substituted on-circle fixed point. A legacy synthetic
fixture previously containing only an Edge1 index was completed with an explicitly
synthetic owned line and fraction; production does not admit the incomplete stub.
Selected-edge tests **3/3** cover183 genuine native-local queries on19 families
under a synthetic rotation/translation, malformed basis/count/frame rejection and
edge domain outside its supporting basis. The domain witness failed before its fix.
Another183-query test goes through actual Circle History source admission with
synthetic valid numeric envelopes using native D0/D1 expectations. Existing replay
normalization prepares the envelope; the new source evaluator never computes the
expected source point/tangent. These are not183 live native History operations.

Full Rust **953 passed/2 failed/0 ignored**, actual cargo exit101. One retained
failure is raw native offset D3. The second is a new concrete contact witness: both
probes still correspond to vertical x=1 and horizontal y=0, but center(5,5),radius4
is accepted even though it cannot be tangent to y=0. This explicitly disproves full
Tangent plan/source validation; probe correspondence does not close that requirement.

Runtime91 standalone plugin build exits0; shared SDK reused and fingerprints
unchanged. The existing all-mode native suite is109/109, cold reopen62/62, total
**171/171**, two actual host exits0. Twenty stored current results survive cold
reopen with exact bytes and source-edit replay. All1475 frozen candidate inputs
match after build and native checks; this is not a compiler dependency certificate.

Remaining requirements:

- Full original RCORE-01 remains open: stable GeometryRef identity/revision/representation/world, complete owned geometry/topology/attributes/units, adapters1–7, C01–C12, T01–T11, and every enabled GeometryResult consumer.
- Final Tangent contact/plane/radius/source correspondence is not implemented. The new unchanged RED witness retains both correct curve probes but moves the circle to center(5,5),radius4, which is not tangent to its horizontal source y=0. Probe checks are not contact checks. Direction and radius-endpoint source semantics also need complete independent correspondence.
- Raw native offset D3 discrepancy remains enabled RED: Rust matches independent mathematical derivatives at45 queries, native differs at36. No dependency patch/rebuild. Higher-order parity, extreme asymmetric weights, UV/retained trim/nested native chains and periodic parameter continuation beyond the captured natural basis need native acceptance evidence.
- AroundCurve and Tangent curve probes now bind selected owned edge, fraction, absolute domain, native location applied once and D0/D1. Nineteen captured families/183 queries pass selected-edge world tests and source admission. These are synthetic envelopes around genuine native-local oracle rows, not183 live native History commands or complete family/backend acceptance. Native unflattened chain parity remains unproven.
- Circle History atomic rejection/lifetime are unchanged: RED355/432/48 callbacks, pending caller journal pollution, changed metadata/source state, and owner-close recompute hang witness. Native savepoint proposal remains pending human review; no new interface implemented. Full ownership/removal/lock/dependency/cache, Undo/Redo/cancellation and notification invariants remain required.
- Task2/3 remain open: generic snap/pick/mouse/render and parameter provenance; snapshot sharing, aggregate allocation/traversal/worker/cancellation bounds; Rebuild and all other enabled result consumers. Decoder per-basis counts/depth/payload bounds are not an aggregate budget certificate.
- Global conversion/deviation/locus exactness, complete parameter/topology maps and native CAD-versus-mesh solid validity/orientation/manifold/cavity coverage remain unproven. Maps remain null. Complete independent read/edit/retain/export and all supported attributes/persistence families remain required.
- Native History109/cold62 checks are named existing coverage with20 exact cold results, not every option/representation/backend, GUI mouse/render, full lifecycle or production deployment. Frozen candidate inventory is not an exact compiler dependency certificate.

Next: Use the existing native Tangent solver contacts to preserve resolved contact fractions, then independently evaluate owned D0/D1 against circle plane/radius/tangency and FromFirst. Preserve legacy/current History behavior and every supported curve family; first retain the new plan-with-correct-probes RED witness and obtain fresh native packet/result/cold evidence. Extend live native family/world/periodic-continuation coverage. Keep the native savepoint proposal pending review and continue all original RCORE-01 adapters1–7/C01–C12/T01–T11, result consumers, conversion/maps, budgets, persistence and lifecycle.

[Machine evidence](2026-10-12-rhino-core-circle-history-owned-source-binding.json).
