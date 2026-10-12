# Solid GeometryResult implementation plan

**Goal:** Route the existing Box/Sphere solver through the shared owned GeometryResult producer, retaining accepted numeric input and actual native source snapshots before publication.

**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`, existing Solid contract and shared prepared result/native adapters.

**Architecture:** Add a Rust `solid_result` validator to the existing `prepared_result::assemble` dispatcher. It admits the owned construction recipe through the existing ConstructSolid capability policy, binds native reference identity/units/attributes and selected point ranges, and returns owned input/output snapshots. C++ captures native masters/attributes and invokes this existing bounded result ABI; the existing Part solver and checked publication transaction remain in use.

**Full scope retained:** This producer is necessary for full RCORE-01 but cannot certify complete conversion maps/deviation, native source channels not inspected by existing readers, actual preview leases, every representation/consumer or original adapter1–7/C01–C12/T01–T11. All remain required. Native observed facts and unknown global conversion facts remain distinct.

## Global constraints

- Rust owns plain data, construction/options admission and serialization. Native OCCT/FreeCAD capture and GUI-thread transaction calls require C++; no host pointer escapes into Rust/results.
- Reuse the single SDK and standalone OpenMatrix9Gui plugin cache; never rebuild/copy the FreeCAD runtime for OM9-only edits. Native hosts run serially and use isolated profiles.
- Construction and result bytes, Shape, units and command metadata must publish atomically. Reject stale generation/context/owners and changed/removed output metadata with original operation/input diagnostic.
- Selected FitPoints retain capture-time sources and point ranges. Undo removes an entire selected batch and its guard/evidence; a later source edit must not produce a stale sphere.
- Curve masters returned by existing nativeShapeInput are already document-world. Discrete snapshots retain native-local geometry/location plus an explicit native-parent-to-world matrix. Never silently apply a second world transform.
- Global parameter/topology maps and deviation remain unknown until independently proven. A valid solid does not establish conversion preservation.

## Tasks

- [x] Write failing Rust tests for ConstructSolid dispatch, owned numeric/native inputs, native source/recipe mismatch, point-range and unit/frame rejection, solid output validity, no false NativeBuilder/import/conversion certificate and FFI canaries. Observe current dispatcher reject real completed Solid recipes.
- [x] Implement `rust/src/solid_result.rs`, shared dispatch and tests; reuse NativeSolid capability, source attributes, frame/stamp and discrete count/value/index policy. Run focused and full Rust suites.
- [x] Add a native command fixture that requires OM9GeometryResult beside Shape, reconstructs owned output master and checks input provenance, generation, units, source immutability, rollback, Undo/Redo and persistence; observe Runtime35 fail before integration.
- [x] Add `Gui/CoreSolidResult.h/.cpp`, capturing selected native source masters/attributes before callbacks. Keep FitPoints batches and their capture-time guards, prune on input Undo/mode changes, and revalidate before construction/publication. Publish/freeze/compare OM9GeometryResult through the existing transaction.
- [ ] Build a fresh standalone runtime; verify actual warm/cold result and relevant Solid regressions, independent source review, public audit, authoritative evidence/README/spec/ledger/manifest sync. Leave full RCORE-01 in_progress until every original requirement has proof.

## Review focus

Missing/replaced native source after numeric capture; source changes during output callbacks; selected FitPoints Undo and typed gaps; document-world curve versus native-local discrete geometry; altered/missing result metadata after creation; same native output with changed placement/tolerance; stale unit context and replacement command preservation; bounded request/output and ABI ownership.

## Progress — 2026-10-11T18:24:11+07:00

Final Rust825/0/0 (9 focused) and standalone Runtime38 build0. Independent review repairs are implemented. Native/C++ task checkboxes remain open until actual warm/cold/guards and aggregate-budget witnesses run. Three startup-only attempts excluded; fourth diagnostic host has a log and remains unaccepted. Full chapter is unchanged.

Native update — 2026-10-11T18:33:45+07:00: five owned startup-only attempts (including software OpenGL) are excluded and verified terminal. No native Solid RED/GREEN acceptance exists; pending native task boxes stay unchecked.

## Progress — 2026-10-11T19:19:24+07:00

Runtime39 matching offscreen native524/524 checks/10 actual exits0; separate cold host opens5 FCStd. Same large-source witness Runtime38 RED3/5 →39 GREEN5/5 after exact owned snapshot sharing with full-input indices. Full Rust826/0/0;10 focused;standalone build0. Native fixture and C++ integration tasks are checked for their stated scoped evidence. Final audit/sync task stays open for full requirements, actual mouse/render, unique-source admission and indexed54-reference cold reload; chapter remains in_progress.

Indexed restore update — 2026-10-11T19:23:09+07:00: actual warm12/12 and distinct cold12/12 preserve54 alternating Vertex1/Vertex2 refs, two owned1000-vertex masters,56 points, exact bytes and one warm Undo/Redo. Matching Runtime39 total548/548 over12 host exits0, two cold hosts. Indexed cold gap is closed for this fixture; remaining unique-source admission, actual render/mouse and full chapter requirements remain open.

Distinct-source update — 2026-10-11T19:28:16+07:00: native Runtime39 RED3/5, actual host exit0, no fixture exception;55 accepted/56th byte-budget reject then Undo/Enter fails54-source prepared result. Aggregate admission remains open; not included in548 passing checks. Capture exact failing packet before repair.

## Progress — 2026-10-11T19:51:28+07:00

Actual Runtime40 failed request captured:16,308,922 bytes/1,630,773 nodes. Necessary exact source-envelope admission gate added; unchanged unique-source fixture RED3/5 → Runtime41 GREEN5/5,33 accepted then rejection before batch acceptance;Undo→32 refs/34 points produces sphere. Current matching553/553 over13 exits0,2 cold hosts/6 FCStd;Rust829/0/0;standalone retry build0;437 frozen inputs match. Full request/output overhead, global budgets and all original chapter requirements remain open.
