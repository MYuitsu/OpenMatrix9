# OM9-L3-03-10-005 — Command History Readable Panel

- Sources: `crates/om9-app/src/transcript.rs, crates/om9-app/src/ui.rs`
- Positive oracle: ordered bounded history with prompt/draft separation
- Negative oracle: history cannot overwrite mutable input/caret
- Verification: command_transcript 3 passed; command_sessions 6 passed
- Manual M01-UI review pending.
