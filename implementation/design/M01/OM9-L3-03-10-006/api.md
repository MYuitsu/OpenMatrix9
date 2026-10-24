# OM9-L3-03-10-006 — Commandline Escape Repeat Shortcuts

- Sources: `crates/om9-app/src/ui.rs, crates/om9-app/src/session.rs`
- Positive oracle: Escape aborts and Enter/Space repeat only allowlisted command
- Negative oracle: non-repeatable file/destructive command never repeats
- Verification: command_shortcuts 3 passed; command_options_and_repeat 6 passed
- Manual M01-UI review pending.
