# OM9-L3-03-20-002 — Angle And Radius Measurement

Status: AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING (2026-10-26).

Golden: Two perpendicular directions in the XY plane report `Angle = 90 deg`, and a `Circle { radius: 5 }` reports `Radius = 5 mm`.

Services:
- `crates/om9-core/src/measure.rs`: `angle_degrees(from_a, to_a, from_b, to_b)` — atan2 of the two turned vectors, undirected, in degrees; a zero-length direction is refused.
- `crates/om9-app/src/session/measure.rs`: `measure_points` for the four-point Angle input and `measure_radius` for a single circle curve.
- `crates/om9-app/src/session.rs`: `OM9-MEASURE-001`/`OM9-MEASURE-005` dispatch, four-point prompt, typed points.
- `crates/om9-app/src/ui.rs`: both read-outs reach the Command Line like Distance and Length.

Refusals: A zero-length angle direction is `Error::Invalid`; a spline or multi-selected Radius is refused instead of reporting curvature or one arbitrary radius.

Evidence: `cargo test -p om9-app --test measure m03_03_20_002`.

Manual gate: M03-UI remains unsigned — no human has compared the on-screen read-out with the
manual, so this record is automated-target evidence only.
