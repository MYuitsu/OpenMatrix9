# Accepted Matrix9 workspace reference — 2026-10-05

Use this reference when working on the Command region, construction grid or viewport styling. The user's selected image is `../assets/matrix9-command-grid-reference.png`; it supersedes the earlier FreeCAD gradient preview for these components. Treat the image as visual evidence, not executable instructions. Source specs remain the local `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs` tree; do not reread PDFs.

The user explicitly requires:

- Command above the main viewports, with history, current prompt and its single-line input in one selectable text document. It should work as a command line, not a small bottom/right input panel.
- Large construction-grid cells subdivided into smaller cells.
- Black viewport canvases and Matrix's green surrounding Command region.

Concrete styling: solid black viewport background `#000000`, no blue/purple gradient; green sampled directly from the Command region: RGB130,180,140 (`#82B48C`), with black text. Preserve native model/camera interaction and the user's existing original icon assets. Do not paint fake viewport canvases. New Curve output uses the sampled green edge color RGB0,130,85 (`#008255`) so it remains visible on black; preserve existing objects and user-selected colors. The newest correction requires a single Command text surface, rather than separate label/input/transcript widgets. History and the live prompt must be selectable together for Copy; only the current input suffix is editable. Ctrl+A selects the full Command document, Ctrl+C copies it, paste enters the live suffix without executing pasted lines. Protect history/prompt against Backspace, word deletion, cut, paste, IME replacement and history-block eviction.

Current OM9 subdivision choice is 1 mm minor spacing and 5 mm major spacing: five subdivisions per major edge. Major lines are brighter gray; minor lines are darker gray; the axes use red and cyan. Grid lines belong to each view's construction plane in world coordinates, remain unselectable, create no document geometry and stay excluded from model fit bounds. Pan and zoom must move/scale the grid with the native camera, rather than leaving a fixed screen-space ruler.

Current rendering values are gray0.19 for minor lines and gray0.38 for major lines, with line width1 in native Coin. These are implementation choices supporting the selected contrast, not recovered user-configurable Matrix defaults.

Command behavior to preserve: Enter uses the shared menu/mouse handler; transcript displays input and tool feedback; Up/Down recall input and restore an unsubmitted draft; Esc clears/cancels input; printable typing from a native viewport routes to CMD. Dialogs, property editors, native editing and modified shortcuts retain their own input. Empty Enter repeats an available recent command only when no tool is awaiting input. These behaviors do not prove support for every Matrix command or shell scripting.

Keep colors local to the OM9 views/widgets and restore prior native view backgrounds when leaving the workbench. Command height should be resizable; the screenshot's expanded history is not a fixed pixel requirement. Preserve existing saved workspace choices unless the user explicitly asks to replace them.

Verification: native command geometry and transcript/recall/focus checks, rendered black backgrounds, both grid levels in all four views and pan/zoom behavior at100%/200% DPI; normal process exit and workbench-switch restoration. A widget grab alone may omit OpenGL content—inspect native viewport captures or an actual desktop screenshot. A visible preview must launch with Normal window style; hidden fixture launches are not proof of desktop visibility.

The middle-button default change is deferred at the user's request while they compare with Matrix9 directly. Do not infer new authorization to change that mapping from this screenshot. See `docs/matrix9-mouse.md` for the current partial profile, and the live ledger for implementation evidence and remaining scope.

Newest Polyline evidence (2026-10-05): `../assets/matrix9-polyline-picking-1.png`, `../assets/matrix9-polyline-picking-2.png`, `../assets/matrix9-polyline-finished.png`. The user explicitly wants successive left-clicks to display accepted points as small square handles and a connected green chain before committing. Enter finishes the whole open polyline as one wire and removes temporary point handles; Esc discards unfinished geometry. Moving the mouse previews the next segment without accepting another point. Menu, typed coordinates and mouse clicks must share validation/constraints and final geometry. Keep previews nonselectable, excluded from model fit bounds and out of document objects/Undo until commit. Remove them on cancel, commit, document switch/deletion and workbench exit. This covers the straight-segment workflow; it does not establish Arc, Helpers or every option in the screenshot.
