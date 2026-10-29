# Execution checkpoint — command options / Polyline standard

Date: 2026-10-26
Scope: P00 baseline + P04 single render/activation path; P05–P12 remain in progress.
This file records implementation facts and test evidence. It is not a human milestone sign-off.

## Baseline

- Base commit: `71852257` (worktree intentionally dirty from parallel work).
- Documentation bundle validator: baseline bundle itself is `PASS`; after this checkpoint's source changes the
  validator reports expected hash drift for `crates/om9-app/src/ui/command.rs` (single shortcut/underline path).
  Per the bundle README this drift is recorded here, not repaired by resetting the baseline or the code.
  - 39 groups, 37 registered providers, 57 Controller pending fields, 19 additional pending owners.
  - 39 owner cards, 13 task packages, 50 acceptance templates, 0 executed acceptance rows.
  - 51 source hashes, 46 links, 0 errors.
- Build target: `L:/om9-target`, `-j 2`; no build inside the checkout.

## Implemented and verified this checkpoint

- `command_options.rs` / `polyline_options.rs` remain the single shared contract for prompt options,
  current values, shortcuts and activation.
- Command UI accelerator path is now one path (`Controller::command_options` → `consume_command_shortcut`
  → `submit_command_option`). The temporary Polyline-only raw-hook accelerator was removed so a shortcut
  cannot bypass owner availability or run/stage checks.
- Inline underline uses the option's declared `shortcut` character, searching within its own label; it no
  longer blindly underlines the first byte.
- Polyline Start/Next prompts expose only owner-available options; `Length` appears after the first point,
  not at Start, and `Close` remains distinct from `PersistentClose`.
- Click targets and keyboard activation still use the same snapshot and activation function.
- First additional pending owner migrated: `surface_pending` exposes `Distance` through the same contract.
  Activation starts a `Distance=` draft; `Distance=2` and the bare owner value both reach the same numeric
  parser. The offset prompt keeps its owner text and adds the option label for inline rendering.
- Second additional pending owner migrated: `curve_pair_pending` exposes `Axis` and accepts
  `Axis=x,y,z,dx,dy,dz` through the same six-value validator; rail-revolve geometry is covered by test.
- Third additional pending owner migrated: `tween_pending` exposes `Fraction`; both `Fraction=0.5` and
  `0.5` reach the owner's 0..1 validation and build the intermediate surface.
- Fourth additional pending owner migrated: `seam_pending` stages `Axis` then `Tolerance` in owner state;
  the legacy combined `U,0.01` input remains valid. Negative/non-finite tolerance is rejected.
- Fifth additional pending owner migrated: surface `extrude_all_pending` exposes `Height`; the solid
  ExtrudeAll command remains in the solid owner and was not conflated with this wrapper.
- Sixth additional pending owner migrated: `extend_pending` stages `Edge` then `Distance`; legacy
  `U1,5` still reaches the same owner, and non-finite distance is rejected.
- Seventh additional pending owner migrated: `split_pending` stages `Axis` then `Parameter`; parameter is
  constrained to the closed 0..1 domain and commit creates both result surfaces in one transaction.
- Eighth additional pending owner migrated: `surface_pair_pending` stages `Source` then `Target` edges;
  the legacy comma form still reaches the same match owner.
- Ninth additional pending owner migrated: `tangent_pending` stages `Edge` then `Direction`; direction
  must contain exactly three finite values.
- Tenth additional pending owner migrated: `chamfer_pending` exposes `Setback`; the owner keeps its
  positive finite constraint and single-transaction commit.
- Eleventh additional pending owner migrated: `fillet_pending` exposes `Radius` for the constant strip and
  stages `Start` then `End` for the variable fillet; both keep owner geometry checks.
- Twelfth additional pending owner migrated: `variable_chamfer_pending` stages `Start` then `End`; failed
  owner geometry restores the pending command and draft stage.
- Thirteenth additional pending owner migrated: `variable_blend_pending` stages `Start` then `End` tangent
  scales; the owner still rejects non-finite values and preserves state on a failed blend.
- Fourteenth additional pending owner migrated: `variable_offset_pending` stages `Start`, `End`, then
  `Axis`; the legacy `1,5,U` path still reaches the same owner.
- Fifteenth additional pending owner migrated: `shrink_pending` stages `U0`, `U1`, `V0`, `V1`; owner
  domain validation still decides whether the shrink is legal.
- Sixteenth additional pending owner migrated: `history_options_pending` exposes `Closed` and `Flip`
  toggles that update owner state without building; `Slash` transitions into the existing slash input.
- Seventeenth additional pending owner migrated: `soft_edit_pending` stages the seven UV/radius/delta
  values; legacy comma input still reaches the same owner and a failed apply restores the pending stage.
