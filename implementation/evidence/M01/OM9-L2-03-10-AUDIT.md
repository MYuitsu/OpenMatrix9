# OM9-L2-03-10 — Febe Matrix Commandline Feedback Adapter: audit of the current code

Date: 2026-10-20. Authority order (user direction): **Matrix9/Rhino5 documentation** first, Matrix9 decompiled
reference second, QymCAD Rust reference third (reference only).

Documentation read for this audit: Rhino 5 p18–20 (start from the command line), p22 (command window, options,
click/type/underlined letter), p49–53 (coordinates), p21/p22 (undo, repeat), Matrix 8 p27–29 (builders vs
commands) and p36 (command history).

| L3 | Title | Status | Evidence |
| --- | --- | --- | --- |
| 03-10-001 | Commandline Entry And Submit | **PASS** | aliases `Line`/`Ln`, `Point`/`Pt`; canonical feature id echoed after submit; unknown alias reports `unknown command: <text>` with no document node; `command_feedback_adapter` 5 tests |
| 03-10-002 | Interactive Command Prompts | **PASS** | prompt derived from the one session state (`Start of line (BothSides=No): `, `End of line…`, `Location of point: `); a committed document change during a prompt cancels it and refuses the stale point (`a_document_change_during_a_prompt_cancels_it_safely`) |
| 03-10-003 | Command Options Clickable Tokens | **PASS** | declared options render as clickable tokens above the entry row; a click runs the same `Name=Value` parser path; clicking twice only toggles, never commits (`command_tokens`); underlined letter `B` also toggles (`the_underlined_option_letter_toggles_the_flag`) |
| 03-10-004 | Numeric And Coordinate Feedback | **PASS (behaviour) / PARTIAL (message language)** | `Length=12.5` produces a 12.5 mm line and `RotateZ 90` a 90° turn; `Length=12,5` and `Length=NaN` are refused as typed errors with no mutation (`a_locale_comma_and_nan_are_refused_without_mutating_the_command`); messages are English-only, no localisation catalogue |
| 03-10-005 | Command History Readable Panel | **PASS (behaviour) / PARTIAL (policy surface)** | one ordered transcript, 64 KiB history budget and 256-command recall ring, whole lines retired; file-dialog messages now keep only the file name (`transcript::redact_paths`, 3 unit tests); the text of the retention policy is not shown in the UI |
| 03-10-006 | Commandline Escape Repeat Shortcuts | **PASS** | Esc aborts without geometry or history; empty Enter repeats the last allowlisted command; Space repeats when the editor is idle and never repeats while editing (`command_shortcuts`, 3 tests) |

## Code changes made while auditing

- `crates/om9-app/src/session.rs`: `resolve_alias` (declared aliases, `Ln`, `Pt`), `active_feature`, `both_sides`/`set_both_sides`, underlined-letter `B` toggle, `point_descriptor`.
- `crates/om9-app/src/ui.rs`: canonical id echo after a successful submit; `Length` underlined letter starts `Length=`.
- `crates/om9-app/src/ui/command.rs`: clickable option tokens and the Space repeat rule.
- `crates/om9-app/src/transcript.rs` + `ui/files.rs`: path redaction for history messages.

## Tests

```
command_transcript 5  command_sessions 5  command_state_machine 4  command_options_and_repeat 6
command_events_surface 6  command_context 3  command_feedback_adapter 5  command_tokens 1  command_shortcuts 3
workspace clippy: pass  (cargo clippy -p om9-app --all-targets -- -D warnings)
```

## Remaining

- Localisation catalogue for typed-value messages (the behaviour is correct; the text is English-only).
- The retention policy (64 KiB / 256 commands) is explicit in code but not shown to the user.
- M02-UI style human review of the command line remains a human gate.

Changed-file SHA-256:

```
16b5293b6d75b6ff088411ea733f29d63c645072586ff0462828de62095fa7cf  crates/om9-app/src/session.rs
62b4da82b44bfc53cbc71dcd726245bb753c4ef2790add47d256edc4970f93e5  crates/om9-app/src/ui.rs
891f40e65e77ea3f5e3b674730c5b9a9b18b68ea214818e0bdf55ae2240d12bb  crates/om9-app/src/ui/command.rs
df5ad74a1d12826d531348b6fe6d2ff37b785dab10341d98fe8b5357ddb117a3  crates/om9-app/src/transcript.rs
30489cd9f9f769a0da492fdc5a5ac8dee3a8e63691fd72b695cac0e685597d89  crates/om9-app/src/ui/files.rs
```
