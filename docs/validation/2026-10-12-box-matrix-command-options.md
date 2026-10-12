# Matrix-style common Command Options and Box — 2026-10-12

User approved Native execution in the current chat and requested a reusable
engine, phase prompts/options, click, D mnemonic and CMD history. The attached
Matrix9 screenshot defines the initial Box question/order and styling. This
extends the existing [common reducer and CMD–Builder binding](2026-10-12-command-line-options.md).
RCORE-04, Builder Framework and Box remain partially implemented outside the
supported slice below.

## Result and boundary

Box begins with **First corner of base** `( Diagonal 3Point Vertical Center ): `.
The full option names are underlined live links. Rust derives the next question,
visible options and numeric defaults from the actual Box session. D/d selects
Diagonal; after its first point, `Other corner of box ( Cube ): ` exposes Cube.
Width defaults to the accepted Length and Height to the accepted Width. Clicks
and keyboard options feed the same native solver. A rejected number/point keeps
the raw input and produces no document geometry; Esc clears the command/draft.
Existing mode/frame, signed extents, tolerance, preview and transaction remain.

The generic engine now supports a question prefix, a UTF-16 question length,
hidden presentation descriptors, revision-bound schema refresh and bounded
command invocation formatting. Other commands provide their own phase adapter;
no geometry command names or Box mode rules are added to the reducer. Existing
toggles, nested choices, numeric input and CMD–Builder binding remain available.
Box's numeric dimensions still use its native ordered construction input; there
is no newly invented Box Builder dialog.

The green CMD transcript records actual `Command: _Box` invocations, accepted
typed/click input with the preceding question, and accepted mouse/F4 phase
questions. Up/Down recalls accepted input. Ortho/Osnap consume their submitted
drafts. Consecutive menu invocations are retained, including cancelled starts.
The Matrix-private `_gvCommand` wrapper is not executed by OM9 and is not
fabricated in the transcript. Legacy typed Cube remains available at the root,
but is only displayed at the Diagonal step. History means the CMD transcript;
this does not add associative geometry History or full macro/repeat semantics.

