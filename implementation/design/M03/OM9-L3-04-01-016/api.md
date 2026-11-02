# OM9-L3-04-01-016 API

- `om9_app::session::transform::ORIENT_CURVE` — feature id constant (`OM9-TRANSFORM-016`)
- `om9_app::session::transform::curve_path_row(feature: &str) -> bool`
- `om9_app::session::transform::fields("OM9-TRANSFORM-016") -> &'static [Field]`
- `Controller::begin_transform(feature: &str) -> Result<()>`
- `Controller::accept_transform_point(pending, point: Point3) -> Result<()>`
- `Controller::submit_transform_text(text: &str) -> Option<Result<()>>`

Kernel owners used by this row: `curve_ops.rs` (path station) and
`transform_session.rs`. The commit is one `TransactionRequest` of
`Mutation::Placement` (in place) or `Mutation::Insert` (array copies).

Errors are typed (`Missing`/`Invalid`/`Permission`/`Unsupported`); a refused value leaves
the document and the Undo journal untouched.
