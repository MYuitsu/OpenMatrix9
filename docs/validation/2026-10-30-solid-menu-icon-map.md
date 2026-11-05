# Solid menu order and icon-to-command map (automated checkpoint)

Scope: `OM9-L2-03-18` Solid menu palette and the command each icon dispatches.

## Order

The authored Matrix main menu (`Config/MainMenu.ini` in the installed product, mirrored by
`Resources/menu/MainMenu.ini`) has **30 Solid icons in two pages of twenty and ten**. The
manual's MAIN MENU page for the Solid category (Matrix book 1 "The Main Menu", Solid
screenshot) shows page one as:

- row 1: Union, Difference, Intersection, Boolean Two Objects, Cap Planar, Extract Surface,
  Fillet Edge, Text, Extrude Curve Straight, Solid Extrude All;
- row 2, first icon **Pipe**: Pipe, Box, Sphere, Ellipsoid, Torus, Cylinder, Tube, Pyramid,
  Cone, Truncated Cone;
- page 2: Boss, Rib, Slab, Make Hole, Array Hole, Array Hole Polar, Move Hole, Move Face,
  Shell, Solid Points ON.

The working palette had been expanded to 43 icons by inserting `Boolean Split`,
`Create Solid`, `Solid Multi Pipe`, the Box variants and the Sphere variants after the
booleans, which moved Pipe to the third slot of row 2. The authored 30-icon order is
restored, and the 13 recovered variants now follow Solid Points ON (they are not separate
icons in the documented Matrix menu; their commands stay reachable from the menu tail and
from CMD).

`menu_catalog::tests::every_palette_matches_the_authored_menu_ini_order` compares every
menu (17 sections) to `Resources/menu/MainMenu.ini`, and
`solid_menu::tests::documented_solid_page_one_starts_row_two_with_pipe` pins the documented
page-one rows. The manual page order is still enforced by
`solid_menu::tests::every_solid_icon_dispatches_its_documented_command`.

## Mapping defects fixed

Five Solid icons had an empty command binding, so the menu rendered them disabled and a click could not start the documented command:

| Icon | Command now bound |
| --- | --- |
| `SolidUnion` | `OM9-SOLID-003` |
| `SolidDifference` | `OM9-SOLID-001` |
| `SolidIntersection` | `OM9-SOLID-002` |
| `SolidExtrudePlanarCurveStraight` | `OM9-SOLID-009` |
| `SolidGVDAllExtrude` | `OM9-SOLID-010` |

`SolidPtOn` (last documented palette entry) had no binding; it now points at its documented Transform row `OM9-TRANSFORM-019`, which stays disabled with that row's recorded reason.

Variant rows keep their own mode command and resolve to the primitive feature: `SolidBoxDiagonal`/`SolidBox3PointsHeight`/`SolidBoxVerticalBase`/`SolidBoxCenterofBaseCornerHeight` → `OM9-SOLID-012`; `SolidSphere2Points`/`3Points`/`4Points`/`AroundCurve`/`TangenttoCurves`/`FitSpheretoPoints` → `OM9-SOLID-014`; `SolidGVDMultiPipe` → `OM9-SOLID-011`.

Visible but disabled, no owner: `SolidRib`, `SolidSlab`, `SolidSolidEditToolsHolesMakeHole`, `SolidSolidEditToolsHolesArrayHole`, `SolidSolidEditToolsHolesArrayHolePolar`, `SolidSolidEditToolsHolesMoveHole`, `SolidSolidEditToolsFacesMoveFace`, `SolidPtOn`.

## Checks run

- `cargo test -p om9-app --lib -- menu_catalog solid_menu`: 7 passed, 0 failed.
- `cargo test -p om9-app --lib`: 275 passed, 2 failed — both `ui::icon_size_audit`, which counts the Top 11 row (11 extra rail slots, 31 vs 20) and were already failing before this change (recorded in `docs/openmatrix9-progress.json` under `unrelated_open_failures`).
- `cargo check -p om9-app -j 2`: exit 0.
- `python tools/public_source_audit.py`: 14804 source files, 594 icon bindings, 0 errors.
- `cargo test -p om9-app --test chrome_actions`: 28 passed, 0 failed, including simulated clicks on the six Sphere variant icons, Boolean Split/Create Solid, four Box variants, Multi Pipe and the icon-triggered secondary-click variants. Those tests now find an icon by its authored name and derive its page, so a reorder cannot make them click a different command.

