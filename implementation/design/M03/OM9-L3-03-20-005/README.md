# OM9-L3-03-20-005 — Measurement Label And Command Feedback

Status: AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING (2026-10-26).

Golden: Measuring a 5 mm line shows `Length = 5 mm` in the Command Line and on the status surface with no geometry operation, the document hash is unaffected, and a later selection change never shows the old label.

Services:
- `crates/om9-app/src/session/measure.rs`: `MeasurementLabel` tagged with the selection revision, `last_measurement` (drops a stale label) and `take_measurement_label` (consumed once).
- `crates/om9-app/src/session.rs`: `dispatch_effect` clears the previous label when a new command starts; `active_feature` echoes the running measure feature.
- `crates/om9-app/src/ui.rs`: `MatrixApp::feedback` prefers the read-out over `complete_success` and appends it to the transcript.

Refusals: A measurement is read-only: it never opens a transaction, so the Undo journal and the document revision stay identical; a stale label is dropped instead of being shown.

Evidence: `cargo test -p om9-app --test measure m03_03_20_005` and `cargo test -p om9-app --test measure_ui`.

Manual gate: M03-UI remains unsigned — no human has compared the on-screen read-out with the
manual, so this record is automated-target evidence only.
