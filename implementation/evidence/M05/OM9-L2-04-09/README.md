# OM9-L2-04-09 Gem Asset Shapes And Loader — evidence

Automated run on 2026-10-27 (`Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=L:/om9-target; -j 2`), source revision `71852257e59f4117c3b3bfca73f047ac546f76c2`
(dirty checkout).

## What was run

- `cargo test -p om9-core --test gem_assets -j 2` -> exit 0
- `cargo test -p om9-app --test gem_panel -j 2` -> exit 0
- `cargo test -p om9-core --test atomic_transactions --test transaction_scopes -j 2` -> exit 0
- `cargo check --workspace --all-targets -j 2` -> exit 0
- `cargo clippy -p om9-core -p om9-app -p om9-kernel --all-targets -j 2 -- -D warnings` -> exit 0
- `cargo test -p om9-kernel --features 3dm-brep --test gem_brep -j 2` -> exit 0

## Per-L3 result

| L3 | Gate | L4 receipts |
| --- | --- | --- |
| `OM9-L3-04-09-001` | `AUTO_ONLY` | — |
| `OM9-L3-04-09-002` | `AUTO_ONLY` | — |
| `OM9-L3-04-09-003` | `OPEN` | OM9-L4-09-003-03 |
| `OM9-L3-04-09-004` | `OPEN` | OM9-L4-09-004-03 |
| `OM9-L3-04-09-005` | `OPEN` | OM9-L4-09-005-03 |
| `OM9-L3-04-09-006` | `OPEN` | OM9-L4-09-006-03 |
| `OM9-L3-04-09-007` | `OPEN` | OM9-L4-09-007-03 |
| `OM9-L3-04-09-008` | `AUTO_ONLY` | — |

Backend L3-001/002/008 are `AUTO_ONLY`: their record asks for no human-facing test, so
`ui-review.json` declares `AUTOMATED_EVIDENCE_ONLY`. UI L3-003/004/005/006/007 keep the
M05-UI manual gate **open**.

## Not claimed

- M05-UI manual acceptance of the gem window, its gestures and the viewport preview overlay.
- Matrix gemstone assets: none were read, copied or shipped.
- Analytic/NURBS gem faces: the committed B-Rep is sewn from the authored planar facets.
