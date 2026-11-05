# Command option visibility — rules taken from the local contracts (2026-11-02)

The user asked to re-read the documents to decide which options are shown, which are
hidden, and which appear only after an action. This record lists the sources and the
rules implemented on 2026-11-02.

## Sources read

- `OM9_Milestones/M01 .../L3-005 - Command Option Parser/reference/text/rhino5__p22-22__command-options.md`
  (Rhino 5 manual p22): options are shown in parentheses at the prompt; click, type the
  name or the underlined letter. The article's worked example is exact:
  `Center of circle ( Deformable Vertical 2Point 3Point Tangent AroundCurve FitPoints )` —
  the default/current mode is not repeated as an option.
- `implementation/design/M01/command-options-polyline-standard-20261026/SPEC.vi.md`
  §8 (prompt format, one prompt/one render path), §12.1 (Line → Polyline stage example:
  first point has no Length; after the point Length/Undo appear) and §12.2–12.3.
- `.../OWNER-CARDS.vi.md`: §2 Polyline stage table; §3.1 Line (`BothSides` before the
  first point, adapter was exposing it wider); §3.2 Rectangle (`Length` only when the
  parser already has the corner); §3.3 Arc (mode choices locked after the first point;
  `Radius`/`Diameter` only in Center mode); §4.2 Ellipse (modes at the pre-point stage);
  §6 Polygon/Helix/Spiral; §9.2 Sphere (mode list is replaced by constraint actions in
  AroundCurve/Tangent stages).
- `.../COMMAND-MATRIX.vi.md`: owner → provider/file/function inventory used to find each
  provider and parser.
- `skills/openmatrix9-feature-port/references/solid-prompt-contract.md` and
  `solid-box-sphere-contract.md`: approved stage behaviour for Box/Sphere and the other
  primitives (Cube after a diagonal corner, Radius after two 3Point picks, Center return
  after switching bases, WallThickness after the first radius, ...).
- User reference screenshots for Circle (Matrix): seven mode options at the center
  prompt, then the other measures plus Orientation.

## Rules applied

| Command | Rule (document) | Before | After |
|---|---|---|---|
| Line | Matrix Line: `BothSides` is the only option; the end point is picked or a length is typed (decision 2026-11-02) | BothSides **and** Length listed at both stages | start: `BothSides=No`; end: no options, `End of line: `; a bare number sets the length (`Length=` is a typed alias) |
| Rectangle | Matrix Rectangle: modes before the corner, a square with Shift; `Length` only after the corner (decision 2026-11-02) | `Square` + `Length` listed as options | modes only at start (no Square); `Undo`/`Length=` after the corner; Shift at the corner pick sets the square |
| Arc | Mode choices only before the first point, locked afterwards; Radius/Diameter only in Center mode (`OWNER-CARDS` §3.3) | current mode listed (`3Point=Yes`), Radius only in Center | current mode omitted; other modes plain words; choosing Center adds `Radius`/`Diameter`; after the first point the mode/radius options disappear |
| Ellipse | Construction modes only at the pre-point stage (`OWNER-CARDS` §4.2) | current mode listed (`Center=Yes`) | current mode omitted; other modes plain words |
| Sphere | Construction modes pre-center; constraint stages replace the list (`OWNER-CARDS` §9.2) | current mode listed (`Center=Yes`) | current mode omitted; other modes plain words |
| Circle | Rhino p22 list + Matrix screenshots (previous checkpoint) | 18 options after the center | 7 modes at the center prompt, 4 measures after it |
| Box | Approved prompt (previous checkpoint) | — | unchanged: 4 modes, Cube only after a diagonal corner |

The current mode/measure is the prompt word (`Center of sphere`, `Start of line`,
`Diameter`), so it is never an option; the options are the alternatives allowed in that
stage.

## Evidence

- `command_options_interface` (78 passed) adds four stage fixtures:
  `line_options_follow_their_stage`, `rectangle_length_is_hidden_until_the_first_corner`,
  `arc_modes_are_pre_point_and_the_current_mode_is_the_prompt`,
  `sphere_modes_omit_the_current_mode`; `line_prompt_carries_its_options_inline` now
  checks both Line stages.
- Updated fixtures/tests to the stage-scoped prompts: `command_feedback_adapter` (5),
  `command_sessions` (5), `command_snapshot` (3), `command_values` (7),
  `command_options_and_repeat` (6, JSON fixture `OM9-L3-01-02-005`),
  `command_state_machine` (3, JSON fixture `OM9-L3-01-02-004`), `arc` (7),
  `ellipse` (22), `rect` (16), `rectangle_regressions` (5),
  `command_transcript` (3), `command_events_surface` (5), `ui_replay` (2).
- Re-ran unchanged suites: `solid` (196), `solid_workflows` (113),
  `solid_difference_ui` (128), `chrome_actions` (30), `pinned_command` (7),
  `command_inline_options` (9, catalog sweep), `top11_options` (10),
  `command_context` (5), `command_shortcuts` (3).

## Not done / open

- The design SPEC §8 asks the live row to stay unwrapped with horizontal scrolling; the
  current bar wraps an idle prompt when the option signature is wider than the panel
  (previous checkpoint). Tokens stay clickable, but this is a deliberate divergence from
  that line of the SPEC.
- Live values (`Radius <17.559>`) are still not rendered; only the current word.
- Polygon's set choices still list both Circumscribed/Inscribed with `=Yes/No`; the
  contracts do not define hiding the current one.
- Cylinder/Tube/Cone/Torus/Ellipsoid were not re-audited beyond what their existing tests
  cover.
- Full `--tests` run was not repeated in this checkpoint (disk on the build drive was
  near full); the targeted suites above are the evidence.
