# OM9-L3-04-01-020 API

- `om9_app::session::transform::020` — the row constant (`OM9-TRANSFORM-020`)
- `om9_app::session::transform::fields("OM9-TRANSFORM-020") -> &'static [Field]`
- `om9_core::cage::{box_corners, box_cage, cage_from_corners, box_mesh, mesh_corners,
  displacement, capture_parameters}`
- `om9_core::cage::Cage::{invert, corners, with_corner, evaluate}`
- `om9_core::model::CAGE_EXTENSION_SLOT` (`om9.cage`), registered in
  `model::extension_slot_known`
- `om9_core::transaction::Mutation::{Insert, Extensions, Geometry}`

The commit is one `TransactionRequest`, so the command is one Undo entry. Errors are typed
(`Missing`/`Invalid`/`Unsupported`).
