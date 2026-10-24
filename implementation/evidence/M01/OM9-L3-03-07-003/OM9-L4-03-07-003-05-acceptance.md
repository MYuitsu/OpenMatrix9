# OM9-L4-03-07-003-05

- L3: `OM9-L3-03-07-003`
- Date: 2026-10-22
- Acceptance: **AUTOMATED PASS; MANUAL M01-UI PENDING**
- Command: `cargo test -p om9-app --test chrome_actions --test left_stack_layers --test panel_ops --test shell_docking --test dock_layout_default --test dock_layout_persistence --test command_sessions --test command_options_and_repeat --test startup_contract -j 2`
- Results: chrome 13, left stack 8, panel ops 33, shell docking 10, dock default 7, dock persistence 6, command sessions 6, options/repeat 6, startup 3.
- Log: `build/standalone-validation/ui-artifacts/m01-l2-03-07-20261022/acceptance.log` (SHA-256 `9b778a25107cab9d6a2f0cbb4b311cdfc8f72c0cdc595a4b0527bdbce7c1905b`)
- Manual visual review is not claimed.
