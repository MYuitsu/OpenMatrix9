# OM9-L3-03-20-002 API

- `om9_core::measure::angle_degrees(from_a: Point3, to_a: Point3, from_b: Point3, to_b: Point3) -> Result<f64>`
- `om9_app::session::measure::measure_radius(curve: &Curve) -> Result<Measurement>`
- `Controller::measure_angle(a0, a1, b0, b1)`, `Controller::measure_selected_radius`

Errors are typed (`Missing`/`Invalid`/`Unsupported`/`Busy`) and no measurement opens a
transaction, so a refused read-out never changes the document.
