# RCORE-01 native reference contexts — 2026-10-10

**Scope:** Bounded owned native group/link/array reference paths, explicit definition names/labels, collapsed alias fallback and exact-once frames using existing geometry readers. Full original RCORE-01 remains in progress.

Rust owns UTF-8 byte-offset path plans, names/labels, checked decimal indices and
collapsed-array aliases. It validates4096 bytes/64 steps and2047-element bounds
before cloning or native numeric parsing. The path-step/summary/alias ABI uses
16-byte structs. The array mode flag is validated; physical names/labels never
use collapsed naming heuristics. Pointer/address/capacity/alignment envelopes
reject before borrowing; short/invalid path output leaves canaries unchanged.
Initialized live immutable input and live nonoverlapping writable output remain
caller obligations. No native pointer is retained in Rust.

C++ is necessary for FreeCAD group membership, linked targets/configured paths,
LinkElement suppression and native matrix/identity APIs. It reuses source
guards, owned definition graph and existing Part/support/edge/discrete readers.
Depth/count bounds precede native traversal. Explicit intermediate link and
definition tokens, label paths, numeric/physical/collapsed element names and
unindexed occurrence0 member paths retain native frames. A bounded collapsed
alias outside the actual count falls back to a definition/member name, while
numeric indices and overflowing aliases still reject. Explicit definition
normalization happens once. Original selected paths and suppressed topology
traversal are checked before exposing geometry; terminal configured occurrence
selection retains ordered Face1/Face3 collections.

The grammar/budgets are OM9/FreeCAD decisions, not Rhino-compatible path syntax.
Rhino5 [Block help](https://docs.mcneel.com/rhino/5/help/en-us/commands/block.htm)
provides definition/instance context, not this native path grammar.

**641 full Rust tests and3096 native checks passed, all15 matching host exits0.**
540 reference checks plus2556 previous array/boundary/edge/discrete/cold/instance/
world/support/topology/descriptor/import checks share plugin SHA256
`7124d4e8aa7ed42867316f6d4ffb7ad68ecafdf41d9f55f5e1fb14bf7da21f45`. The shared host SHA256 stayed unchanged. Only standalone
OpenMatrix9Gui was rebuilt with the existing SDK/cache and14 workers.

Independent native path/link matrices, distinct occurrence placements and
actual point vertices prove target identity and world XYZ, including both
LinkTransform settings, mirrored/nonuniform root transforms, configured member
paths and direct/group/link/physical/virtual array selection. Source geometry,
placements, labels, link/array properties and Undo/Redo remain unchanged by
inspection/rejection. Meaningful RED reports reproduced missing context,
physical label retargeting, count-dependent fallback, direct explicit names,
double normalization, suppressed object/topology traversal and configured
occurrence-only selection. Initial collision-fixture failures were invalid
authoring: native label uniqueness renamed the requested duplicate label.
Renaming element1 first created the intended native label collision.

[Machine evidence](2026-10-10-rhino-core-reference-contexts.json) records current
source/dependency/runtime/harness/report fingerprints and verification counts.
[Read-only review](../reviews/2026-10-10-reference-contexts-review.md) records the
reproductions and corrections. Existing OCCT deprecation and Rust cache/path
warnings remain in logs; they do not certify or contradict global memory safety.

**Original RCORE-01 remains in progress.** Configured topology traversal and
overlapping selections are still explicitly unsupported. Remaining reference
ownership/cycles, attributes/channels, global topology/certificates/reports and
every original allocation/worker/commit/lifecycle obligation remain open in the
[closure audit](../reviews/2026-10-10-rcore01-closure-audit.md).
