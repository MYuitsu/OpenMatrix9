# OM9-L3-03-13-006 — History Commandline And Preview Status

## Goal

informative status when rebuild happens

## Controller contract

- `Controller::submit(..)` / the sidebar/menu feature IDs dispatch to the same history methods.
- Positive: HistoryRecord/HistoryUpdate/HistoryReport/ClearHistory/EditHistory drive the same controller switches; recompute reports complete with the rebuilt feature count or failed with the node id and reason
- Negative: ClearHistory with no selection -> Missing(history object); closing the window while a recipe is Pending -> Busy, document untouched

## Tested entry points

- `the_command_line_drives_the_switches_report_and_clear`
- `closing_the_window_is_refused_while_a_recipe_is_pending`

## Status text

- complete: `History update complete: <n> feature(s)`
- failed: `History update failed at <recipe id>: <reason>`
- lifecycle: a Pending recipe (or an in-flight rebuild) makes window close/open return `Busy`.

## Command events

- `Started / History / updating <recipe id>`
- `Finished / History / complete: <n> feature(s)` with outcome Committed
- `Warn / History / failed at <recipe id>: <reason>` with outcome Refused
