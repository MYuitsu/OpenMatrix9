# RCORE-01 instance definitions implementation plan

> **For agentic workers:** Use superpowers:executing-plans inline. Fresh read-only review after this integrated change; no implementation delegation.

**Goal:** Complete the missing group/member/subelement portion of the existing
owned instance descriptor, preserving canonical native definitions and world
placement once. This is work toward the full original RCORE-01, not a replacement
completion criterion. Arrays and mesh/cloud owners still need subsequent native
geometry adapters/evidence; unsupported paths must not be silently flattened.

**Architecture:** Reuse native GroupExtension ordered membership, getSubObject
for rigid container/member placement, and getLinkedObject for the actual affine
link path. Existing owned Part support/topology/attributes remain the leaf
adapter. Rust owns/validates a bounded parent-before-child definition graph containing
source stamps, parent occurrence, representation and world frame; repeated
definition identities across separate instance occurrences remain distinct.

**Tech:** Safe Rust model + initialized-buffer C ABI; minimal native C++ reader;
Python only real FreeCAD fixture/bootstrap. Shared SDK/plugin cache only.

**Spec:** OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md,
especially representation/schema/ownership C01/C06/T06/T11. Existing full audit
remains authoritative for every original requirement not covered here.

## Constraints and review focus

Max4096 graph nodes, max64 structural/link/reference depth; stable session stamps
with all five fields nonzero. Parent indices precede children; only group/instance
may own children. Child identity can repeat at distinct occurrences. Native
sources are GUI-thread guarded before/after, including member/reference owners.
No native handle escapes and no Shape bake/copy/transaction is introduced.
Rust frame policy and bounded8MiB atomic JSON ABI are reused. Aggregate serialized
snapshot accounting is bounded, but excludes native/global allocator overhead.

Critical cases: plain vs placed group; definition ancestor outside instance path;
nested links inside groups; root/member subelement selectors and missing paths;
shared identities with different transforms; cycles/oversized trees before
recursive native reads; unknown attributes/unsupported representations remain
explicit; cancellation/source/history must be unchanged by read-only failure.

## Task 1: portable graph contract

Files: rust/src/instance_graph.rs, instance_graph_ffi.rs, lib.rs,
rust/tests/instance_graph.rs; representation11 group in geometry_contract_ffi.
Node184-byte ABI: Stamp40, parent usize, representation/reserved u32, matrix16.
Graph owns Vec<Node>, validates count/parent order/depth/container categories and
finite affine nonsingular frames before clone. Serializer retains stamps as
decimal strings, parent indices, occurrence index, world matrix/sign; empty
group is valid but unknown/malformed parents, leaf children and invalid frames
reject. Count/envelope gate precedes borrowing; invalid/short output is atomic.

- [x] Tests fail with absent graph API/group representation, then valid typed
  graph, shared identities, malformed structure/depth/frame/envelopes pass.

## Task 2: native definition/reference reader

Files: Gui/CoreGeometryContract.cpp, Gui/CoreDefinitionSnapshot.cpp/.h;
tests/rhino_core_instance_definition_smoke.FCMacro.
Extend describe to group roots and group definitions. Preserve ordered nested
members and reuse Part leaf descriptors. A group's native member matrix includes
its rigid placement; Rust removes that embedding once before composing into the
instance path. Child links are resolved with their own native affine path, not
with inherited global definition ancestors. Active traversal path rejects cycles;
global occurrence/count budget includes every copied node. Source guards cover
group memberships, links, targets and member paths.

Preflight link object paths against ordered GroupExtension membership before
native recursion. Capture every intermediate group/member owner. Single and
multiple actual Part subelements use the existing revision-scoped Shape adapter,
without losing selectors. Nested overlapping selection contexts need explicit
handling or a declared unsupported diagnostic, never wrong whole-object data.

- [x] Real host RED proves current group/member/subelement rejection.
- [x] Native GREEN proves ordered groups and member selectors; world points via
  independent native evaluation under rotations/mirror/nonuniform/nested links.
- [x] Live source/definition/sibling and Undo/Redo unchanged; owned tree survives
  source replacement/close; malformed references/depth reject without mutation.

## Task 3: integrated verification and synchronization

- [x] Full Rust tests and only OpenMatrix9Gui plugin build; new host fixture plus
  previous current-plugin world/support/topology/descriptor/import regressions.
- [x] Fresh read-only review; reproduce/fix substantive findings with RED→GREEN.
- [x] Source/runtime/log/report fingerprints, actual process outcomes, original
  full audit, spec/README/ledger/capability follow-up and package MANIFEST agree.

Ruling: continue the existing user-authorized full RCORE-01 implementation inline;
this is an extension of the agreed descriptor/ownership design. No reapproval of
routine native adaptation or rebuilding the shared FreeCAD host is needed.

Ruling: permit a group or instance graph root and retain each one-step native
link target as an occurrence — nested link identity and appearance would be
lost if every link were flattened to the final Part target. A separate native
reader keeps this adaptation out of the Part support reader. The portable graph
requires parent-before-child ordering, rather than imposing an unused strict
contiguous preorder traversal on callers.

Review fix: container metadata must be counted before descending, not only after
the final JSON allocation. The native RED with70 individually valid64KiB label/
Label2 containers reached an unsupported sentinel instead of rejecting the
already-oversized prefix. Count each shallow container header, member separators,
instance reference/placeholder and all graph copies before expansion. The final
serialized size must not exceed the accumulated bound. Native/global allocator
overhead and peak single-node staging remain outside this serialized budget.

Fix verification note: nonconst QJsonObject lookup inserted an unaccounted
definition_geometry:null field when testing a direct group. Independent Qt
measurement of the old valid tree:33181 actual/33183 accounted; the regression
tree:33208 actual. Value lookups remove the extra27 bytes. Keep a separate Qt
graph byte measurement as well, since Rust and Qt JSON encodings are not a
portable assumption. Final native fixture passed48 checks, including the new
oversized-prefix rejection; final matching-plugin regressions passed and were
recorded with source/runtime/report/log fingerprints.

Accepted evidence:48 new+863 regression=911 host checks;624 full Rust tests;
all nine matching selected processes exit0. See docs/validation/2026-10-10-rhino-core-instance-definitions.md. Original RCORE-01 remains in progress.
