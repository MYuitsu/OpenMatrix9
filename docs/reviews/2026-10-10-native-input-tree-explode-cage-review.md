# Independent review: Explode and NURBS cage preflight

**Ruling:** Approved for this bounded post-decode increment. No actionable
findings remain in the reviewed Explode/cage paths.

The reviewer identified one P2 gap: finite cage knot values can still overflow
when adjacent differences or active-domain width are computed. The implementation
now checks both with `std::isfinite` before the pinned SDK reaches
`ON_NurbsCage::IsValid()` or `IsoSurface()`. A real serialized 3DM fixture with
finite extreme knots requires the specific cage preflight error and confirms
both SDK hooks remain unreachable.

The reviewed change also validates supported original Explode curves, BReps and
surfaces before `Duplicate()`, validates cages before CV extraction, and bounds
the cage control-point product, knot storage, strides, capacities, coordinates
and weights. The native fixture result is 73/73 with zero failures; six related
archive/block CTests and the existing positive `OM9ThreeDmExplodeTest` pass.

This approval covers only post-decode preflight. It does not cover allocations
inside `ONX_Model::Read()`, process-wide allocation or worker budgets, other
cage-edit APIs, or completion of RCORE-01.T07/all P0 requirements.
