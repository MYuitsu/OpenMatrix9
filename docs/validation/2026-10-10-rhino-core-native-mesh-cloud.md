# Mesh and PointCloud native input preflight — partial RCORE-01.T07

This follow-up extends the existing Rust-backed synchronous input gate to
`ON_Mesh` and `ON_PointCloud`. It bounds live direct-array envelopes and logical
bytes, checks mesh face/N-gon references and supported numeric fields, and runs
before the existing SDK validity call, Explode clone, or PointCloud copy. The
current openNURBS/FreeCAD adapters remain in use; no replacement mesh or cloud
representation was introduced.

## Implemented scope

- Mesh preflight checks array count/capacity, pointer span/alignment and the
  shared 64 MiB per-owner-tree logical byte gate. It validates direct and
  double-precision vertices, optional normals/texture/surface-parameter array
  shapes, finite coordinate values, face indices, N-gon maps and their bounded
  index arrays, and per-vertex texture-set storage before `ON_Mesh::IsValid()`.
- PointCloud preflight checks point/optional-array count and storage envelopes,
  finite points/normals/intensities and an active plane before value traversal or
  copying. Runtime hidden-point arrays are bounded but retain the SDK's optional
  semantics.
- Archive member staging validates Mesh and PointCloud before a transformed
  owner is duplicated. Explode validates Mesh before cloning. PointCloud has no
  editable Explode representation, so that path now reports unsupported input
  before cloning it.
- The code reuses the existing Mesh and PointCloud archive adapters and the
  existing Line/Polyline command/controller and `Part.makePolygon` path.

The validator assumes each incoming SDK owner and its referenced memory are
already live, initialized and readable during the synchronous call. Address and
capacity checks do not prove allocation lifetime or accessibility. The 64 MiB
value budget applies to each validated root owner; it is not an archive-wide,
process-wide or worker aggregate memory certificate.

## Verification

- The fixture RED run recorded 79 checks with two failures: malformed mesh and
  PointCloud storage were not stopped by the new expected preflight boundary.
- The current `ThreeDmNativeInputTreeTests` run passes **92/92**. It also proves
  paired-NaN curvature sentinels and finite set curvatures remain accepted,
  partial-NaN curvature is refused, unmanaged zero-capacity N-gons are refused
  before index-buffer reads, and nonfinite PointCloud points, normals and
  intensities are rejected before value traversal. Empty PointCloud sources
  remain valid for overlays, while malformed source storage is rejected before
  copy. Existing valid archive Mesh/PointCloud adapters remain reachable.
- All six focused archive/block CTests pass: CurveOnSurfaceArchive,
  NativeInputTree, NativeTransform, Preservation, Archive and Blocks. The
  positive `OM9ThreeDmExplodeTest` integration passes.
- `build/build-om9-plugin.ps1` rebuilt only `OpenMatrix9Gui` against the shared
  FreeCAD SDK. A fresh shared-host run at
  `H:/FreeCAD-src/build/native-input-tree-mesh-cloud-current-plugin-20261010`
  loaded the current plugin and passed **21/21** checks with process exit code
  0. Host SHA-256 is
  `98eef899354a8aa84cfa2934e6e1a3c8558c7e6fff3b89c347c06731f06018a8`; plugin
  SHA-256 is `85f1ed567d3a68097cb264972b4bee109719560365182fbc0a680b25279fbff2`.
- Rust was unchanged in this increment; the prior full run remains 572 passing
  tests across 73 suites.

The bounded independent review found no code-level blocker and confirmed the
added numeric branches and 92/92 fixture result. Its scope excludes raw archive
decoding. Exact source and binary fingerprints are recorded in
[the evidence JSON](2026-10-10-rhino-core-native-mesh-cloud.json). The all-P0
objective and T07 remain open: `ONX_Model::Read` allocations happen before this
gate, process-wide allocation and aggregate worker memory are not measured, and
other analytic, cage-edit and native-owner paths still need coverage.
