# Top11 Split with object cutters and Shrink — 2026-10-31

Previously Shrink was available only after choosing Isocurve. The shared Split cutter-selection stage now offers Shrink=No/Yes when the selection includes eligible nonperiodic typed surfaces or nonperiodic single-face BReps, optionally mixed with curves. The setting remains available after cutters are selected or removed and carries into Isocurve if that option is chosen. No is the provisional OM9 default. Multi-face/solid and periodic surface targets currently prevent advertising Shrink for the selection.

When Yes is selected, each split surface result passes through the existing exact BRep shrink adapter before being converted back to the source frame. Curve results are unchanged. All geometry is prepared before the shared transaction, so an error on any target leaves the document untouched. Cutters and uncut targets remain unchanged.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- The new mixed-target regression failed before implementation because Shrink was absent. Afterward it covers typed surface, BRep face and curve targets with selection before or after command invocation, option activation, invalid text, cutter removal/reselection, bounded underlying surfaces, area, `.om9` save/open and Undo/Redo.
- An overlapping BRep cutter is rejected after a curve target has been prepared; the document, Undo stack and running session remain intact. A separate fixture verifies the setting carries into Isocurve. The simulated egui viewport cutter-selection test passes with Shrink=Yes.
- Top11/options/BRep/snapshot suites passed 133 tests before the final shared-parser consolidation and cutter-removal assertions. Eight Shrink-related tests passed after the parser change, and the strengthened mixed fixture passed separately afterward. No-default Top11/options/snapshot: 68 passed after the parser change.

Limits: Shrink for periodic surfaces and connected multi-face/solid targets remains unimplemented; a selection containing any such target does not advertise the option. General complex pcurves, topology, performance, aggregate transient budgets, 3DM interchange, external SDK plugin parity, physical desktop input and human M03-UI acceptance remain open. Native BRep shrink is the capability-scoped OCCT adapter recorded in `2026-10-31-top11-split-shrink-brep.md`; this batch did not perform openNURBS measurements.

SHA-256: `session/top11.rs` ac8f6ae1a3a99d4cc0f03e94713e6b09d35f81b514818024bb37c2a10357d7dc; `session/surface_commands.rs` 4a25c33c54bc7bd82a37a06e6c2e32678f89a97a1fa7d8d940b0612801908bfa; `session/command_options.rs` 359a5f1b0ed3cb5cbff2effc318cb6a9f8046bda8eb3606d79adc81200588cad; `ui/chrome.rs` c7e2969c2f9188bbc965c58c30094b018dabd63eaf8f0fe8c817948f58a63b12; `tests/top11_brep.rs` fd6b1d59c1958c649d84adbb79e4c6cd2b7834284c19aa2bd18c5e3d210e2d.
