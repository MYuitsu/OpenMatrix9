# RCORE-01 instance/group definition review — 2026-10-10

Fresh read-only reviewer: `/root/instance_definitions_review`, scoped to the
instance/group/member/subelement extension and its integration with existing
Part geometry readers. This is not a review certifying full RCORE-01 completion.
The plan and original chapter were supplied; no edits or concurrent native host
runs were delegated. The reviewer found no other substantive transform, member
resolution, source guard, graph structure or ABI issue in this reviewed scope.

## Finding and disposition

**P2, fixed:** Container metadata was omitted from aggregate serialized accounting
until the complete JSON string was allocated. Only Part leaves were counted.
Individually valid64KiB Label and Label2 values on many empty groups could consume
far more than the8MiB snapshot policy before rejection.

The real native RED at
`H:/FreeCAD-src/build/instance-definitions-container-budget-red3-20261010/results.json`
records `Unsupported native definition leaf representation`:70 valid large-label
containers had already exceeded the budget, yet traversal reached the final
unsupported sentinel. The intended budget-before-traversal assertion failed.
The first setup attempt used an unavailable App::Feature type and is not accepted
as a behavioral RED; the final reproducer uses native App::FeaturePython.

The adapter now counts each shallow container header and attributes before
descending, including its instance reference/placeholder, member separators and
every graph copy. The Rust serialized graph size is gated before copying; Qt's
own compact graph serialization is also measured before embedding. The complete
JSON cannot exceed the accumulated bound or the8MiB policy. These checks exclude
native/Qt/global allocator overhead and peak single-node/graph staging.

During the fix, direct group regression exposed27 unaccounted bytes: a nonconst
QJsonObject lookup inserted `definition_geometry:null` while testing whether a
group contained an instance child. An independent Qt accounting probe of the
previous valid tree measured33181 actual bytes against33183 accounted, whereas
the failing tree measured33208. The reader now uses value lookups, and the fixture
requires absence of the spurious instance field on a direct group. The original
hypothesis about floating point spelling did not explain this27-byte difference;
the separate Rust/Qt measurement is retained to avoid relying on equal encodings.

GREEN: `H:/FreeCAD-src/build/instance-definitions-native-final3-20261010`, actual
host exit0 and48 passed checks. The oversized prefix now produces
`Definition serialized payload budget exceeded` before the unsupported sentinel,
with source BRep, membership, placement, object count and history unchanged.
Existing valid direct and instanced group/member/subelement captures still pass.
The matching-plugin regression results are recorded in
[the evidence report](../validation/2026-10-10-rhino-core-instance-definitions.md).

No second review was dispatched; the integrated fix is verified by its native
RED→GREEN and the selected-plugin regressions. Arrays, mesh/cloud owner adapters,
remaining reference contexts and all other original chapter requirements remain
open in the [closure audit](2026-10-10-rcore01-closure-audit.md).

## Implementation rulings

- A group or instance may be the graph root. Parent-before-child order is
  sufficient to prove acyclicity; contiguous preorder is not required by a
  consumer. Native membership order is retained in the actual copied tree.
- Resolve one link step at a time and retain each intermediate identity and
  attributes. Flattening to the terminal Part would lose that information.
- Native GroupExtension membership remains authoritative, including an ordinary
  group member also retained directly by its GeoFeatureGroup. Both real paths
  are separate occurrences, with independent native world checks.

These choices cost a larger occurrence snapshot; the4096-node/64-level and
serialized-payload gates bound the supported record. They do not certify all
definition owners or edit/History/persistence behavior.
