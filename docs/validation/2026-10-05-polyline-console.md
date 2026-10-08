# Polyline picking and one-document Command — 2026-10-05

The user supplied three Matrix screenshots and explicitly requested successive Polyline clicks followed by Enter, plus history/prompt/input in one selectable text area for copying. The local `specs/02-curve/om9-curve-001-polyline.md` confirms the click/Enter workflow; no PDF reread was required. Matrix's running window was found, but its Computer Use capture showed other desktop content, so no new live-Matrix behavior is claimed. The packaged screenshots remain the accepted evidence.

## Implementation

`CommandConsole` is one native QPlainTextEdit document containing history, current prompt and current input. Ctrl+A selects it all; Ctrl+C/native Copy includes history and the active line. Only the input suffix is editable. Up/Down, draft restoration, Esc, viewport typing and shared native dispatch are retained. Deletion, word deletion, Cut/Paste and IME replacement cannot edit history/prompt; pasted/IME newlines do not execute commands. History is bounded to10000 blocks and recall to1000 entries. Existing fixtures now use QTextCursor helpers on this visible console instead of detached QLineEdit/QLabel widgets; those helpers are fixture-only.

Rust exposes accepted preview points and computes a hover point by applying the existing click constraints to a cloned session. Native Coin overlays show connected green segments, square accepted-point markers and the pending hover segment in all document views. They are nonselectable, outside document objects/Undo and excluded from model fit bounds. Enter commits one native Part::Feature wire in one transaction; markers/preview clear on commit, Esc, document change/deletion and workbench deactivation. Existing green output style and black/major-minor grid theme remain. Full Arc/Helpers/AutoClose and continuously closed PersistentClose preview remain unimplemented; this is the straight-segment slice.

Curve now reinstalls its application event filter at command start before prioritizing CoreMouse, matching Distance/PictureFrame precedence. At200%, traces showed the mouse filter receiving four presses but the older Curve priority receiving only the final press. Reprioritization restores all picks; mouse navigation retains first precedence. Middle-button default remains deferred.

## Verification

Matching MSVC14.44/C++23, Qt6.11.2, Python3.13, Coin4 and the clean patched host `D:/FreeCAD-src/Mod/OpenMatrix9/build/host-camera-diagnostic/bin/FreeCAD.exe`. Latest module is `build/polyline-console-final-runtime/OpenMatrix9Gui.pyd`. Native fixtures run sequentially in isolated profiles, requiring ok=true plus normal process exit0.

All59 Rust tests, fmt, Clippy all-targets -D warnings and release build passed (`build/polyline-rust-*.log`); new preview semantic test observed RED before implementation. Native RED on the prior theme build: `build/polyline-console-red-1/b5b61aea03304531963e8a41b8dc0943/results.json` shows missing unified console and transient chain/markers. Review exposed Ctrl+Backspace/IME boundary risks and history-eviction prompt offsets; bounded editing and suffix-position recomputation repaired them. The live menu-start preview additionally revealed an initial boundary treating the literal Command prompt as draft; a new initial-empty-input check reproduced it (`build/polyline-initial-red.log`) and constructor initialization repaired it. Synthetic QTest.mouseMove did not reliably deliver hover to hidden fixtures; explicit native QMouseEvent delivery is used instead. DPI fixtures use coordinates within the viewport, avoiding out-of-widget constants.

| Fixture | Scale | Checks | Result |
| --- | --- | --- | --- |
| Polyline + console | 100% | 33 | `build/polyline-final-polyline_console_smoke-1/aba59699797045649ebc1746fc1cfd9b/results.json` |
| Polyline + console | 200% | 33 | `build/polyline-final-polyline_console_smoke-2/20f23ae17f594919b8a9251848ee7e6b/results.json` |
| Command/theme | 100% | 38 | `build/polyline-final-core_command_theme_smoke-1/94f057846dfe4b27ad662957832893a3/results.json` |
| Curve | 100% | 26 | `build/polyline-final-curve_smoke-1/1e9c4f0289d446ecbb3df49ffe13affe/results.json` |
| Mouse | 100% | 46 | `build/polyline-final-core_mouse_smoke-1/d468e1bf6f254b83bc26fd40dc8e3692/results.json` |
| Keyboard | 100% | 28 | `build/polyline-core_keyboard_smoke-1/88d361bb7ec94042a01294db1f520acd/results.json` |
| PictureFrame | 100% | 43 | `build/polyline-core_picture_frame_smoke-1/60f4bbede4684a9884c94778b35365c5/results.json` |
| Distance | 100% | 24 | `build/polyline-core_distance_smoke-1/1d97346bcef24ccdb1e942b63912d8d6/results.json` |
| Capture | 100% | 16 | `build/polyline-core_capture_smoke-1/8d40019ab531451b9c5e1be7a97a9e35/results.json` |

Final-module33 checks per scale cover the initial empty input, four clicks/segments/markers before geometry commit, hover, one-wire Enter, accepted-point equality, Undo/Redo, validation with one point, Esc cleanup, whole-document copy, history editing guards, Ctrl+Backspace, IME, history eviction, CMD geometry, document switching and workbench exit. Keyboard/PictureFrame/Distance/Capture regressions were run on the preceding module; the final module differs only in the initial input-boundary constructor fix, rechecked by dedicated fixtures and relevant Command/Curve/Mouse regressions. Existing menu-registration warnings remain separately tracked.

## Skill persistence

Source and installed openmatrix9-ui contain the newest unified-Command and Polyline rules plus three byte-preserved screenshots. Baseline independent retrieval found both requirements missing/conflicting in the old reference; forward retrieval recovered the new click/preview/Enter/Esc behavior and protected, selectable Command document. It retained middle-button deferral and full-feature limitations. All skill artifact SHA-256 values match, source/installed quick_validate passed with Pixi Python, and eight reference-index tests passed. Git whitespace checks passed.
