# OM9-L2-03-21 — Model-space annotations

Owns thirteen existing Measure catalog commands, now implemented in one typed Rust owner.
The L3 records map their original ids/icons and aliases; L4 records separate model, UI and acceptance scope.

- Contract: `implementation/design/M03/measure-completion-20261026/ANNOTATION-CONTRACT.md`
- Model: `crates/om9-core/src/annotation.rs`; session: `crates/om9-app/src/session/annotation.rs`
- Tests: `crates/om9-app/tests/annotation.rs`, `annotation_ui.rs`
- Evidence and limitations: `docs/validation/2026-10-26-measure-completion/README.md`

Eight semantic tests and three separate UI-frame tests cover values, all thirteen bindings, creation/edit,
style/recenter, Unicode, preview/Cancel, identity, atomic Undo/Redo, disconnected strokes and native OM9 archive.
Native desktop replay and M03-UI human signature remain open. Rhino dimension/style/layout interchange
is a separate open owner, not proven by a versioned OM9 extension. Session defaults do not persist as document styles.
