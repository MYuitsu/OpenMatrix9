# Circle spatial tangent and AroundCurve History

Approved continuation of OM9-CURVE-005's remaining supported branches.
Source reread: Matrix8 Book1 PDF138–140, Circle through continuation before Ellipse.

- Rust derives a plane from all native line endpoints/conic or NURBS poles,
  keeps the existing CPlane frame when valid, and rejects incompatible geometry.
  This supports arbitrarily oriented coplanar constraints, not general nonplanar
  tangencies. A Tangent `Vertical=Yes` constraint requires a plane perpendicular
  to CPlane. Native OCCT validates exact bounded contacts and selected branches.
- Rust accepts the solved normal explicitly and validates bounds, Radius and
  Vertical before commit. Native objects remain GUI-owned.
- Optional `History=Yes` for analytic AroundCurve records a normalized edge
  parameter fraction and fixed output radius. Defaults remain snapshot/No.
  The native feature updates center/normal from exact world-placed source edge;
  source and parent placement changes, graph scheduling, Record/Update/Lock,
  detach, Undo/Redo and cold FCStd restore use existing History infrastructure.
  Deformable and other Circle constructions continue as snapshots.
- Test Rust plane inference and acceptance first; then matching native build,
  spatial/History runtime cases, Circle/Curve and shared Solid/History regressions.
- Update Circle feature notes, spec_v1 catalogs/checklist/status/manifest and ledger
  only after verified native results, preserving the original contract/sample.

OM9 decisions: plane tolerance 1e-6 mm; normalized parameter rather than arclength;
History snapshot Radius and full analytic native edge validation; no claim of
original Matrix defaults or automatic topology correspondence.
