# RCORE-01 prepared result snapshots — 2026-10-10

**Scope:** Prepared native CAD GeometrySnapshot fields, bounded independently restorable OCCT V3 master, owned typed topology/support observations and frozen binding attributes; Rust result/report assembly with exact unit revision. Full original RCORE-01 remains in progress.

Prepared native CAD objects now carry all original GeometrySnapshot fields:
schema_version, unit_context_revision, geometry_data, topology_data and
source_attributes. The complete geometry master uses the existing OCCT ASCII
V3 serializer without triangulation/normals. Encoding and SDK are explicit;
native root location/orientation and ordered native structure remain available.
Existing topology/support observations retain rational weights/knots/domains,
hole loops, seam pcurve occurrences and native validity reports.

Rust owns bounded parsing and result/report JSON assembly, reusing the existing
admission, binding-field and unit-context policies. Native capture is required
for Part/OCCT serialization, frozen Python values and document context; Rust
never retains a native/Python/document pointer. Master8MiB and request/output
16MiB limits, bounded nodes/depth and synchronous query/copy output apply.
These logical budgets are not process-wide allocation certificates.

Explicit unit revision retains the complete u64 as a decimal string. Legacy
documents without context retain null revision. Native geometry remains in
host millimeters; reading display/model unit context does not rescale it.
Attributes retain the exact supported binding fields and declare arbitrary
metadata and Rhino userdata uninspected. Snapshot-owned optional fields are
validated even on non-working rows before native getters can set Python errors.

Conversion reports have every original named field. Upstream exactness,
deviation, parameterization and topology maps remain explicitly uninspected or
null; successful native reconstruction is not a conversion certificate.
Reports remain immutable import evidence, distinct from current edited shape
validity. This advances the full chapter's schema, without declaring all
operation families or all original requirements complete.

**665 full Rust tests;3701 host checks;all24 matching actual host exits0.**
Plugin SHA256 `2ec561858015035d9bbc59ccf769dbd31bc5a2d9c928663f86c3e3e123ca3fb7`.107 new full-snapshot/warm-cold checks,
95 metadata checks and3499 existing regression checks share this plugin.
The full snapshot fixture covers10 native CAD cases: Unicode Point, rational
and reversed curve, moved/rotated circle, ordered open wire, face with hole,
periodic seam, closed Shell, Solid and mixed Compound. It reconstructs after
document close, checks exact report bytes through one Undo/Redo and FCStd,
then restores the snapshots independently in a second process.

The first cold oracle reported three differences in cached BoundBox values.
Source inspection confirmed FreeCAD's BRepBndLib::Add includes display mesh;
the no-mesh snapshot had matching topology and edge lengths. Comparing detached
native copies with copyMesh=false made all40 cold checks pass, with no CAD or
tolerance change. Failed runs and actual signatures remain fingerprinted.

RED evidence: the old native result failed64 of67 full-schema checks; initial
Rust result stub failed4 of5 tests. The optional geometry_kind=None regression
failed two identity/error-category assertions; its fix passes the final95-check
metadata fixture and preserves executable Redo. All other listed modeling,
preservation, solid/topology, instance/edge/support and import fixtures passed.

Only standalone OpenMatrix9Gui was built, reusing the shared SDK/cache with14
workers. Shared host hash remains unchanged. [Source review and limits](../reviews/2026-10-10-prepared-result-snapshots-review.md).
[Machine evidence](2026-10-10-rhino-core-prepared-result-snapshots.json) binds
sources, matching SDK/runtime, test harness, process outcomes and RED artifacts.

**Full original RCORE-01 remains in progress.** Upstream reports/parameter maps,
geometry/topology contracts across other families, remaining attributes, non-CAD contracts, all enabled
consumers, workers and every other original item remain in the
[closure audit](../reviews/2026-10-10-rcore01-closure-audit.md).
Native writer-budget errors currently use KernelFailure; uniform typed-error
coverage still needs work. No global FFI/lifetime/allocation safety is claimed.
