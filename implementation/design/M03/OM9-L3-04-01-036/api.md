# OM9-L3-04-01-036 API

- `om9_app::session::transform::ROW_035` — the row constant
- `om9_app::session::transform::fields("OM9-TRANSFORM-035") -> &'static [Field]`
- `om9_app::session::transform::handled_rows() -> Vec<&'static str>`
- `Controller::begin_transform(feature: &str) -> Result<()>`
- `Controller::submit_transform_text(text: &str) -> Option<Result<()>>`
- `om9_core::deform::Deform` / `om9_core::deform::apply(deform, geometry, limits)`
- `om9_core::deform::apply_placed(deform, geometry, placement, limits)`

Kernel owners used by this row: `deform.rs`, `curve_ops.rs`.

The commit is one `TransactionRequest` of `Mutation::Geometry`, so the whole row is a single
Undo entry and the object keeps its placement: the map runs in world space and the result is
written back into the object's own frame. Errors are typed
(`Missing`/`Invalid`/`Permission`/`Unsupported`); a refused value leaves the document and the
Undo journal untouched.
