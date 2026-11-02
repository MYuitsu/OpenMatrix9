# OM9-L3-03-20-004 API

- `om9_app::session::measure::summarize(document: &Document) -> ObjectSummary`
- `om9_app::session::measure::ObjectSummary::{label, total, by_kind, gems, heads, rails, cutters, unknown_slots, definitions}`
- `Controller::document_summary`, `Controller::object_summary`

Errors are typed (`Missing`/`Invalid`/`Unsupported`/`Busy`) and no measurement opens a
transaction, so a refused read-out never changes the document.