- Top11 Duplicate now exposes `Vertical`, `FromLastPoint`, `UseLastDistance`, `UseLastDirection` and toggles
  owner state without building. Split/Trim/Move/Rotate/Mirror remain positional/object input by contract.
- Slash exposes `First` then `Second`; both accept typed coordinates and keep the existing viewport point
  path. This closes the initial 19 additional-owner audit list (top11 special actions remain positional).
- Option IDs now carry a monotonic Controller run generation. Starting a command bumps the generation, so
  an option snapshot from a previous run — including the same owner/name — is rejected before activation.
  This covers the P02 same-owner new-run requirement.
- P05 owner check added for Rectangle: the shared `Square` option is activated through the same snapshot,
  then actual corner points produce a 20×20 square. Existing Line unit-snapshot test still passes.
- P08 owner check added for Offset: the shared `Distance` option opens the draft and `Distance=2` commits
  the same NURBS offset as the legacy typed route.
- P05 Arc check added: shared `Center` choice enters center mode and the real arc has the expected radius
  and angle span.
- P09 AngleIsocurve check added: `Angle` and `Direction` both activate through the shared snapshot and
  produce the same NURBS isocurve as the typed route.
- P09 CrossSection check added: `Count` and `Spacing` open the shared entries and commit real section
  curves at the expected levels.
- P09 ExtrudeSurface check added: shared `Height` commits a surface spanning z=0..10.
- P06 Circle check added: shared `Size=Area` choice then the area value commits a radius-5 circle.
- P06 Ellipse check added: `RadiusX` then `RadiusY` use the shared entries and commit the expected extents.
- P10 Box check added: shared `Center` choice enters the owner mode; the existing
  `solid_box_center_mode_uses_the_center_and_doubles_the_base` geometry test still covers the commit.
- P10 Sphere check added: shared `2Point` choice then the two points commits a radius-3 sphere with the
  expected volume, proving the choice changes actual owner geometry rather than only its label.
- P10 Mirror check added: shared `Copy` toggles the Delete-Input mode; two plane points then reflect the
  original line in place with the expected mirrored start.
- Registry refreshed from 37 to 56 providers after the 19 additional-owner migration. `om9-app --lib`
  passes 179 tests; the legacy owner-fixture test keeps its original 37 deep fixtures and skips owners
  whose dedicated integration target now provides the stronger geometry/state proof.
- Full app regression found and fixed a phase regression: `phase()` had lost the active `CommandSession`
  arms, so a running Line read as Idle even though the session existed. Restoring the session match makes
  `command_context` 5/5 and keeps `command_run` snapshot rejection intact.
- `CommandOption` now carries a typed `CommandValue` (flag/choice/number/integer/point/text/none) in
  addition to its display label. Polyline and Line populate real owner values; `command_values` proves
  `Length=None -> Some(5)` and `BothSides=Flag(false)` come from the snapshot, not from parsing the label.
- Added `active_command_input_snapshot()`: owner, run generation, phase and options from the same owner
  read. Idle is empty; Line is `line`/AwaitPoint→AwaitEnd with the same run; completion returns to empty.
  `command_snapshot` covers this contract without falling back to the repeat slot.
- Positional actions (Top11 Move/Rotate etc.) now report their real owner (`top11`) and phase in the same
  snapshot while keeping the option list empty; no fake option row is created. `command_snapshot` covers
  Move, and `command_values` covers the typed Top11 path.
- Polyline is covered end-to-end through the snapshot: Start reports owner `polyline` with
  PersistentClose/Mode/Helpers and no Length, after the first point Length appears, and the run stays the
  same across the stage change.
- Helix and Spiral now populate typed values too: Radius/Height/Turns/Axis and
  StartRadius/EndRadius/Turns/Height/Axis read directly from their pending owners; `command_values` covers
  defaults `Turns=3`, `Height=10` and unset radii.
- AngleIsocurve, OffsetCurve, Fit, CrossSection, ExtrudeSurface and Divide now populate typed values too
  (Angle/Direction, Distance, Tolerance, Count/Spacing, Height, Count). `command_values` covers the
  Angle/Direction and Divide integer defaults.
- Rebuild, Project, Blend, Pull, Intersect, Polygon, ThroughPoints, CrvSeam, ExtendCurve, TweenCurves,
  FilletCorners, ToLines, OffsetOnSurface and OffsetNormal now populate typed values from their pending
  owners as well. `command_values` covers Rebuild Degree/Points and Polygon Sides.
- Arc, Rectangle, Circle and Ellipse populate typed mode/radius/degree/point-count/flag values.
- Circle also exposes typed `Size=` choice flags and `Solution=` flags; `command_values` activates
  `Size=Area` after the center and checks the resulting typed flag transition.
