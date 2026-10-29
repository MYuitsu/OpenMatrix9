# OM9-L2-04-10 Gems On Curves And Surfaces — evidence

Automated run on 2026-10-27 (`Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=H:/OpenMatrix9/cargo-target; -j 2; app tests need the OCCT bin and VC runtime on PATH`), source revision `71852257e59f4117c3b3bfca73f047ac546f76c2`
(dirty checkout).

## What was run

- `cargo test -p om9-core --test gem_placement -j 2` -> exit 0
- `cargo test -p om9-app --test gem_placement -j 2` -> exit 0
- `cargo test -p om9-core --test gem_assets -j 2` -> exit 0
- `cargo test -p om9-app --test gem_panel -j 2` -> exit 0
- `cargo test -p om9-core --test atomic_transactions --test transaction_scopes -j 2` -> exit 0
- `cargo check --workspace --all-targets -j 2` -> exit 0
- `cargo clippy -p om9-core -p om9-app --all-targets -j 2` -> exit 0

## Per-L3 result

| L3 | Command | Gate | L4 receipts |
| --- | --- | --- | --- |
| `OM9-L3-04-10-001` | `BuilderGemOnCrv / gvGemOnCrv` | `M05-UI` | OM9-L4-10-001-03 |
| `OM9-L3-04-10-002` | `BuilderGemOnCrvAdvanced / gvGemOnCrvAdvanced` | `M05-UI` | OM9-L4-10-002-03 |
| `OM9-L3-04-10-003` | `BuilderGemCountOnCrv / gvGemCountOnCrv` | `M05-UI` | OM9-L4-10-003-03 |
| `OM9-L3-04-10-004` | `BuilderGemListOnCrv / gvGemListOnCrv` | `M05-UI` | OM9-L4-10-004-03 |
| `OM9-L3-04-10-005` | `BuilderGemOnCrvMulti / gvGemOnCrvMulti` | `M05-UI` | OM9-L4-10-005-03 |
| `OM9-L3-04-10-006` | `BuilderGemProfile / BuilderGemGuides / gvGemProfile / gvGemGuides` | `M05-UI` | OM9-L4-10-006-03 |
| `OM9-L3-04-10-007` | `BuilderGemOnSurface / gvGemOnSurface` | `M05-UI` | OM9-L4-10-007-03 |
| `OM9-L3-04-10-008` | `BuilderGemFollow / gvGemFollow` | `M05-UI` | OM9-L4-10-008-03 |
| `OM9-L3-04-10-009` | `BuilderGVDGemFlow / gvGemFlow` | `M05-UI` | OM9-L4-10-009-03 |
| `OM9-L3-04-10-010` | `BuilderGVDGemSplop / gvGemSplop` | `M05-UI` | OM9-L4-10-010-03 |
| `OM9-L3-04-10-011` | `GemPlacementLinks / gvGemPlacementLinks` | `M05-UI` | OM9-L4-10-011-03 |

All eleven L3 are commands with a human-facing M05-UI gate, so none is marked DONE and no
`ui-review.json` is written. Each L3 is WIP with automated evidence and an open manual gate.

## Not claimed

- M05-UI manual acceptance of the eleven placement commands.
- Matrix assets: none were read, copied or shipped.
- A shaded viewport preview proxy for an open placement session.
- The crate-wide app lib run: 188 of 192 pass; the 4 `ui::layer_dialog_audit` failures need
  `build/standalone-validation` write access that the restricted test token denies in this
  session (see `app-lib-os-restriction.log`).
