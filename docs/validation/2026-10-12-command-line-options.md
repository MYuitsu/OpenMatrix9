# Command Line Options — native validation 2026-10-12

Supported slice: one reusable Rust engine, dynamic exact-feature-ID schema
registration, a shared native Qt widget binder and existing native Curve,
Rebuild, Sweep and Loft consumers. This is not full RCORE-04, all original
builders or an OffsetBezier-specific implementation.

## Architecture and acceptance

`rust/src/command_options.rs` owns typed accepted values, cursor/draft,
constraints, aliases/scopes and generation/revision/input-epoch checking.
`command_options_ffi.rs` owns opaque identities and copied descriptor/input
buffers. `command_option_schemas.rs` declares existing consumers. Generic
registration does not require a family branch; native Rebuild uses it.

`Gui/CommandOptions.{h,cpp}` supplies the common bridge/binder. CommandConsole
renders descriptor spans; Curve, Surface and Rebuild adapters validate actual
solver/native candidates before accepting them. Qt numeric controls preserve
raw text; their declared millimetre unit and explicit units use the same Rust
quantity parser. CMD bare lengths follow document input units. Numeric Enter
does not commit a Builder. Unresolved CMD or numeric widget drafts block commit.

[Register another command](../command-options-adapter.md).

## Build and execution

Reuse `H:/FreeCAD-src/build/relWithDebInfo` and the existing plugin cache
`H:/FreeCAD-src/build/openmatrix9-history-cage`. Build only `OpenMatrix9Gui`:

```powershell
rtk proxy pwsh -NoProfile -ExecutionPolicy Bypass -File H:/FreeCAD-src/build/build-om9-plugin.ps1 -SourceDirectory H:/FreeCAD-src/Mod/OpenMatrix9 -BuildDirectory H:/FreeCAD-src/build/openmatrix9-history-cage -RuntimeDirectory C:/Users/nguye/AppData/Local/Temp/om9-command-options-runtime-20261012 -RustTargetDirectory C:/Users/nguye/AppData/Local/Temp/om9-command-options-plugin-rust-20261012 -NativeAppManifestPath C:/Users/nguye/.codex/om9-native-tests/native-app-sdk141.json
```

