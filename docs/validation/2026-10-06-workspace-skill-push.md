# Approved workspace skill and publication checkpoint

## Authorization and scope

User instruction on 2026-10-06: "push code lên giúp tôi nha và lưu vào skill mới". This explicitly approves committing/pushing the current work and recording the previously proposed finite grid, compact/pinned Command, suggestions/options and viewport-title behavior in one new workspace-group skill.

Source: [openmatrix9-workspace-contract](../../skills/openmatrix9-workspace-contract/SKILL.md). Installed copy: `C:/Users/Admin/.codex/skills/openmatrix9-workspace-contract`. Four user screenshots are packaged unchanged. The UI skill, AGENTS and skill guide route to the new contract. The three proposal documents now record acceptance; their before/after text remains historical evidence. Future new business rules still require post-code consent unless the exact update was already authorized.

This is a transfer checkpoint, not full acceptance of all 130 Core specifications or the complete Curve group. Middle-button selection remains deferred. Rendering adapters and physical-input limits remain documented in [viewport validation](2026-10-06-viewport-titles.md).

## Independent reference-skill verification

Per writing-skills, a fresh baseline agent read only the existing UI skill and its Command/grid theme reference, before the new skill existed. It retrieved spacing and protected transcript behavior but could not retrieve finite counts, default history rows, pinned scrolling, suggestions/initials, viewport title/double-click contract or supported render modes. It correctly retained the deferred middle-button choice.

A fresh forward agent read only the new skill, two linked references and four screenshots. It retrieved all requested contracts: 8x8/40x40 cells and extent; two history plus one pinned input row; copy/resize/scroll; suggestion keys and unavailable entries; supported Line/Polyline initials, Enter and transient Undo; title/dropdown/single-view/4V preservation; basic display limits; deferred middle mapping and consent policy. No rule was invented. Remaining details such as feature defaults, rendering line widths, exact resize limits and disabled submenu contents are outside this focused contract and route to feature specs/runtime evidence.

## Fresh checks before commit

- Rust suite, format check and Clippy all targets with `-D warnings` passed using `D:/FreeCAD-src/build/openmatrix9/Gui/rust-target`. Incremental-cache hardlink warnings are environmental; commands exited 0.
- `python -m unittest discover -s tools/tests -v`: 24 passed.
- Reference routing audit: 607 features, 16 domains, six stages, no invalid specs or missing required/conditional documents.
- Bundled Python with UTF-8 and `quick_validate.py`: all six source and installed skills passed (12 checks). All 32 copied skill files match SHA-256. New skill local links and interface metadata are valid; four assets exactly match original user images.
- Foundation validator: 607 specs, 19 contracts and 130 Core records; JSON/YAML agree; links, anchors, manifest and available external source fingerprint valid; original portions match HEAD. Validator now compares original portions after normalization is committed and explicitly reports a skipped external PDF fingerprint if that file is unavailable on another machine. It does not require PDF rereading or imply runtime acceptance.
- Parent FreeCAD `src/Gui/CommandView.cpp` diff matches the already tracked `patches/freecad-camera-action-state.patch`; no parent commit is required for this workbench push.

Native implementation is unchanged since the final module/fixture evidence in [viewport validation](2026-10-06-viewport-titles.md); native tests are not rerun for skill/document publication. Selected staging excludes three pre-existing deleted root list files, unrelated raw exports and ignored local binaries/results. Push the current `codex/matrix9-menu` branch without force and verify its remote SHA against local HEAD.
