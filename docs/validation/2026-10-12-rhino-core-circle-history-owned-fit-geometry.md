# RCORE-01 — Circle History selected geometry, 2026-10-12T05:06:07+07:00

**Status: in_progress. Full RCORE-01 and Task2/3 remain open.**

Safe Rust independently reconstructs selected CAD points with the existing native
FitPoints precedence: one direct NURBS/Bezier surface uses its full U-major control
net; otherwise one direct NURBS/Bezier curve uses full control poles; otherwise
unique vertices follow native_topology_index. Explicit VertexN/FaceN resolve their
unique native rows. Surface bases use existing typed rational/periodic validation;
curves reuse the bounded fallible decoder. Inspection may unwrap a rectangular
trim, so selection checks the original native_class and does not falsely promote
its support poles. Control-pole selection is independent of edge parameter
evaluation. Absent Curve3d preserves vertex fallback. CAD native_location is
applied once; captured document_world already includes own/parent placements.
Discrete selection retains the independent full XYZ/index/frame checks and now
requires complete native order, allowing only the retained prefix role to shorten.

The new native fixture is **RED62/70** on unchanged Runtime97: direct Face1 NURBS
and Bezier constructors work, but both unchanged and edited History replays lose
paired output. Actual host exit0, no fixture exception. The same unchanged macro
on Runtime102 is **GREEN70/70**. It exercises rational and periodic NURBS curves,
rational Bezier curve, NURBS/Bezier surfaces both whole and Face1, multi-face vertex
fallback, point cloud and mesh, with nontrivial own and parent rotations/translations.
Native Python poles/Vertexes/Points provide the independent order/world oracle.

Rust TDD first **0/5→5/5** fixes stale selected points plus consistent replay,
invalid basis, native vertex order and whole-versus-prefix selection. Two review
findings independently fail then pass: periodic control selection must not require
edge evaluator continuation, and absent Curve3d must keep vertex fallback. Final
focused **7/7**, Curve results **56/56**, full Rust **972 passed/1 failed/0 ignored**,
actual cargo exit101. The unchanged native offset D3 discrepancy test remains
enabled RED. The initial test typo and unavailable Result helper compile attempts
are retained as excluded attempts; neither is counted as behavioral RED evidence.

Fixtures reconstruct eight actual saved native replays; two Face cases use
synthetic History envelopes around genuine successful native constructor records,
because their original replay is RED. Stamps, geometry, topology, weights, classes,
location and attributes are native facts, not invented. The fixed live native run
now supplies complete genuine Face replay evidence separately. Existing earlier
JSON fixtures are immutable. Literal absent-Curve3d and synthetic extended edge
domain tests are not independent native positive family certification.

Fresh Runtime102 plugin-only build exits0, reusing the single SDK/cache. The first
cold test is41/50: nine intermediate captures differ only in the parent revision
after later Group additions, while archived bytes/masters/replay pass. This oracle
timing error is retained. A wrapper reuses the unchanged70 gates and captures all
final live native records before save; the whole-result comparison is preserved.
Independent
cold process verifies exact result bytes against persisted FCStd native String
properties, complete saved owned results, independently read native masters and
parent-change replay for all10 cases: **50/50**. Existing all-mode109, cold62 and
spatial65 regressions also pass. Including the final-record capture70, total
**426/426**, six actual host exits0. Shared
SDK/App/Gui/host/probe, C++ replay and native solvers remain unchanged. Candidate
inventory1491 matches after build. The later archive wrapper/cold oracle change
two test files only (current1492); all production inputs match, full inventory
false. These are not compiler dependency certificates.
Offscreen evidence does not certify mouse/OpenGL rendering or all source lifecycle.

Remaining original requirements:

- Full original RCORE-01 remains open: stable GeometryRef identity/revision/representation/world, complete owned geometry/topology/attributes/units, adapters1–7, C01–C12, T01–T11, every enabled GeometryResult consumer, and independent read/edit/retain/export of all supported families.
- Circle History selected-point binding now follows direct native surface poles, direct curve poles, then unique vertex enumeration; FaceN, rational/periodic NURBS/Bezier, cloud/mesh, prefix and world-once named fixtures pass. Ten cases are necessary coverage, not every native class/degree/weight/trim/placement/topology option/backend. Absent Curve3d fallback and extended periodic edge interval are literal/synthetic Rust tests, not native positive family proofs. Whole retained control basis is distinct from edge evaluation; evaluator periodic continuation remains open. Initial ConstructCurve input admission/publication still needs equivalent complete source selection/contact checks.
- Raw native offset D3 discrepancy remains enabled RED: independent mathematical derivatives match45 queries, native differs at36. No dependency patch/rebuild. Higher-order parity, extreme asymmetric weights, complete UV/retained trim/nested native chains and periodic parameter continuation beyond captured natural basis still require native acceptance evidence.
- Nineteen captured families/183-query synthetic tangent circles pass owned contact admission; live native nonplanar checks exercise retained contacts on actual spatial NURBS History. This is named fixture coverage, not all native solver roots, curve families/options/backends, global conversion or rigorous deviation bounds. The spatial archive is reopened in the same host; independent cold spatial contact acceptance remains separate.
- Circle History atomic rejection/lifetime are unchanged: RED355/432/48 callbacks, caller journal pollution, changed metadata/source state and owner-close recompute hang witness. Native savepoint proposal remains pending human review; no new FreeCAD journal/lifetime interface implemented. All original owner/removal/lock/dependency/cache and Undo/Redo/cancellation/notification invariants remain required.
- Task2/3 remain open: generic snap/pick/mouse/render and parameter provenance; snapshot sharing, aggregate allocation/traversal/worker/cancellation bounds; Rebuild and every other enabled result consumer. Per-basis decoder bounds and sampled tests do not establish full aggregate budgets or lifecycle.
- Global conversion/deviation/locus exactness and complete parameter/topology maps remain unproven, with maps null. Full CAD-versus-mesh solid validity/orientation/manifold/cavity coverage, supported attributes and persistence families remain required.
- Fresh reconstructed Runtime93 packets retain genuine resolved fractions and all20 original recipes/points/plans/roles/probe/path parameters. Old Runtime59 fixtures remain immutable. Historical stored results may retain their old bytes; current recompute obtains new complete evidence. Raw legacy producer requests lacking contact metadata are rejected instead of bypassing verification. This does not replace all historical-format acceptance or production deployment requirements.
- Prebuild1491 candidate inventory matches at build completion; later final-capture wrapper and corrected cold oracle change only two test files (current1492). Production inputs match; full inventory false, not a compiler dependency certificate. Full Rust precedes addition of cold/final-capture macros only; Rust production/test inputs remain unchanged. No C++/FreeCAD journal/lifetime interface, root/dependency rebuild, production deployment or Git publication was performed.

Next: Reuse independent selection validation in initial native Curve input admission/result replay, first reproduce stale point/FitPoints source geometry accepted through the real native-input API. Preserve native action semantics, full source/ranges, branch Undo and all existing families. Complete initial Tangent source/contact provenance and independent cold spatial coverage. Keep raw native D3 RED and pending native savepoint review; continue every original RCORE-01 adapter1–7/C01–C12/T01–T11, GeometryResult consumer, conversions/maps, aggregate budgets, read/edit/retain/export, persistence and lifecycle.

[Machine evidence](2026-10-12-rhino-core-circle-history-owned-fit-geometry.json).
