# Command options: inline-only presentation (2026-11-01)

Scope: the Command Line presentation of command options. The user reported that the
commands still showed a row of option buttons above the command row and asked for the
Polyline presentation — options inside the prompt — in every command.

## Before

- `ui/command.rs` had two option presentations. A prompt that already spelled its option
  labels was underlined in place; every other command fell back to a floating `Area` of
  button widgets drawn above the pinned row (`option-tokens`).
- The fallback appeared whenever the stored prompt did not carry the current labels. That
  state is reachable because only `MatrixApp::feedback` refreshed the prompt. Toolbar
  actions (`ui/top11::top11_action`), the typed Top11 path (`DispatchOutcome::Top11`) and
  MCP/controller starts never called `feedback`, so the live row kept its previous prompt
  (`Command: `) while the session published options: the reported row of buttons.
- Long option signatures also overflowed the single command row, leaving labels past the
  right edge without a clickable rectangle (Circle publishes 18 options).

## After

- `command_ui` adopts `Controller::prompt_with_options()` every frame. Every invocation
  path (typed, menu, toolbar, MCP) shows the same inline signature; a stale prompt can no
  longer produce a second presentation.
- The floating chip `Area` and its `shortcut_button` helper are deleted. Options exist only
  as underlined, clickable spans inside the prompt text.
- Idle prompts wrap inside the command panel, so every inline option stays visible and
  clickable; the history area above the row is sized from that measured height. While a
  draft is typed the pinned row keeps its documented single-line horizontal-scroll
  behaviour (`pinned_command`).
- When the prompt changes, the editor caret is re-anchored to the draft end; a paste or
  keystroke after a stage change can no longer be dropped by a stale character boundary.
- Box keeps its documented plain-word mode prompt (`Diagonal 3Point Vertical Center`), so
  `commands_share_keyboard_pointer_and_typed_activation` now expects the plain Box label
  only; the prompt text itself shows the active mode.

## Evidence

Filed as tests in `crates/om9-app`:

- `command_inline_options` (9): every starter publishes an inline-only prompt; Polyline
  keeps its own signature; Idle never gains one; suggestions drop below the row; a
  toolbar Top11 start (`top11_invoke`) reaches the owner prompt inline; Box and Copy
  options resolve to rectangles on the command row; clicking `Center` reaches the owner;
  and a catalog sweep over `assets/features.json` starts **67** commands headlessly,
  **41** of which publish options — every one names all of its labels in one prompt line.
- `command_options_interface` (74): keyboard, pointer and typed activation through the
  shared snapshot, including stale-run rejection and long-signature pointer clicks.
- `pinned_command` (7): pinned live row, history scroll, copy-as-one-document and the
  unwrapped draft contract.
- `chrome_actions` (30): Box command-line echo and option tokens inside the prompt.

## Limitations

- No manual M01-UI gate; the product window was not driven by a human after the change.
- Full `cargo test -p om9-app --tests --no-fail-fast` passes every command-option suite but
  ends with 9 failing targets: `--lib` (icon-size audit), `command_transactions` (panic in
  `session.rs:2837`), `escape_commands` (dropped texture deltas in its own helper),
  `file_widgets` (7, already recorded as a baseline failure in the 2026-10-26 checkpoint),
  `global_status_surface` (status kind after Escape), `mesh_fill` (grid fill),
  `shell_docking` (left column/Builder/status), `transform_corrections` (command
  availability) and `ui_events` (grid colours).
  - `ui_events` reproduces on an 18:47 build made before this change; `--lib` icon-size
    reproduces on an 18:39 build. Neither touches the command row.
  - `escape_commands` fails identically with this change's prompt refresh disabled, so the
    texture-delta panic is not caused here.
  - The remaining targets fail in owners this change never edits and coexist with parallel
    in-flight work in the same worktree.
- The change is not committed; the worktree carries unrelated in-flight changes.
