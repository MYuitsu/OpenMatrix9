# RCORE-01 — Circle History stored roles/current replay, 2026-10-12T04:45:27+07:00

**Status: in_progress. Full RCORE-01 and Task2/3 remain open.**

Safe Rust reconstructs current replay points from immutable recorded points and
source-role triples. Point roles use current selected point evidence; Tangent
probes use the existing independent owned curve evaluator and stored fraction.
Whole FitPoints replaces original spans with the entire current selection,
including growth/shrink; retained prefixes preserve their original count. Spans
are sorted by original offset, numeric gaps/suffixes retained, overlap rejected.
Direction derives endpoint minus current center; picked radius derives endpoint
distance to current anchor. Every other stored recipe field remains unchanged.
Legacy AroundCurve explicitly has empty recorded storage and keeps its existing
independent current path/fraction validation. No native solver is rewritten.

The new role tests first fail **0/6**, then pass **6/6**. A seventh regression
covers unsorted whole/prefix spans with a numeric gap. The actual source snapshots
are unchanged native captures; altered points/options/plan envelopes are synthetic.
The test replay helper prepares a valid numeric plan only, never expected source
coordinates. Six stale point modes, direction, picked radius, unbound numeric
storage, malformed/overlapping spans and retained prefix are checked. All20 native
reconstructed baselines still assemble, including whole FitPoints growth4→5.

The first full run shows six setup failures plus the existing raw D3 failure.
Old synthetic fixtures omitted recorded storage or changed a recipe/typed fixed
point without changing their own stored record. These setups now contain complete
honest metadata; the source evaluator expectations still come from native D0/D1
oracles. The typed-only fixture now retains its original numeric point unchanged.
No authored native JSON fixtures were edited. Final Curve result group **50/50**;
full Rust **965 passed/1 failed/0 ignored**, actual cargo exit101. The unchanged
failure is `owned_curve_jets_match_native_value_and_three_derivatives_for_nineteen_families`
at native OffsetBezier D3; it remains enabled and is not masked.

Runtime97 standalone plugin build exits0 using the existing SDK/cache. Fresh
all-mode History109/109, independent cold62/62 and spatial65/65 pass: **236/236**,
three actual host exits0. The spatial archive reopens in its own host; this is not
independent cold spatial proof. Shared SDK/App/Gui/host/probe and the existing C++
replay/solver stay unchanged. Candidate inventory1486 matches after build/native;
it is not a compiler dependency certificate. Offscreen tests do not certify
mouse/OpenGL rendering. No production deployment or Git publication is claimed.

This step binds selected evidence to replay inputs. It does **not** yet establish
all selected CAD points against full supporting geometry: native FitPoints uses
direct NURBS/Bezier surface or curve poles before falling back to unique vertices.
FaceN source validation and complete ordered native selection provenance remain
open. No vertex-only fallback or narrower family support policy was added.

Remaining original requirements:

- Full original RCORE-01 remains open: stable GeometryRef identity/revision/representation/world, complete owned geometry/topology/attributes/units, adapters1–7, C01–C12, T01–T11, every enabled GeometryResult consumer, and independent read/edit/retain/export of all supported families.
- Stored Circle History roles now reconstruct current points and recipe across supported modes, preserving whole FitPoints growth and prefix/numeric spans. Complete CAD whole/Face FitPoints selected-point geometry correspondence still needs independent direct NURBS/Bezier control-pole versus vertex provenance, native-location/world-once and ordered native enumeration coverage. FaceN is still rejected by the existing generic result source validator despite native FitPoints supporting direct spline/Bezier faces. No vertex-only fallback or reduced family policy was added. Initial ConstructCurve Tangent contact/source publication remains open.
- Raw native offset D3 discrepancy remains enabled RED: independent mathematical derivatives match45 queries, native differs at36. No dependency patch/rebuild. Higher-order parity, extreme asymmetric weights, complete UV/retained trim/nested native chains and periodic parameter continuation beyond captured natural basis still require native acceptance evidence.
- Nineteen captured families/183-query synthetic tangent circles pass owned contact admission; live native nonplanar checks exercise retained contacts on actual spatial NURBS History. This is named fixture coverage, not all native solver roots, curve families/options/backends, global conversion or rigorous deviation bounds. The spatial archive is reopened in the same host; independent cold spatial contact acceptance remains separate.
- Circle History atomic rejection/lifetime are unchanged: RED355/432/48 callbacks, caller journal pollution, changed metadata/source state and owner-close recompute hang witness. Native savepoint proposal remains pending human review; no new FreeCAD journal/lifetime interface implemented. All original owner/removal/lock/dependency/cache and Undo/Redo/cancellation/notification invariants remain required.
- Task2/3 remain open: generic snap/pick/mouse/render and parameter provenance; snapshot sharing, aggregate allocation/traversal/worker/cancellation bounds; Rebuild and every other enabled result consumer. Per-basis decoder bounds and sampled tests do not establish full aggregate budgets or lifecycle.
- Global conversion/deviation/locus exactness and complete parameter/topology maps remain unproven, with maps null. Full CAD-versus-mesh solid validity/orientation/manifold/cavity coverage, supported attributes and persistence families remain required.
- Fresh reconstructed Runtime93 packets retain genuine resolved fractions and all20 original recipes/points/plans/roles/probe/path parameters. Old Runtime59 fixtures remain immutable. Historical stored results may retain their old bytes; current recompute obtains new complete evidence. Raw legacy producer requests lacking contact metadata are rejected instead of bypassing verification. This does not replace all historical-format acceptance or production deployment requirements.
- Prebuild/full Rust/native candidate inventory1486 is unchanged through verification; this is not an exact compiler dependency certificate. No new C++/FreeCAD journal/lifetime interface, root/dependency rebuild, production deployment or Git publication was performed.

Next: Complete independent current selected-point geometry binding for every supported Circle History source: retain native precedence for direct NURBS/Bezier surface poles, direct curve poles and unique topology vertices; preserve prefix, whole growth, point-cloud/mesh and world-once semantics. First add genuine native whole curve/Face/control-net fixtures and stale selection witnesses, then reuse typed Rust basis/frame validation. Extend initial Tangent contact provenance and independent cold spatial coverage. Preserve raw D3 RED and pending native savepoint review; continue all original RCORE-01 adapters1–7/C01–C12/T01–T11, result consumers, conversions/maps, budgets, persistence and lifecycle.

[Machine evidence](2026-10-12-rhino-core-circle-history-owned-role-replay.json).
