# Circle prompt stages — inline options match the Matrix reference (2026-11-02)

Scope: the user compared three Matrix screenshots with the OM9 Circle prompt and
rejected the over-advertised option list ("thì làm gì có 18 option").

Reference (Matrix, screenshots):

1. right after the icon: `Center of circle ( Deformable Vertical 2Point 3Point Tangent AroundCurve FitPoints ):`
2. after the center point: `Diameter <17.559> ( Radius Orientation Circumference Area ):`
3. after the second point: `Command:` — the circle is committed.

## Before

`circle_option_tokens` returned every mode plus the deformable entries, all four size
entries, Undo and Cancel in one list. After the center it published 18 options
(`Center=Yes 2Point=No … Size=Diameter (No) … Radius= Diameter= … Undo Cancel`), and the
prompt said only `Radius`.

## After

- The prompt word is the *current* mode/measure; the option list holds the alternatives.
  - Before the center: `Deformable Vertical 2Point 3Point Tangent AroundCurve FitPoints`
    (the current `Center` mode is omitted), seven options and nothing else.
  - After the center: the three other measures plus `Orientation` —
    with the default `Radius`: `Orientation Diameter Circumference Area`;
    after `Diameter` is chosen: `Radius Orientation Circumference Area`, exactly the
    reference order.
  - No `Undo`, `Cancel`, `Approx` or `Size=` tokens; Esc cancels, `Approx` stays a typed
    alias, and the current measure is the prompt word.
- `Orientation` is now implemented (the feature record claimed it but the command never
  parsed it): after the center it asks for a normal-direction point, then for the size
  (point or numeric). The circle frame uses that normal; the preview follows it.
- Typed `Radius=`, `Diameter=`, `Circumference=`, `Area=` are case-insensitive, and a bare
  measure word (`Area`, `Diameter`, …) switches the measure like clicking it.
- The generated inline signature now uses the Matrix spacing
  (` ( Deformable ... )`), so prompts read like the reference.

## Evidence

`crates/om9-app/tests/circle.rs` (32 passed) adds:

- `circle_prompt_and_options_match_the_reference_stages` — exact prompt text and option
  lists for the two reference stages, four options after the center.
- `circle_orientation_takes_a_normal_then_a_radius` — the orientation normal becomes the
  circle plane (+Y) and the next pick sets the radius.
- `circle_orientation_accepts_a_measure_word_then_a_value` — `Area` + area value commits
  radius 5 on an oriented circle.

Updated to the plain-word contract: `command_options_interface` (74), `command_values`
(7), `command_inline_options` (9) including the catalog sweep, `command_options_and_repeat`
(6), `command_snapshot` (3), `command_shortcuts` (3), `command_context` (5),
`top11_options` (10), `curve_menu` (5), `solid` (196), `solid_workflows` (113),
`solid_difference_ui` (128), `pinned_command` (7), `chrome_actions` (30).
The `om9-app` unit tests pass 330 with only the two pre-existing icon-size audit failures;
`session::command_options_tests` now records Circle alongside Solid as an owner that
cancels with Esc instead of a `Cancel` token. The full integration run still ends with the
same unrelated targets (`command_transactions`, `escape_commands`, `file_widgets`,
`global_status_surface`, `mesh_fill`, `shell_docking`, `transform_corrections`,
`ui_events`), none of which touch the Circle prompt.

## Limitations

- The live value (`Diameter <17.559>`) is not shown yet: the controller prompt has no
  access to the hover point, so only the measure word appears.
- `Orientation` with a normal parallel to the CPlane x-axis uses the recorded
  `commit_circle_normal` fallback/no-frame path; degenerate normals are rejected rather
  than converting to another plane.
- No manual M01-UI gate; the product window was not driven by a human after the change.
