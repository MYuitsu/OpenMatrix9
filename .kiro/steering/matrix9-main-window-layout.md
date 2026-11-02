# Matrix9 main window layout (remembered from user screenshot, 2026-10-16)

Full analysis: `docs/analysis/matrix9-main-window-layout.md`. Use it for any OM9 UI layout, docking or L3-002 work. The screenshot bitmap is not stored in the repo.

- Left column is a fixed-width (~21% of a 1024 px capture) stack of collapsible panels, top to bottom: ICON HISTORY, MAIN MENU, DISPLAY, SNAPS, INFO & SETTINGS, LAYERS, PROJECTS, then builder panels such as GEM ON CRV ADVANCED (bottom, clipped by the window edge).
- LAYERS sits directly below INFO & SETTINGS, in the left column. The user requires it to match Matrix9 exactly at any resolution.
- Right workspace, top to bottom: green two-line `Command:` box, toolbar tab strip with `>>` overflow, one toolbar icon row, viewport tabs (Perspective, Through Finger, Side View, Looking Down, +), then a 2x2 viewport grid.
- No right dock appears in the screenshot. User decision (2026-10-16): keep the right dock, but it is hidden by default and appears only after the user drags panels to it. Never put Layers (or any panel) in the right dock by default, in audits, fixtures or screenshots that claim to match Matrix9. Right-dock test fixtures use a neutral section (Snaps) and are labelled as user-docked cases.
- LAYERS must be in the left column, directly below INFO & SETTINGS and above PROJECTS (user confirmed, 2026-10-16).
- Parity with Matrix9 is required across the supported range only (>= 1280x720 logical, 1366x768 capped by the work area). When the left stack is taller than the window it scrolls/clips like Matrix9; no proportional compression.
- Left column: default width 300 px, user can widen it (resizable) so SVG icons can be larger; icon size should follow the widened column. Do not invent other proportion rules; ask for a second capture first.
- Minimum supported window: 1280x720 logical (user decision, 2026-10-16).
- Not evidenced: how proportions change with resolution or DPI.
