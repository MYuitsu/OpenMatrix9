# Cylinder return to Center — 2026-10-29

The default Cylinder base takes a center and radius. After choosing 2Point, 3Point, FitPoints, AroundCurve or Tangent, OM9 previously accepted a typed `Center` before the first point, but did not offer the corresponding command option. OM9 now exposes Center in those stages, using the same existing mode switch.

The simulated command interface checks each of the five modes with keyboard shortcut, pointer and typed input, then constructs a closed cylinder and checks its analytic volume; no history entry is made by the mode switch. Existing Cylinder option and controller regressions were also run with `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`.

The original Matrix command wrapper identifies `_Cylinder` as the ordinary menu command and `_Cylinder _P` for the 2Point shortcut (`docs/analysis/matrix-command-layers.md`); this does not establish the original prompt options. Actual desktop and Matrix behavior remain unverified.
