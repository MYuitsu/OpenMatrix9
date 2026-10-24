# OM9-L3-03-15-001 — Matrix File New Open Save Actions

## Contract

- Sources: `crates/om9-app/src/file_lifecycle.rs, crates/om9-app/src/session/files.rs`
- Owner: `OM9-L2-03-15`
- Positive oracle: file lifecycle New/Open/Save/Save As + cancel/failure preserves document
- Negative oracle: cancelled/failed picker does not discard the current scene
- Verification: file_lifecycle 20 passed; file_widgets 21 passed
- Manual M01-UI review pending.
