# Prepared-CAD import commit gate review

Independent reviewer: `/root/import_commit_gate_review`, fresh context,
read-only review of the new `ThreeDmPython.cpp` gate and host fixture. This
reviews the bounded change, not the entire RCORE-01 chapter or branch.

## Findings and disposition

- P1: a list/tuple subclass can override `__len__`; `PySequence_Size` followed
  by unchecked `GET_ITEM` can index past actual storage. Fixed: accept exact
  built-in pair containers, then use their intrinsic native storage length.
- P2: overridden list iteration or dictionary containment can make the Python
  binder consume different CAD than native raw inspection preflighted. Fixed:
  exact built-in rows, pairs, metadata, archive and preview dictionaries.

The safe runtime RED uses a list subclass holding a valid native curve while
its iterator yields an out-of-policy native curve. It bypassed the preflight.
The pair regression uses three actual items and a reported virtual length of
two; the intentionally empty/crashing baseline case is not executed.
GREEN requires specific plain-container rejection and unchanged complete
document/object/view property payloads and Undo/Redo for all three overrides.
See [runtime evidence](../validation/2026-10-10-rhino-core-import-commit-gate.md).

## Review conclusions retained

For ordinary built-in containers, the reviewed ordering is correct: GUI-thread
rejection precedes document lookup; prepared `host_geometry` and CAD
`host_previews` pass shared Rust/native basis traversal before BRepCheck and
transaction opening. Wrappers stay borrowed synchronously under the GIL;
preflight exceptions become Python errors before mutation.

Ruling: reject custom prepared container subclasses instead of interpreting
virtual iteration/length/dictionary callbacks — supported import preparation
already returns ordinary built-ins, and raw native indexing must inspect the
same storage the binder consumes — cost if wrong: a caller using a custom
container must convert it to plain containers before calling the import API.

The input contract still requires live immutable native owners for the
synchronous call. Full metadata/type validation, aggregate batch/worker memory,
raw archive decoder allocation and arbitrary callback/native owner lifetimes
are required open work. The reviewer did not certify those requirements or
whole RCORE-01 completion.
