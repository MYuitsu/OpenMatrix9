---
name: openmatrix9-headless-testing
description: Design, run, or assess precise headless tests for OpenMatrix9 egui interactions, CAD commands, geometry, and offscreen viewport rendering, minimizing diagnostic output while reporting evidence limits.
---

# OpenMatrix9 headless testing

Use for OM9 testing without opening its desktop window. Start with `openmatrix9-workflow` and `openmatrix9-build-validation`, root AGENTS.md, the relevant milestone/contract and current code. Select tests for the requested behavior, not a generic full suite. Do not start implementation or change business rules merely because a test workflow is requested.

## Choose independent evidence

1. **CAD behavior:** Rust/controller tests or `om9 --mcp` can verify typed geometry, prompt/options, selection, layers, transaction atomicity, Undo/Redo and persistence. Specify independently expected values and tolerances. For native geometry exercise the actual configured adapter, check errors and ownership as relevant, and measure geometric/topological properties, not only object counts or screenshots.
2. **egui interactions:** `crates/om9-app/src/ui_replay.rs` runs `MatrixApp::show` with `egui::Context` and synthetic pointer/keyboard events, returning per-step receipts. Prefer existing real widget paths and assert the state change, not just that a widget opened. For menus/CMD test both entry paths to the same command and actual result where applicable. Check layout rectangles across relevant resolutions and DPI. Existing examples: `tests/ui_replay.rs`, `tests/window_minimum.rs`, `tests/workbench_regions_sizes.rs`.
3. **Offscreen viewport:** `--render project.om9 out-dir [WIDTHxHEIGHT]` writes four PNGs; `--render-4 project.om9 out.png [WIDTHxHEIGHT]` writes one 2×2 sheet (size is per tile). `src/headless.rs` loads a saved document, renders the published scene and grids via wgpu without egui or a window. It needs an available GPU adapter. Use a controlled fixture, camera and resolution, and assert meaningful visible content or a reviewed image comparison. PNG signature, dimensions, or successful process alone do not prove geometry, visibility, orientation, or visual parity. Verify skip/no-adapter outcomes explicitly; never count an unexecuted GPU test as a pass.
4. **Visual UI:** viewport captures do not include egui chrome. When UI appearance matters, use a suitable existing offscreen egui rendering harness if present, or propose/add one only when authorized and justified. Confirm whether it renders the actual viewport GPU callbacks before claiming a complete workbench image. Pin fixture, font, dimensions, DPI and renderer; examine baseline and diffs, using narrow tolerances. Do not automatically accept new baselines. Image comparisons supplement semantic assertions.

For CAD commands involving points, use independent expected world coordinates, relevant signs and view orientations, and compare preview and committed geometry where applicable. Test cancellation, invalid input and Undo when material. Test true UI click/keyboard routing separately from direct controller/dispatch calls: a dispatcher call does not prove a menu click, and scene publication does not prove GPU presentation.

## Replay and MCP limits

Inspect current implementation and schemas before relying on it; source may change. At the time this skill was authored, each `om9_ui` call clones the server document into a fresh `MatrixApp`, returns receipts, and does not write replay changes to the MCP server. Put dependent gestures in one replay and do not assume state persists across calls or that a later MCP save contains UI replay edits. Replay manually activates a view for some pointer events and feeds synthetic focus/events; it does not prove native OS focus or identical desktop event routing. Direct `om9_dispatch` tests dispatch, not clicking the menu. A saved-project render does not capture a running command preview unless explicitly implemented and verified.

## Run and report

- Follow repo rules: root workspace, `CARGO_TARGET_DIR=L:\om9-target`, Cargo `-j 2`, RTK; retain the product's default features for product evidence. If using `--no-default-features`, label it as fallback and do not claim default OCCT/BRep coverage. Resolve current binaries, flags, native runtime and fixture setup from code before running.
- Run focused checks first. Example PowerShell: `$env:CARGO_TARGET_DIR = 'L:\om9-target'`, then `rtk cargo test -p om9-app -j 2 --test ui_replay` or `rtk cargo test -p om9-app -j 2 --test headless_render`. Check exit status, test count and skips; don't treat filtered zero tests as success. Run the applicable broader checks and public source audit when changes require them. Follow the artifact retention instruction after a validation batch.
- Minimize tokens: obtain concise status/counts and targeted assertions first; keep full logs and image artifacts on disk, inspect failure details or diffs only when needed. Never suppress failure status or discard needed evidence for brevity.
- State fixture, configuration, checks, results, skips and the exact scope proved; distinguish semantic tests, simulated egui input, offscreen GPU and real desktop. Record evidence and sync progress as required by project instructions after meaningful verified work. Do not label an unrun desktop, OS focus, actual swapchain/present, native adapter or release gate as verified. If desktop testing is outside the request, report it as a remaining limit rather than opening the app.
