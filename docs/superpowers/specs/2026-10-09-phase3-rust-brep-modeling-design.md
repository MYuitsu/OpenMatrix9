# Phase 3 — Rust-owned modeling on imported surfaces, BRep and solids

Date: 2026-10-09. Status: **accepted_scoped;18 requirement groups verified in isolated source/runtime**.

Binding scope: `2026-10-08-rhino-modeling-five-phase-design.md`, Phase 3, its application acceptance amendment, and `../plans/2026-10-08-rhino-modeling-phase-3-brep-modeling.md`. The user now requests complete Phase 3. This document updates the execution architecture for the subsequently adopted Rust-first rule without reducing P3.1–P3.3.

## Outcome and acceptance

Rhino 5 geometry imported or pasted into OM9 remains native CAD. Users can construct Loft/Sweep from its curves, Trim/Join its open faces, Explode a polysurface and BooleanDifference its closed solids. They can construct a cutter and a new surface detail, undo/redo, save a self-contained FCStd after deleting the source 3DM, and export the current modified and new selection. Actual Rhino 5 opens, saves and exports the result; OM9 rereads those exact files and checks geometry against independent oracles.

History, rendering, materials and textures remain outside the requested modeling workflow. Surface CV, fillet and offset have separate capability entries and remain disabled unless their actual adapters and matching proof exist. This does not exempt any promised Loft/Sweep/Trim/Join/Explode/Boolean workflow, trim fidelity, failure behavior or application gate.

## Historical pre-implementation evidence and gaps

- Accepted optimized Phase 1/2 baseline: source `H:/FreeCAD-src/build/om9-perf-dev`, runtime `H:/FreeCAD-src/build/om9-perf-sdk`. Its full user replay `4f741966ee8541e096b4a4f80a90dc4f` passed 14 fixtures, 621 Rhino assertions, 38 owned OM9 reports / 770 assertions and 175 saved reread assertions. Phase 3 is not implied by that result.
- That source contains the exchange converters and native curve editor but no SurfaceController/EditController integration. Its `tests/run_modeling_phase.ps1` explicitly refuses phases above 2.
- `H:/FreeCAD-src/Mod/OpenMatrix9` contains Rust surface/edit sessions and native controllers/adapters, including Sweep1/Sweep2/Loft and Join/Explode/Trim/solid Booleans. Source presence and historical tests do not prove they work with the accepted exchange runtime.
- `SurfaceGeometry::surfaceWire` applies a parent placement to a current shape, but the surface input structure stores object/subelement references, not a captured signature. Surface preview/commit requires a coupled stale-input check in the integrated runtime.
- The source controllers depend on SurfaceLoft, SurfaceSeams, SurfaceRefit, SurfaceConstraints, EditSpecialTypes/EditMesh and history adapters. Integration must trace these dependencies selectively; copying only the four controller/geometry files cannot establish a working build.
- Main-checkout `EditGeometry::editInput` currently admits surface-only inputs for Boolean commands. The Phase 3 binding plan explicitly requires rejection of open-shell Boolean. Rust capability policy must enforce that restriction for this modeling workflow before calling the reused adapter; the existing permissive source is not evidence of the promised behavior.
- Root discovery currently resolves the candidate to the parent FreeCAD Git repository, rather than a self-contained OM9 worktree. Do not mistake parent Git status for a clean OM9 source baseline or repair old Git metadata implicitly.

## Considered approaches

1. **Recommended: integrate the existing Rust sessions and native adapters selectively, with a Rust-owned Phase 3 request/capability/stale-check layer.** This retains command IDs/options, the accepted curve editor and current-geometry exchange. Kernel geometry stays in FreeCAD/OCCT. Each reused path receives new imported-geometry and application proof.
2. Replace the entire accepted source with the main checkout. This introduces unrelated differences and may lose the accepted Rust snap/editor and clipboard behavior; it is unsuitable for a requirement-bound Phase 3 acceptance.
3. Create a separate Python modeling workflow. This duplicates user commands and portable policy and conflicts with the Rust-first requirement; it is unsuitable for the requested product implementation.

## Source, runtime and isolation

Prepare `H:/FreeCAD-src/build/om9-phase3-dev` from the accepted optimized source, excluding generated build/Cargo target trees, preserving reference locations and recording a sorted SHA-256 source inventory. Build a separate module/native suite and runtime at `om9-phase3-module`, `om9-phase3-native` and `om9-phase3-sdk`. This extends the already used isolated source/runtime pattern; do not mutate either accepted Phase 1/2 runtime, the user's live Rhino document or the main source.

