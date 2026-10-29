# OM9-L3-03-20-003 — Bounding Size And Center Measurement

Status: AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING (2026-10-26).

Golden: A 10 x 20 x 30 box whose corner is at (3,4,5) reports `Bounding Box = 10 x 20 x 30 mm, center (8, 14, 20)` — the size is unchanged and no box geometry is created.

Services:
- `crates/om9-core/src/utilities.rs`: `Bounds::{size, center}` and `bounds(&Geometry)`.
- `crates/om9-app/src/session/measure.rs`: `world_bounds` transforms the eight local corners through the entity placement, so a moved object reports its moved box; `bounds_measurement` is the `Output=None` report.
- `crates/om9-app/src/session.rs`: `OM9-UTIL-010` dispatch through `measure_selection`.

Refusals: An empty selection is `Error::Missing("bounding box selection")`; geometry without vertices is `Error::Unsupported`; the document is never touched.

Evidence: `cargo test -p om9-app --test measure m03_03_20_003`.

Manual gate: M03-UI remains unsigned — no human has compared the on-screen read-out with the
manual, so this record is automated-target evidence only.
