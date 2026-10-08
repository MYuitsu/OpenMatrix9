# Matrix Command, grid and viewport theme — 2026-10-05

The user's selected screenshot is preserved at `skills/openmatrix9-ui/assets/matrix9-command-grid-reference.png`. The three explicit requirements are a Command region above the viewports with command-line interaction, major grid cells subdivided into minor cells, and black viewport canvases with Matrix's green Command background. The sampled green is RGB130,180,140 (`#82B48C`). No PDF was reread.

## Changes

`CurveController` retains original command dispatch and widget IDs. The Command dock is now at the top, with a read-only bounded transcript above the current prompt and one input row. It echoes submitted input, Curve prompts/errors/commit results, measurement feedback and PictureFrame errors/success. Up/Down recall inputs and restore an unfinished draft; Esc clears/cancels input. Printable typing in a native viewport routes to CMD, while modified shortcuts, dialogs, native editing and active native tasks retain their input. Empty Enter repeats an available recent command after checking active point-tool state. Menu, mouse and CMD geometry still use the same handlers.

Rust supplies minor/major/axis classification. Each native construction plane has nonselectable world-space 1 mm minor lines, 5 mm major lines, red/cyan axes and extent±100 mm. Major/minor brightness is0.38/0.19; line width1. Grid geometry remains excluded from fit bounds and document objects. These values are OM9 rendering choices to implement the reference, not recovered configurable Matrix defaults.

Visual review reproduced two contrast gaps in `build/theme-isolated-1/ed01e2f4dd4f41a4a8d4b3813622af15/results.json`: a white prompt-label background and default black Curve edges. The input row/prompt now shares the sampled Command green. New Curve objects use sampled edge green RGB0,130,85 (`#008255`); existing objects retain their colors. Both regressions are included in the final38 checks.

Each OM9 native viewer uses solid black without a gradient. Original background color/gradient are saved per viewer and restored on leaving the workbench. Global FreeCAD color preferences are not overwritten. The middle-button default switch remains deferred per the user's latest request.

## Verification

Matching MSVC14.44/C++23, Qt6.11.2, Python3.13, Coin4 and FreeCAD SDK build; native module `build/theme-contrast-runtime/OpenMatrix9Gui.pyd`, clean patched host `D:/FreeCAD-src/Mod/OpenMatrix9/build/host-camera-diagnostic/bin/FreeCAD.exe`. Native fixtures run sequentially in separate profiles and require both `ok=true` and process exit0.

- All58 Rust tests, format, Clippy all-targets with `-D warnings`, release build and native compilation/link passed. Logs: `build/theme-rust-tests.log`, `build/theme-rust-clippy.log`, `build/theme-rust-release.log`.
- Rust RED observed missing `grid_line_kind`, then semantic test passed. Native UI RED: `build/theme-red-1/e3710b7965394bd392ec75256a5f7103/results.json`, showing the old bottom input panel, missing transcript/levels, gradient canvases and viewport typing/repeat gaps.
- Read-only review found lost Curve feedback and viewport key interception during native task editing. Both reproduced in `build/theme-isolated-1/f8545d7ead5c44f3a0f67761157eec2a/results.json`, then fixed by transcript logging and native editing/task guards. Reviewer checked both repairs without further P1/P2 findings.

Final dedicated fixture: **38 checks per scale**, all passed with normal exit:

| Scale | Result |
| --- | --- |
| 100% | `build/theme-contrast-isolated-1/35c8b169ec6942a591f43ab432b2252b/results.json` |
| 200% | `build/theme-contrast-isolated-2/4437aec915954df68c0c4b8540f0b3bb/results.json` |

Checks cover top dock geometry, sampled green, history/input layout, native four views, grid spacing/contrast/no model objects, black native rendered pixels, typed geometry, Up/Down/draft/Esc, repeat, measurement output, retained Curve error/success, modal/native task focus, workbench switching and native background restoration. Native `native-view-*.png` captures include the visible world grid; black background probes temporarily hide and restore the grid to avoid mistaking dense grid pixels for background. The first fixture used the wrong Pivy SoMFColor accessor; correcting it was a fixture repair, not a product fix.

## Skill persistence

Updated source and installed `openmatrix9-ui`: entrypoint, UI routing reference, `references/command-grid-theme.md` and packaged selected image. Both skill frontmatters pass `quick_validate`; all8 reference-index tests pass. An independent baseline retrieval could not recover any of the three concrete requirements from the old skill. Forward retrieval from the updated skill recovered command position/behavior, colors, grid levels/world-space semantics, restoration and middle-button deferral. This preserves the user's accepted reference without treating visual similarity as implementation of every Matrix feature.

The command transcript is bounded to10000 blocks and recall to1000 inputs. Full Rhino scripting/history customization and unimplemented CAD commands remain outside this slice. Existing Unknown command menu-registration warnings and the prior Mid-sidebar input failure remain separate tracked gaps.

## Final regressions and visible preview

Latest contrast module passed sequential normal-exit regressions at100%:

| Fixture | Checks | Result |
| --- | --- | --- |
| Curve |26| `build/theme-final-curve_smoke-1/19405283ea084a49bdd4380dc3fd1fa7/results.json` |
| Mouse |46| `build/theme-final-core_mouse_smoke-1/809374b736ff47cf993deb61b36c9863/results.json` |
| Capture |16| `build/theme-final-core_capture_smoke-1/8355e754197449a195d5e4eff825daa4/results.json` |

The previous theme build also passed keyboard28, workspace36, view-controls20 and PictureFrame43 native regressions. The final two changes affect scoped prompt CSS and the new Curve object's edge color; the dedicated38-case fixture and three relevant regressions were rerun after those changes. Read-only review found no P1/P2 issue in setter ownership, transaction rollback or CSS scope.

Visible `OpenMatrix9WorkspaceFinal` preview (PID9212) loads `build/theme-contrast-runtime/OpenMatrix9Gui.pyd`. Status: `build/workspace-matrix-preview-status.json`; Qt grab: `build/workspace-matrix-preview.png`; launcher: `build/workspace-matrix-preview.FCMacro`. Actual Windows.Graphics.Capture was inspected: full-width top green Command transcript/input, four black native canvases, differentiated world grids and green visible geometry. Distance via shared CMD returned5mm and Line created actual geometry. Native dock overlays are disabled only in this isolated preview's startup config (`build/theme-preview-layout.cfg`), preserving the user's real profile. Sample-box color tuples in the preview helper were corrected to all-float values after a helper-only type error; product fixtures remained passing.

Source and installed UI skills passed quick_validate using the existing Pixi Python runtime; the default shell Python lacked PyYAML. All four changed skill artifacts have matching SHA-256. Eight reference-index tests and Git whitespace checks passed. Core completion remains in progress.
