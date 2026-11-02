# OM9-L3-04-01-027 API

- `om9_app::session::transform::ROW_027` — the row constant
- `om9_app::session::transform::fields("OM9-TRANSFORM-027") -> &'static [Field]`
- `om9_core::deform::flow_along_curve(source, target, geometry, placement, limits)`
- `om9_core::deform::flow_along_surface(source, target, geometry, placement, limits)`
- `om9_core::deform::splop(surface, geometry, placement, limits)`

Kernel owners used by this row: `deform.rs`, `curve_ops.rs`, `curve_surface_ops.rs`, `transform_session.rs`.

The commit is one `TransactionRequest` of `Mutation::Geometry`, so the row is one Undo entry
and the object keeps its placement. Errors are typed
(`Missing`/`Invalid`/`Permission`/`Unsupported`).
