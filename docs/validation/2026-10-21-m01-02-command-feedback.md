# Plugin progress and warning reports (OM9-L3-01-02-010) — 2026-10-21

A plugin command can now report progress and warnings into the host's own command event log,
so the Command line and its observers read plugin feedback through the same bounded surface
as built-in commands. The work belongs to the command/event owner (`OM9-L2-01-02`,
`OM9-L3-01-02-010`); the app's `EventLog` and `Finished` outcome contract stay the only
feedback log.

## Changed

| Layer | Change |
|---|---|
| Public SDK | `MessageLevel` (`progress`/`warning`), `PluginMessage`, `Host::progress(text)` and `Host::warning(text)`; one new `HostRequest::Message` and two capability entries. |
| Host | `Staged` collects at most 32 reports of at most 512 one-line bytes each; `PluginRun` returns them as `messages`. Exceeding either bound is a `budget` refusal (empty text is `budget`, a control character is `message`). |
| App | The Controller emits each staged report as a `Progress`/`Warn` event under the same `plugin:command` label it already uses for `Started`/`Finished`, before publishing the command's transaction. |
| Example plugin | `FeedbackDemo` reports "started", optionally "half way", then a warning; `{"fail":true}` returns an error after reporting. |
| Tests | Host tests for the staged records and every refusal, and a native DLL test that proves the committed and refused event sequences, no document change and preserved Redo. |

## Semantics fixed by this slice

- Reports are command feedback, not document state: they create no Undo entry and no
  transaction; `PluginRun::request` stays `None` for a reporting-only command.
- The event sequence for a committed reporting command is `Started`, the reports in order
  (`Progress`/`Warn`), then `Finished(Committed)`; a refused command keeps `Started` then
  `Finished(Refused)` and its reports are discarded, because a failed invocation publishes
  nothing.
- Reports are bounded (32 per command, 512 bytes each, one line without control
  characters). The host refuses rather than truncates, so a chatty plugin learns the limit.
- The app does not gain a second feedback path: reports join the existing `EventLog` that
  `command_events` owns, which the Command line surface and observers already consume.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone Controller/document/scene/archive path; no
OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 43 passed, 8 suites | Staged reports, the message bounds/grammar and every other host contract still pass. |
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites | No core change in this slice; the document/transaction contracts are unchanged. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 4 passed, 135 filtered | The Controller publication/Undo boundary still holds. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 8 passed | A real `.om9p` DLL: `FeedbackDemo` produces `Started/Progress/Progress/Warn/Finished(Committed)` and changes nothing; the failing variant produces `Started/Finished(Refused)` only, with the document and the existing Redo unchanged. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-feedback` | PASS, 10 commands | Package/inspect, `CreateProfile`, cold `--inspect` hash equality, cancel/unknown-command preserving the `.om9` bytes, SDK ZIP built here and its example compiled offline outside the checkout. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features, incl. `occt-brep`) | exit 0 | The product-feature build still compiles. |
| `python tools/public_source_audit.py` | 8791 source files, 511 icon bindings, 510 SVG, 0 errors | No private/commercial source or binary was introduced. |
| `rustfmt --check` on the edited Rust files; `git diff --check` | PASS | Formatting and tracked whitespace are clean. |

The [JSON receipt](2026-10-21-m01-02-command-feedback.json) records commands, exit codes,
source hashes and artifact fingerprints; `observed.json` remains under the ignored
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk-feedback/` run directory.

## Not proved here

- Only reporting is exported. Interactive prompts, point/object input sessions, option
  tokens, preview overlays, dynamic menus and observer callbacks remain open under the M01
  command/UI owners.
- A refused command discards its reports; piping failure-path diagnostics through the ABI
  would need an error-carrying channel and is not claimed.
- macOS host and plugin execution are unverified; interactive UI acceptance was not
  performed. No manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: take the next plan-named owner slice — `OM9-L2-01-12` file lifecycle (host file
service with atomic save/open and no document swap inside a command) or the interactive
prompt/input session work at M01 `03-10`/`01-02`.
