# RCORE-01 prepared result snapshots implementation plan

> **For agentic workers:** Use superpowers:executing-plans inline, with meaningful
> RED/GREEN and matching standalone-plugin evidence.

**Goal:** Supply the original GeometrySnapshot fields and complete report field
structure for prepared native CAD results, retaining independently restorable
native geometry and the existing owned support/topology observations.

**Architecture:** Reuse OCCT BRep serialization for the complete private native
master, existing topology/support readers for typed observations, and existing
document unit-context provenance. Rust owns bounded request parsing, unit revision
decoding, representation/admission, result schema and JSON output. Native C++ only
captures native geometry, exact frozen binding attributes and host context.

**Tech stack:** Existing Rust bounded JSON parser/serializer patterns, OCCT
BRepTools V3 ASCII without triangulation, FreeCAD Part/units and FCStd properties.

**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`,
GeometrySnapshot/ConversionReport/GeometryResult and adapter items1–6. Full original
RCORE-01 remains the objective; upstream conversion and other operation families
are not closed by assembling the required result fields.

## Global constraints

- Shared SDK and existing standalone OpenMatrix9Gui cache/runtime only.
- No new solver/repair, mesh fallback or tolerance/default change.
- Master encoding is explicitly native OCCT V3 with matching SDK provenance;
  it is not an openNURBS or cross-SDK preservation certificate.
- Bound native writer, request, parser nodes/depth and report before publication.
- GUI thread/GIL; no native owner crosses Rust ABI; caller initializes bytes and
  destination, allocator/free stay on the same side, no panic/exception crosses.
- Legacy absent unit context remains null revision; explicit contexts retain the
  complete u64 revision as a decimal string and native geometry stays in mm.
- Immutable import evidence remains distinct from validity after later edits.
- Report source attribute coverage; unknown upstream exactness/parameterization
  and topology maps stay explicit rather than inferred from successful copy.

## Review focus

Periodic/rational basis and restricted domains; seam/hole topology; native
placement/orientation and ordered wire structure after detached restore; Unicode
metadata and full-width context revision; bounded serialization; source/Redo
isolation on rejection and report persistence across another host process.

## Task1 — Owned Rust result schema and policy

- [x] Add `rust/src/prepared_result.rs` and focused tests; meaningful RED first.
- [x] Validate native request fields, reuse existing admission and JSON bounds,
  decode units with existing Context, retain unknown provenance and own output.
- [x] Export a synchronous bounded UTF8 JSON ABI; test query/copy/null/capacity.

## Task2 — Native capture and result binding

- [x] Add `Gui/CorePreparedSnapshot.h/.cpp` and CMake entry; bounded stream writer
  preserves native master without display mesh, using existing OCCT serializer.
- [x] `CorePreparedCad.cpp` supplies frozen binding attributes, native private
  shape/topology/validation and document context to Rust result assembly.
- [x] Add a real FreeCAD RED/GREEN fixture for full schema, detached native
  reconstruction, rational/periodic/wire/hole/seam/root representation, exact
  attributes, explicit context, Undo/Redo and warm/cold FCStd reports.
- [x] Full Rust, matching native regressions, source review and original chapter
  progress/checklist/evidence synchronization. Retain every remaining obligation.

Accepted snapshot evidence:665 Rust;107 snapshot/warm-cold+95 metadata+3499
regression=3701 host checks;24 matching actual exits0. One fresh source review;
optional metadata finding reproduced RED and fixed. Cold BoundBox oracle uses
no-mesh detached CAD copies after source-confirmed display-cache differences.
Minor deferred: native writer budget exceptions map to KernelFailure. Full
original chapter obligations remain in the closure audit.
