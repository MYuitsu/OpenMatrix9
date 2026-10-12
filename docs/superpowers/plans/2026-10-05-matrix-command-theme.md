# Matrix command region and viewport theme

> Execution: use the existing checkout and shared original handlers. Implement inline with native verification; preserve unrelated uncommitted work.

Goal: match the user's 2026-10-05 reference: top green command history with an inline entry row, black native viewports, and major grid cells subdivided into minor cells. Update the source and installed openmatrix9-ui skill with this accepted reference.

Architecture: CurveController retains command dispatch and owns the improved command region. Rust core_views supplies grid classification, while C++ builds nonselectable world-space Coin lines and scopes black backgrounds to OM9 views. Original tool names and menu/CMD/mouse handlers stay shared.

Tasks:

- [x] Native RED fixture for top command dock, history/recall/viewport typing, default repeat, black render and differentiated world-space grid. Rust RED for minor/major/axis classification.
- [x] Improve Command UI and input handling; preserve existing widget IDs and point-tool prompt adapters, Escape, modal/edit guards.
- [x] Build major/minor/colored-axis grid and local black backgrounds, restoring original backgrounds on workbench exit.
- [x] Native100%/200% fixtures, existing keyboard/Curve/mouse/capture regressions and clean shutdown. Inspect real images and open a visible workspace with the new module.
- [x] Store source screenshot, update source/installed UI skill and evidence/ledgers. Do not claim all130 core features complete.

Review focus: text entry outside CMD must preserve dialogs/native edit and modified shortcuts; history must include results and preserve drafts; empty Enter must use the shared available command; black styling must not leak into other workbenches; grids must stay world-space and excluded from selectable geometry/fit bounds.
