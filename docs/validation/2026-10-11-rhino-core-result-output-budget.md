# RCORE-01 — Common result output budget, 2026-10-11T20:03:10+07:00

**Status: in_progress. Full original RCORE-01 is still required.**

The common Rust result encoder now checks the actual owned output tree before
writing JSON. Existing limits remain16MiB,1million nodes and depth64 (root0).
Result wrappers and metadata consume those limits too. Oversize output returns
InvalidBasis2 without copying bytes into the caller buffer. No snapshot fields,
native geometry or source references are dropped to meet the cap. Native solvers
and the C++ bridge are reused unchanged. Solid source-envelope depth derives from
the same depth constant, accounting for its extra request ancestor.

## Actual evidence

- Focused Rust RED1/4 → GREEN4/4: independently valid exact1million-node input
  becoming overbudget output; depth64 input becoming depth65 output; legal
  depth64 output retaining its full extension; untouched initialized FFI output
  buffer on rejection. These are schema/ABI tests, not native geometry proof.
- Full Rust833 passed/0 failed/0 ignored. Standalone plugin build exit0.
- Matching Runtime42:892/892 checks across18 distinct actual host exits0.
  Solid553 plus prepared CAD47, discrete81, Builder20, Surface81, Edit110.
  Existing native geometry/Undo/Redo/History/FCStd fixtures were not weakened.
- Solid's two cold hosts open five recipe FCStd files and the indexed54-ref
  result from their matching warm Runtime42 hosts. Exact saved result bytes
  survive; six cold-loaded archive hashes are recorded. Other consumers' warm
  persistence files are recorded separately, not counted as cold hosts.
- All437 frozen producer-input fingerprints match. Shared SDK hashes unchanged.
- Fresh independent source review found no actionable findings.

## Limits of this evidence

The full output tree is bounded before serialization; building/cloning the tree
and all native allocations still need their original aggregate proofs. The source
envelope gate does not reserve unknown final solver/output overhead. Native
regressions verify existing consumers only within their fixture scopes. All runs
use offscreen QPA; actual mouse, OpenGL rendering and viewport acceptance remain
required. Earlier Runtime41/40 results remain historical, not pooled into892.

- Every original RCORE-01 requirement remains required: GeometryRef/Snapshot/Result, rational/periodic basis, UV trims/pcurves/seam occurrences, complete attributes, every enabled consumer, independent family read/edit/retain/export, adapters1–7, C01–C12 and T01–T11.
- Common produced GeometryResult now enforces bytes16MiB/nodes1million/depth64 before serialization/caller-buffer copy. Source-envelope admission is still necessary only: complete request/output overhead reservation and aggregate allocation/traversal/worker/cancellation bounds remain required.
- Global conversion/deviation bounds and complete parameter/topology maps remain unproven. Valid closed native Solid is not a certificate of these contracts.
- Existing CurveController/CurveGeometry commands still require the common GeometryResult path using their current FreeCAD/Line/Polyline/curve solvers.
- Offscreen evidence covers native kernel, command, Qt keys, callbacks and scoped FCStd persistence only. Actual mouse, OpenGL rendering and viewport acceptance remain required; historical graphics cohorts are not current reruns.
- The two cold hosts verify five named recipes and one indexed54-reference result, with OM9 imported by bootstrap. Unsignalled native root-frame mutations, actual Solid preview leases and complete original lifetime/cache invariants still require evidence.
- Production deployment/ABI acceptance remains pending. An isolated App19/camera-guard GUI host reuses the single shared SDK; no root FreeCAD build or full runtime copy was made.

The [machine checkpoint](2026-10-11-rhino-core-result-output-budget.json) records
matching source/binary/fixture/launch/log/archive hashes. Plugin SHA256:
`595a05c159f06991608ab40ad6848297515352af96c0c1c78a668c71291de861`.

Next: common Curve GeometryResult via existing CurveController/CurveGeometry
solvers, complete request/output overhead reservation and the original audit.
