# RCORE-01 topology validation implementation plan

> **For agentic workers:** Use superpowers:executing-plans to implement each
> task with RED/GREEN and matching standalone-plugin evidence.

**Goal:** Implement the original RCORE-01.C04 and adapter item5 geometry/topology
validation and report requirements, retaining the complete chapter objective.

**Architecture:** Reuse owned face/loop/edge-use snapshots. Rust checks the
combinatorial surface complex, including vertex links and face orientation.
C++ reads OCCT validity and solid orientation/shell observations after bounded
native input preflight; Rust combines independently stated facts. Native validity
and combinatorial facts stay distinct, with method/scope recorded in reports.

**Tech stack:** Safe Rust, bounded value-only FFI, existing FreeCAD/OCCT SDK/host.

**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`
representation table, C04, adapter items2/4/5 and standard diagnostics. This
implements the existing user-authorized contract; it introduces no new command,
solver or implicit source repair.

## Global constraints

- Rust owns portable validation and copied values; native objects remain native.
- Keep existing16384-row topology bound; add no unbounded graph traversal.
- Native exceptions stay in C++; initialized live FFI storage remains a caller
  obligation. Do not claim whole-process/native memory safety.
- Use only shared SDK and standalone OpenMatrix9Gui build; no FreeCAD rebuild.
- Inspection/rejection preserves geometry, attributes, placements and history.
- Do not infer a valid solid from native shape type, edge closure or a surface.
- Preserve native inward/outward orientation; mirrored frames must be explicit.
- Record OCCT algorithm validity separately from global geometric embedding or
  mathematical error bounds that have not been established.

## Review focus

Vertex pinches with otherwise manifold/closed edges; seam endpoint germs;
degenerate poles and collapsed loops; opposite face orientations; cavity/multiple
shells and mirrored/nonuniform instances; missing/invalid native observations.

## Task1 — Owned surface-complex validation

- [x] Add literal Rust fixtures for disks, open cylinder, sphere poles, torus,
  vertex pinches, shared-edge branching, closed face orientation and invalid
  input/budgets. Observe meaningful failures before implementation.
- [x] Compute each vertex link from canonical nondegenerate edge-endpoint germs,
  retaining parallel edges/self loops, and require one path or cycle. Ignore
  degenerate edges only while preserving their incident vertex checks.
- [x] Check shared-edge orientation after applying each face's orientation.
- [x] Integrate scalar results into the existing synchronous topology FFI and
  native JSON, keeping scope distinct from kernel validity/solid decisions.
- [x] Native positive/negative fixtures and full matching regressions; review.

## Task2 — Native validity/solid observations and common reports

- [x] Read matching OCCT analyzer/classifier contracts and bounded native shape
  traversal before invoking native validators. Reuse existing APIs.
- [x] Native fixture: box, sphere, cone/torus, reversed solid, hollow/cavity,
  open shell, invalid orientation, vertex-pinched boundary and loose geometry.
  Compare facts independently, preserving source/history.
- [x] Rust owns tri-state combination and report reasons. Native shape type
  remains representation provenance; it does not alone establish solid validity.
- [x] Preserve native and world orientation separately; derive each reframe from
  native observation once, including mirrored links/arrays.
- [x] Full Rust and matching host regressions; fresh read-only review; synchronize
  evidence, chapter/README, ledger/capability lookup and package manifest.

The original closure audit remains authoritative for every other RCORE-01
requirement. Completing either task alone does not complete the chapter.

Accepted inspection evidence:655 Rust;287 new+3096 regression=3383 native checks;
17 matching host exits0. Uniform operation/conversion/commit consumers remain
required by the original chapter; this plan validates inspection reports only.
