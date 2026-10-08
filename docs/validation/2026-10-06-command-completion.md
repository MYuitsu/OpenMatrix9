# Pinned Command and completion — 2026-10-06

User requested the horizontal separator above the live row, Matrix-style named-command suggestions and a Command input that stays visible as the dock height changes. Local specs were used; no PDFs reread.

## Implementation

`CommandConsole` retains one protected/selectable QTextDocument. A read-only history viewport scrolls independently above a horizontal separator; the live input is pinned below it. The default is two history rows and one unwrapped live row, sized from font metrics. Current native fixture reports dock69 logical pixels; lower native dock separator drag expands69 to129. History and input Copy together, selection sync preserves history scroll, and resizing never deletes the draft.

Rust embeds original names generated from local spec `command:` frontmatter, supplemented by supported core mappings. Exact available names take priority over retained selection, followed by prefixes and contained names. Native availability is checked using the existing registered command. Unsupported entries are disabled; no command implementations are inferred from suggestions. Up/Down skip disabled entries, Tab fills, Enter/click dispatches through CMD, Esc first dismisses the popup while preserving the draft. Popup is suppressed during point/interactive tools and closed on focus/workbench exit. Existing recall works with popup hidden.

Pasted and committed IME input normalizes CR/LF and Unicode U+2028/U+2029 to spaces. Qt's single-line-height plain-text viewport could compute a scrollbar maximum past the last document line; centered scroll range plus a last-line clamp fixes the observed blank paint while preserving the live input. A native pixel assertion catches that rendering regression.

## Verification

Rust62 tests, fmt check, Clippy all-targets `-D warnings` and release passed. Incremental-cache hardlink warnings caused copying rather than a compilation failure. Matching MSVC14.44/C++23 native compile/link succeeded. Runtime `build/command-completion-runtime/OpenMatrix9Gui.pyd`, SHA-256 `b341838b141bcf65d9eefbde955fe8db3b458e7be640715c71a02920327d5d2c`; host `build/host-camera-diagnostic/bin/FreeCAD.exe`.

| Final fixture log | Passed checks | Report |
| --- | --- | --- |
| `command-completion-final-command_completion_smoke-1.log` | 29 | `build/command-completion-command_completion_smoke-1/043d08e968f442178ecf6603a6fb0a10/results.json` |
| `command-completion-final-command_completion_smoke-2.log` | 29 | `build/command-completion-command_completion_smoke-2/b57ca3f222354f4ba010ca2ae42f09c1/results.json` |
| `command-completion-final-core_command_theme_smoke-1.log` | 38 | `build/command-completion-core_command_theme_smoke-1/6d2f446138244b93b49508a1bbf14c91/results.json` |
| `command-completion-final-core_keyboard_smoke-1.log` | 28 | `build/command-completion-core_keyboard_smoke-1/d19e609288404f5fb83d1cc1c9dd3576/results.json` |
| `command-completion-final-core_mouse_smoke-1.log` | 46 | `build/command-completion-core_mouse_smoke-1/7e229d6054334479a89892226dc3df07/results.json` |
| `command-completion-final-curve_smoke-1.log` | 26 | `build/command-completion-curve_smoke-1/c1fa972e001842539f72320b7e1b5f24/results.json` |
| `command-completion-final-matrix_grid_command_smoke-1.log` | 37 | `build/command-completion-matrix_grid_command_smoke-1/f57a6d3f75ef4748a75bba86d56bd731/results.json` |
| `command-completion-final-polyline_console_smoke-1.log` | 33 | `build/command-completion-polyline_console_smoke-1/77abcc68868a477f8a6bfae19b6023b6/results.json` |

All eight native processes completed through the smoke runner with exit0. Actual live-viewport and suggestion captures inspected at100%/200%. Native events are Qt-generated; physical desktop input has not been claimed verified.

RED evidence: initial no-separator/history/completion fixture; Unicode multiline paste (`command-completion-render-red.log`); blank prompt paint with invalid firstVisibleBlock (`command-completion-layout-diagnostic.log`); retained Polyline selection overriding exact Line (`command-completion-exact-red.log`). Each regression passed after its fix. The200% initial completion probe had an inactive test window; activating that fixture's own Qt window fixed the focus prerequisite, without removing the product focus guard. Independent read-only review identified Unicode and exact-name issues, reproduced before fixing; final forward review found no further issue.

## Scope and skill persistence

This validates the bounded UI slice. Many core and Curve spec commands remain unavailable; full130-core completion is still in progress. Middle-button mapping is still deferred. Existing unrelated worktree/spec changes were preserved; no commit/push was requested.

Source and installed business skills have not changed for this slice. `2026-10-06-command-completion-skill-proposal.md` extends the prior pending grid/default-height/option proposal. Record rules only after the user's post-verification consent.

Visible preview: `OpenMatrix9CommandUI`, PID2828, Normal launch with separate profile via `build/command-completion-preview.FCMacro`. Preview initialization sets the compact three-row height after hiding native auxiliary docks and places the sample `pol` draft at the editable suffix. Startup status must show the current module, two history rows, visible suggestions and visible live cursor.

2026-10-06 follow-up: the user explicitly approved saving these rules into the new workspace-group skill and committing/pushing this work. The pending-consent statements above describe the state when this slice was first verified. See [approval and skill verification](2026-10-06-workspace-skill-push.md).
