# RCORE-01 native reference contexts

Original objective remains the complete RCORE-01 chapter. This task extends the
existing owned definition graph; it does not replace the original closure audit.

Rust owns bounded UTF-8 path syntax, byte offsets, decimal/collapsed-array index
classification and ABI validation. C++ is necessary for live FreeCAD group,
Link/LinkElement/array membership, suppression, configured reference paths and
native placement queries. It retains no native pointers in Rust. Python is a
native host fixture only. Reuse all existing geometry readers and source guards.

The path grammar and budgets are OM9/FreeCAD decisions, not recovered Rhino
syntax. Rhino 5 Block help documents definition/instance identity; it does not
specify FreeCAD member paths. Keep configured topology traversal and ambiguous
overlapping selections explicitly unsupported until validated.

- [x] Reproduce missing references on the prior native runtime; Rust parser RED
  then GREEN with byte/count/numeric/ABI/output-canary checks.
- [x] Integrate bounded group/link/array preflight, optional explicit definition
  names/labels, chained links, collapsed aliases and unindexed member fallback.
- [x] Reproduce and fix physical alias retargeting, count-dependent alias
  fallback, direct definition context, repeated prefix and suppressed traversal.
- [x] Verify configured occurrence-only selection and suppressed final topology
  selection; finish review and rerun matching plugin regression suite.
- [x] Synchronize evidence, chapter, README, ledger, capability lookup and package
  manifest, preserving whole-chapter in-progress status.

Remaining chapter obligations are tracked in
[the original audit](../../reviews/2026-10-10-rcore01-closure-audit.md).

Accepted evidence:641 Rust;540 new+2556 regression=3096 native checks;15 matching
host exits0. Full original chapter remains in progress.
