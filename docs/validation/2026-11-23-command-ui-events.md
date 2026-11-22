# CMD, clipboard and workspace event regression

Windows standalone, 2026-11-23. The starting checkout reproduced five failures in 37 tests. The existing F6 Extrude Tapered test passed at baseline.

| Scenario | Cause and verified result |
|---|---|
| Type Point in a viewport | The focus handoff required an active command. Printable text now focuses CMD and reaches its native text editor when the keyboard is free. The test types five separate letters, moves the selected suggestion, then presses Enter once. The exact Point command opens its coordinate prompt. A separate focused editor receives its own text. |
| Draw Line, Undo, Redo and Escape | Initial typing had the same focus problem. After it was fixed, the test needed to let the history worker publish through subsequent egui frames. The typed Undo or Redo also needs to place the caret after the echoed transcript line so the next command can be entered. The test checks both Line point prompts, endpoints (0,0,0) and (12,6,0), restored object identity on Redo, and one Undo entry after Escape cancels a second Line. |
| Ctrl+Z | The original test stopped while creating Point. A later run exposed an egui text Undo snapshot with an old protected prefix. Starting a draft now starts text Undo against the current transcript. With a draft, Ctrl+Z removes the edit and keeps the point. With an empty draft and idle CMD, a second Ctrl+Z runs document Undo and creates a Redo entry. |
| Select all, Cut and Paste | After Ctrl+A and Cut, egui put the caret at the start of the copied text, inside protected history. CMD now places it back at the draft boundary. History and prompt survive both Cut and Paste. Pasting three lines yields one draft without running the command. |
| Four-view docking | The 150-point threshold predated the removal of the horizontal toolbar strip. At 1280 × 800 logical points, the menu ends at Y=21, CMD occupies Y=21 through Y=111, the toolbar reserves zero height, and the top viewports start at Y=111. The test checks adjacency, two history rows at the current font size, the pinned live row, the left rail and both viewport columns. |
| F6 Extrude Tapered | The existing egui replay opens Surface/3Crv, clicks the factory row, chooses options, previews, picks the height in Through Finger and undoes all three results in one step. |

The shortcut handler consumes E and its paired text before the idle CMD handoff, so the key still arms End Snap. An exact typed command takes precedence over a different highlighted suggestion. Tab fills a suggestion and Enter invokes through the existing CMD dispatcher.

## Verification

- Windows debug build using the existing L:/om9-target, with Cargo at -j 2 and both pinned BRep workers. The running OM9 window held debug/om9.exe open. Cargo linked the integration binaries under debug/deps, which were run directly with the SDK runtime on PATH.
- ui_events 37/37, including the six scenarios above. f6_context 39/39. pinned_command 7/7, command_inline_options 9/9 and snap_panel_modes 48 passed with two ignored. The IME and long Unicode paste fixtures activate CMD before replaying editor events.
- The supplemental workbench_regions_sizes run passed 9/10. Its 911 × 480 layout at the 460-point rail clips Layers by 7.5 points. The test reproduces with no keyboard events. The requested 1280 × 800 idle layout passed.
- The user approved recording viewport keyboard ownership in openmatrix9-ui and openmatrix9-workspace-contract. Both repository and installed copies were updated.
- The change affects command input and history publication timing. Point and Line are checked through their existing geometry producers. The F6 replay covers preview, point picking and final placement. Physical Windows keyboard routing and desktop layout were outside this validation run.
