# RCORE-01 — Initial Curve owned parameters, 2026-10-12T05:53:17+07:00

**Status: in_progress. Full RCORE-01 and Task2/3 remain open.**

The small plugin bridge retains source_curve_fraction for actual native projected
Tangent probes and typed/seeded/mouse AroundCurve centers. Existing native projection
and solving are reused. Rust independently resolves the selected owned edge,
absolute edge interval and native_location once; its existing evaluators bind D0
and normalized D1 to the admitted point/normal. AroundCurve reuses the actual path
reference on the accepted immutable branch, including Undo. Tangent also binds its
single selected probe evidence. Validation precedes session/log/options mutation.
Construction replay retains and verifies the same fraction through the input API.
This is plugin JSON metadata, not a new FreeCAD journal/lifetime interface or ABI.

TDD **RED1/3→GREEN5/5**: Circle/Ellipse previously accept wrong center, normal or
fraction; Tangent accepts off-curve probe, wrong derivative or fraction. Positive
points/fractions are independently derived from unchanged native line snapshots.
The proposed fractions in these first tests are synthetic metadata, not intercepted
C++ packets. Missing/out-of-range Tangent fractions and removed-path replay reject
atomically.183 input queries across19 actual native-local curve families pass using
native D0/D1 oracles; envelopes are synthetic and do not certify live solver roots.

Typed historical OnCurve=fraction can validate its original token without adding a
new field to canonical old event bytes. Current producers explicitly serialize the
fraction. Old raw Tangent/seed/mouse packets without sufficient parameter evidence
reject; old stored result bytes remain historical evidence until current recompute.
This is not a complete historical-format/deployment certificate. Initial solved
contacts remain distinct from these probes and are not yet independently bound.

The initial full run has three incomplete path fixture setups plus unchanged raw
D3 failure. Literal supporting line data now makes those setups complete; the
legacy absent-unit-context case supplies the true line midpoint. Final full Rust
**982 passed/1 failed/0 ignored**, actual cargo exit101; only the unchanged raw
native offset D3 discrepancy is failed. No native fixture data is rewritten.

Runtime109 plugin-only build exits0 using the existing SDK/cache. New native24/24
checks typed/seeded Circle, Ellipse and Tangent fractions against actual kernel
D0/D1, publication, physical output and same-host FCStd reopen. Existing Curve
result33/33 includes both Circle/Ellipse native paths with full two-edge parent,
original tokens, native Undo/Redo and exact result bytes. Local4, Fit70/cold50,
History109/cold62 and spatial65 also pass: **417/417**, eight actual host exits0.
Shared SDK/App/Gui/host/probe and existing solver stay unchanged.1498 candidate
inputs match after build/native; they are not compiler dependency certificates.
Actual mouse/OpenGL, independent cold new-parameter family coverage, periodic
continuation and every original lifecycle remain required.

Remaining original requirements:

- Full original RCORE-01 remains open: stable GeometryRef identity/revision/representation/world, complete owned geometry/topology/attributes/units, adapters1–7, C01–C12, T01–T11, every enabled GeometryResult consumer, and independent read/edit/retain/export of all supported families.
- Initial Circle/Ellipse AroundCurve and Tangent probe admission now retains normalized source fractions, evaluates owned D0/unitD1 and preserves branch Undo/canonical typed OnCurve tokens. Seeded/mouse C++ producers pass fractions as plain values; actual mouse/OpenGL acceptance remains open. Initial solved Tangent contacts/FromFirst/plane/radius still need independent publication binding, separate from probes. Generic CAD Edge/Face snap point provenance, old-format acceptance and every native family/option/backend still require complete evidence. Nineteen native-local oracle families/183 synthetic input envelopes and named live line/path tests are scoped coverage, not all solver roots or global conversion bounds.
- Raw native offset D3 discrepancy remains enabled RED: independent mathematical derivatives match45 queries, native differs at36. No dependency patch/rebuild. Higher-order parity, extreme asymmetric weights, complete UV/retained trim/nested native chains and periodic parameter continuation beyond captured natural basis still require native acceptance evidence.
- Nineteen captured families/183-query synthetic tangent circles pass owned contact admission; live native nonplanar checks exercise retained contacts on actual spatial NURBS History. This is named fixture coverage, not all native solver roots, curve families/options/backends, global conversion or rigorous deviation bounds. The spatial archive is reopened in the same host; independent cold spatial contact acceptance remains separate.
- Circle History atomic rejection/lifetime are unchanged: RED355/432/48 callbacks, caller journal pollution, changed metadata/source state and owner-close recompute hang witness. Native savepoint proposal remains pending human review; no new FreeCAD journal/lifetime interface implemented. All original owner/removal/lock/dependency/cache and Undo/Redo/cancellation/notification invariants remain required.
- Task2/3 remain open: generic snap/pick/mouse/render and parameter provenance; snapshot sharing, aggregate allocation/traversal/worker/cancellation bounds; Rebuild and every other enabled result consumer. Per-basis decoder bounds and sampled tests do not establish full aggregate budgets or lifecycle.
- Global conversion/deviation/locus exactness and complete parameter/topology maps remain unproven, with maps null. Full CAD-versus-mesh solid validity/orientation/manifold/cavity coverage, supported attributes and persistence families remain required.
- Fresh reconstructed Runtime93 packets retain genuine resolved fractions and all20 original recipes/points/plans/roles/probe/path parameters. Old Runtime59 fixtures remain immutable. Historical stored results may retain their old bytes; current recompute obtains new complete evidence. Raw legacy producer requests lacking contact metadata are rejected instead of bypassing verification. This does not replace all historical-format acceptance or production deployment requirements.
- Prebuild1498 candidate inputs match after build/native; not a compiler dependency certificate. Plugin bridge changes only plain parameter serialization; FreeCAD SDK/host and native solving are reused. No new FreeCAD journal/lifetime interface, root/dependency rebuild, deployment or Git publication.

Next: Bind initial solved Tangent contacts to each owned source and final circle/plane/radius/FromFirst using actual retained native solver fractions; first reproduce a publication plan that is numerically valid with correct probes but invalid contacts. Preserve probe/contact distinction, all supported families, branch Undo and old stored evidence. Extend generic edge/face snap and independent cold spatial coverage. Keep raw D3 RED and pending native savepoint review; continue every original RCORE-01 adapter1–7/C01–C12/T01–T11, consumer, conversion/maps, aggregate budgets, persistence, read/edit/retain/export and lifecycle.

[Machine evidence](2026-10-12-rhino-core-initial-curve-owned-parameters.json).
