# Phase3 acceptance

Phase3 **implementation_verified=true / application_accepted=true (accepted_scoped)**: 18/18 requirement groups;15 fixture Rhino5 / 599 checks, 14 OM9 reports / 504 checks. Actual two-way clipboard11 fixtures /496 Rhino /218 host +120 saved reread checks on the same module. Rust152 tests; native10 suites; tools72 tests; format/strict Clippy PASS; one independent whole-change review resolved. Source/runtime isolated at om9-phase3-dev/sdk; no primary integration. Full openNURBS remains incomplete: seven broader packages open, zero prepared Phase3 batches pending, future total unknown. [Summary](summary.json).

Manifest `bc22bb5c662e0e4c268a03f253d3c2a6e211842ee1e13d431f136da6e01ad622`; module `5d6f16ed8df443d6dd6c240cf72b00f2531608f67702dd0e804a33144f6ea673`.

| Requirement | Result | Evidence |
|---|---|---|
| P3.1-A — Hole/seam/pole/torus/cavity import stays valid native CAD | verified | results.json; LastTest.log; results.json |
| P3.1-B — Open-shell Boolean rejected, invalid topology causes no mutation | verified | results.json |
| P3.1-C — Edited cylinder keeps common UV basis under knot insertion | verified | LastTest.log; results.json |
| P3.2-A — Imported rational curves drive Loft/Sweep | verified | results.json; results.json |
| P3.2-B — Imported open faces Trim/Join, polysurface Explode | verified | results.json; results.json; results.json |
| P3.2-C — Imported closed solids BooleanDifference | verified | results.json; results.json |
| P3.2-D — Menu/CMD/mouse use the same implementation | verified | results.json; results.json |
| P3.2-E — Read-only/task/stale/Cancel fails without mutation | verified | results.json; results.json; results.json |
| P3.2-F — Nested transformed inputs apply global placement once | verified | results.json |
| P3.2-G — Failed or empty kernel result retains inputs | verified | results.json; results.json; results.json |
| P3.2-H — Operation capability is truthful, CAD tessellation stays CAD | verified | results.json; results.json |
| P3.3-A — Snap → cutter → cut → new Loft/Sweep detail | verified | results.json; results.json |
| P3.3-B — Undo/Redo every modeling step | verified | results.json |
| P3.3-C — Delete source 3DM, save/close/reopen FCStd | verified | results.json; results.json |
| P3.3-D — Selected V5 export includes edited and new current shapes | verified | results.json; results.json; results.json |
| APP-GATE-P3 — Actual Rhino5 Open/SaveAs/Export Selected then OM9 reread | verified | phase3-verification.json; results.json |
| REG-P12 — Preserve accepted Phase 1/2 and worker/snap/clipboard rules | verified | LastTest.log; rust-final.log; rhino5-results.json; results.json; results.json; results.json; results.json; results.json; results.json; results.json |
| REVIEW-P3 — Resolve important findings, audit every requirement | verified | final-review.md |

Rust owns portable policy/options/state/request identity/signatures; C++ owns required Qt/FreeCAD/Part/OCCT/openNURBS bridges. Python/PowerShell are test/bootstrap glue. Native unsafe dependencies are not certified memory-safe.

Rhino user entry: `_-RunPythonScript "H:/FreeCAD-src/build/om9-phase3-dev/tests/rhino5_verify_phase3.py"`.

Decisions and costs are retained in migration-ledger/progress.md.

