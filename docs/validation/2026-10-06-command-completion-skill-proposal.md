# Proposed UI skill update: pinned Command and suggestions

Status: accepted 2026-10-06. User: "push code lên giúp tôi nha và lưu vào skill mới". Saved in the new workspace-group skill `skills/openmatrix9-workspace-contract/`; the historical proposal below records the pre-approval state.

User evidence: `C:/Users/Admin/AppData/Local/Temp/codex-clipboard-4aba9356-5f10-4d9e-a244-1edd68e04a6f.png`, supplied 2026-10-05. Local specs remain authoritative; no PDF reread.

This proposal extends the still-pending `2026-10-05-grid-command-options-skill-proposal.md`. Approval can cover both together; a new implementation request alone is not approval to save the earlier proposal.

Before: Command/history scrolled together, allowing the live input to disappear when viewing old history. No separator immediately above the live row or command-name suggestion list.

After: one protected, selectable text document with independently scrolling history, a horizontal separator above the pinned live row, and named-command suggestions while idle. The input remains visible even at the minimum dock height.

## Exact proposed additions to openmatrix9-ui

Target: `skills/openmatrix9-workspace-contract/references/command-and-curve.md`, with a pointer in `skills/openmatrix9-ui/SKILL.md`. The screenshot is packaged in the new skill; source and installed copies are synchronized.

- Command defaults to two history rows above one live input row. Drag the lower dock separator to change history height. Keep the current prompt/input visible at the bottom at every supported dock height and when history is scrolled. Draw a horizontal separator immediately above that live row.
- History, current prompt and input remain one selectable/copyable plain-text transcript. Protect history and prompt from editing; keep draft input on one unwrapped line, including pasted/IME text. Scrolling history must not discard or move the draft.
- While idle, typing a command name opens suggestions below the input. Use original command names from the local specs and supported command mappings. Rank exact matches, prefixes, then contained matches. An available exact name takes priority over an earlier selected suggestion.
- Up/Down select available suggestions; Tab fills the selected name without running it; Enter or a mouse click runs the selected command through the same CMD/menu handler. The first Esc dismisses the suggestion list while retaining the draft; a further Esc cancels/clears as usual. Commands that are unavailable are visibly disabled and cannot be selected to execute.
- Hide named-command suggestions while a point/interactive tool is awaiting options or coordinates, on no match, on focus loss, or on workbench exit. Preserve existing command history recall when the suggestion list is hidden.

This records only implemented behavior. It does not imply that every suggested spec command, or the full core/Curve group, is implemented. The middle-button mapping remains deferred.
