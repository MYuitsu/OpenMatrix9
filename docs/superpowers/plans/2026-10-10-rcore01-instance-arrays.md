# RCORE-01 native instance array implementation plan

> For agentic workers: use superpowers:executing-plans inline with TDD and a
> fresh read-only review after integration. Preserve the original RCORE-01
> chapter and closure audit; this plan extends the existing owned definition
> graph/frame architecture rather than replacing the full completion criteria.

Goal: retain native array occurrences, their revision-scoped indices/identities,
definitions and independently transformed geometry in the existing snapshot.
Architecture: existing group representation holds kind `instance_array`, with
ordered instance members. No new representation tag, kernel, backend or editor.
Rust owns array metadata and validates count/index/stamps/frames before cloning;
C++ synchronously reads native Link APIs and reuses existing capture/readers.
Python authors independent FreeCAD fixtures only. Tech: Rust, Qt/FreeCAD native.
Spec: OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md;
docs/reviews/2026-10-10-rcore01-closure-audit.md.

Ruling: the existing user-authorized Rust/native owned snapshot design applies
unchanged. Native arrays are collections of instance occurrences, compatible
with the existing group/instance graph. `kind=instance_array` and native source
type distinguish arrays from ordinary group definitions; shared source stamps
are not deduplicated. This routine extension does not require a new backend or
design approval. Full attributes/reference authoring and the original gates
remain separate required work; unsupported native states must reject explicitly.

## Global constraints

- Shared SDK H:/FreeCAD-src/build/relWithDebInfo; only OpenMatrix9Gui target,
  existing openmatrix9-history-cage cache and reusable-history-runtime outputs.
- No native source edits, implicit array expansion, flattening or copied runtime.
- Same existing 4096-node/64-depth graph and 8MiB shared serialized support bound.
  Array metadata uses at most2047 elements (one root plus minimum2 nodes/element),
  1MiB retained numeric rows and8MiB serialized data; count gate before native
  collection traversal, staging or FFI slices. Selected rows must remain ordered
  and retain their actual0-based indices. Source/physical element/definition
  stamps remain guarded; virtual elements use array stamp plus native path.
- Rows224 bytes: index u64, element_present/reserved u32, optional element Stamp40,
  definition Stamp40, object_to_world16 doubles. All present stamps nonzero;
  absent element stamp allzero. Checked affine frames; no pointers retained.
- Native/Qt/global allocation and peak staging remain outside these model limits;
  original allocator/worker/mutation/lifetime requirements are not removed.

## Files/interfaces

- rust/src/array_occurrences.rs/.ffi: `Occurrence` repr(C), `Snapshot::capture(total,
  rows)`, bounded JSON and `om9_array_occurrences_counts(total,selected)` /
  `om9_array_occurrences_json(ptr,count,total,out,capacity)`. Generated numeric
  native paths and explicit optional physical element/definition references.
- Gui/CoreDefinitionSnapshot.cpp: array branch before regular instance branch;
  count/owner/selection preflight, native per-element getSubObject matrix and
  guarded one-step getLinkedObject, shared Rust metadata and existing captureAt
  leaves/definitions. Existing group containers hold instance_array data.
- rust/src/lib.rs and Rust/native tests; no FreeCAD root or API tag change.

## Tasks

- [x] Rust RED→GREEN: ordered repeated definition identities and selected indices,
  owned independent values; count/index/order/flags/stamp/matrix rejection before
  cloning; ABI layout/count/null/alignment/output canaries; full Rust suite.
- [x] Native RED→GREEN: ShowElement false/true and LinkTransform false/true,
  individual placements/scales, mirrored/nonuniform root scale, real element
  identities and virtual paths, Part/group definitions, nested arrays/links and
  revision-scoped element selection. Compare world points with independent
  native paths. Exact source/history preservation, sibling isolation, update/
  Undo/Redo/close, count and aggregate serialization rejection without mutation.
- [x] Plugin-only build, fresh source review/fixes and all previous matching
  edge/discrete/cold/instance/world/support/topology/descriptor/import regressions;
  evidence/audit/spec/status/README/ledger/lookup/MANIFEST synchronization.

## Review focus

- Expanded LinkElement placement is already in getSubObject path; cancel it
  exactly once before the existing instance reader adds its placement/scale.
- Virtual array paths resolve directly to the definition; don't invent a native
  element object or mutate ElementCount/ShowElement to simplify capture.
- Source placement may be included depending on LinkTransform; native path and
  inverse source placement must match the existing frame contract, including links
  to array definitions under unrelated external parents.
- Ordered physical/virtual element selection, nested definitions and configured
  subelements must retain actual owner/stamps, not silently inspect whole source.
- Preflight counts/owners/cycles before native recursion; source attributes and
  all child payloads count toward shared serialization before descending further.

Full original chapter completion is unproven until every original requirement,
capability and named fixture has authoritative evidence. This plan adds array
data; it does not reinterpret unknown attributes or certificates as completed.

Accepted evidence:635 full Rust tests;609 array+10 boundary+1937 regression
=2556 host checks;all14 matching processes exit0. See docs/validation/2026-10-10-rhino-core-instance-arrays.md.
Original RCORE-01 remains in progress; full requirements stay in the audit.