Ruling: Use a separately seeded filesystem-isolated source/runtime with sorted source hashes rather than pretending the parent FreeCAD Git status represents an OM9 worktree — accepted OM9 source is ignored build content and prior work uses this source/runtime isolation — cost if wrong: no Git commit-range provenance; keep exact source snapshots/diffs and perform independent final review.
Ruling: The user's explicit request to implement the presented written design authorizes inline execution of the existing Phase3 scope; the updated plan supplies concrete Rust/file/gate bindings without another permission loop — direct user authorization takes precedence over a skill's redundant implementation approval prompt — cost if wrong: user may prefer changing implementation sequencing; product changes stay in the separate Phase3 source/runtime.
Ruling: snapshot selected current main Surface/Edit sources; main now has native ObjectSnapshot guards, unlike earlier historical audit. Replace guards with Phase3 Rust-owned signatures; retain host shape adapters; disable excluded history/special-type mutation. Cost if wrong: integration needs further regression proof, no completion inferred.
Ruling: raw FreeCAD mass and OCCT adaptive mass on an extrusion miss rational knot spans; accurate test oracle converts exact faces into NURBS (no resampling) before adaptive integration1e-13, retaining original face orientation and count — same Sweep area raw152.5630719958 versus normalized150.7964472054 / analytic48*pi — cost if wrong: oracle could hide geometry distortion; independent analytic source, bounds, topology, pcurves and Rhino mass must also agree.
Tool suite70 PASS after restoring accepted ref junction and using owned TEMP. Missing excluded fixture catalog/sandbox TEMP were environment failures, preserved separately; source/reference originals unchanged. Actual clipboard failure smoke with new reader PASS19 PID/Exit0 bound. Final attempt6 running with scripts/oracle frozen. Ruling: retain migration-ledger and filesystem review package rather than delete them as a Git scratch workspace — there is no authorized Git commit history for this isolated source — cost if wrong: extra retained disk artifacts; removal would destroy provenance. Integration remains the explicitly approved isolated runtime; no Git merge/push menu or primary integration is appropriate.
Attempt6 geometry15/599 PASS but RhinoApp.Exit left owned PID23420 alive after report, title empty; exact PID/exe/start timestamp verified before stopping only that process, preserving failed launch/report. Attempt7 repeats full matrix. Ruling: use established clipboard bootstrap controlled System.Environment.Exit(0) for the isolated Rhino verifier after all output/report streams close — deterministic owned test lifecycle, caller Rhino untouched — cost if wrong: this does not test native Rhino/plugin graceful shutdown hooks; report explicitly records controlled termination. No geometry/assertion/tolerance changed.
Final: Ruling: native Part_Primitives Box is the supported user cutter command in the shared OM9 document — existing FreeCAD command provides actual dimensions, placement and transaction, without importing unrelated main-workspace Solid/Sphere features — cost if wrong: cutter UI requires the Part task panel and does not establish custom Matrix Box-command parity.
Final: Ruling: excluded history/render/materials/textures/Surface CV/fillet/offset remain outside this scoped acceptance — approved modeling scope has explicit unsupported behavior — cost if wrong: workflows requiring those features need future implementation.
Final: Ruling: seven broader full-openNURBS packages remain future work and total batch count unknown — named modeling/application matrices cannot certify every openNURBS type/property — cost if wrong: untested data may retain or reject rather than become editable CAD.
Final: Ruling: command-first Join still requires Phase2 preselection — no new interaction contract was authorized for this continuation — cost if wrong: users must select objects before Join.
Final: Ruling: named analytic Loft/Sweep options evidence scoped workflows rather than universal Rhino parity — advanced option solver coverage is bounded by actual fixtures — cost if wrong: additional shapes/options may reject and require new solver work.
Final: Ruling: completion documents must bind fresh binary, script and output hashes before acceptance — old candidate pass is not final proof after fixes — cost if wrong: stale evidence could misstate support.
Final: Ruling: valid initialized caller-owned native storage remains an FFI precondition — Rust copied/bounded views do not certify arbitrary native pointers or dependencies — cost if wrong: native memory bugs remain possible outside the checked ownership policy.
Final Ruling: finish by retaining the explicitly authorized isolated source/runtime, rather than offer a fabricated Git branch integration menu — parent Git does not represent this OM9 baseline and integration/push were excluded — cost if wrong: primary application is not automatically replaced.
