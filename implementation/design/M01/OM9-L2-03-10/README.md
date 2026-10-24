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

## Remaining

- Manual M01-UI visual review on the real Windows window is still required.
- No human sign-off is claimed.