If a native managed worktree can safely represent the actual OM9 source, reuse it; otherwise record the filesystem-isolation ruling and retain source snapshots and exact diffs. Do not commit, push, publish or repair Git metadata in this work. Source isolation is preparation, not a claim of Git commit provenance.

## Ownership and interfaces

### Safe Rust

- Reuse `surface.rs` and `edit.rs` command aliases, ordering, session phases and numeric algorithms after comparison with the selected feature contracts.
- Own operation classification, input cardinality, allowed type combinations, numeric options, ordered selections, cancel/commit transitions and stale-request decisions.
- Add typed BRep capability facts: point/curve/face/open shell/closed solid/mesh/cloud/retained, topology validity, read-only/preservation protection, and operation-specific support/reason. A CAD display mesh is not a mesh input. Closed-shell detection comes from the native kernel, never object names.
- Own immutable identities/signatures/options for a modeling request. Surface preview and final commit use the same validated request generation. Geometry, placement, ancestor placement, subelement, document lifecycle or relevant input availability changes invalidate it.
- Resolve Join command dispatch by actual classified input: preserve accepted Phase 2 curve Join; route faces/shells to the Phase 3 surface Join adapter. Mixed/unsupported sets fail before mutation.
- Keep worker policy at `max(1, floor(0.60 * logical CPU))`, limited by independent tasks and RAM. No mutable document or widget pointer enters workers. Do not invent unsafe parallel kernel export without separate proof.

### Native C++/Qt/FreeCAD/OCCT/openNURBS adapters

- Read topology and current world-space shape/subelement on the GUI thread, including nested parent placement once. Capture native facts and signatures; send independent values to Rust.
- Use existing typed kernel adapters to build native results, including holes, seam/pole/cavity topology. C++ geometry operations whose API is exposed through FreeCAD Part Python objects may call that required native binding while holding the GIL; this is an API bridge, not new Python business logic.
- Retain Qt widgets/event filters and render preview shapes. Rust holds session decisions; widgets only present and submit options.
- Before any document mutation, compare current snapshots with the request, validate result topology, reject empty/null/invalid results and recheck read-only/task/document state. Commit one FreeCAD transaction; on failure abort it and retain every input.
- Preserve the existing command names and all paths: menu, CMD and mouse call the same controller. Copy/Paste and selected export consume current result shapes rather than original archive snapshots.
- Native FFI and dependencies are not certified memory-safe by their Rust callers. Confine unsafe pointer/slice/string adapters; document lifetimes, bounded counts, allocation/free ownership, panic/error handling and GUI-thread requirements.

### Test and bootstrap exception

Rhino 5 IronPython scripts, FreeCAD FCMacro host tests and PowerShell process/build orchestration remain test/bootstrap glue because their applications expose those APIs. No new portable business logic or heavy geometry loops are implemented in Python. These exceptions are recorded in the requirement/evidence ledger.

## Geometry and command behavior

- Reuse converters until a named independent fixture reproduces a fidelity error. Do not rewrite a passing converter.
- P3.1 covers a planar face with hole, capped cylinder seam, sphere poles, torus, cavity solid and an open shell. Check native validity, topology, analytic bounds/mass properties and coupled UV/pcurve/basis where applicable; a mesh, object count or CRC cannot establish CAD fidelity.
- BooleanDifference accepts supported valid closed solids. Open shells are not silently converted into solids. Disjoint sets follow the selected existing command contract; empty/null/invalid results and kernel failures abort without mutation. Do not introduce a new rejection rule merely because a valid Difference leaves the minuend unchanged.
- P3.2 covers imported rational profile/rail curves for Loft and Sweep, imported open faces for Trim and Join, imported polysurface for Explode and imported closed solid for Difference. Preserve selections and original command options. Any additional already implemented Boolean operations activated by integration also require matching regressions.
- Keep surface trim semantics coupled to actual surface selection, not a curve-only Trim implementation counted as face Trim. Complementary regions, holes and singular trims receive named expected results.
- P3.3 uses an independently redistributable analytical ring/representative torus as the baseline. The user's private ring may supplement it but cannot be the only release oracle.
- Cutter construction uses actual native CAD snap through the accepted Phase 2 path. Make the expected cut identifiable by geometry/topology. Construct a new Loft/Sweep detail from imported and newly drawn inputs; export both the modified ring and new geometry with explicit selection.

