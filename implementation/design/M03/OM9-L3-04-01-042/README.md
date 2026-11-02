# OM9-L3-04-01-042 — Transform menu action availability

- Owner: `OM9-L2-04-01`
- Row: the menu-wide availability answer for the Transform group

## What the row proves

`Controller::availability` answers for every one of the 41 catalogued Transform feature
ids. Fourteen rows answer `Ok(())` because `session::transform` owns a body for them; the
other 27 answer `Err(Error::Unsupported(reason))` with a non-empty reason, which is exactly
what makes a menu row visible but disabled instead of dead.

`transform_menu::BINDINGS` is the single table the answer comes from, so the state in the
record and the state in the menu cannot drift.

Each row also carries `command_key`, the authored `MainMenu.ini IconN=` / `icons.rs`
identifier. The parity test reads the real menu block, `assets/features.json` and
`icons::feature_for`, so the catalog cannot claim a row the authored menu does not have, and
an authored icon cannot point at the wrong feature.

The audit behind that test fixed one mis-binding: `TransformArrayAlongCurveonSurface` was
bound to `OM9-TRANSFORM-021` (Array Along Curve) and is now bound to
`OM9-TRANSFORM-026` (Array Curve on Surface). `OM9-TRANSFORM-017` (Orient on Curve) and
`OM9-TRANSFORM-022` (ReleaseFromCage) have no authored artwork at all; inventing an icon is
not allowed, so they keep their catalog row, stay disabled and render the fallback glyph,
and the test pins that set.

## What it refuses

A feature id that is not in the catalog answers `Ok(())` from this module (it is not a
Transform row), and the rest of the availability chain decides its fate.

## Verification

`cargo test -p om9-app --test transform_menu m03_04_01_042`

The test walks all 41 ids, compares each answer with the catalog row, and asserts that an
enabled row has a body in `transform::FEATURES` and a disabled row has a reason the error
message actually carries.

## Limits of this row

- The availability answer is menu state, not a UI review: `M03-UI` is unsigned.
