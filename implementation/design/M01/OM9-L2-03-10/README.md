# M01 OM9-L2-03-10 — Febe Matrix Commandline Feedback Adapter

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Target:** standalone Rust OM9
**Owner:** `OM9-L2-03-10` (M01)

## Delivered

- `Transcript` (`crates/om9-app/src/transcript.rs`) owns the ordered, bounded history, current
  prompt, draft, caret/IME text buffer and history scrolling.
- `MatrixApp` routes Enter/typed input through the same Controller used by menus and shortcuts:
  aliases resolve to canonical feature IDs, and an accepted command echoes `[feature-id]`.
- Interactive prompts display the exact Controller prompt while a command awaits a point,
  object or option; cancelled/stale prompts are refused instead of writing to the wrong state.
- Clickable option tokens use the same typed option parser as keyboard entry.
- Numeric/coordinate input is validated through the typed parser; locale comma/NaN are refused
  without mutating the active command.
- Escape aborts the running command; Enter/Space repeat only the allowlisted completed command.
- Command history is a readable panel with bounded retention and scroll position.
- Feedback/errors are surfaced on the status line and in history; no silent failure.

## Verification — 2026-10-22

| Test target | Result |
|---|---:|
| `command_feedback_adapter` | **5 passed** |
| `command_options_and_repeat` | **6 passed** |
| `command_shortcuts` | **3 passed** |
| `command_state_machine` | **3 passed** |
| `command_tokens` | **1 passed** |
| `command_transcript` | **3 passed** |
| `command_sessions` | **6 passed** |
| `status_bar_overlap` | **4 passed** |

Total: **31 passed, 0 failed**.

Log:
`build/standalone-validation/ui-artifacts/m01-l2-03-10-20261022/acceptance.log`

## Shared Command interface — 2026-10-25

The common `CommandOptionInterface` now covers all 39 existing declaration groups; the UI no longer chooses a per-command parser. Value entry, advertised unique shortcuts, IME/draft/focus guards and current owner values are shared. 466 distinct tests pass; the known preexisting Box assertion and two manual fixture omissions are documented. Native Circle/Helix/Rectangle were checked in the new debug window. [Evidence](../../../../docs/validation/2026-10-25-command-parameter-interface/README.md).

## Remaining

2026-10-25 parameter-interface follow-up: all eight Polyline options are now described by one typed `PolylineOptionInterface`. Prompt values, underlines, pointer targets and typed initials/names derive from that snapshot; a 24-scenario route/parameter matrix and 216 app tests pass. [Evidence](../../../../docs/validation/2026-10-25-polyline-parameter-interface/README.md).

2026-10-25 follow-up: Polyline inline options now share the typed session handler; M toggles Line/Arc before global Mid snapping, P/H update their flags, and the prompt refreshes without adding shortcut letters to the draft. See [targeted validation](../../../../docs/validation/2026-10-25-command-polyline-shortcuts/README.md). This follow-up does not sign the manual gate below.

- Manual M01-UI visual review on the real Windows window is still required.
- No human sign-off is claimed.

## Detailed Polyline-based migration specification — 2026-10-26

[Handoff package](../command-options-polyline-standard-20261026/README.md): target contract, source inventory and per-owner cards, 13 implementation packages, 50 acceptance templates and a copyable GPT Luna prompt. This is documentation-validated planning; it does not mark the future entry/render/owner migrations implemented or close any human gate.
