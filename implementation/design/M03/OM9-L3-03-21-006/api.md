# OM9-L3-03-21-006 API

- `om9_app::session::annotation::FEATURES[5]` — `OM9-MEASURE-010`
- `Controller::begin_annotation("OM9-MEASURE-010")`, `accept_annotation_point`, `submit_annotation_text`
- `Pending::prompt()` — the documented question sequence above
- `om9_core::annotation::{Annotation, Kind, layout, SLOT}`
- `om9_core::transaction::Mutation::Extensions` — the annotation lives in its `annotation` slot

One command is one `TransactionRequest`, hence one Undo entry. Errors are typed
(`Invalid`/`Permission`).
