# OM9-L3-03-15-009 — Project Manager Dock Interaction

## Contract

- Sources: `crates/om9-app/src/ui/chrome.rs, crates/om9-app/src/dock_layout.rs`
- Owner: `OM9-L2-03-15`
- Positive oracle: Projects above Builder, normal collapse/reorder/float/resize
- Negative oracle: dock changes do not save/restore a Job Bag
- Verification: panel_ops 33 passed; shell_docking 10 passed
- Manual M01-UI review pending.
