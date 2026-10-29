# OM9-L3-04-01-038 API

- `om9_app::session::transform::REMAP_CPLANE` — feature id constant
- `om9_app::session::transform::fields("OM9-TRANSFORM-038") -> &'static [Field]`
- `Controller::begin_transform(feature: &str) -> Result<()>`
- `Controller::accept_transform_point(pending, point: Point3) -> Result<()>`
- `Controller::submit_transform_text(text: &str) -> Option<Result<()>>`
- `om9_app::transform_menu::by_feature` / `by_command` / `by_label`

Kernel owners used by this row: `om9_core::gumball` (scale/rotate/array) and
`om9_core::transform_session` (align solve). The commit is one `TransactionRequest` of
`Mutation::Placement` (in place) or `Mutation::Insert` (array copies).

Errors are typed (`Missing`/`Invalid`/`Permission`/`Unsupported`); a refused value leaves
the document and the Undo journal untouched.
