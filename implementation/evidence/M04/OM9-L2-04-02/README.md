# M04 builder UI and history journal

This directory records the vertical slice that connects the associative ring chain to the
application. The L3 records stay in the plan tree until their own manual gates are signed.

## What is implemented

- `om9_core::builder::BuilderSession` -- typed fields with domains, a live preview through
  `ProducerRegistry`, viewport handle positions and a one-transaction commit plan.
- `Controller` builder API -- `begin_builder`, `builder_set_field`, `builder_adjust_field`,
  `builder_refresh`, `builder_reset`, `builder_cancel`, `builder_apply`, `builder_text`.
- Command frame -- `RingRail`/`gvRingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`,
  `Ring Resizer`, Enter/Apply, `Diameter=`, `Width=`, `Height=`, `Offset=`, `Target=`,
  `Thickness=`, `preset=`, `Solid=`, Reset and Cancel; the Top 11 Ring Rail row opens the
  builder.
- Mode tabs -- the BUILDERS panel switches between the ring modes; every tab keeps its own
  typed parameters, a return refreshes against the current document, and a failed switch
  restores the previous tab (`OM9-L3-03-11-002`). The jewelry ring family tabs
  (`Eternity Band`, `Signet Ring`, `Raised Band`, `Award Ring`) are owned by
  `OM9-L2-04-03`; the ring-rail chain remains the five modes below.
- Ring size -- `RingRail` accepts an explicit `Diameter=` or `standard=US` plus `size=`
  1..15 through `om9_core::ring_size`, with the frozen table provenance in the summary
  (`OM9-L3-04-02-001`).
- Profile placement -- `RailProfile` adds `Position=` (0..1) and `Roll=` (degrees); a third
  handle edits the position in rail units (`OM9-L3-04-02-003`, `-009`).
- Custom rail -- `OM9-TOOLS-004` with fewer than five selected curves records a validated
  closed/planar/non-self-intersecting rail; `OM9-TOOLS-035` is the advanced alias
  (`OM9-L3-04-02-004`).
- Four rail profile -- `OM9-TOOLS-032` is an alias of the existing `OM9-TOOLS-004` metadata
  command, so both icons record the same command and label (`OM9-L3-04-02-005`).
- Profile merge/end cap -- `OM9-TOOLS-034` joins compatible sections; `OM9-TOOLS-033` caps a
  closed profile as a planar surface and an open B-Rep through the kernel cap under
  `occt-brep` (`OM9-L3-04-02-006`).
- Associative sweep -- a ring-size edit rebuilds the dependent profile and one dirty sweep;
  a failed profile rebuild leaves the committed sweep revision unchanged
  (`OM9-L3-04-02-008`).
- Profile selector window -- the `Library…` button lists rectangle, rounded rectangle and
  circle; choosing changes only the preview, Apply commits the chosen shape, and removing
  the selected preset drops the preview and refuses Apply (`OM9-L3-03-11-007`).
- Outside rail -- `OutsideRailProducer` offsets a closed circular rail by `offset_mm`; a
  non-circular rail or non-positive offset is refused (`OM9-L3-04-02-002`).
- Ring resizer -- `RingResizerProducer` creates an independent ring at
  `target_diameter_mm` on a new object; the source ring is untouched and a target at or
  below twice `thickness_mm` is refused (`OM9-L3-04-02-007`).
- Panel -- the reserved BUILDERS slot paints the tabs, fields, sliders, preview summary,
  Apply/Reset/Cancel, the library button and handle read-outs.
- Viewport handles -- projected squares with a drag that changes the same field the panel
  edits (`diameter_mm`, `width_mm`, `height_mm`, `offset_mm`, `target_diameter_mm`).
- F6 -- the Builder context emits the typed prefill and opens the mode the selection fits.
- Transaction -- `Mutation::History` also carries published results (a builder inserts entity,
  recipe and result in one atomic step); `Mutation::HistoryUpdate` replaces the recipes and
  results a recompute touched; `Controller::recompute_history` runs the registry and journals
  the whole rebuild as one Undo unit.

## Verification

See `observed.json` and the logs in this directory. The focused suites are:

- `cargo test -p om9-core --test builder_session --test history_mutation`
- `cargo test -p om9-kernel --features 3dm-brep --test associative_history` (real OCCT sweep)
- `cargo test -p om9-app --test builder_panel --test top11`
- `cargo test -p om9-plugin-host --lib history`

## Not claimed

- Human acceptance of the panel, handle gestures, mode tabs, selector window, outside rail,
  resizer, ring-size table, custom rail, four-profile alias, merge/end-cap commands or the
  associative sweep; those L3 keep their plan status and the M04-UI gate stays open.
- Screenshot/pointer-frame evidence; the tests prove the model, command frame and transactions.

See [`OM9-L2-04-02-COMPLETION`](../OM9-L2-04-02-COMPLETION/README.md) for the final
milestone note, per-L3 evidence counts and the exact next action.
