# OM9-L3-04-01-024 API

- `om9_app::session::transform::ROW_023` — the row constant
- `om9_app::session::transform::fields("OM9-TRANSFORM-023") -> &'static [Field]`
- `om9_app::session::transform::handled_rows() -> Vec<&'static str>`
- `Controller::begin_transform(feature: &str) -> Result<()>`
- `Controller::submit_transform_text(text: &str) -> Option<Result<()>>`
- `Controller::commit_transform_mutations(name, targets, steps, copies) -> Result<()>`

Kernel owners used by this row: `surface_kernel.rs`, `transform_session.rs`.

The commit is one `TransactionRequest`: `Mutation::Placement` for an in-place row and
`Mutation::Insert` for every copy, so the whole row is a single Undo entry. Errors are typed
(`Missing`/`Invalid`/`Permission`/`Unsupported`); a refused value leaves the document and the
Undo journal untouched.
