# Solid broad regression and stack safety — 2026-10-29

The accumulated Solid work was checked across full controller, option, simulated viewport and internal input-regression suites. Two stack overflows were reproduced in the default Windows test thread: linked Fillet CurrentRadius recursively entered the large Solid session handler, and a Boolean command with an inline option entered the same handler recursively while opening a session. Linked CurrentRadius now takes the existing SetAll branch directly; the lightweight dispatcher now separates starting a command from processing a pending session. The targeted failing cases pass without increasing stack size.

Three internal Pyramid/Cylinder fixtures still expected the older numeric-only prompts. They now test the current center-first sessions while retaining invalid-input, retry and no-mutation assertions.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: the complete `solid` integration suite passed 130 tests with one ignored; `command_options_interface` passed 68; `solid_difference_ui` passed 100; `solid_input_regressions` passed 56. The ignored test requires a native-UI-saved ellipsoid fixture via `OM9_ELLIPSOID_UI_FIXTURE`. The two previously overflowing tests passed individually with the final dispatcher. The broad integration runs were started before the final `inline(never)` annotation, which preserves the tested separation and was checked by the targeted reruns.

These are automated controller and simulated egui checks, not proof of original Matrix equivalence, physical desktop interaction, all recovered Solid variants, or the ignored native UI fixture. No reusable business rule changed.
