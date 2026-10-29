# OM9-L3-04-01-012 API

- `om9_app::session::transform::SMART_FLOW_RIGID` — the row constant (`OM9-TRANSFORM-012`)
- `om9_app::session::transform::fields("OM9-TRANSFORM-012") -> &'static [Field]`
- `om9_core::deform::rigid_flow`
- `om9_core::model::FLOW_EXTENSION_SLOT` (`om9.flow`), registered in
  `model::extension_slot_known`
- `om9_core::transaction::Mutation::{Geometry, Placement, Extensions}`

The commit is one `TransactionRequest`, so the command is one Undo entry. Errors are typed
(`Missing`/`Invalid`). Owner L3: `OM9-L3-04-05-003`.
