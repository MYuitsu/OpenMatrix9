# RCORE-01 — Initial Curve owned selection, 2026-10-12T05:29:03+07:00

**Status: in_progress. Full RCORE-01 and Task2/3 remain open.**

Safe Rust now shares the existing owned CAD selection validator between initial
native input and Circle History. Full FitPoints uses native surface-pole/curve-pole/
vertex precedence; native VertexN and standalone-point inputs bind their selected
coordinates to actual owned geometry. Validation runs before any session mutation.
Construction result replay calls the same input API, so the check also applies to
owned input evidence retained at publication. Existing solvers and C++ adapters
are reused; no external ABI or FreeCAD interface is added.

Real `accept_native_json` tests around immutable native captures are **RED1/4 →
GREEN4/4**: eight CAD families previously accept jointly falsified selected/input
points, while cloud/mesh already reject through their full-coordinate checks.
All10 native selections remain accepted. Vertex APIs for Line/Polyline/Rectangle/
InterpCrv/Circle/Ellipse and Direction/fixed Tangent point actions now reject the
same stale source coordinates. Failure preserves prompt/options/log/points/input
revision/context; the original valid packet remains admissible immediately after.
No intercepted C++ packet or live kernel failure is claimed by these Rust tests.
Existing discrete explicit indices/subset ordering remains supported; the stricter
whole/prefix History reader policy was not imposed on the generic input API.

A separate native oracle finds a local/world policy error: retained NURBS poles
near1e12 translated by-1e12 yield world points within±5mm. Initial constructor
works, but old History rejects its local poles. Runtime102 **RED3/4**, actual exit0,
no fixture exception; Vertex is already normalized by native capture and passes.
Rust input reproduction is **RED4/5→GREEN5/5**. Retained local coordinates now use
the existing support basis1e150 policy; transformed command inputs still obey the
existing1e9 world limit. No bounds were widened for command points. Runtime106
with the same unchanged native fixture is **GREEN4/4**.

The first full Rust run has seven setup/ABI failures plus the unchanged raw D3
failure. Old unit FitPoints envelopes claimed Edge1 with no geometry, and point
fixtures omitted vertices. Complete literal vertex compounds/points now provide
the same independently specified coordinates; ABI schema tests retain their opaque
BREP limitation. Native captures are immutable. Final full Rust **977 passed/
1 failed/0 ignored**, actual cargo exit101. Only the unchanged raw native offset D3
discrepancy remains RED. All23 native input unit tests and the existing ABI test
pass, including explicit discrete subsets, branch Undo, sharing and1025-point
Polyline capacity.

Standalone Runtime106 build exits0 with the shared SDK/cache. Local/world4,
Fit archive70/cold50, all-mode109/cold62 and spatial65 pass: **360/360**, six actual
host exits0. The two cold hosts verify exact archived result properties and current
physical replay; offscreen is not mouse/OpenGL proof. C++ replay/solvers and shared
SDK/App/Gui/host/probe fingerprints remain unchanged. Candidate1496 inputs match
after build/native; this is not a compiler dependency certificate.

Remaining original requirements:

- Full original RCORE-01 remains open: stable GeometryRef identity/revision/representation/world, complete owned geometry/topology/attributes/units, adapters1–7, C01–C12, T01–T11, every enabled GeometryResult consumer, and independent read/edit/retain/export of all supported families.
- Initial accept_native_json now reuses owned CAD full FitPoints selection and native VertexN/standalone-point validation before changing session state. Point/reference Direction/fixed Tangent point actions and six constructor vertex API cases pass; construction replay uses that same admission. Discrete explicit subset indices retain their existing independent full-coordinate contract. Generic CAD Edge/Face mouse/snap point inputs are not yet independently bound; initial AroundCurve OnCurve parameter/normal and Tangent curve probe/contact/source publication remain open. Named ten-family input/cold fixtures and local1e12/world±5 witnesses are scoped evidence, not every family/options/backend or full source lifecycle.
- Raw native offset D3 discrepancy remains enabled RED: independent mathematical derivatives match45 queries, native differs at36. No dependency patch/rebuild. Higher-order parity, extreme asymmetric weights, complete UV/retained trim/nested native chains and periodic parameter continuation beyond captured natural basis still require native acceptance evidence.
- Nineteen captured families/183-query synthetic tangent circles pass owned contact admission; live native nonplanar checks exercise retained contacts on actual spatial NURBS History. This is named fixture coverage, not all native solver roots, curve families/options/backends, global conversion or rigorous deviation bounds. The spatial archive is reopened in the same host; independent cold spatial contact acceptance remains separate.
- Circle History atomic rejection/lifetime are unchanged: RED355/432/48 callbacks, caller journal pollution, changed metadata/source state and owner-close recompute hang witness. Native savepoint proposal remains pending human review; no new FreeCAD journal/lifetime interface implemented. All original owner/removal/lock/dependency/cache and Undo/Redo/cancellation/notification invariants remain required.
- Task2/3 remain open: generic snap/pick/mouse/render and parameter provenance; snapshot sharing, aggregate allocation/traversal/worker/cancellation bounds; Rebuild and every other enabled result consumer. Per-basis decoder bounds and sampled tests do not establish full aggregate budgets or lifecycle.
- Global conversion/deviation/locus exactness and complete parameter/topology maps remain unproven, with maps null. Full CAD-versus-mesh solid validity/orientation/manifold/cavity coverage, supported attributes and persistence families remain required.
- Fresh reconstructed Runtime93 packets retain genuine resolved fractions and all20 original recipes/points/plans/roles/probe/path parameters. Old Runtime59 fixtures remain immutable. Historical stored results may retain their old bytes; current recompute obtains new complete evidence. Raw legacy producer requests lacking contact metadata are rejected instead of bypassing verification. This does not replace all historical-format acceptance or production deployment requirements.
- Prebuild1496 candidate inputs match after build/native; not a compiler dependency certificate. Shared SDK/host and C++ adapters/solvers are unchanged. No new FreeCAD journal/lifetime interface, root/dependency rebuild, deployment or Git publication.

Next: Complete initial native AroundCurve/Tangent parameter, probe/normal and solved-contact provenance against owned sources, first reproduce wrong native source point/normal/fraction admitted by the real input/publication API. Preserve source roles, original parameter text, branch Undo and every existing family. Extend generic CAD edge/face snap provenance and independent cold spatial coverage. Keep raw native D3 RED and pending native savepoint review; continue all original RCORE-01 adapters1–7/C01–C12/T01–T11, consumers, conversions/maps, aggregate budgets, read/edit/retain/export, persistence and lifecycle.

[Machine evidence](2026-10-12-rhino-core-initial-curve-owned-selection.json).
