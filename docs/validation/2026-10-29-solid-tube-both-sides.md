# Tube BothSides — 2026-10-29

The normalized Tube reference describes BothSides as extending the tube on both sides of its starting base by the entered height. OM9 now exposes a `BothSides` toggle, default No. Yes uses the chosen signed distance on each side of the captured base plane; preview shows both end rings and wall generators, and commit creates one closed body of twice the absolute height. Switching the option does not create history. For a staged session with BothSides enabled, direct three-number construction without a center is rejected rather than silently ignoring the option.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: nine Tube controller tests passed, including positive and negative height on a rotated CPlane, preview endpoints, analytic volume and Undo. One simulated viewport BothSides test and a keyboard/pointer/typed option test passed. Tube option and Truncated Cone regressions were also run.

The normalized reference shares a two-column boundary page; exact original prompt persistence and solver behavior are unverified. Actual desktop and GPU rendering, other Tube options and full Solid acceptance remain open.
