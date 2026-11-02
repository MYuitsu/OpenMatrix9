# OM9-L3-03-20-001 API

- `om9_core::measure::distance_mm(a: Point3, b: Point3) -> Result<f64>`
- `om9_core::measure::distance(a: Point3, b: Point3, unit: Unit) -> Result<f64>`
- `om9_core::curve_ops::arc_length(curve: &Curve, limits: &Limits) -> Result<f64>`
- `om9_app::session::measure::measure_points(feature: &str, points: &[Point3]) -> Result<Measurement>`
- `om9_app::session::measure::measure_length(curves: &[Curve], limits: &Limits) -> Result<Measurement>`
- `Controller::measure_distance`, `Controller::measure_selected_length`

Errors are typed (`Missing`/`Invalid`/`Unsupported`/`Busy`) and no measurement opens a
transaction, so a refused read-out never changes the document.
