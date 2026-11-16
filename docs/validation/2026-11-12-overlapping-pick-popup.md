# Overlapping B-rep picker remains usable — 2026-11-12

The user reported that ordinary selection of overlapping objects opened `Select object`, but clicking a row made the chooser close and reappear. The user's active document and window were not operated during diagnosis.

The offscreen GPU regression `overlapping_brep_popup_row_selects_once_on_gpu_viewport` reproduces the issue with two coincident B-rep spheres named `sphere` and `Boolean Union`. It opens the chooser by a viewport pointer click, waits for it to appear, then clicks a row with separate move, press and release frames. Before the fix, the popup remained open and no object was selected. Instrumentation showed that `published_scene_current` became false on the popup press: its primary-gesture safeguard expects a viewport press capture, which a popup click does not have. Consequently the chooser was not drawn during the click frames, so its button could not receive the click.

The UI now distinguishes whether the published scene content is current from whether the current pointer gesture was captured by that viewport. The popup uses the content check, while viewport picking retains the stricter gesture check. The popup also blocks the same pointer click from reaching the viewport under it; existing candidate epoch and command-context validation remain in use. After the change, both chooser rows selected the intended object and the chooser remained closed, with document and pre-existing Redo unchanged. A real GPU adapter was used; the test did not skip. The ordinary chooser integration suite passed 6 tests; separate stale-publication selection and hover GPU tests passed. The Solid viewport integration suite passed 160 tests with 3 ignored. Those automated tests alone do not establish that the user's exact file or desktop interaction has been retested.

The checked debug build is `L:/om9-target-surface/debug/om9.exe`. The original `L:/om9-target` build and the user's open window were not replaced. The new executable was launched in non-window fixture mode with its own runtime DLLs and exited successfully; it is not a release build.

An isolated desktop follow-up used `write_overlapping_brep_desktop_fixture` to save two coincident sphere B-reps named `sphere` and `Boolean Union`, then opened that fixture in the new executable. On the actual Looking Down viewport, a mouse click opened the chooser; a physical click on `sphere` selected exactly that object and closed the chooser. After clicking blank space and reopening the chooser, a physical click on `Boolean Union` selected exactly that object and closed it. The live window state confirmed each label and an unchanged two-object document with zero Undo entries. This is a synthetic fixture, not the user's file; other topology, DPI and viewport arrangements remain unverified.

Per the user's instruction to keep only two builds, the two active Cargo targets are `L:/om9-target` and `L:/om9-target-surface`. Unused compilation caches in `L:/om9-head-target`, `L:/om9-nav-target/release` and `L:/om9-integration-check/debug` were removed after checking for processes using them; the reported free-space increase was 11.85 GiB. Historical standalone executables and associated DLLs in the latter two directories, the user-recorded trace, and the DupEdge `.om9` evidence were retained; those evidence directories are no longer complete Cargo build caches.

## Pick query failure handling — 2026-11-13

The ordinary object-click handler previously discarded `pick_all` errors and
continued as though the query produced no candidates. It now reports that
error and returns before changing the selection. The existing error path is
shared with fallback-source failures.

An egui fixture reserved managed memory until `pick_all` itself returned an
error: selection, document and one Redo entry were retained, the error was
visible, and retry after releasing pressure selected the requested point. The
same fixture also passed without this change because another admission failure
kept the old selection and surfaced an error; therefore it does not reproduce
a prior selection loss or independently establish every error path. All six
`matrix_selection_sequence` tests passed with the change. Physical OOM and
actual desktop behavior were not tested.


## Grip selection and pick failure investigation

Code inspection shows that, when grip mode is active and a plain click misses a grip, the handler clears selected grips before the object candidate query can report an error. This ordering is a potential selection-loss path if that error is reachable after the clear; no actual loss was established. An exploratory egui fixture selected a polyline control point and held the managed budget until a direct scene pick returned an error, but the subsequent simulated click left the grip selected and the status at Ready. The fixture did not establish that the viewport object-click handler was reached, so it cannot prove or disprove the suspected path. The exploratory test was removed and no product behavior changed. Before changing the handler, a valid regression needs to exercise the same click branch and inject or otherwise trigger a query error after entering it; do not count a general memory-pressure error elsewhere as this evidence.

### Reproduced grip selection loss and fix

A revised egui fixture clicks the midpoint of an on-screen polyline while its control point is selected. With managed budget exhausted, a direct scene pick and the viewport click report budget refusal. Before the fix the click cleared the selected control point; after the fix it retains it, the document and an existing Redo entry. Releasing the pressure and clicking again clears the grip on successful ordinary object selection. The click exercises an early control-grip miss and the ordinary candidate query: the former now defers clearing when no command is active, and the latter clears only after the query succeeds. Existing command/point-session handling is unchanged. All ten matrix selection integration tests and five UI Top 11 unit tests passed; the existing debug target built. This is simulated egui pressure, not physical OOM or desktop evidence.

A follow-up also clears the UI grip-selection mirror after a successful ordinary pick, matching the controller state after deferred clearing. The ten selection integration tests passed again, the no-default-features check and existing debug build succeeded. The refusal path remains scoped to the simulated fixture above.
