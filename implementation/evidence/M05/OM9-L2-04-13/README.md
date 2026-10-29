# OM9-L2-04-13 Gem Instance Edits And Reports — evidence

Automated run on 2026-10-28 (`Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=H:/OpenMatrix9/cargo-target; -j 2; app tests need the OCCT bin and VC runtime on PATH`), source revision `71852257e59f4117c3b3bfca73f047ac546f76c2`
(dirty checkout).

## What was run

- `cargo test -p om9-core --test gem_edits -j 2` -> exit 0
- `cargo test -p om9-app --test gem_edits -j 2` -> exit 0
- `cargo test -p om9-app --lib -j 2 gem_edits` -> exit 0
- `cargo test -p om9-core --test gem_assets -j 2` -> exit 0
- `cargo test -p om9-app --test gem_panel -j 2` -> exit 0
- `cargo test -p om9-core --test atomic_transactions --test transaction_scopes -j 2` -> exit 0
- `cargo check --workspace --all-targets -j 2` -> exit 0
- `cargo clippy -p om9-core -p om9-app --all-targets -j 2 -- --cap-lints warn` -> exit 0

## Per-L3 result

| L3 | Command | Gate | L4 receipts |
| --- | --- | --- | --- |
| `OM9-L3-04-13-001` | `BuilderGemPositioner / gvGemPositioner` | `M05-UI` | OM9-L4-13-001-03 |
| `OM9-L3-04-13-002` | `BuilderGemControl / gvGemControl` | `M05-UI` | OM9-L4-13-002-03 |
| `OM9-L3-04-13-003` | `OthersMatchAttributes / gvMatchAttributes` | `M05-UI` | OM9-L4-13-003-03 |
| `OM9-L3-04-13-004` | `OthersSaveStyle / gvSaveStyle` | `M05-UI` | OM9-L4-13-004-03 |
| `OM9-L3-04-13-005` | `OthersLoadStyle / gvLoadStyle` | `M05-UI` | OM9-L4-13-005-03 |
| `OM9-L3-04-13-006` | `BuilderGemUpdate / gvGemUpdate` | `M05-UI` | OM9-L4-13-006-03 |
| `OM9-L3-04-13-007` | `BuilderGemMap / gvGemMap` | `M05-UI` | OM9-L4-13-007-03 |
| `OM9-L3-04-13-008` | `BuilderGemReporter / gvGemReport` | `M05-UI` | OM9-L4-13-008-03 |
| `OM9-L3-04-13-009` | `GemAttributeHistory / gvGemHistory` | `M05-UI` | OM9-L4-13-009-03 |

All nine L3 are commands with a human-facing M05-UI gate, so none is marked DONE and no
`ui-review.json` is written. Each L3 is WIP with automated evidence and an open manual gate.

## Not claimed

- M05-UI manual acceptance of the nine edit/report commands.
- Matrix assets: none were read, copied or shipped.
- M05-UI pixel acceptance of the painted Gem Map overlay and the reporter key window.
- A gemological certificate: the reporter prints a computed mass with a material density and a
  trade-formula cross-check, not a certified weight.