No FreeCAD-root rebuild or full runtime copy was made. The plugin links with
`/INCREMENTAL:NO`; this resolved repeated LNK1140 while retaining debug symbols.
The failed feature PDB was preserved, following
[Microsoft's description of the PDB limit](https://learn.microsoft.com/en-us/cpp/error-messages/tool-errors/linker-tools-error-lnk1140?view=msvc-170).
Disk exhaustion on H caused native/Rust build failures. New Rust archives,
test profiles/reports and plugin output are on C; the cache location is optional
and defaults to the plugin checkout's original cache location.

Early runs use `build/run-om9-plugin.ps1` with distinct profiles on the standard
shared host. Concurrent unrelated App changes later required an extended App
ABI absent from that host. The final plugin uses the existing, source/hash-bound
`native-app-sdk141.json` DLL/import pair; no FreeCAD root was built for this task.
The stale App135 manifest was rejected before selecting the valid existing pair.

Final native runs reuse the existing `OM9NativeGuiTest.exe` from
`C:/Users/nguye/.codex/om9-native-tests/atomic-host-app-booking141-20261012` with
the same shared SDK home and dependencies. The task's preserved runner is
`C:/Users/nguye/AppData/Local/Temp/om9-command-options-validation-20261012/run-compatible141.ps1`.
It uses real Qt windows and isolated profiles. `shared-host-launch.json` records
host/plugin hashes and actual exit code; `native-loaded.json` verifies the loaded
module path/hash. `loaded-native-app.json` and `loaded-native-gui.json` prove the
actual DLL selections.

- App141 SHA-256: `2e6f57828496f35fbbb629b155ba345c95d27a6d57de63fa1418796dd58ff15e`.
- Test launcher SHA-256: `edacc467a1bbf247b7339496ea3e7a6fd81d73dd0dca1e33647aa3a6abd351da`.
- Existing Gui DLL SHA-256: `8819fc482151dfef1964d7d3071ebd67ab3137a5a9d660b99a708ce13c395677`.

The latest binary requires this matching App ABI. Standard shared-host deployment
is **not claimed**: its import failed before any fixture in `final04`. Matching
kernel/SDK integration belongs to the concurrent App work. No shared DLL was
replaced and no complete FreeCAD runtime was copied.
New fixtures additionally retain source fingerprints, Qt screenshots and FCStd
models. Test code sends real Qt clicks/keys; it does not proxy product input.

Artifacts are retained under
`C:/Users/nguye/AppData/Local/Temp/om9-command-options-validation-20261012`.
RED runs were observed before the corresponding fixes: initial schema missing,
Polyline choice absent, Loft choice absent, Rebuild raw numeric text clamped,
unresolved numeric draft committed after an unrelated toggle, and stale Curve
candidate accepted after an external point. Final run results are recorded below.

<!-- command-options-native-results:start -->
Final plugin SHA-256: `862585451bdfd8e848715e8a462ad50b9931b55c48189c85f8ec819fe2f092a2`.
Native execution: **13 passing suites / 692 checks**, plus Ellipse **60/61**
with the unchanged baseline Layer swatch failure. Every real host process
exited 0; fixture checks, not process exit alone, determine pass/fail.

| Native fixture | Passed checks | Result |
|---|---:|---|
| `builder_command_options_smoke.FCMacro` | 25/25 | PASS |
| `command_line_options_smoke.FCMacro` | 26/26 | PASS |
| `rebuild_command_options_smoke.FCMacro` | 15/15 | PASS |
| `circle_advanced_smoke.FCMacro` | 64/64 | PASS |
| `command_completion_smoke.FCMacro` | 29/29 | PASS |
| `curve_smoke.FCMacro` | 27/27 | PASS |
| `curve_spline_smoke.FCMacro` | 42/42 | PASS |
| `ellipse_smoke.FCMacro` | 60/61 | Baseline Layer swatch failure |
| `rectangle_smoke.FCMacro` | 37/37 | PASS |
| `rhino_core_native_rebuild_binding_smoke.FCMacro` | 47/47 | PASS |
| `surface_commands_smoke.FCMacro` | 97/97 | PASS |
| `surface_history_smoke.FCMacro` | 65/65 | PASS |
| `surface_options_smoke.FCMacro` | 181/181 | PASS |
| `matrix_grid_command_smoke.FCMacro` | 37/37 | PASS |

The three new fixtures pass 26 + 25 + 15 checks. They exercise actual controls,
accepted parameters, native Part geometry, Enter/Esc, no premature document
objects, transaction Undo/Redo and Loft FCStd round-trip. Loft additionally
proves that `SectionMode=Rebuild` then `History=Yes` retains pending PointCount
and the accepted None mode; only the supplied child number accepts Rebuild.

Selected final reports are `review-green01`, with corrected Loft and Matrix
grid fixture assertions in `review-green02`. `final-native-summary.json`
consolidates checks, actual process exits and independently loaded plugin/App/Gui
proof. An earlier Loft assertion used a boolean pending property as a string;
it is not counted as native product RED evidence. Matrix grid's three older
exact prompt/order assertions were updated for schema-rendered domains/options;
all existing grid, geometry, clipboard, AltGr and IME assertions remain and pass.

Ellipse's failed assertion is `Layer swatch recolors ellipse and both focus
markers`, also reproduced on the pre-feature
`initial-parameters-runtime109-20261012` in `ellipse-baseline01`. No Layer changes
were made to conceal it. The earlier `compatible01` run used App125 and an older
plugin; its wrapper metadata errors are not part of the final passing evidence.

Fresh full Rust: **1,019 passed / 1 preexisting failed, 131 targets**, log
`C:/Users/nguye/AppData/Local/rtk/tee/1791768408_cargo_test.log`.
Targeted reducer/ABI: **30/30 passed**. Source audit: 2,830 files, 511 icon
bindings, 510 SVGs, no errors. Targeted Clippy: no errors, 111 existing-file
warnings. Python tooling: 50 tests run, OK, one skipped. Formatting and scoped
whitespace checks pass.
<!-- command-options-native-results:end -->

## Rust and source checks

The targeted schema/ABI suites pass 30 tests after the final review fixes. RED→GREEN covers
toggle/explicit idempotence, nested selection/Esc, raw finite/domain/unit errors,
aliases/collision, private scopes, draft conflicts, generic registration, malformed
candidates and Curve generation/input-revision guards. All new Rust files pass
rustfmt; targeted Clippy has no errors and only existing-file warnings.

The full suite is run with `--no-fail-fast`. One preexisting failure remains:
`support_snapshot::evaluation_tests::owned_curve_jets_match_native_value_and_three_derivatives_for_nineteen_families`.
For OffsetBezier at parameter 0.1 the third derivative is
`[-13.535143916888291,-0.5573614450242469,-1.864017480263903]` versus the native oracle
`[-353.1490199479945,268.1061862322634,-1.8640174802639011]`.
The same failure was observed before product implementation. It is not a Command
Options failure and is not hidden by a targeted green run. Fresh full-suite totals
and audit results are added to the results block.

## Limits and rulings

Private CMD-only/Builder-only scopes are reducer/ABI proven; the first native
widgets are the currently shared controls. Original Gem solvers, arbitrary
viewport handles/sliders, Reset workflows, `.mss` exchange, transparent commands
and full repeat/macro semantics remain outside this supported slice.
Builder source identity controls keep their existing native guards/bindings.
Circle/Ellipse size inputs retain existing geometric construction behavior.

Invalid proposals restore accepted controls. Two older negative fixtures were
adapted to this contract: Rebuild submits the invalid dependent count last;
Loft recovers with an explicit `Style=Normal` after rejection has already restored
Normal. Original native geometry and no-mutation assertions remain.

A whole-file edit hit disk exhaustion and truncated `curve_ffi.rs`. The complete
pre-task file was restored from saved tool output and matched the independent
`curve-focus-producers57-20261012.json` SHA-256
`4e018b2db9011e1186964e86a38a8e148f61244d0eb003c3a3b7287d98ba92c0`.
Only the task's two additions were replayed, followed by targeted/full tests.
Recovery artifacts are preserved. Further document writes use verified temporary
files and atomic replacement.

The shared checkout contains unrelated in-progress changes. Task changes are
kept reviewable without whole-file staging, committing unrelated work, merging
or pushing. Preserve validation artifacts and execution ledger rather than
deleting them. The final review and any fixes are recorded with the fresh results.

## Final independent review and fix pass

One fresh-context reviewer inspected the reusable engine, ABI and real native
consumers. No Critical or Minor findings; five Important findings were retained
after technical verification. A single fix pass added seven regressions, all
observed RED before implementation and GREEN afterwards:

| Finding | Fix and regression |
|---|---|
| Candidate survives Esc/navigation/draft changes | Separate input epoch; retire ABI token on new input. `input_phase_changes_invalidate_unaccepted_candidates`, `ffi_escape_and_draft_replacement_retire_candidate_tokens`. |
| Unrelated CMD assignment accepts unfinished parent | Only active-child completion consumes staged ancestors; unrelated proposals preserve the branch. `unrelated_cmd_proposals_preserve_unfinished_branch` and native Loft History/PointCount. |
| Bare choice collides with global key | Active phase resolves its choice first. `pending_choice_names_take_precedence_over_global_options`. |
| Case-only duplicate canonical key | Reject ASCII case-insensitive identity collisions during registration/seal. `case_only_canonical_key_collisions_are_rejected`, `ffi_registration_rejects_case_only_duplicate_identity`. |
| Builder-only child entered through CMD | Check child scope/gate before traversal, disable its CMD choice and skip it in keyboard selection. `builder_only_child_cannot_be_traversed_or_selected_in_cmd`. |

No second review was dispatched. Fresh full Rust, the affected native suites and
existing regressions were run after that fix pass. There are no deferred Minor
findings. Exhaustive OS-level IME/focus delivery, all original Reset workflows
and unimplemented Gem consumers are outside the evidence; native synthetic Qt
input coverage is identified explicitly rather than claimed exhaustive.

The review's five items declined for judgment were explicitly ruled on: retain
both independently proved baselines (cost: underlying geometry/layer defects
remain), do not claim standard-host deployment (cost: matching App ABI required),
complete loaded metadata normalization (cost if wrong: invalid native evidence),
limit IME/focus claims to executed native Qt tests (cost: untested OS paths), and
retain gates/partial scope for Gem/Reset/profile/handles and unrelated WIP
(cost: those consumers require their own later adapters/solvers).

Scoped whitespace checking enables `cr-at-eol` for the preserved Windows files;
their existing CRLF endings are not normalized across unrelated registry/source
content. New files, formatting, task-owned manifest hashes and document links
were also checked independently.

## Execution rulings retained

- Keep the approved existing checkout/branch and unrelated WIP; no staging,
  commit, merge or push. Coupled current native foundations require this checkout;
  source fingerprints guard concurrent changes. The cost is shared-checkout risk.
- Use one generic exact-ID schema registration API and declared constraints,
  with built-in schemas only as consumer conveniences. The extra ABI validation
  enables future commands without command-name branches in the reducer.
- Preserve native Circle/Ellipse geometric size/action semantics; changing those
  to parameter-only input would break construction. Their solvers remain native.
- Restore accepted controls after rejected input. Rebuild/Loft legacy negative
  fixtures were adjusted to submit the invalid dependent value last or recover
  explicitly; Matrix grid checks schema-rendered prompts. Geometry assertions
  and invalid-input no-mutation assertions are retained.
- Keep SDK/cache on H; place optional Rust cache, plugin runtime and test artifacts
  on C because H filled. Preserve failed symbols and recovery evidence; no
  recursive cleanup through junctions. The cost is an optional cache flag.
- Disable incremental linking only for the plugin while retaining debug symbols;
  the cost is full plugin links. Never rebuild the FreeCAD root for an OM9 change.
- Accept only independently proved loaded plugin path/hash when a legacy macro
  lacks its own module metadata; preserve profiles, results, screenshots and models.
- Reuse the existing matching App141/native launcher after the concurrent App ABI
  change; standard-host deployment remains unverified. No shared SDK replacement.
- Preserve this plan's ignored execution ledger and validation workspace under
  `.superpowers/sdd/2026-10-12-command-line-options`, following report-preservation
  policy. The cost is retained disk data. Disk-write recovery is documented above.