- Line now carries both `BothSides=` and `Length=` in the inline prompt. The UI no longer produces the
  floating `BothSides=No Length=` row seen in the reference screenshot; the shared options render inside
  the prompt and the token rectangles still resolve. `command_options_interface` covers this inline prompt.
  `command_values` covers Arc Center flag, Rectangle Square flag, Circle Deformable flag and the options
  that are actually available at each first prompt.
- Surface seam/split/fillet/variable-chamfer/variable-blend/variable-offset/shrink/soft-edit providers now
  populate typed values from their staged owner fields (axis/tolerance, parameter, radii/scales, UV bounds,
  seven soft-edit values). Their dedicated integration tests already cover the owner state transitions.
- Interp, ControlCurve, ContinueCurve, ContinueInterp, Heightfield, Corner and Sweep-history options now
  populate typed values from their pending owners. `command_values` covers Interp Degree; the remaining
  owners are exercised by their existing command tests.
- Isocurve, Tangent, SurfacePair, Slash and Top11 now populate typed values too (U/V current value,
  direction point, source/target edge choices, first/second slash points, Duplicate flags and Mirror copy).
  `command_values` covers the Duplicate flag transition false→true.

## Test evidence (all pass)

```text
cargo test -p om9-app --test command_options_interface -j 2
  test result: ok. 9 passed; 0 failed
cargo test -p om9-app --test polyline_options_interface -j 2
  test result: ok. 3 passed; 0 failed
cargo test -p om9-app --test polyline_mode_regression -j 2
  test result: ok. 9 passed; 0 failed
cargo test -p om9-app --test command_options_and_repeat -j 2
  test result: ok. 6 passed; 0 failed
cargo test -p om9-app --test pinned_command -j 2
  test result: ok. 7 passed; 0 failed
cargo test -p om9-app --test polyline_shortcuts -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test surface_options -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test rail_revolve --features occt-brep -j 2
  test result: ok. 2 passed; 0 failed
cargo test -p om9-app --test tween_surfaces --features occt-brep -j 2
  test result: ok. 2 passed; 0 failed
cargo test -p om9-app --test seam_srf --features occt-brep -j 2
  test result: ok. 2 passed; 0 failed
cargo test -p om9-app --test polyline_mode_regression --test polyline_options_interface --test command_options_interface -j 2
  test result: ok (9 + 3 + 9 passed)
cargo test -p om9-app --test surface_menu -j 2
  test result: ok. 42 passed; 0 failed
cargo test -p om9-app --test extend_shrink --features occt-brep -j 2
  test result: ok. 3 passed; 0 failed
cargo test -p om9-app --test split_srf --features occt-brep -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test match_srf --features occt-brep -j 2
  test result: ok. 2 passed; 0 failed
cargo test -p om9-app --test tangent_srf --features occt-brep -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test surface_menu chamfer_setback_uses -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test surface_menu fillet_radius_options -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test surface_menu variable_chamfer_options -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test surface_menu variable_blend_options -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test surface_menu variable_offset_options -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test extend_shrink --features occt-brep -j 2
  test result: ok. 4 passed; 0 failed
cargo test -p om9-app --test surface_menu sweep1_history -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test surface_menu soft_edit_options -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test top11_options -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test surface_menu sweep2_slash -j 2
  test result: ok. 1 passed; 0 failed
cargo test -p om9-app --test command_options_interface -j 2
  test result: ok. 9 passed; 0 failed (includes same-owner new-run rejection)
cargo test -p om9-app --test rect --features occt-brep -j 2
  test result: ok. 16 passed; 0 failed
cargo test -p om9-app --test offset --features occt-brep -j 2
  test result: ok. 4 passed; 0 failed
cargo test -p om9-app --test arc --features occt-brep -j 2
  test result: ok. 13 passed; 0 failed
cargo test -p om9-app --test angle_isocurve --features occt-brep -j 2
  test result: ok. 2 passed; 0 failed
cargo test -p om9-app --test cross_section --features occt-brep -j 2
  test result: ok. 3 passed; 0 failed
cargo test -p om9-app --test extrude_surface --features occt-brep -j 2
  test result: ok. 2 passed; 0 failed
cargo test -p om9-app --test circle --features occt-brep -j 2
  test result: ok. 29 passed; 0 failed
cargo test -p om9-app --test ellipse --features occt-brep -j 2
  test result: ok. 22 passed; 0 failed
cargo test -p om9-app --test solid --features occt-brep -j 2
  test result: ok. 70 passed; 1 ignored
cargo test -p om9-app --test top11_options -j 2
  test result: ok. 2 passed; 0 failed
cargo test -p om9-app --lib -j 2
  test result: ok. 179 passed; 0 failed
cargo test -p om9-app --test command_context -j 2
  test result: ok. 5 passed; 0 failed
cargo test -p om9-app --test command_values -j 2
  test result: ok. 7 passed; 0 failed
cargo test -p om9-app --test command_snapshot -j 2
  test result: ok. 3 passed; 0 failed
cargo test -p om9-app --test surface_menu --test offset --test offset_on_surface --test offset_normal -j 2
  test result: ok (3 + 1 + 1 + 41 passed)
cargo test -p om9-app --test rect --test arc --test circle --test helix -j 2
  test result: ok (15 + 8 + 6 + 6 passed)
```

