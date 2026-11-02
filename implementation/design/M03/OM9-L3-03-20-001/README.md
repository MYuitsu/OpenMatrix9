# OM9-L3-03-20-001 — Distance And Length Measurement

Status: AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING (2026-10-26).

Golden: The typed points `0,0,0` and `3,4,0` report `Distance = 5 mm`, and a `Line` from `0,0,0` to `3,4,0` reports `Length = 5 mm`.

Services:
- `crates/om9-core/src/measure.rs`: `distance_mm(a, b)` (Euclidean distance with the `MAX_MEASURE_COORDINATE` guard) and `distance(a, b, unit)`.
- `crates/om9-core/src/curve_ops.rs`: `arc_length(curve, limits)` — the adaptive-Simpson exact length behind the Length read-out.
- `crates/om9-app/src/session/measure.rs`: `measure_points`, `measure_length`, `Measurement`, `MeasureUnit`, `MeasurePending` (two picked points).
- `crates/om9-app/src/session.rs`: `OM9-MEASURE-002`/`OM9-MEASURE-003` dispatch, point prompt, typed point input and `active_feature` echo.
- `crates/om9-app/src/ui.rs`: the read-out is appended to the Command Line once and shown on the status surface.

Refusals: An empty selection for Length is `Error::Missing("curve selection")`; a non-curve selection is `Error::Invalid("curve selection")`; neither publishes a label.

Evidence: `cargo test -p om9-app --test measure m03_03_20_001` and `cargo test -p om9-app --test measure_ui`.

Manual gate: M03-UI remains unsigned — no human has compared the on-screen read-out with the
manual, so this record is automated-target evidence only.
