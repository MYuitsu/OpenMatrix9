# RCORE-01 instance/group definitions — 2026-10-10

The existing owned geometry descriptor now retains ordered App::Part and plain
GroupExtension definitions, empty groups, nested one-step App::Link identities,
member object paths and single/multiple Part subelements. Shared native identities
are separate occurrences with distinct checked world matrices and attributes.
Part support/topology/attribute readers are reused; no solver or source definition
is recreated and no native Shape bake or document transaction is introduced.

Native getSubObject supplies rigid group/member frames; getLinkedObject(false)
supplies one native affine link step. Rust checked composition removes the native
source placement once. The external ancestor of a canonical definition is omitted
when reached through an instance, while direct group inspection includes its real
external ancestors. Intermediate nested links remain separate descriptors and
retain their own identity and appearance. Group representation11 is distinct from
instance10; solid facts stay unknown on containers, on their inspected Part leaves
where applicable. Multiple selected subelements form an ordered group collection.

Rust owns a bounded graph of revision-scoped occurrences:4096 nodes,64 levels,
nonzero stamps, parent-before-child order, container-only parents, exactly one
child per instance and finite nonsingular affine frames. Empty groups are valid.
The184-byte node ABI validates count/alignment/address envelopes before borrowing
initialized live input; output queries and short/invalid buffers are atomic.
Native reads are synchronous on the GUI thread, with guarded source identities,
memberships, path owners and a final source revalidation. No native handle escapes.
The aggregate8MiB serialized bound excludes allocator overhead and peak/native
staging. This is not a global allocation or memory-safety certificate.

**624 full Rust tests and 911 FreeCAD checks passed** on the same
selected plugin, all nine actual host process exits0:48 instance definitions,
85 world/attributes,91 derived,76 Bezier,174 analytic,54 NURBS,177 topology,
151 descriptor and55 import-gate checks. Plugin SHA256:`7ef1af43f3e11a54aa1a7807b77f03c689f381143c9c36496fa31b319347fd75`.
[Machine evidence](2026-10-10-rhino-core-instance-definitions.json) records the
source/native/runtime/report/log hashes and actual process outcomes.

The old plugin RED rejected an App::Part definition. Rust graph/group tests also
failed before implementation. Independent native curve and plane evaluation now
checks nested rotations, mirror/nonuniform scale, all four direct nested
LinkTransform combinations, plain/placed groups, member paths and Face1/Face3
references. Native GroupExtension membership is authoritative: a plain-group
member also retained directly by its GeoFeatureGroup is copied at both real paths.
Fixtures check live A and canonical definitions after B edits, Undo/Redo, retained
JSON after replacement/close, unchanged source BRep/group/placement/history, and
atomic rejection of missing paths, malformed subelements and a65-level tree.
The review's container-metadata undercount has a native RED→GREEN: an oversized
prefix of individually valid group labels rejects before the following unsupported
leaf is read. Shallow container headers/references, child separators and each
embedded graph copy are counted before expansion. Qt's floating point JSON
spelling is measured separately from Rust's query byte count; the final string
must fit the accumulated bound. Temporary/native allocator overhead is excluded.
These samples do not prove global parameter maps or Rhino semantic parity.

[Fresh read-only review](../reviews/2026-10-10-instance-definitions-review.md)
records its findings and their disposition. Existing Line/Polyline and native
solvers remain reused.

**Full original RCORE-01 remains in progress.** Arrays, mesh/cloud native
definition leaves, references through link containers and overlapping nested
subelement contexts need further adaptation. Full attributes/userdata, edge3D/UV
correspondence, native reversals, manifold/solid/cavity certificates, uniform
reports/capability decisions and the remaining allocation/worker/mutation/lifetime
requirements are still tracked in the [original audit](../reviews/2026-10-10-rcore01-closure-audit.md).
