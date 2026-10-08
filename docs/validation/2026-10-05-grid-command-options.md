# Grid extent and compact Command options — 2026-10-05

The user's two new Matrix screenshots specify eight by eight major grid cells / forty by forty minor cells; an adjustable Command area defaulting to two history rows and one input row; parenthesized questions with underlined initial-letter option inputs. Implementation follows the existing UI/Curve slice and local specs, without PDF rereads.

## Implementation and limits

Native world-space Coin grid now spans -20 to +20 mm, with current 1 mm minor / 5 mm major spacing. All four construction planes share the bounded grid. Geometry remains unpickable, excluded from fit bounds, and outside document objects/Undo. Camera pan and zoom change the rendered grid.

The top Command dock uses font-based sizing (default native dock 60 logical pixels at the verified font), not a fixed expanded 180 pixel height. Native dock separator mouse dragging expands it to 120 pixels; shrinking is retained. One unwrapped QPlainTextEdit document still combines history, current prompt and input for Copy. Only input suffix is editable.

Line and Polyline prompts now show supported options inside `( )`, with initial letters underlined via ExtraSelections, preserving copied plain text. Case-insensitive P/P=Y/P=N, C, M/M=L, L/L=value, U and Line B/B=Y/B=N use the Rust session; full option names remain supported. Close appears after three points, Undo/Length after the first, BothSides before Line's first point. Clicking an option name submits through the same input handler only with an empty draft; text selection and existing drafts are preserved. Undo removes the last accepted point and updates the preview without a document transaction. Enter still commits one complete wire.

This styles the implemented Line/Polyline options; Arc, Helpers and continuously closed PersistentClose previews remain unfinished. Other core-command prompts have their existing contracts. Full 130-spec core acceptance remains in progress. Middle-button mapping is unchanged and still deferred for original-Matrix comparison.

## Verification

Rust 61 tests, fmt, Clippy all-targets -D warnings and release passed. Matching native MSVC 14.44/C++23, Qt6.11.2, Python3.13 and Coin4 compiled and linked. Runtime host: `build/host-camera-diagnostic/bin/FreeCAD.exe`; latest module: `build/matrix-grid-command-runtime/OpenMatrix9Gui.pyd`.

Module SHA-256: `a7f443f43b1a40b85582537c4fe62053ebf699e292d3f421f7ae8d234483e55f`.

| Fixture log | Coverage | Passing report |
| --- | --- | --- |
| matrix-grid-command-final-1 | 37 checks | `build/matrix-grid-matrix_grid_command_smoke-1/aa4d9bc6b799489cb95030f8c6e8b527/results.json` |
| matrix-grid-command-final-2 | 37 checks | `build/matrix-grid-matrix_grid_command_smoke-2/dc9134097c2845568ad2acfe3b2a89fb/results.json` |
| matrix-grid-polyline-final-1 | 33 checks | `build/matrix-grid-polyline_console_smoke-1/f0af6429e6bd4dedbed176815c3ee368/results.json` |
| matrix-grid-polyline-final-2 | 33 checks | `build/matrix-grid-polyline_console_smoke-2/5db87912cc6d462891684e80c66535d3/results.json` |
| matrix-grid-theme-final-1 | 38 checks | `build/matrix-grid-core_command_theme_smoke-1/3a0d705067cc4d64ac1d6da5f3afbcba/results.json` |
| matrix-grid-distance-final-1 | 24 checks | `build/matrix-grid-core_distance_smoke-1/ff2c4cfb00e64e8fbe7a277c46538b62/results.json` |
| matrix-grid-picture-final-1 | 43 checks | `build/matrix-grid-core_picture_frame_smoke-1/de811cb40ab549e5ba3cdd6a09e31fd5/results.json` |
| matrix-grid-curve-1 | 26 checks | `build/matrix-grid-curve_smoke-1/2e31c7b29de0470a9a61fc45efbb8656/results.json` |
| matrix-grid-mouse-1 | 46 checks | `build/matrix-grid-core_mouse_smoke-1/44f7e2ae894445b3b11d7963050a4fd6/results.json` |
| matrix-grid-camera-1 | 4 views | `build/matrix-grid-core_grid_camera_probe-1/c161399a1a9946e6a8e11b04d6fb671c/results.json` |

Final module: dedicated 37 checks at100% and200%, Polyline/console33 at both scales, Command/theme38, Distance24 and PictureFrame43; every process exited0 through the smoke runner. Curve26, Mouse46 and four-view pan/zoom regressions ran before the final console-only input-boundary repair; camera/grid/Curve session code is unchanged after those runs. Native events are Qt-generated, not proof of a physical desktop mouse/keyboard trial.

Red evidence: `build/matrix-grid-command-red.log` reproduces old extent/default-height/prompt/shortcut failures; new Rust prompt test failed before implementation. Read-only review exposed AltGr and selected-draft IME bypasses, reproduced by four failing native checks (`build/matrix-grid-console-boundary-red.log`). Further surrogate-pair/Unicode-format/Alt text bypasses reproduced by six checks (`build/matrix-grid-console-unicode-red.log`). Guarded insertion now covers all Qt-accepted text classes, bounds preserve surrogate pairs, and IME range/length calculations use the cursor after selection removal. Forward review found no remaining actionable issue. Qt input acceptance reference: https://raw.githubusercontent.com/qt/qtbase/6.8/src/gui/text/qinputcontrol.cpp (native tests use the actual6.11.2 host).

The separator fixture now sends hover before press, matching Qt's native separator hit tracking; an earlier200% probe without hover failed to start the drag. Final both-scale probes pass. Native grid captures and `prompt-options.png` inspected; QWidget whole-window grabs omit OpenGL content and are not used as viewport evidence. The fixture restores the prior clipboard after Copy tests.

## Skill persistence

No business rules were written into source or installed skills for this task. A concrete before/after and exact text proposal is recorded in `2026-10-05-grid-command-options-skill-proposal.md`, pending the user's post-verification decision. Technical protection fixes restore the prior agreed behavior and do not need a separate business consent question.

Preview launched with Normal window style, separate profile and matching module: document `OpenMatrix9GridCommand`, PID19248, launcher `build/matrix-grid-preview.FCMacro`. No physical desktop trial or live screenshot is claimed.

2026-10-06 follow-up: the user explicitly approved saving these rules into the new workspace-group skill and committing/pushing this work. The pending-consent statements above describe the state when this slice was first verified. See [approval and skill verification](2026-10-06-workspace-skill-push.md).
