# Box CMD focus, origin input and readable history — 2026-10-12

The user's native screenshots reproduced three omissions in the previous
[Matrix-style common engine slice](2026-10-12-box-matrix-command-options.md):
icon start did not focus CMD, bare0 was rejected as a dimension, and internal
scientific-notation limits occupied the live prompt. The reference Matrix
example accepts0 then lengths10/20/10 and leaves the accepted questions/input
in history. Existing inline Native/Rust-first authorization was retained.

## Corrected behavior

- Left/right Box icon click focuses the live input after the icon/menu releases
  focus. Queued focus is bound to the active Rust session generation; cancelled
  or replaced commands cannot steal another editor's focus.
- Bare `0` in a point-only Box step is shorthand for the current CPlane origin,
  equivalent to `0,0,0`, and is recorded with the preceding question. The shared
  PointInput parser preserves the original token and world resolution. Initial
  points and Diagonal/Cube opposite points admit it; scalar Length/Width/Height
  phases still reject a zero dimension. Other commands retain phase-owned
  scalar/point routing; this is not a blanket claim of all command parity.
- Width says `Width. Press Enter to use length <10 mm>: `; Height says
  `Height. Press Enter to use width <20 mm>: `. Defaults are actual accepted
  native values under the existing Solid mm input policy.
- Invalid Width0 stays editable and records
  `Error: Width: Enter a value other than zero` in history. A too-large number
  says `This value is too large. Enter a smaller value`. The phase prompt remains
  clean. The existing technical bounds are unchanged: finite dimensions,
  minimum1e-7 mm, maximum1e9 mm, signed widths/heights and positive typed Length.
  These limits belong in engineering documentation rather than routine prompts.
- Accepted input is consumed exactly once; invalid drafts are preserved for
  correction, with no geometry. The example0→10→20→10 produces one valid native
  Box, dimensions10/20/10 mm, volume2000 mm³, with Undo/Redo.

No geometry solver, unit conversion, layer/History integration or global
completion status was changed. Native Part geometry remains mandatory. No
synthetic `_gvCommand`/mesh-building transcript was added; only actual events
are recorded. The existing two-history-row default and user-resizable Command
dock remain unchanged. Original command names and English UI wording remain.

