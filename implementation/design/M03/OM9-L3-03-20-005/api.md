# OM9-L3-03-20-005 API

- `Controller::report_measurement(measurement: Measurement)`
- `Controller::last_measurement() -> Option<&str>`
- `Controller::take_measurement_label() -> Option<String>`
- `MatrixApp::feedback` read-out branch

Errors are typed (`Missing`/`Invalid`/`Unsupported`/`Busy`) and no measurement opens a
transaction, so a refused read-out never changes the document.
