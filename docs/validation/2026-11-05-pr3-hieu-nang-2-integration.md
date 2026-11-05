# PR #3 integration into hieu-nang-2 — 2026-11-05

Base `b1c7f2c7dcbd372dc056ba065356af503ed19ccd`; add-on head
`afa2dd0efa576b1b2de28c5e18705bda5a4c686f`. Resolved conflicts in workspace
dependencies, dialogs, command/UI dispatch, Top11 grips, scene sampling and
document geometry. Add-on transactions use the destination's candidate-scene
publication; grip moves retain its tolerance and stale-gesture guards.

Windows, `CARGO_TARGET_DIR=L:\om9-target`, Cargo `-j 2`:

- `cargo check -p om9-app` and `cargo check --workspace`: passed; the add-on session reports one pre-existing unreachable-pattern warning.
- `cargo test -p om9-app --test addon_command --test addon_session --test addon_smoke --test grip_selection`: add-on command 1/1, session 11/11 and smoke 2/2 passed; initial grip fixture lacked the required document tolerance and 2/3 failed. The fixture was corrected and the grip test was rerun.
- `cargo test -p om9-app --test grip_selection`: final run 4/4 passed, including stale handle and changed-selection guards.
- `cargo test -p om9-app --lib ui::top11::tests`: 5/5 passed.
- `cargo test -p om9-addon-curves --lib`: 37/37 passed.
- `cargo test -p om9-scene --test nurbs_corners`: 1/1 passed.
- `tools/public_source_audit.py`: 15,819 source files, 594 icon bindings, 593 SVGs, zero errors.

The PR head's GitHub Windows build and native-exchange jobs passed against
`main`; they did not test this integration. No complete workspace run, desktop
pointer/GPU acceptance, or comprehensive Curve geometry certification is
claimed. Further validation: execute the actual Matrix UI menu/CMD, grip drag,
snap and gumball paths on the integrated build and audit scene memory under
large add-on geometry.