First-step wording/order/style follows the supplied screenshot. Later wording
uses OM9's accepted native construction phases, consistent with the documented
[Rhino 5 Box workflow](https://docs.mcneel.com/rhino/5/help/en-us/commands/box.htm).
Whole Matrix9 visual/solver parity is not claimed. Native mouse/F4 history records
the accepted phase question, without inventing typed coordinates.

## Ownership and interface

Rust owns question/presentation, aliases, gates, actual phase defaults, accepted
state, generation/revision/input epoch and invocation validation. Qt/FreeCAD/OCCT
are necessary for document references, real controls, font/layout/hit testing,
Part geometry, preview and transactions. ABI strings use owned copied buffers;
existing public event/item structs remain 16 bytes. A restarted Box session
invalidates the preceding option bridge. Unicode spans use QTextCursor absolute
UTF-16 endpoints rather than grapheme movement.

Files: `rust/src/command_options.rs`, `command_option_schemas.rs`,
`command_options_ffi.rs`, phase adapter in `solid.rs`, `Gui/CommandOptions.*`,
`CommandConsole.*`, `CurveController.*`, `SolidController.*`, `RustBridge.h`.
See [adapter API](../command-options-adapter.md).

## Fresh verification

Targeted Rust: **57/57** across `command_options`, `command_options_ffi`,
`command_options_matrix`, `solid_session`, `solid_options`, `solid_commands`.
Full Rust: **1,023 passed / 1 existing OffsetBezier derivative failure**, 132
targets, log `C:/Users/nguye/AppData/Local/rtk/tee/1791772361_cargo_test.log`.
That failure predates this task; no all-product-green claim. Targeted Clippy has
zero errors and 111 existing warnings, including existing common-engine style
warnings; log `1791772409_cargo-json-issues.log`. A new Solid style warning was
fixed. No wholesale formatting of the unrelated large Solid source was applied.

Final native: **17 suites / 1051 checks**, all fixture assertions
pass and all actual host exits are 0. The same plugin/App/Gui DLLs are proved
loaded independently. Exit 0 alone is not treated as fixture success.

| Native fixture | Checks passed |
|---|---:|
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

Box tests cover D/d and click paths, phase-specific Cube, actual volumes27/24/8,
Enter defaults, error-draft preservation, click guarding, cancellation,
Undo/Redo and invocation counts. Separate history tests cover Ortho/Osnap,
Up/Down, F4, repeated invocations and no premature geometry. Existing Solid
fixtures cover construction modes and geometry. Common fixtures cover actual
toggle/child/numeric/Builder controls; Matrix grid retains IME, AltGr, clipboard
and native grid assertions. The Solid helper now replaces a retained invalid
draft before its next input, matching product recovery; all geometry assertions
were preserved. The grid underline assertion checks the complete link text.

Standalone native Qt component `Gui/tests/CommandConsoleUnicodeTest.cpp`
compiles the actual CommandConsole/bridge/Rust archive. Unicode question/link
spans, unchanged raw draft, actual document bold font and rendered history text
pass. Script/results are under `renderer` and `build-renderer-probe.ps1` in the
artifact directory. Full OS-level IME delivery is not claimed by synthetic Qt
events. Source audit: 2,842 files, 511 icon bindings, 510 SVGs, no errors (before
this final documentation addition).

## Build and native provenance

Only `OpenMatrix9Gui` was built via `H:/FreeCAD-src/build/build-om9-plugin.ps1`,
using the single shared SDK `H:/FreeCAD-src/build/relWithDebInfo` and existing
plugin cache `H:/FreeCAD-src/build/openmatrix9-history-cage`. Rust archive,
plugin runtime, native reports/profiles and snapshots are on C because H has
little free space. No FreeCAD root build or full runtime copy was made.

Concurrent native App source changes made prior App154 validation stale. It was
rejected, then the already built source/hash-bound App162 manifest was validated
and reused. No manifest fingerprint was fabricated and no shared DLL replaced.
Final runtime needs that matching App ABI; standard unextended-host deployment
is not claimed. The manual instance uses the same tested pair and SDK home.

- Plugin: `C:/Users/nguye/AppData/Local/Temp/om9-box-matrix-options-runtime-20261012/bin/OpenMatrix9Gui.pyd`; SHA-256 `ce356b7a6522533f1ce3ba1835f824b5b0ef3956ae08a44f324583403cd6ed59`.
- Loaded App: `C:/Users/nguye/.codex/om9-native-tests/atomic-host-app-redo162-20261012/FreeCADApp.dll`; SHA-256 `321608c480c06c54e92988281dd531237a320b560e8f22ab90cd845e41eec91c`.
- Loaded Gui: `H:/FreeCAD-src/build/camera-close-host-green2-20261011/FreeCADGui.dll`; SHA-256 `8819fc482151dfef1964d7d3071ebd67ab3137a5a9d660b99a708ce13c395677`.
- Matching manifest: `C:/Users/nguye/.codex/om9-native-tests/native-app-sdk162.json`.
- Runner/artifacts: `C:/Users/nguye/AppData/Local/Temp/om9-box-matrix-options-validation-20261012/run-compatible162.ps1`.

`final03` contains the fresh17-suite batch; `final04/box_command_options_smoke`
corrects a fixture cursor-position assumption to use the actual input boundary.
Its previous failed bold assertion selected the wrong character, not a product
font failure. `final-native-summary.json` consolidates selected fixture reports,
loaded module proofs, real exits and current selected-source fingerprints.
Screenshots `matrix-box-prompt.png` / `matrix-box-native.png` are preserved.
Earlier native runs are retained rather than overwritten.

## Review, RED evidence and rulings

One fresh-context read-only review was used, followed by one repair pass and
fresh verification. No second review was dispatched. All five findings were
accepted after source/runtime verification and repaired:

| Finding | Resolution |
|---|---|
| Ortho/Osnap left submitted draft active | Consume native accepted input once; history regression. |
| Accepted Box input skipped recall bookkeeping | Share rememberInput with the ordinary console path; Up/Down regression. |
| Mouse/F4 lost the preceding question | Record only native accepted phase; F4 regression. |
| Dedup dropped consecutive real menu invocations | Dedicated invocation logging bypasses message dedup. |
| UTF-16 spans used grapheme stepping | Absolute endpoints; Unicode native component RED→GREEN. |

The last two were labelled Minor by review; they were repaired inline because
the first violates the explicit CMD-history request and the second is a small
common-renderer correctness fix. No findings remain deferred. The reviewer
declined inherited solver/tolerance/layer/publication rules, legacy Cube/private
wrapper policy, exact mouse-coordinate echo, unrelated units/concurrency and
independent native execution. Scope reasoning and tests above resolve the
applicable items; inherited foundations retain their prior evidence/limits.

REDs retained: old plugin failed the Matrix initial prompt (`red01`); Cube alias
C collided with hidden Center; native history failed accepted-input tests
(`history-red01`); Unicode selection included extra text; actual bold document
font failed before font application; first native history text was offscreen
(`history-visual-red01/02`). The renderer now applies question font to the live
document and follows the initial blank transcript without forcing scroll when
the user is browsing meaningful earlier history. Native visible-text pixels
and real font assertions pass after the fixes.

Rulings: retain approved shared checkout/branch/WIP without staging/commit;
source fingerprints reduce shared-edit risk. Preserve old manual window and
models; open the new runtime/profile separately (cost: two windows). Reuse SDK,
cache and valid existing App pair (cost: matching ABI deployment requirement).
Retain legacy Cube typed compatibility and only actual invocation history
(documented visual difference). Preserve ledger, failed runs and reports (cost:
disk space). No unrelated Layer/OffsetBezier fix or broad feature status upgrade.

## Manual window reopened

The tested plugin/App pair was reopened in a separate visible maximized window,
PID24408, document `OM9BoxMatrixTest` labelled `OM9 Box — Matrix CMD`, zero
objects, active Box first-step prompt. `manual01/manual-ready.json` confirms
visible/ready, exact prompt/feature and the same plugin hash as all final tests.
`manual-command.png` and `manual-window.png` preserve the displayed result.
The old interactive window and its models were retained. The helper uses
`WindowStyle Normal` only for this explicitly requested interactive window.
Scoped whitespace and Rustfmt edition2024 checks pass; JSON/YAML Box records,
six owned manifest rows and relative documentation links are synchronized.

## Later corrective evidence

User testing exposed actual-icon focus and bare0 cases omitted by the earlier
fixtures. This report preserves the original run, not evidence of those cases.
The [Box usability correction](2026-10-12-box-command-usability.md) supersedes
its focus/origin/Width/Height/error presentation behavior and records fresh native
verification on a separate runtime. Original geometry/ABI evidence is retained.

Later user correction: only a single declared shortcut character is underlined; full-name click remains. See [current mnemonic evidence](2026-10-12-command-mnemonics.md). Earlier runtime hashes/results above are preserved as historical evidence.
