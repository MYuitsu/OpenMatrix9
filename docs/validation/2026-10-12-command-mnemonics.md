# Single-character command mnemonics — 2026-10-12

User corrected the previous full-name underline: the underline identifies one
shortcut character. This supersedes that presentation in the earlier
[common engine](2026-10-12-box-matrix-command-options.md) and preserves the
[Box focus/origin/error fixes](2026-10-12-box-command-usability.md).

Rust supplies only a declared unambiguous single-character alias present in
the displayed key. Qt underlines that character, including a surrogate pair
when it is one Unicode character. The full key remains the click target and
selection highlight; copied prompt text has no extra markup. Options without
a declared one-character shortcut do not advertise a fabricated shortcut.
Disabled/ambiguous aliases retain their validation; full-name input/click works.
Pending choice labels retain full click spans and arrow/Enter handling; this
does not invent new choice-key mappings.

Box marks D/3/V/C for Diagonal/3Point/Vertical/Center. The existing3P alias stays
supported;3 is active only before the first point. At length/width/height steps,
3 remains numerical input. Diagonal's Cube marks C only in its applicable step.
Question bold styling and the earlier focus/origin/history fixes are preserved.

PromptItem owns mnemonic text in Rust. Existing item_text field3 copies it into
the caller buffer. Event/item ABI structs remain16 bytes; no foreign pointer is
retained. Qt reads that string and decorates only its exact UTF-16 range while
the full hit-test range remains unchanged. Rust owns alias policy; Qt/FreeCAD
remain the necessary native font, widget and geometry adapters.

RED: old native plugin failed the one-character Box assertion (`mnemonic-red01`)
and Rust lacked the3 alias/owned metadata. A phase regression then rejected
numeric3 after the first point; the alias is now declared only in the first
phase. The first native batch also exposed an old release archive compiled
before that source correction. Explicitly rebuilt the current Rust archive
and relinked the plugin before final verification. This was not concealed by
source fingerprints or weakening numeric assertions. `mnemonic-final01`
failures are retained; all selected final evidence is `mnemonic-final02`.

Targeted Rust40/40; fresh full Rust **1029 passed / 1 preexisting
OffsetBezier third-derivative failure**, 133 targets, log `C:\Users\nguye\AppData\Local\rtk\tee\1791777580_cargo_test.log`.
The six usability tests include3Point alias metadata and numerical3 after
phase transition. Standalone real native Qt Unicode renderer passes exact
question text, single L underline, unchanged draft, actual bold font and
visible history. Full-name click remains exercised by Box Diagonal/Cube and
existing common-control tests. No all-product-green or full Matrix parity claim.

Final actual native **18 suites / 1068 checks**, all fixture
assertions and host exits0 on one independently proved plugin/App/Gui pair.
This includes actual left/right icon focus, cancellation focus guard,0 origin,
clear Enter/errors,17 usability checks, full-name option clicks, numeric3 and
native geometry/Undo/Redo, plus common CMD–Builder, clipboard/IME and regressions.

| Native fixture | Passed checks |
|---|---:|
| `box_command_usability_smoke.FCMacro` | 17 |
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

Plugin `C:\Users\nguye\AppData\Local\Temp\om9-mnemonic-runtime-20261012\bin\OpenMatrix9Gui.pyd`, SHA-256 `b988fb1eef2bbd765cb27d99b9e441b340ee14c2fa49405cb132390d2382bec9`.
Loaded App `C:\Users\nguye\.codex\om9-native-tests\atomic-host-app-python200-20261012\FreeCADApp.dll`, SHA-256 `ba0cc05965df7bd7b2d2e3d3d7f0833423d30d5fcdc62b41d57729a873752bc2`.
Loaded Gui `H:\FreeCAD-src\build\camera-close-host-green2-20261011\FreeCADGui.dll`, SHA-256 `8819fc482151dfef1964d7d3071ebd67ab3137a5a9d660b99a708ce13c395677`.
Artifacts/runner/selection manifest under `C:\Users\nguye\AppData\Local\Temp\om9-box-usability-validation-20261012`; consolidated final proof
`mnemonic-final-summary.json`. Runtime/profile on C preserves old user windows.
Only standalone OpenMatrix9Gui/shared cache/SDK was built; no root FreeCAD
build, shared DLL replacement or complete runtime copy. Existing source-bound
App200 DLL/import/build receipt was reused with its captured190 source rows,
never refreshed to disguise stale code. Candidate hashes are not a full
compiler-dependency certificate. Standard unextended-host deployment not claimed.

This is the user's precise correction to the existing approved UI contract,
not a new solver or independent architecture. No additional review loop,
staging/commit/push or skill business-rule write. Prior review fixes remain.
Exact native source/module fingerprints, scoped formatting, source audit,
local documentation links and owned manifest/feature records are verified.

Native manual window reopened and confirmed ready/visible/focused: PID50076, document `OM9BoxUsabilityTest`, label `OM9 Box — Phím tắt đã sửa`, zero objects and exact first Box prompt, same tested plugin SHA `b988fb1eef2bbd765cb27d99b9e441b340ee14c2fa49405cb132390d2382bec9`. Start used an actual icon click without forcing CMD focus. Proof/screenshots: `manual03/manual-ready.json`, `manual-command.png`, `manual-window.png`. Old windows/models retained. Six owned manifest rows, Box JSON/YAML and 422 relative local links verified; scoped formatting/whitespace/source audit pass. The user explicitly confirmed closing the manual window after it opened; the launcher reported exit 0. This was a user closure, not an application failure. Task complete within the supported slice.

Later extension: [common root and suboption mnemonics across current commands](2026-10-12-common-command-mnemonics.md). Earlier counts/runtime remain historical evidence.
