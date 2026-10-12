# RCORE-01 instance arrays review — 2026-10-10

Fresh read-only review covered `array_occurrences.rs`, its FFI and tests,
`CoreDefinitionSnapshot.cpp`, routing and both native array fixtures.

Two actionable findings were reproduced on the current shared host:

- Virtual occurrences discarded a nonempty native member path, while physical
  elements retained it. The member fixture failed only the virtual path check;
  definition identity and source/history preservation passed in both modes.
- Standalone array count validation omitted nodes already in the graph and the
  minimum child depth. A4091-node prefix followed by a3-element array reached a
  missing member lookup; an array at depth63 reached reference-depth traversal.
  Both must reject before those native lookups. The boundary fixture recorded
  these specific late errors with an unchanged document.

The bridge now retains the already bounded member path in each pending entry,
publishes/accounts it in virtual instance headers and checks existing nodes plus
one array root and at least two nodes per selected occurrence, at depth+2,
before native element/path staging. Rust owns ordered indices, optional physical
element stamps, definition stamps and affine occurrence frames as before.

The fresh post-fix review found no remaining actionable defect in this increment.
The expanded fixture includes eight mesh/cloud combinations (both ShowElement
and LinkTransform modes), independent raw-data/native-frame comparisons and
unchanged native sources. A serialized-overflow fixture initially hit an authored
zero scale at element20; explicit positive scales now isolate the budget case.

Review is source evidence, not a claim of runtime success. Matching final
build/host outcomes and fingerprints are in
[validation](../validation/2026-10-10-rhino-core-instance-arrays.md).
Malformed physical ownership and array cycles still lack dedicated native
fixtures. Remaining reference contexts, arbitrary attributes, global allocation,
workers, reports and all original RCORE-01 requirements remain in the closure
audit; this review does not establish full chapter completion.
