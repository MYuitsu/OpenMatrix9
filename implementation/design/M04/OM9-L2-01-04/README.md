# M04 `OM9-L2-01-04` — associative dependency rebuild

**Milestone:** M04 Builder and History · **Gate:** AUTO_ONLY · **Evidence:** `implementation/evidence/M04/OM9-L3-01-04-001..009/`

## What was already there

W7 ported the deterministic dependency kernel (`stale_recipes`, `mark_stale`,
`plan_recompute`, `mark_clean`, `mark_failed`, `detach`, `freeze`, `state_counts`) onto
`HistoryGraph`. That kernel is reused unchanged; this batch did not rewrite it.

## What this batch adds

| L3 | Title | Tests | Positive oracle |
|---|---|---:|---|
| `OM9-L3-01-04-001` | Feature Record And Typed Dependency Edges | 4 | 3 nodes, edges follow typed IO; a curve fed into ScalarMm is rejected |
| `OM9-L3-01-04-002` | Feature Provenance From Command | 2 | re-open the exact command, typed inputs and unit preserved |
| `OM9-L3-01-04-003` | Dirty Propagation And Topological Order | 2 | dirty set {A,B,C,D}, producer before consumer, no duplicate evaluation |
| `OM9-L3-01-04-004` | Stable Semantic Reference Linking | 2 | the reference resolves to the same intended edge after the allowed edit; the bound id is unchanged |
| `OM9-L3-01-04-005` | Incremental Recompute And Cache | 2 | only A and B recalculate; X and Y cache entries (revision and sha256) are unchanged |
| `OM9-L3-01-04-006` | Builder Preview Vs Committed Features | 2 | committed document id/geometry/hash unchanged; one Apply yields only the final value |
| `OM9-L3-01-04-007` | History Recompute Failure Rollback | 3 | recompute fails, the previous geometry/published result/cache entry are retained, the feature is Failed(reason) |
| `OM9-L3-01-04-008` | Delete Parent Or Change Input Policy | 3 | child marked broken; the report offers KeepBroken/DetachDependents/RemoveDependents and the chosen policy leaves no dangling reference |
| `OM9-L3-01-04-009` | Associative Rebuild Geometry Fixtures | 4 | deterministic baseline hash across sessions; an edited rebuild matches a fresh build |

The code is `crates/om9-core/src/associative.rs` plus additive fields on
`Recipe` (`input_ports`, `output_ports`) and `HistoryGraph` (`references`), and two
borrow-splitting helpers in `history_ops.rs`. Both new `Recipe` vectors and the reference
map are `serde(default)` and skipped when empty, so an existing document round-trips byte
for byte.

## Decisions worth keeping

1. **A type is declared, never inferred.** `Recipe.input_ports` records what the feature
   asked for. A recipe saved before typed ports is reported `Untyped`; the code never
   looks at today's geometry and calls that the recorded type.
2. **A scalar port is not an object slot.** Binding a curve to a `Length` port is
   `Invalid("object bound to scalar input")`, which is the fixture's "CurveRef into
   ScalarMm" refusal.
3. **Provenance is durable and unit-safe.** A recorded `Value::Number` cannot reopen in
   millimetres, so it is refused when the provenance is read, not guessed on reopen.
4. **A semantic reference needs a unique name, the recorded kind and a fingerprint inside
   tolerance.** A duplicate name is `ambiguous`; movement past tolerance is `moved` and
   requires an explicit `relink_reference`.
5. **The cache key is (revision, parameters, input stamps, outputs).** A change to any of
   them is a miss even while the graph still calls the feature `Clean`.
6. **Preview stores deltas, not a second document.** `PreviewSession::begin` copies only the
   base identity/revision; a parameter edit stages one recipe, preview geometry one entity.
   `commit` applies the staged deltas to the still-current document and refuses if the base
   moved. A preview of one parameter in a large document therefore holds no geometry copy.
7. **A failed rebuild restores a captured snapshot** (recipe, outputs, published results)
   and records `Failed(reason)`; the previous geometry and result stay current. A successful
   rebuild re-stamps its recorded inputs, so its own output does not look stale next time.
8. **A rebuild re-checks the feature's references before it commits.** A produced subshape
   set that drops the name, duplicates it, changes the kind or moves past tolerance turns
   the rebuild into `Failed(...)`; there is no silent relink.
9. **Deleting a parent reports before it chooses.** `delete_source_input` returns the
   broken children with `KeepBroken`/`DetachDependents`/`RemoveDependents`, refuses with
   `Busy` while a child is rebuilding, and the applied policy leaves no reference to the
   deleted object.
10. **A baseline is content-ordered.** Nodes are sorted by `(command, Merkle label over
    content and producer labels)`, so two sessions agree without depending on random
    `HistoryId`s; the hash includes parameters, ports and output geometry.

## Real producer chain (follow-up)

`crates/om9-core/src/producers.rs` supplies the chain and the registry:

- `RingRailProducer` — closed circle centerline at `diameter_mm`;
- `RailProfileProducer` — closed rectangle on the rail start frame (`width_mm`, `height_mm`),
  derived from the rail tangent rather than a hard-coded world plane;
- `RailSweepProducer<B>` — profile swept along the rail through a `SweepBackend`; the
  analytic reference backend is `surface_kernel::sweep1`, the OCCT backend is
  `om9_kernel::associative::OcctSweepBackend` (`creation::pipe_shell`);
- `SnapshotTopology`/`OcctTopology` — names `edge:*`, `face:*`, `vertex:*` (or
  `curve:0`/`face:0`) with anchor/measure fingerprints from the B-Rep snapshot;
- `ProducerRegistry` + `recompute_registered` — one call from a host, and the registry
  validates produced subshapes against the feature's references before commit.

`crates/om9-kernel/tests/associative_history.rs` proves the chain end to end with the real
OCCT sweep: a B-Rep whose source is the OCCT bridge, published edge/face/vertex names, and a
semantic reference that still resolves after a profile edit.

## SDK export (follow-up)

The SDK owns the history read contract (the host keeps the write/transaction side):

- `HostRequest::{HistoryList, HistoryFeature, HistoryDependencies, HistoryProvenance,
  HistoryRebuildPlan}` and the typed `Host` wrappers;
- `FeatureRecordInfo`, `FeatureDependencyInfo`, `FeatureProvenanceInfo`, `FeatureInputInfo`;
- capability names in the host's `capabilities` reply;
- `sdk/examples/hello-plugin` command `HistoryDemo` calls all five through the public SDK.

Recompute itself stays host-internal: it needs the native kernel producer and the host's
atomic transaction/Undo owner, so a plugin gets the plan and the record, not a direct
mutation route.

## Still outside this owner

- The builder UI that calls the chain (live handles, panels, the command frame) belongs to
  the M04 builder owner (`OM9-L2-03-11`, `OM9-L2-04-02`); this owner exposes the headless
  producer and rebuild service it calls.
- A host transaction mutation for a rebuilt history recipe is not added: the application
  owner decides how a recompute is journaled, and the current app replay path still edits
  geometry through `Mutation::Geometry`.

## Re-run

```text
python tools/record_m04_l2_01_04_evidence.py --workspace-log build/m04-l2-01-04-evidence/workspace-tests.log
python tools/record_m04_l2_01_04_followup.py
python tools/om9_milestone_dashboard.py
```
