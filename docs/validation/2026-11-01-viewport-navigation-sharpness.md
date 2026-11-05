# Viewport navigation: full resolution at 30/24 Hz

The navigation scheduler now starts at 30 Hz and can fall back to 24 Hz after sustained missed frames, then recover to 30 Hz with hysteresis. `OM9_FRAME_HZ` accepts fixed 24 or 30; unsupported values fall back to automatic 30. The scheduler retains Q0 under load, so the normal document viewport requests full physical resolution and 4x MSAA while moving; existing stress-render paths have their own sample count. Actual achieved FPS is hardware/workload dependent and can fall below 24.

Source: `crates/om9-app/src/frame_scheduler.rs` SHA-256 `DC71EE3DCA4204FA3A6CFEDC6E32E967CD4B85C00348330144923BDC199515FA`; base revision `89ef2676b597ebc5ab38fa5c6acf0ad47a2f261b`, with a pre-existing dirty worktree. Windows, default product features, `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`.

- `cargo test -p om9-app -j 2 --lib frame_scheduler`: 4 passed.
- `cargo test -p om9-app -j 2 --test navigation_deadlines --test ui_replay --test gpu_navigation_publication`: 10 passed across 3 suites; GPU tests can silently skip if no adapter, so this count alone does not establish GPU coverage.
- `python tools/public_source_audit.py`: 0 errors before this evidence update.

Scheduler unit checks establish policy and simulated frame timing; the egui replay uses synthetic input. Real OS pointer input, GPU presentation, visual sharpness, achieved FPS and released executable remain unverified. Next: measure navigation at 30/24 targets on the actual GPU with interaction log, and inspect moving and settled frames for representative light and heavy models.
