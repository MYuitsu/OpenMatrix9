# OM9-L3-03-20-004 — Jewelry Object Count And Type Summary

Status: AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING (2026-10-26).

Golden: A scene with 2 curves, 1 surface, 1 solid, 1 gem and 1 unknown placeholder reports `Objects: 6 total`, `gems 1` and `unknown 1`; the unknown slot is never called a gem and block definitions never add to the total.

Services:
- `crates/om9-app/src/session/measure.rs`: `summarize(&Document) -> ObjectSummary` counts by `GeometryKind::tag()`, reads the gem/head/rail/cutter extension slots, counts unknown slots separately and reports definitions without adding their members.
- `crates/om9-app/src/session.rs`: the `ObjectSummary` command (typed name, no menu row) through `measure_selection`.
- `crates/om9-core/src/model.rs`: the typed extension slots and `extension_slot_known`.

Refusals: There is no error path that invents a type: an unknown slot increments `unknown_slots` only, and the summary is read-only.

Evidence: `cargo test -p om9-app --test measure m03_03_20_004`.

Manual gate: M03-UI remains unsigned — no human has compared the on-screen read-out with the
manual, so this record is automated-target evidence only.
