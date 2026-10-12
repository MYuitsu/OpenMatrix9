# RCORE-01 owned instance arrays — 2026-10-10

**Scope:** Rust-owned ordered native instance-array occurrences, indices/identities/member paths and exact-once world frames with existing Part/group/mesh/cloud definition readers. Full original RCORE-01 remains in progress.

Native arrays reuse FreeCAD Link/PlacementList/LinkElement APIs and the existing
group/instance definition graph. Root representation stays `group`, with
`geometry_data.kind=instance_array` and ordered instance members. Rust owns
actual zero-based indices, generated numeric native paths, optional physical
element stamps, definition stamps and affine frames. Virtual members use the
array owner stamp plus a revision-scoped native path; these are not persistent
topological names. No expansion, source baking, new solver or editor occurs.

`Link::isLink()` is false for arrays with ElementCount>0; the router now also
detects the native LinkBaseExtension and uses the existing definition reader.
Selected elements retain their original ordinal. Physical elements use the
existing instance branch; virtual members preserve the bounded native member
path. Placement, per-element scales and LinkTransform follow independently
queried native matrices. Existing Part/group/support/mesh/cloud readers are
reused and source/physical-element/definition identities remain guarded.

Count/index/order/boolean/stamps/affine values validate before Rust cloning.
Occurrence ABI is224 bytes, align8, frame offset96. Null/misaligned/count and
address envelopes reject before slice creation; invalid values and short output
preserve output canaries. Model limits are2047 total elements and1MiB retained
numeric metadata, within the shared4096-node/64-depth graph and8MiB serialized
payload. Cumulative minimum nodes and required depth reject before staging native
elements/paths. Shared accounting includes attributes, metadata, member paths,
children and the final Qt-serialized graph. These model bounds do not certify
native/Qt/process peak allocations or initialized FFI pointer lifetimes.

**635 full Rust tests and 2556 FreeCAD checks passed**, all14
matching host exits0:609 array checks,10 boundary checks and1937 previous
edge/discrete/cold/instance/world/support/topology/descriptor/import regressions.
Only OpenMatrix9Gui was built using the existing shared SDK/cache,14 workers;
the FreeCAD host stayed unchanged. Plugin SHA256:`898d6863cb232fee686d8dcc2d287daeac19ee0000a315150f0712203f163163`.
[Machine evidence](2026-10-10-rhino-core-instance-arrays.json) records source,
native dependency, harness, log, report and runtime fingerprints.

The native fixture covers physical/virtual elements with both LinkTransform
settings, individual rotations/scales, mirrored nonuniform root transforms,
group definitions, nested arrays, regular links to arrays, selected indices and
native element names, configured Face1/Face3 collections and eight mesh/cloud
mode combinations. Independent native path/vertex queries and authored raw
discrete points/facets check exact-once frames and source identities. Inspection
and count/selector/singular/serialized rejection preserve native geometry,
placements, labels, collections and history. Moving one occurrence leaves a
sibling array and canonical definition unchanged; Undo/Redo and source
replacement/close exercise owned snapshot lifetime.

The boundary fixture reproduced the two reviewer findings before correction:
virtual member path lost while physical retained; cumulative/depth limits reached
malformed native reference lookup instead of rejecting first. After correction,
both modes retain the path, and a4091-node prefix plus3-element array and an
array at depth63 reject before native member lookup. The serialized fixture's
initial zero scale at element20 was a test-data fault; explicit positive scales
now prove the separate count-valid aggregate serialized rejection.
The [fresh read-only review](../reviews/2026-10-10-instance-arrays-review.md)
found no remaining actionable source defect in this increment. Existing SDK
OCCT deprecation warnings remain in build logs.

**Full original RCORE-01 remains in progress.** Remaining reference contexts,
array ownership/cycle fixtures, attributes/discrete channels, global topology/
parameter certificates, reports and all original allocation/worker/commit/
lifecycle gates remain in the [closure audit](../reviews/2026-10-10-rcore01-closure-audit.md).