## Required verification matrix

| ID | Requirement | Authoritative evidence |
|---|---|---|
| P3.1-A | Hole/seam/pole/torus/cavity import stays valid native CAD | New native fixture suite plus actual OM9 lifecycle assertions |
| P3.1-B | Open-shell Boolean rejected, invalid topology causes no mutation | Rust capability tests and host object/transaction snapshot checks |
| P3.1-C | Edited cylinder keeps common UV basis under knot insertion | Coupled surface/pcurve native regression and V5 reread |
| P3.2-A | Imported rational curves drive Loft/Sweep | Rust session/options tests, native adapter and actual command-result geometry |
| P3.2-B | Imported open faces Trim/Join, polysurface Explode | Named face-region/topology oracles and host command tests |
| P3.2-C | Imported closed solids BooleanDifference | Independent expected volume/topology/section checks |
| P3.2-D | Menu/CMD/mouse use the same implementation | Actual host command and event-path assertions, not dialog-open checks |
| P3.2-E | Read-only/task/stale/Cancel fails without mutation | Native host snapshots for each trigger; Rust state tests |
| P3.2-F | Nested transformed inputs apply global placement once | Nested App::Part fixtures and independent world-space geometry |
| P3.2-G | Failed or empty kernel result retains inputs | Injected/real failing native operations with document snapshot proof |
| P3.2-H | Operation capability is truthful, CAD tessellation stays CAD | Rust classification tests and poisoned mesh/cloud accessors |
| P3.3-A | Snap → cutter → cut → new Loft/Sweep detail | End-to-end actual native command workflow with independent oracle |
| P3.3-B | Undo/Redo every modeling step | Actual document geometry and transaction lifecycle assertions |
| P3.3-C | Delete source 3DM, save/close/reopen FCStd | New process opens self-contained file with expected geometry |
| P3.3-D | Selected V5 export includes edited and new current shapes | Exact selection, native reread, no source snapshot resurrection |
| APP-GATE-P3 | Actual Rhino5 Open/SaveAs/Export Selected then OM9 reread | Owned isolated Rhino process, exact source/runtime/module/fixture/output hashes and corresponding host result |
| REG-P12 | Preserve accepted Phase 1/2 and worker/snap/clipboard rules | Fresh scoped regression on the final matching Phase 3 binary |
| REVIEW-P3 | Resolve important findings, audit every requirement | Independent whole-change review and explicit requirement-to-evidence audit |

Bounds tolerance for the ring remains 0.001 mm. Area/volume tolerance remains `max(0.001, 0.0001 * abs(reference))`; analytical fixtures have their explicitly justified independent tolerances. Do not relax a check to make an implementation pass.

## Evidence, progress and delivery

The plan continues through P3.1, P3.2 and P3.3; no smaller subset closes Phase 3. Preserve failures and reproducible RED→GREEN regressions. A reused passing converter is recorded as a verified baseline rather than creating a fake failure.

Publish a phase-specific source/runtime manifest, requirement ledger, native/Rust/host results, actual Rhino output and matching OM9 reread reports. Bind owned PIDs and exit codes, fixture/output hashes, module hashes and source immutability. Historical reports are context only.

Provide one user-invokable Rhino 5 verification entrypoint after integration, preserving the user's open document and logging command timings. A verifier passing fewer requirements cannot certify the phase.

Update progress, roadmap JSON/Markdown, phase acceptance and README consistently. Record prepared batches only when executable fixtures/gates are actually ready. Seven broader full-openNURBS packages remain open and the total future batch count stays unknown until independently enumerated. Phase 3 completion must not be represented as full openNURBS completion.

## Review checkpoint

Self-review: all three original tasks and the original Review Focus are included; Rust-first ownership is explicit; native/API exceptions are stated; each promised operation has a matching test scope; accepted baselines remain immutable. No product code or new runtime has been changed by writing this proposal.

User authorized inline implementation with `okie triển khai khôi`. Continue the approved plan through all 18 requirement groups; no additional approval loop. Execution evidence is recorded in `migration-ledger/progress.md` and the Phase3 application reports.

Final checkpoint: ../../validation/modeling-phase-3/summary.json and requirements.json supersede historical gaps for this scoped runtime.
