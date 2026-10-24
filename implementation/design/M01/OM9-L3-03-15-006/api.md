# OM9-L3-03-15-006 — Jobbag Restore And Insert

## Contract

- Sources: `crates/om9-app/src/session/jobbag.rs, crates/om9-app/src/project_manager.rs`
- Owner: `OM9-L2-03-15`
- Positive oracle: restore into one transaction with new IDs
- Negative oracle: repeated restore never collides
- Verification: project_manager 8 passed; project_manager_ui 3 passed
- Manual M01-UI review pending.
