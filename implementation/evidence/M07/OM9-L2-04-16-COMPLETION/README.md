# OM9-L2-04-16 — 01-febe-single-gem-seat-cutter-workflows

Gem Cutter (Gem/Azure/Advanced) for one stone and for a selected stone group, the ten-profile Gem
Cutter Library, the dimensional Bezel Cutter Builder through the shared `gvBezelCutter` command, the
pavilion clearance/depth controls measured on the real stone profile, the Interactive Boolean cut of
the metal body and the Cutter Body Preserve/Remove answer. Automated Rust evidence only — the M07-UI
manual review is open.

## Commands

| run | exit | tests | log |
| --- | --- | --- | --- |
| `cargo check -p om9-core -j 2` | 0 | — | `implementation/evidence/M07/OM9-L3-04-16-001/logs/core_check.log` |
| `cargo check -p om9-app -j 2` | 0 | — | `implementation/evidence/M07/OM9-L3-04-16-001/logs/app_check.log` |
| `cargo test -p om9-core --lib cutters -j 2` | 0 | 26 | `implementation/evidence/M07/OM9-L3-04-16-001/logs/core_cutters_lib.log` |
| `cargo test -p om9-core --test cutters -j 2` | 0 | 15 | `implementation/evidence/M07/OM9-L3-04-16-001/logs/core_cutters_it.log` |
| `cargo test -p om9-app --test gem_cutters -j 2` | 0 | 7 | `implementation/evidence/M07/OM9-L3-04-16-001/logs/app_gem_cutters.log` |
| `cargo test -p om9-app --test head_commands -j 2` | 0 | 9 | `implementation/evidence/M07/OM9-L3-04-16-001/logs/app_head_commands.log` |
| `cargo test -p om9-core -j 2` | 0 | 901 (74 targets) | `implementation/evidence/M07/OM9-L3-04-16-001/logs/core_full.log` |
| `cargo test -p om9-app --lib -j 2` | 0 | 197 | recorded in the L2 record (`implementation.automated_evidence`) |
| `python tools/public_source_audit.py` | 0 | 14480 source files, 511 icon bindings, 0 errors | recorded in the L2 record |

## Per L3

| L3 | title | feature | design | fixtures |
| --- | --- | --- | --- | --- |
| OM9-L3-04-16-001 | Gem Cutter Azure For Single Stone | `BuilderGemCutter` | [api](../../../design/M07/OM9-L3-04-16-001/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-16-001/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-16-001/negative.json) |
| OM9-L3-04-16-002 | Gem Cutter Multiple Stones | `BuilderGemCutter` | [api](../../../design/M07/OM9-L3-04-16-002/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-16-002/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-16-002/negative.json) |
| OM9-L3-04-16-003 | Gem Cutter Library Profiles | `BuilderGemCutterLibrary` | [api](../../../design/M07/OM9-L3-04-16-003/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-16-003/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-16-003/negative.json) |
| OM9-L3-04-16-004 | Bezel Cutter Builder | `BuilderBezelCutter` | [api](../../../design/M07/OM9-L3-04-16-004/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-16-004/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-16-004/negative.json) |
| OM9-L3-04-16-005 | Cutter Pavilion Clearance And Depth Controls | `BuilderGemCutter` | [api](../../../design/M07/OM9-L3-04-16-005/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-16-005/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-16-005/negative.json) |
| OM9-L3-04-16-006 | Interactive Boolean On Metal | `BuilderGemCutter` | [api](../../../design/M07/OM9-L3-04-16-006/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-16-006/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-16-006/negative.json) |
| OM9-L3-04-16-007 | Cutter Body Preserve Remove Option | `BuilderGemCutter` | [api](../../../design/M07/OM9-L3-04-16-007/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-16-007/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-16-007/negative.json) |

## Code owners

- `crates/om9-core/src/cutters/mod.rs` — ring loft/solid admission, `GemCutterReference` (measured stone profile), edge profiles.
- `crates/om9-core/src/cutters/gem.rs` — `CutterMode`, `GemCutterParams`, per-stone plans.
- `crates/om9-core/src/cutters/library.rs` — the ten manual presets and the 2D/Z scales.
- `crates/om9-core/src/cutters/bezel.rs` — `BezelCutterDims`, `CutterPosition`, block set.
- `crates/om9-core/src/cutters/boolean.rs` — `BooleanMode`, `CutterKeep`, the shared mesh-CSG cut.
- `crates/om9-core/src/settings/bezel.rs` — `ScallopParams.dims` routes the M06 `gvBezelCutter` to the block builder.
- `crates/om9-app/src/session/gem_cutters.rs` — staging, prompt/options, commit, `om9.cutter` / `om9.cutter.cut`.
- `crates/om9-app/src/ui/gem_cutters.rs` — window and viewport preview.

## Gates

- `M07-UI` manual review: **not signed**; the dashboard keeps the seven L3 at WIP.
- Enter commits one host transaction and Undo restores: the app tests exercise both.
- `BuilderBezelCutter` remains the M06 `gvBezelCutter` command; this owner publishes the dimensional
  contract (`Dims=on`) and the block geometry that the shared scallop owner dispatches to.
- Cutter bodies are lofted ring stacks (48 segments by default), not B-Rep solids.
- Percentage controls are relative to the measured gem: girdle radius, pavilion depth, crown height;
  `ZOffset` is a pavilion-depth percentage for the Gem Cutter and millimetres for the library.
- The Interactive Boolean needs a closed mesh target and reuses `mesh_csg::mesh_boolean`.
- `Preserve=Remove` applies to the committed cut only; a cutter committed with the Boolean off always
  stays as its own body.
- Unrelated open failure in the working tree:
  `crates/om9-app/tests/command_snapshot.rs::positional_top11_owner_is_visible_without_a_fake_option_row`
  (untracked Top11 work) — not touched by this owner.