The origin shortcut is documented in the
[Rhino 5 guide](https://docs.mcneel.com/rhino/5/usersguide/en-us/html/ch-08_transforms.htm).
[Rhino 5 Box help](https://docs.mcneel.com/rhino/5/help/en-us/commands/box.htm)
describes choosing base corners or a length and Enter reusing width for height.
The supplied Matrix screenshot supports the displayed example. This does not
claim full Matrix solver/private wrapper parity.

## Root cause and ownership

Solid start never requested input focus. Its lexical Scalar0 path called
dimension validation before checking the input phase; shared PointInput rejected
a single-component0. Box's stored error was prefixed to its question, and numeric
prompts showed bare defaults without Enter instructions. Earlier fixtures used
`0,0` and explicitly set CMD focus, so did not test the user's actual sequence.

Rust owns grammar, phase choice, numeric validation, default wording and accepted
point evidence. Qt C++ owns focus/event delivery, real widget lifetime, transcript
display and native geometry. The focus callback consults Rust active/generation
instead of retaining a native document pointer. Existing solid_message ABI uses
copied caller buffers; no new ABI was introduced. Box errors are separate
messages; Sphere retains its existing prompt/error flow.

New shared point admission also required Curve's accepted-input log to resolve
bare0 when no scalar option/length is pending. This preserves the original
world point in source snapshots rather than only normalized geometry. A rotated,
translated CPlane Line regression verifies `[12,34,56]` is retained as the first
resolved origin. Scalar options retain their independent admission.

## RED→GREEN and independent review

Four Rust regressions first failed on origin parsing and Enter wording.
`red01` native actual-icon/origin/history tests failed on plugince356b7. A fresh
read-only reviewer found one Important source-evidence omission for shared Curve
origin input and one Minor stale queued-focus issue. Both were verified and
repaired in one pass; no second review or deferred finding. The Curve evidence
test first showed resolved_point_world=null, and native `review-red02` showed a
visible focused editor losing focus to CMD after cancellation. Both now pass.

The reviewer declined unrelated WIP, inherited solver/default contracts,
OffsetBezier baseline and independent GUI execution. Parent performed the actual
native verification below. The existing Solid zero-base test was updated to
require a separate error history row, clean phase prompt, unchanged document and
retained raw0,2 input. Old exact Width/Height strings were updated to the explicit
Enter prompts. No geometry or negative-input assertions were removed.

## Fresh verification

Targeted Rust before review:72/72 across8 suites. After review repair:65/65
across7 suites, including all5 new usability tests and Curve construction.
Full Rust: **1028 passed / 1 existing failure**, 133
targets, log `C:/Users/nguye/AppData/Local/rtk/tee/1791775693_cargo_test.log`. Failure:
`support_snapshot::evaluation_tests::owned_curve_jets_match_native_value_and_three_derivatives_for_nineteen_families`,
OffsetBezier third derivative at0.1. It predates this change; no full-green claim.

Final actual native: **18 suites / 1067 checks**, all fixture
assertions and actual host exits pass on the same independently proved loaded
plugin/App/Gui pair. New usability fixture checks real left/right icon focus,
bare0 history/draft, readable prompts/errors, defaults, native volume and
Undo/Redo, plus focus cancellation. Common CMD–Builder toggles/children/numeric
controls, clipboard/IME/AltGr, Solid modes, Curve and Surface regressions pass.

| Native fixture | Checks passed |
|---|---:|
| `box_command_usability_smoke.FCMacro` | 16 |
| `box_command_options_smoke.FCMacro` | 26 |
| `box_command_history_smoke.FCMacro` | 11 |
| `solid_commands_smoke.FCMacro` | 70 |
| `solid_options_smoke.FCMacro` | 252 |
| `matrix_grid_command_smoke.FCMacro` | 37 |
| `command_line_options_smoke.FCMacro` | 26 |
| `builder_command_options_smoke.FCMacro` | 25 |
| `rebuild_command_options_smoke.FCMacro` | 15 |
| `command_completion_smoke.FCMacro` | 29 |
| `curve_smoke.FCMacro` | 27 |
| `curve_spline_smoke.FCMacro` | 42 |
| `rectangle_smoke.FCMacro` | 37 |
| `circle_advanced_smoke.FCMacro` | 64 |
| `surface_commands_smoke.FCMacro` | 97 |
| `surface_options_smoke.FCMacro` | 181 |
| `surface_history_smoke.FCMacro` | 65 |
| `rhino_core_native_rebuild_binding_smoke.FCMacro` | 47 |

Reports/profiles/screenshots are preserved under `C:/Users/nguye/AppData/Local/Temp/om9-box-usability-validation-20261012`.
`final01/native-summary.json` is consolidated as `final-native-summary.json`.
Earlier green and RED runs are preserved separately. Fixture source fingerprints
are checked against current owned source; actual loaded module hash/path and
process exits are checked independently. A native exit0 alone is not success.

## Build provenance and limits

Only standalone OpenMatrix9Gui was built using the existing cache
`H:/FreeCAD-src/build/openmatrix9-history-cage`, script
`H:/FreeCAD-src/build/build-om9-plugin.ps1` and single shared SDK
`H:/FreeCAD-src/build/relWithDebInfo`. Runtime, Rust output and native profiles
remain separate on C. No root FreeCAD build or complete runtime copy.

Concurrent unrelated App work made App162 and then App175 selection stale.
Both were rejected; App188 also became stale during the corrective build.
The final build reuses already built `atomic-host-app-refresh191-20261012` with its
successful build receipt and190 source candidates captured by that build.
A task-local selection manifest retains those recorded source hashes EXACTLY;
no old source fingerprints were refreshed to fabricate matching provenance.
Repository validator confirmed them before build. No shared DLL was replaced.
This is candidate source/output evidence, not an exhaustive compiler-dependency
certificate. Standard unextended-host deployment is not claimed.

- Plugin `C:/Users/nguye/AppData/Local/Temp/om9-box-usability-runtime-20261012/bin/OpenMatrix9Gui.pyd`, SHA-256 `fd35db61d111d07885beffecb657bbc7de34f529d9d1fcafe034508be640b9c9`.
- Loaded App `C:/Users/nguye/.codex/om9-native-tests/atomic-host-app-refresh191-20261012/FreeCADApp.dll`, SHA-256 `59062ad677b3c64f90c90c71ea6e2ad89de583fcd2b932a9e93e5e956ff0d219`.
- Loaded Gui `H:/FreeCAD-src/build/camera-close-host-green2-20261011/FreeCADGui.dll`, SHA-256 `8819fc482151dfef1964d7d3071ebd67ab3137a5a9d660b99a708ce13c395677`.
- Existing App selection/runner: `selected-app.json`, `run-compatible.ps1` and
  `atomic-host-app-refresh191-20261012.manifest.json` under the artifact directory.

Shared checkout/branch and unrelated WIP were retained; no staging/commit/push.
Old manual windows/models remain intact. The tested runtime will be reopened
with its own native profile. Scoped formatting/whitespace, source audit, local
links, feature JSON/YAML and owned manifest checks are recorded at completion.
This repairs approved behavior; no new skill business rule is being saved.

Native manual window reopened and confirmed ready/visible/focused: PID70512, document `OM9BoxUsabilityTest`, label `OM9 Box — CMD đã sửa`, zero objects and exact first Box prompt, same tested plugin SHA `fd35db61d111d07885beffecb657bbc7de34f529d9d1fcafe034508be640b9c9`. Start used an actual icon click without forcing CMD focus. Proof/screenshots: `manual01/manual-ready.json`, `manual-command.png`, `manual-window.png`. Old windows/models retained. Six owned manifest rows, Box JSON/YAML and 407 relative local links verified; scoped formatting/whitespace/source audit pass. Task complete within supported slice; native manual window remains open.

Later user correction: only a single declared shortcut character is underlined; full-name click remains. See [current mnemonic evidence](2026-10-12-command-mnemonics.md). Earlier runtime hashes/results above are preserved as historical evidence.
