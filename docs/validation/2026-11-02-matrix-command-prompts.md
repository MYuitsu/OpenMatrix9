# Matrix manual prompts vs the OM9 command line (2026-11-02)

The user noted that some Matrix commands override their Rhino-style options, and that the
manual text already lives in the per-L3 reference folders. This record keeps the
family-level findings read from the manual; the machine-derived, all-family inventory
lives in [matrix-ref-command-options-2026-11-02.md](matrix-ref-command-options-2026-11-02.md)
(371 command sections parsed from 1130 `OM9_Milestones/**/reference/text` files) and the
option diff in [matrix-ref-option-diff-2026-11-02.md](matrix-ref-option-diff-2026-11-02.md).
Page numbers below are the manual's printed pages.

## Decision 2026-11-02 — Matrix direction for the two conflicts

The user chose the Matrix direction for the two open conflicts:

- **Line** no longer publishes a `Length` option at any stage. Matrix Line takes a point or
  a typed length at the end prompt: a bare positive number now sets the length (converted
  through the session unit), and `Length=` remains a typed alias. The M01
  `OM9-L3-01-02-005` fixture now declares `BothSides` only, and the end prompt is
  `End of line: `.
- **Rectangle** no longer publishes a `Square` option. Matrix makes a square by holding
  Shift while picking the corner: `Controller::set_rect_square` drives the owner flag from
  the viewport pick, and the typed `Square`/`s` word remains a hidden alias. Matrix
  `Rounded` (Arc/Conic corners) is still a feature gap.

Evidence: `command_options_interface` 78, `command_values` 7, `command_snapshot` 3,
`command_feedback_adapter` 5, `command_sessions` 5, `command_options_and_repeat` 6,
`command_state_machine` 3, `rect` 16, `rectangle_regressions` 5,
`command_inline_options` 9, `command_tokens` 1, `pinned_command` 7 and `preview_drag` 5
(including the new Shift-square pointer test) pass. The skill reference
[command-option-visibility.md](../../skills/openmatrix9-ui/references/command-option-visibility.md)
records the decision.

## Curve tools (Matrix 8 book 1, chapter 19)

| Command | Matrix steps / options (printed pages) | OM9 today |
|---|---|---|
| Polyline (p122-123) | `PersistentClose`; `Close` only after 3 points; `Mode=Line/Arc`; `Length` only in Line mode; `Direction`/`Center` only in Arc mode; `Helpers=Yes/No` | matches |
| Line (p123-125) | `BothSides`, `Normal` (`IgnoreTrims`), `Angled`, `Vertical`, `FourPoint`, `Bisector`, `Perpendicular` (`BothSides`, `Point`, `PointOnCurve`, `FromFirstPoint`, `2Curves`), `Tangent` (same set), `Extension`; length is typed at the end prompt, not an option | only `BothSides` + `Length`; the rest is not implemented |
| Rectangle (p127) | `3Point`, `Vertical`, `Center`, `Rounded` (Corner `Arc`/`Conic`); a square is made with Shift; a typed value at the second-corner prompt is the length | `Vertical`, `3Point`, `Center`, `Square` (Rhino-style), `Length` token after the first corner; `Rounded` missing; Shift-square missing |
| Circle (p128-130) | center + radius; options `Radius/Diameter` toggle, `Orientation`, `Circumference`, `Area`; construction `2Point`, `3Point` (with `Radius` after two picks), `Tangent` (`Point`, `FromFirstPoint`, `Radius`), `AroundCurve`, `FitPoints`, `Vertical`; the user's Matrix screenshot lists `Deformable Vertical 2Point 3Point Tangent AroundCurve FitPoints` at the center prompt | start list matches the screenshot; the size stage matches `Radius/Diameter + Orientation + Circumference/Area`; `3Point`+`Radius` and `FromFirstPoint` are not implemented |
| Ellipse (p130-131) | `Deformable`, `PointCount`, `Vertical`, `Corner`, `Diameter`, `FromFoci` (`MarkFoci`), `AroundCurve`; center is the default flow | matches after hiding the `Approx` alias (this checkpoint); `Degree=` is an extra entry token |
| Arc (p132-133) | **center-based default**; options `StartPoint` (`Direction`, `ThroughPoint`, `Center`), `Center`, `Tangent` (`Point`, `FromFirstPoint`, `Radius`), `Extension` (`Center`), `Deformable`, `Degree`, `PointCount` | default flow is `3Point` (Rhino-style); modes `3Point`/`Center`/`StartPoint`; `Radius/Diameter` only in Center; no `Tangent`, `Extension`, `Deformable`, `Degree`, `PointCount` |
| Arc Direction (p134) | same option set as Arc `StartPoint` | `StartPoint` mode exists |

## Solid primitives (Matrix 8 book 1, chapter 21)

- Box (p222-223): options `Diagonal` (with `Cube` after the diagonal corners), `3Point`,
  `Vertical`, `Center`; the default flow picks first corner, opposite corner or length,
  width (Enter uses length), height (Enter uses width). OM9 matches this option set and the
  Cube stage.
- Sphere (p223): options `2Point`, `3Point` (`Radius` after two picks), `Tangent` (Point
  or radius), `AroundCurve`, `4Point`, `FitPoints`; the center flow is the default, so no
  `Center` option. OM9 matches after hiding the current mode; `Vertical` is an OM9/Rhino
  extension that Matrix does not document.
- Ellipsoid (p223-224): `Corner`, `Diameter`, `FromFoci` (`MarkFoci`), `AroundCurve`.
  OM9 matches the modes; `MarkFoci` appears with the foci branch.

## Consequence for the option-visibility pass

- The current-mode omission already applied (Arc/Ellipse/Sphere) agrees with the Matrix
  prompts.
- The biggest Matrix overrides not implemented in OM9 are: Line's option families, the
  center-based Arc with `Tangent`/`Extension`, Rectangle's `Rounded` and Shift-square, and
  Circle's `3Point + Radius` / `Tangent FromFirstPoint`.
- Two local contracts conflict with the Matrix text and need a decision: the M01
  `OM9-L3-01-02-005` fixture declares `BothSides` and `Length` as Line options, and the
  owner card declares Rectangle `Square`; Matrix treats length as typed input and squares
  as a Shift gesture.

## Applied in this checkpoint

- Ellipse no longer advertises the `Approx` alias (Matrix documents `Deformable` only);
  `Approx` remains a typed alias. `ellipse` 22 and `command_inline_options` 9 pass.