## Window/exe icon

The debug executable had no Windows icon resource (Explorer, pinned shortcuts and the taskbar
showed the generic application icon even though the runtime window icon from
`launcher/OpenMatrix9.ico` was set). `crates/om9-app/build.rs` now compiles that authored ICO
with the installed Windows SDK `rc.exe` and links it into `om9.exe`; machines without the SDK
still build, with a warning and the previous default icon. Verified by extracting the icon from
`L:\om9-target\debug\om9.exe` after the rebuild (authored gem icon) and by reading the running
window icon handle.

One stale expectation in `sphere_variant_icons_start_the_recovered_modes` expected `Sphere AroundCurve` to skip straight to `Pick center on curve`. The implemented two-stage flow asks for the curve first (`Select curve or B-Rep edge`, then the center), which matches the documented workflow; the expectation now names that first prompt.

## Box command line (2026-11-01)

The Matrix command line shows one `Command: Box` echo and one prompt line whose option words
are underlined; OM9 showed the prompt twice: the start echo and a live line carrying every
option with its current value (`Diagonal=No 3Point=No … Cube=No Cancel`).

- The Box first stage now advertises the four documented mode words only. `Cube` appears after
  a Diagonal first corner and `Cancel` is the Esc path, so neither adds a word to the prompt.
- Those mode options keep the plain label (`Diagonal`, not `Diagonal=No`), so
  `prompt_with_options` sees the words already in the prompt and does not append a second
  signature; the existing inline renderer underlines and clicks them in place.
- A command started from a menu/icon records `Command: <feature name>` in the CMD history
  instead of echoing its first prompt; the live row carries that prompt.
- After the first corner, a single positive number is the base **length** in document units
  (the documented "Other corner of base or length" stage). The command then continues with
  `Width. Press Enter to use length ( 3Point ):` and `Height. Press Enter to use width:`,
  converts to the 3Point dimensions and clears the CMD draft on the accepted Enter. Pinned by
  `solid_box_accepts_one_length_after_the_first_corner` (10 × 10 × 10 mm = 1000 mm³) and the
  updated unit-once test (1/2/3 cm = 6000 mm³).
- The transcript mirrors the Matrix command log: the typed command, then one
  `prompt: value` line per input (colon, one space, value), with the `( 3Point )` mode tag on
  the Width prompt. There is no `[OM9-…]` feature-id line in the CMD; that id stays in the
  document history/receipts. `prompt_echo` applies the colon-space rule to every command.
  `chrome_actions::box_typed_stages_match_the_matrix_command_log` pins the exact transcript.

Covered by `chrome_actions::box_command_line_records_the_command_once_without_option_values`
(exact two-line transcript, four clickable inline tokens, no Cube token), the updated
`solid_box_exposes_clickable_mode_tokens`, and the unchanged `command_inline_options` suite.
The command-name suggestion list now drops **below** the command row like the Matrix8
autocomplete (above only when the row is close to the window bottom); pinned by
`command_inline_options::command_suggestions_drop_below_the_command_row`.
`cargo test -p om9-app --test chrome_actions` is 30 passed; `command_inline_options` is 5
passed; the app library is 320 passed with three failures outside this path
(`ui::icon_size_audit` ×2 and `split_point_cmd_widget_preserves_mode_after_object_selection`,
which still fails after the menu echo hunk is reverted and is being changed by parallel work in
the shared tree).

## Limits

The icon-to-command map is proved through `icons::feature_for` plus `Controller::dispatch` for every palette icon and through the simulated egui frame clicks above, not by a desktop mouse click. The Solid rows listed as having no owner are not commands yet; they remain visible and disabled. Native/OCCT geometry and the manual M03-UI picture review are unchanged by this checkpoint.
