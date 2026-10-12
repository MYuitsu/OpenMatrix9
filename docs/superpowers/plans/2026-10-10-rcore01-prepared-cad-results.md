# RCORE-01 prepared CAD result contract implementation plan

> **For agentic workers:** Use superpowers:executing-plans inline, with meaningful
> RED/GREEN and matching standalone-plugin evidence.

**Goal:** Integrate the original geometry/topology validation and Rust commit/
reject requirement into prepared CAD import and preserved affine preview rows.

**Architecture:** Reuse owned topology and OCCT validation reports. Rust admits
native CAD results only after inspected native validity, with the complete
solid prerequisites for native Solid roots. C++ freezes plain metadata and
deep-copies native CAD before the transaction; Python only binds the native
generated immutable report property through the existing host adapter.

**Tech stack:** Safe Rust scalar policy, bounded C++ Python/Part/OCCT adapter,
existing FreeCAD transaction and FCStd property serialization.

**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`,
adapter items3–6, GeometryResult/ConversionReport schema and preview/ownership/
error/Undo invariants. The full original chapter remains the objective.

## Global constraints

- Shared SDK `H:/FreeCAD-src/build/relWithDebInfo`; standalone OpenMatrix9Gui.
- No new solver, implicit repair, mesh fallback or tolerance change.
- GUI thread/GIL only; exceptions/owners do not cross Rust ABI.
- Gate native input before deep copy/evaluation; cumulative rows/depth/metadata
  and geometry-node/basis budgets remain bounded.
- Keep native shape provenance separate from valid-solid admission.
- Reports identify operation/input and are immutable import evidence, not live
  validity after later edits. Upstream archive conversion remains uninspected
  unless an existing converter supplies independently verified evidence.
- Rejection precedes transaction; source, Redo and prepared input stay unchanged.

## Review focus

Invalid native Solid despite type tag; incomplete observations; compound/sheet
policy; forged reports; metadata/container callbacks; observer mutation of
prepared inputs; invalid second row or affine preview; exact Undo/Redo and cold
FCStd report persistence.

## Task1 — Rust result admission

- [x] Add native-valid/invalid/unknown and valid-solid/non-solid scalar fixtures,
  malformed observation rejection and meaningful RED→GREEN.
- [x] Reuse shape-validation Summary and versioned working-CAD registry;
  keep inspect permission distinct from commit permission and typed error codes.

## Task2 — Native owned preparation and report binding

- [x] Add `Gui/CorePreparedCad.h/.cpp`; clone only exact plain metadata values
  under bounded traversal, retaining immutable primitives. Reject custom values,
  cycles, bad numbers and unsupported containers before native publication.
- [x] Reuse input/topology/shape validation, Rust admission and deep native copy
  for all CAD host rows and preserved preview rows, before transaction.
- [x] `ThreeDmPython.cpp` commits the private prepared snapshot, maps failures to
  owned GeometryResult diagnostics and preserves native exception boundaries.
- [x] `ThreeDm.py` binds native-generated OM9GeometryResult string property in
  the same existing transaction. This is API glue, not Python validation policy.
- [x] Native negative/positive/forgery/observer/Undo/Redo/cold-persistence fixtures;
  focused/full Rust, matching host regressions, read-only review and doc sync.

Original upstream conversion, remaining operation families, mesh/cloud/retained
result contracts and every other chapter requirement remain in the closure audit.

Accepted prepared CAD evidence:660 Rust;154 new+3434 regression=3588 host checks;
22 matching exits0. Required metadata/schema and identity findings reproduced
RED and fixed; readable binding reasons preserved after regression failure.
Original full result snapshots/conversion, non-CAD and all remaining chapter
requirements remain in the closure audit.
