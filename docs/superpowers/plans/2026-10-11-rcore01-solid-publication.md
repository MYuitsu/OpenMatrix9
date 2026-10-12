# RCORE-01 Solid publication implementation plan

> **For agentic workers:** Use superpowers:executing-plans inline, task by task.

**Goal:** Verify and repair enabled Box/Sphere publication against the original
RCORE-01 source, atomic transaction, diagnostic and temporary ownership contracts.
This plan does not replace or complete the rest of the chapter.

**Architecture:** Reuse the Rust solid session/admission, existing FreeCAD Part
solver and captured SolidCurveReference/EditInput snapshots. C++ changes are
limited to necessary GUI-thread native transaction, reference and error adapters.
Python/pivy and the existing native fault probe are test infrastructure only.

**Spec:** [01-object-model-kernel.md](../../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md).

## Constraints

- Preserve all original geometry families, consumers, errors, adapters1–7,
  C01–C12 and T01–T11. Full GeometryResult/conversion/maps and remaining lifecycle
  requirements stay required regardless of this task's test results.
- Reuse the single SDK and standalone OpenMatrix9Gui build/cache. Use the
  separately pinned App19/camera-guard GUI test launcher; do not replace SDK DLLs.
- Retain independent historical evidence and create fresh artifact directories.
- GUI/native tests run serially. Do not close the user's FreeCAD application.

## Task: native publication contract

- [x] Add tests/rhino_core_solid_publication_smoke.FCMacro using real Box,
  Sphere Center and Sphere AroundCurve commands. Inject existing native before
  and close transaction failures; mutate the actual AroundCurve input in the
  output-created callback. Compare source Shape/Placement/metadata, native
  wrapper identity and Undo; execute preserved prior Redo after early rejection.
  Include precommit stale rejection, cancellation and positive output controls.
- [x] Run the unchanged Runtime25 on the pinned native test host. Record actual
  failures, selected DLL/module hashes and terminal exits before changing code.
- [x] Repair only demonstrated gaps in Gui/SolidController.cpp using existing
  preserving-Redo API and verifySolidReferences; freeze diagnostic source names
  before publication and contain native errors before they can escape Qt.
- [x] Build the standalone plugin into a fresh runtime. Re-run the exact fixture,
  relevant solid command/options/admission regressions and affected publication
  callers. Compare source/runtime fingerprints; no unsupported success claims.
- [x] Request independent source/evidence review; address actionable findings.
- [x] Synchronize the chapter, implementation status, root/group README,
  docs/features/RCORE-01.md, progress ledger and664-entry manifest. Run the public
  source audit. Keep RCORE-01 in_progress while any original requirement is open.

## Verified checkpoint — 2026-10-11T17:17:29+07:00

Native publication and strengthened callback guards are verified within the scoped fixtures. Runtime33 Solid669/6, Runtime32 unaffected caller703/8, source-identical full Rust811. Historical false-positive BRep candidates and incomplete attempts remain excluded. All original chapter requirements remain required; full Solid GeometryResult and actual Solid preview leases are next. [Evidence](../../validation/2026-10-11-rhino-core-solid-publication.md). Public source audit and document verifier must pass before reporting this checkpoint.