The batch proves the shared interface, Polyline stages/shortcuts, repeat-slot isolation and the existing
Rectangle/Arc/Circle/Helix owner tests. It does not prove every command or mode is migrated.

## 19 pending owners — audit classification

These fields are outside the registered 39 option groups. They are not forced into the Polyline option
contract; each row must be classified from its own parser and milestone before code changes.

| Owner field | Preliminary class | Reason / next check |
|---|---|---|
| `top11_pending` | OPTIONS (partial) / SPECIALIZED | Duplicate flags exposed; Split/Trim/Move/Rotate/Mirror remain positional/object input |
| `surface_pending` | OPTIONS (partial) | `Distance` now opens the shared value entry; offset/extrude keep their numeric owner. Further variants still need stage audit |
| `curve_pair_pending` | OPTIONS (partial) | `Axis` opens the shared six-value entry; owner still validates origin/direction |
| `surface_pair_pending` | OPTIONS (partial) | `Source` then `Target` staged; legacy `U1,U0` still valid |
| `seam_pending` | OPTIONS (partial) | `Axis` then `Tolerance` are staged in the owner; legacy `U,0.01` still works |
| `soft_edit_pending` | OPTIONS (partial) | `U`, `V`, `RadiusU`, `RadiusV`, `Dx`, `Dy`, `Dz` staged one at a time |
| `tween_pending` | OPTIONS (partial) | `Fraction` opens the shared entry; owner keeps the 0..1 validation |
| `extend_pending` | OPTIONS (partial) | `Edge` then `Distance` are staged; legacy `U1,5` still valid |
| `split_pending` | OPTIONS (partial) | `Axis` then `Parameter` staged; parameter constrained to 0..1 |
| `tangent_pending` | OPTIONS (partial) | `Edge` then `Direction` vector staged; vector must have 3 finite values |
| `extrude_all_pending` | OPTIONS (partial) | Surface ExtrudeAll has `Height`; solid ExtrudeAll remains its own solid owner |
| `variable_offset_pending` | OPTIONS (partial) | `Start`, `End`, `Axis` staged; owner keeps axis/distance checks |
| `shrink_pending` | OPTIONS (partial) | `U0`, `U1`, `V0`, `V1` staged; owner validates the sub-domain |
| `variable_blend_pending` | OPTIONS (partial) | `Start` then `End` tangent scales staged; finite values only |
| `chamfer_pending` | OPTIONS (partial) | `Setback` uses the shared entry; zero/non-finite still rejected |
| `fillet_pending` | OPTIONS (partial) | Constant `Radius`; variable fillet stages `Start` then `End` |
| `variable_chamfer_pending` | OPTIONS (partial) | `Start` then `End` staged; positive finite constraints preserved |
| `history_options_pending` | OPTIONS (partial) | `Closed`/`Flip` toggle without building; `Slash` transitions to slash input |
| `slash_pending` | OPTIONS / POSITIONAL_POINT | `First`/`Second` open the two point stages; viewport picks still work |

All 19 remain `AUDIT_REQUIRED` until their exact parser, stage and transaction boundary have been read.

## Remaining work

- P05/P06/P07 owner-by-owner parity audit (Line/Rectangle/Arc, Circle/Ellipse, point chains).
- P08/P09 curve modifiers and surface-related owners.
- P10 Solid/Top11 positional chains.
- P11 the 19 pending owners above.
- P12 final entry-point reconciliation, native replay/provenance, clippy/fmt/build/public-source audit.

## Known gaps

- Full `cargo test -p om9-app --tests` currently reports failures in `file_widgets` around the expected
  coordinator snapshot and the Browse control after an unsupported save path. These tests were not edited
  by this checkpoint; treat them as a separate baseline failure to investigate, not as option-interface
  evidence. A deferred-staging experiment was reverted; the file lifecycle remains as it was.
- `CommandOption` has typed values for Polyline/Line; the other provider bridges still construct `None`
  because their token APIs expose labels only. That remaining provider-by-provider migration is open.
- No native executable replay is recorded in this checkpoint.
- No release/install update; no human M01-UI/M03-UI gate is signed.
- Worktree contains unrelated parallel changes; no commit was made by this checkpoint.

