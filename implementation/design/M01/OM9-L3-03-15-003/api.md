# OM9-L3-03-15-003 — Project Manager Project Row List

## Contract

- Sources: `crates/om9-app/src/project_manager.rs, crates/om9-app/src/ui/chrome.rs`
- Owner: `OM9-L2-03-15`
- Positive oracle: create/rename/select project rows with stable IDs
- Negative oracle: duplicate labels allowed only with distinct IDs; unknown ID rejected
- Verification: project_manager 8 passed; project_manager_ui 3 passed
- Manual M01-UI review pending.
