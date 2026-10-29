# OM9-L3-03-20-003 API

- `om9_app::session::measure::world_bounds(document: &Document, ids: &[ObjectId]) -> Result<Bounds>`
- `om9_app::session::measure::bounds_measurement(bounds: Bounds) -> Measurement`
- `Controller::measure_selected_bounds`

Errors are typed (`Missing`/`Invalid`/`Unsupported`/`Busy`) and no measurement opens a
transaction, so a refused read-out never changes the document.
