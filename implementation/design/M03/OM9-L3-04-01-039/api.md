# OM9-L3-04-01-039 API

- `om9_app::session::transform::MOVE_UVN` — the row constant
- `om9_app::session::transform::fields("OM9-TRANSFORM-039") -> &'static [Field]`
- `om9_core::deform::move_uvn(surface, normal_offset, u_offset, v_offset, limits) -> Result<NurbsSurface>`
- `om9_core::surface_kernel::{greville_parameter, derivatives}`

The commit is one `TransactionRequest` of `Mutation::Geometry` per selected surface, so the
command is one Undo entry and the object keeps its placement. Errors are typed
(`Missing`/`Invalid`).
