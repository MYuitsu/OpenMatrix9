# OM9-L3-04-01-013 API

- `om9_app::session::transform::SMART_FLOW_BREAK` — the row constant (`OM9-TRANSFORM-014`)
- `om9_app::session::transform::fields("OM9-TRANSFORM-014") -> &'static [Field]`
- `om9_core::Mutation::Extensions`
- `om9_core::model::FLOW_EXTENSION_SLOT` (`om9.flow`), registered in
  `model::extension_slot_known`
- `om9_core::transaction::Mutation::{Geometry, Placement, Extensions}`

The commit is one `TransactionRequest`, so the command is one Undo entry. Errors are typed
(`Missing`/`Invalid`). Owner L3: `OM9-L3-04-05-005`.
