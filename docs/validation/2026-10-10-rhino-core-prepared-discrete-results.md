# RCORE-01 prepared discrete results — 2026-10-10

**Scope:** Prepared mesh and current float64 PointCloud owned GeometrySnapshots, Rust admission/result assembly, bounded native copies and immutable report persistence; mesh archive correspondence at declared float32 host precision. Full original RCORE-01 remains in progress.

Prepared mesh/current-cloud rows and affine previews now receive every original
GeometrySnapshot field before publication. Rust reuses the working-V5 registry,
discrete model, numeric/index policies and bounded result assembler. C++ uses
existing native readers and copies exact MeshObject owners; plain cloud fields
are frozen recursively. Python binds the native-generated immutable report.
Native/Python/document pointers never enter or remain in Rust.

Mesh retains raw-local float32 points, triangles and separate native location;
original archive float64 vertices and quad partition remain owned observations.
Rust compares cyclic-normalized oriented triangle multisets at float32 precision
after applying native location once. Reindexing, cyclic rotations and unused
archive vertices are allowed; different geometry, winding and duplicate counts
reject with ConversionLoss before transaction. No source data is repaired.
This scoped correspondence is explicit in validation, not an exact float64 or
global manifold certificate. The exporter retains its existing verified-source
archive behavior; mismatching prepared rows can no longer install its signature.

Current PointCloud schema1 retains float64 points/normals/values, RGBA integer
channels, ordered and plane fields. Channel sizes, exact flags and bounds are
checked by Rust; active plane validity comes from the existing ON_Plane API.
The fixture retains 10000000.00000001 exactly through publication and cold
FCStd. No CAD conversion or display-cloud precision is substituted.

Native mesh owner/segment/index/byte budgets precede source-copy constructors;
count limits100k points/faces and existing8MiB discrete serialization remain.
Cumulative64MiB/depth64/node budgets cover frozen values and report binding.
MSVC cannot delay-load the MeshPy static Type data import. A fixed empty SDK
prototype provides exact wrapper type after Mesh initialization; this constant
native allocation is not a source geometry clone. SDK imports stay deferred.
These logical limits do not prove process-wide allocation safety.

**675 full Rust tests;3790 host checks;all26 matching actual host exits0.**
Plugin SHA256 `6f92a07de16f8a00c565b7e2312279e2fcba366a8a169da79cc714343cbac5f4`. The81-check new warm fixture covers a real
document observer mutating original mesh/cloud inputs, structured prepublication
rejection, source and executable Redo preservation, valid active planes, segment
owner budgets and archive correspondence. Eight second-process checks rebuild
mesh/cloud from owned snapshots and verify exact FCStd report bytes. The3701
prior CAD, metadata, modeling/preservation, topology/reference/support and cold
module checks share this plugin. Only standalone OpenMatrix9Gui was built using
the shared SDK/cache and14 workers; the shared host hash remains unchanged.

Meaningful RED: the original native plugin failed48/51 ownership/schema checks;
the segment budget fixture failed4/65; the archive mismatch fixture failed12/81.
All actual RED host exits were0, with no fixture exceptions. Focused Rust RED
counts were0/6,6/7 and9/10. RED logs/reports remain fingerprinted.

[Machine evidence](2026-10-10-rhino-core-prepared-discrete-results.json) and
[source review](../reviews/2026-10-10-prepared-discrete-results-review.md) preserve
scope, provenance and limitations. Every remaining original obligation stays in
the[closure audit](../reviews/2026-10-10-rcore01-closure-audit.md).
