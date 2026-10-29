# User decisions for OM9-L3-01-01-003

## 2026-10-16: M00 approved by the user
The user wrote "M00 duyệt" (M00 approved) during this run.

- Treat QymCAD legal reuse for this item (and sibling L3-002) as human-approved on 2026-10-16. Replace `PENDING_HUMAN_M00_APPROVAL` with `HUMAN_APPROVED_2026-10-16 (user: "M00 duyệt")` in this item's evidence, source_crosswalk.json reuse status, validation doc, ledger entry and final report.
- Agents still never approve anything themselves; this is the user's decision, quote it as such.
- Adapted code must still keep `QymCAD/LICENSE` and `QymCAD/THIRD-PARTY-NOTICES.md` notices, and OM9 must still have no build-time dependency on QymCAD.
- Scope (user answer, 2026-10-16: "chỉ tái sử dụng QymCAD trước, còn tới phần nào khác thì hỏi tôi tiếp"): the approval covers ONLY QymCAD code reuse. The other M00 parts (OCCT, UI, 3DM, Associative, License, Reproducible) are NOT approved; whenever work reaches one of them, stop and ask the user. Do not edit `OM9_Milestones/M00*` files.
- OM9-L2-01-02 and OM9-L2-01-03 stay BLOCKED_DEPENDENCY (they are unimplemented modules, not approval gates).

## 2026-10-16: Layers panel placement (explicit user instruction, added scope for this run)
User wrote: "layer cũng phải nằm bên trái chứ phía dưới info & setting, phải giống y chang matrix9 dùng ở độ phân giải nào".

- Requirement: the Layers panel must be docked in the LEFT column, directly BELOW the Info & Settings panel, matching Matrix9's layout exactly, at every supported window size/resolution (not just the default size). It must not appear at the bottom of the window or in the right dock.
- Reference: Matrix9 layout as captured in `skills/openmatrix9-workspace-contract/SKILL.md`, `skills/openmatrix9-ui/SKILL.md`, the MATRIX9_UI_CONTRACT (if present), and the bounded manual excerpts in `OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-14 - Febe Matrix Layers Objectinfo Superselect/reference/` and `.../L2-01-01 - Febe Application Shell Docking/*/reference/`. Read only what is needed; never copy manual images/text into the repo.
- Must stay consistent with the L3-002 docking-region rules (left rail narrowing, right dock yielding under ~900 px, minimum 560x286). Decide and document how Layers behaves when the left column is too short (e.g. shrinks/scrolls, Info & Settings keeps priority) and when the user collapses or floats it (L3-003 roll-up/float rules apply; default/restore position is left, below Info & Settings).
- Verify with the real-window size matrix (same sizes as `tools/validate_m01_l3_002_ui_matrix.py`, plus 1920x1080 and 2560x1440, and SIMULATED 150%) and look at the screenshots.
- This changes a business rule (panel placement). The user's instruction authorizes the behavior change in code. Still present the before/after and proposed skill text in the final report and ask before editing skills.
- Panel layout/movement is still UI state only: it must not change document objects or Undo history.

## 2026-10-16: Matrix9 main window layout (user screenshot) supersedes earlier layout assumptions
Source of truth: `docs/analysis/matrix9-main-window-layout.md` and steering `.kiro/steering/matrix9-main-window-layout.md`. The first L3-003 run (wf_c6ba092eed7e7c12) was stopped by the user before review; its design must be revised to these rules.

- Left column: fixed-width stack of collapsible panels, top to bottom: ICON HISTORY, MAIN MENU, DISPLAY, SNAPS, INFO & SETTINGS, LAYERS, PROJECTS, then builder panels (e.g. GEM ON CRV ADVANCED) at the bottom. The column is taller than the window and is scrolled/clipped, not shrunk to fit. Panel headers: dark bar, UPPERCASE title, small close box at right.
- Right workspace, top to bottom: green two-line `Command:` box, toolbar tab strip with `>>` overflow, one toolbar icon row, viewport tabs (Perspective, Through Finger, Side View, Looking Down, +), 2x2 viewport grid.
- No right dock in Matrix9. The default OM9 layout must not show a right dock. Do not delete the existing right-dock code without a user decision; keep it off by default and flag it in the final report.
- Minimum supported window: 1280x720 logical (user decision). This replaces the L3-002 rule "error below 560x286" and the "right dock yields under ~900 px" rule for the default layout.
- Behaviour at other resolutions/DPI is NOT evidenced by the single capture. Use a fixed logical column width scaled only by DPI, and record it as an assumption pending a second Matrix9 capture. Do not invent further proportional rules.

## 2026-10-16: Left-column width and right dock (user decisions in steering, supersede earlier notes)
`.kiro/steering/matrix9-main-window-layout.md` (updated by the user) is authoritative: left column default width 300 px, user-resizable so SVG icons can grow with it; right dock kept but hidden by default, shown only after the user drags panels to it; Layers (or any panel) never in the right dock by default, in audits, fixtures or Matrix9-parity screenshots; right-dock fixtures use Snaps and are labelled user-docked; parity required across >= 1280x720 logical (1366x768 capped by work area); no proportional compression. The design's 300 pt resizable column and hidden-by-default right dock follow these decisions.
