# OM9-L3-03-10-002 — Interactive Command Prompts

- Sources: `crates/om9-app/src/ui.rs, crates/om9-app/src/session.rs`
- Positive oracle: exact prompt visible for each awaited input
- Negative oracle: stale/cancelled prompt cannot accept later input
- Verification: command_state_machine 3 passed; command_feedback_adapter 5 passed
- Manual M01-UI review pending.
