# OM9-L2-04-17 — 02-febe-channel-microprong-brightcut-cutters

The Channel Builder trench through a row of stones, the Micro Prong scallop row, the Bright Cut
Channel along the row, the Bright Cutter around one gem, the channel profile/end controls and the
multi-stone Interactive Boolean. Automated Rust evidence only — the M07-UI manual review is open.

## Commands

| run | exit | tests | log |
| --- | --- | --- | --- |
| `cargo check -p om9-core -j 2` | 0 | — | `implementation/evidence/M07/OM9-L3-04-17-001/logs/check-core.log` |
| `cargo check -p om9-app -j 2` | 0 | — | `implementation/evidence/M07/OM9-L3-04-17-001/logs/check-app.log` |
| `cargo test -p om9-core --test channel_cutters -j 2` | 0 | 10 | `implementation/evidence/M07/OM9-L3-04-17-001/logs/test-core-channel-cutters.log` |
| `cargo test -p om9-app --test channel_cutters -j 2` | 0 | 7 | `implementation/evidence/M07/OM9-L3-04-17-001/logs/test-app-channel-cutters.log` |
| `cargo test -p om9-app --test gem_cutters -j 2` | 0 | 7 (M07-04-16 intact) | `implementation/evidence/M07/OM9-L3-04-17-001/logs/test-app-gem-cutters.log` |
| `cargo test -p om9-app --test head_commands -j 2` | 0 | 9 | `implementation/evidence/M07/OM9-L3-04-17-001/logs/test-app-head-commands.log` |
| `cargo test -p om9-core -j 2` | 0 | 911 (all core targets) | recorded in the L2 record (`implementation.automated_evidence`) |
| `cargo test -p om9-app --lib -j 2` | 1 | 207 passed / 2 failed (`ui::icon_size_audit`, unmodified by this owner) | recorded in the L2 record |
| `cargo test -p om9-app -j 2 --no-fail-fast` | 1 | 8 failing targets, all outside the cutter family (see the L2 `unrelated_open_failures`) | `implementation/evidence/M07/OM9-L3-04-17-001/logs/test-app-full.log` |

## Per L3

| L3 | title | feature | design | fixtures |
| --- | --- | --- | --- | --- |
| OM9-L3-04-17-001 | Channel Builder Cutter | `BuilderChannelBuilder` | [api](../../../design/M07/OM9-L3-04-17-001/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-17-001/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-17-001/negative.json) |
| OM9-L3-04-17-002 | Microprong Cutter | `BuilderMicroProngCutter` | [api](../../../design/M07/OM9-L3-04-17-002/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-17-002/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-17-002/negative.json) |
| OM9-L3-04-17-003 | Bright Cut Channel Builder | `BuilderBrightCutChannel` | [api](../../../design/M07/OM9-L3-04-17-003/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-17-003/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-17-003/negative.json) |
| OM9-L3-04-17-004 | Bright Cutter Single Gem | `BuilderBrightCutter` | [api](../../../design/M07/OM9-L3-04-17-004/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-17-004/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-17-004/negative.json) |
| OM9-L3-04-17-005 | Channel Cutter Profile And End Controls | `BuilderChannelBuilder` | [api](../../../design/M07/OM9-L3-04-17-005/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-17-005/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-17-005/negative.json) |
| OM9-L3-04-17-006 | Multiple Stone Interactive Boolean | `BuilderChannelBuilder` | [api](../../../design/M07/OM9-L3-04-17-006/api.md) | [positive](../../../fixtures/M07/OM9-L3-04-17-006/positive.json) / [negative](../../../fixtures/M07/OM9-L3-04-17-006/negative.json) |

## Code owners

- `crates/om9-core/src/cutters/channel.rs` — `ChannelParams`, `ChannelEnd`, `ChannelMiddle`, `build_row` (swept 4-corner trench with A/B end caps and an optional middle station).
- `crates/om9-core/src/cutters/microprong.rs` — `MicroProngParams`, `build_row` (one dome scallop per gap, optional count distribution and tangent lean).
- `crates/om9-core/src/cutters/brightcut.rs` — `BrightParams`, `BrightMode`, `build_single`/`build_channel` (8-corner bright facet blocks).
- `crates/om9-core/src/cutters/mod.rs` — `sweep(sections)` for the channel trench; the loft/solid/volume services reused from `OM9-L2-04-16`.
- `crates/om9-core/src/cutters/boolean.rs` — the Interactive Boolean reused unchanged.
- `crates/om9-app/src/session/channel_cutters.rs` — `RowTool` (the four command ids and typed aliases), `RowCutterSession`, `RowCutterPayload`, the dispatch/prompt/cancel/commit methods on `Controller`.
- `crates/om9-app/src/ui/channel_cutters.rs` — the window and viewport preview for the four builders.
- `crates/om9-core/tests/channel_cutters.rs`, `crates/om9-app/tests/channel_cutters.rs` — the geometry and command tests behind the numbers above.

## What was missing before this delivery

The four command ids (`BuilderChannelBuilder`, `BuilderMicroProngCutter`,
`BuilderBrightCutChannel`, `BuilderBrightCutter`) and their SVGs existed in the menu catalog and the
icon bindings, and the display filter already read an `om9.cutter` payload. No session resolved the
ids, no kernel built a row cutter and no window existed: a click fell through the unsupported-command
path. This delivery adds the row-cutter kernel once, the staged session, the window, the typed
aliases and the tests.

## Open

- `M07-UI` is not signed: the dashboard keeps the six L3 at WIP until a human reviews the preview,
  the options and the commit/cancel of each builder on the OpenMatrix9 window.
- The manual text for the channel/bright pages is not in the repository; the option names come from
  the L3 domain inputs and the manual's named controls, and each mapping is recorded in the L3
  `api.md`.
- Cutter bodies are closed meshes, not B-Rep solids; the B-Rep swap stays the kernel adapter's gate.
