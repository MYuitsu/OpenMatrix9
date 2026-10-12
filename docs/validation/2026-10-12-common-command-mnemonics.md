# Common Command Options mnemonics — 2026-10-12

User requested the single-character underline rule across commands through the
shared command engine. This extends the earlier
[Box correction](2026-10-12-command-mnemonics.md), without a separate renderer
or geometry implementation for each command.

Rust owns root/choice alias declarations, case-insensitive resolution,
phase-local choice precedence, ambiguity/gate checks and owned mnemonic text.
Qt underlines one character within a snapshot's exact displayed UTF-16 span,
preserves full-label click targets, and binds accepted values to native widgets.
Options with no usable shortcut have no misleading underline. Plain prompt
text, input drafts and existing native geometry ownership remain intact.

All current option schemas cover Line, Polyline, InterpCrv, Rectangle, Circle,
Ellipse, Rebuild, Box, Sphere, Sweep1/History, Sweep2/History and Loft. Rebuild
demonstrates dynamic registration; arbitrary root and choice aliases use the
same bounded ABI. `om9_options_alias` accepts `Parent=Choice` before sealing,
copies the buffers and keeps event/item layouts unchanged. New consumers use
the [registration contract](../command-options-adapter.md).

The aliases are explicit OM9 declarations. Ellipse uses E for Center and C for
Corner; a shared first-letter shortcut would be ambiguous. Sphere 2/3/4 choose
modes before the first point, and later scalar input remains numeric. Radius
appears as a link only where its native mode-switch action is supported;
ordinary radius/diameter phases accept scalar or existing named assignments.
Tangent branch errors are logged in CMD history instead of being discarded
when the engine refreshes the accepted phase question.

RED evidence includes missing Degree metadata, empty choice mnemonics, absent
Sphere schema, the Ellipse collision, absent Sphere 3Point and misleading
numeric-phase Sphere activation. One read-only review found the last three;
all were accepted for repair. No second review loop or unrelated WIP edits.
Native failed batches and diagnostic profiles remain outside the repository.

Validation and exact runtime evidence are recorded below after final checks.
This does not certify the whole RCORE-04, all original Matrix9 commands, Gem
solvers or unrelated geometry. Existing partial feature statuses remain.

## Final validation

Targeted Rust **47/47** (six new cross-command cases, nine ABI cases and existing
reducer/Box coverage). Fresh full Rust **1036 passed / 1 preexisting
failure**, 134 targets. Failure remains
`support_snapshot::evaluation_tests::owned_curve_jets_match_native_value_and_three_derivatives_for_nineteen_families`
for OffsetBezier third derivative at0.1; original native oracle remains intact.
Log: `C:\Users\nguye\AppData\Local\rtk\tee\1791780475_cargo_test.log`.

Final actual native **19 suites / 1585 checks**, all fixture
assertions and host exits0. The new command fixture has181 checks, including
Ellipse distinct choice shortcuts, native Sphere2/3Point, full-label clicks,
numeric units and native Rebuild/Loft control synchronization. Sweep1/Sweep2/
Loft option fixtures also check actual Qt underline spans for every registered
root option. Solid regressions retain geometry, references, ambiguous Tangent
branch admission, persistence and Undo/Redo. No geometry assertion was relaxed.

| Native fixture | Passed checks |
|---|---:|
| `command_mnemonics_smoke.FCMacro` | 181 |
| `box_command_usability_smoke.FCMacro` | 17 |
| `box_command_options_smoke.FCMacro` | 26 |
| `box_command_history_smoke.FCMacro` | 11 |
| `solid_commands_smoke.FCMacro` | 70 |
| `solid_options_smoke.FCMacro` | 252 |
| `command_line_options_smoke.FCMacro` | 26 |
| `builder_command_options_smoke.FCMacro` | 25 |
| `rebuild_command_options_smoke.FCMacro` | 15 |
| `matrix_grid_command_smoke.FCMacro` | 37 |
| `command_completion_smoke.FCMacro` | 29 |
| `curve_smoke.FCMacro` | 27 |
| `curve_spline_smoke.FCMacro` | 42 |
| `rectangle_smoke.FCMacro` | 37 |
| `circle_advanced_smoke.FCMacro` | 64 |
| `surface_commands_smoke.FCMacro` | 97 |
| `surface_options_smoke.FCMacro` | 517 |
| `surface_history_smoke.FCMacro` | 65 |
| `rhino_core_native_rebuild_binding_smoke.FCMacro` | 47 |

Native Qt Unicode probe additionally passes single-character root and nested
choice underlines, unchanged draft, actual bold document font and visible
history. Scoped rustfmt/whitespace checks and public source audit pass (2866
source files,511 icon bindings,510 SVGs, errors[]).

Plugin `C:\Users\nguye\AppData\Local\Temp\om9-all-mnemonics-runtime-20261012\bin\OpenMatrix9Gui.pyd`, SHA-256 `cbe982b7f5129811f85544a2b4736cc5361bb56f53e9b22c30695be46191733d`.
Loaded App `C:\Users\nguye\.codex\om9-native-tests\atomic-host-app-source209-20261012\FreeCADApp.dll`, SHA-256 `fa38d4ba729b245cff4ca27d403fef4030f44d41a0073c546a05fc22b6ebcc32`.
Loaded Gui `H:\FreeCAD-src\build\camera-close-host-green2-20261011\FreeCADGui.dll`, SHA-256 `8819fc482151dfef1964d7d3071ebd67ab3137a5a9d660b99a708ce13c395677`.
Artifacts, captured190 source rows, build receipt, selection manifest, runner
and consolidated `native-final-summary.json` are under `C:\Users\nguye\AppData\Local\Temp\om9-all-command-mnemonics-20261012`. Existing App209
and its paired import library were reused. Only standalone OpenMatrix9Gui was
built on the shared SDK/cache; no FreeCAD root build, shared DLL replacement or
runtime clone. Candidate fingerprints are not a full compiler-dependency or
native memory-safety certificate. Existing user windows and models were retained;
no further manual window was launched after the user confirmed closing it.

Earlier `native-final01` retained a swallowed Sphere Tangent diagnostic, a
misnamed Rectangle fixture and a Sweep2 crown mismatch. The diagnostic is now
preserved in CMD history and Rectangle uses its actual fixture. The initial
crown discrepancy did not reproduce with the old plugin or the rebuilt current
plugin on the same selected host; final unchanged crown/rail checks pass and
report distances. Its earlier cause is not established, and that witness is
preserved. `surface-current-diagnostic` independently passes517 assertions.
No all-product-green or complete private Matrix shortcut parity is claimed.

One read-only review identified the Ellipse alias collision, omitted Sphere
3Point and unsupported bare Radius/Diameter activations. Each has a failing
Rust witness and repair. Reviewer-declined unrelated WIP and broad geometry
parity remain outside this change; parent performed relevant native visual,
geometry and pass-evidence checks. No second review, staging, commit or push.

Later on 2026-10-12, the user explicitly requested reopening the tested native
build for manual Box/Sphere/Circle/Rebuild checks. Window
`OM9 — Test Box Sphere Circle Rebuild` opened with PID60456, responding,
visible and CMD focused at the first Box phase. Loaded plugin SHA matches
`cbe982b7f5129811f85544a2b4736cc5361bb56f53e9b22c30695be46191733d`.
The new document contains one labeled Rebuild sample circle; existing user
models/windows are retained. Ready snapshot and screenshots are in
`manual-four-commands01` under the artifact directory above. The retained
launcher waits for user closure and does not automatically close this window.
