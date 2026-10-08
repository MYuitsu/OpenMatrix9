# Proposed UI skill update: grid and Command options

Status: accepted 2026-10-06. User: "push code lên giúp tôi nha và lưu vào skill mới". Saved in the new workspace-group skill `skills/openmatrix9-workspace-contract/`; the historical proposal below records the pre-approval state.

User evidence: grid image `C:/Users/Admin/AppData/Local/Temp/codex-clipboard-44177292-656b-42b8-85e4-40f3b6aaad54.png` and Command image `C:/Users/Admin/AppData/Local/Temp/codex-clipboard-8c6a65c0-1fc2-4cc0-8429-5a119996dc03.png`, supplied 2026-10-05. Local specs remain authoritative; no PDF reread.

Before: world grid spanned 200 minor cells per edge; Command default dock height was 180 logical pixels; Curve prompts used semicolon-separated options without visible initial-letter markers. Last-point Undo and abbreviated option inputs were absent.

After: finite 40 mm world grid with 40 minor cells and eight major cells per edge; Command default uses three text rows with a native draggable separator; Curve options use parentheses, initial-letter underlines and the shared mouse/CMD dispatcher.

## Exact proposed additions to openmatrix9-ui

Target: `skills/openmatrix9-workspace-contract/references/command-and-curve.md` and `references/grid-and-viewports.md`, with a pointer in `skills/openmatrix9-ui/SKILL.md`. The user images are packaged in the new skill; source and installed copies are synchronized.

- The construction grid is bounded to eight by eight major cells, each divided into five by five minor cells: forty by forty minor cells. With current OM9 spacing, extent is -20 to +20 mm on each construction-plane axis, minor spacing 1 mm and major spacing 5 mm. Grid remains world-aligned, unpickable and excluded from model fit bounds; pan/zoom changes its screen position/scale.
- Command defaults to two visible history rows plus one live input row, sized from font metrics. Drag its lower separator to expand or shrink history. Preserve the single selectable text document, protected history/prompt and current draft; the input line does not wrap. Keep the user's resized height through workbench switches.
- Supported Curve options are displayed inside `( )` with the initial character underlined. Enter confirms typed case-insensitive initials: Polyline P/P=Y/P=N for PersistentClose, C for Close once three points exist, M/M=L for the existing straight Line mode, L/L=value for Length after a point, and U for Undo after a point; Line B/B=Y/B=N controls BothSides before its first point. Full option names remain accepted. A plain click on a supported option name submits it through the same input handler when the draft is empty; dragging selects text and a nonempty draft is preserved.
- Polyline Undo removes the last accepted point and refreshes its transient preview without a document Undo entry. Enter still commits the complete wire in one transaction. Do not infer Arc, Helpers or continuously closed PersistentClose preview support from these options; those remain unfinished.

The consent question applies only to recording this business contract in the skill. AltGr/IME/Unicode fixes restore the already agreed protected-input behavior and do not require a separate business-rule question.
