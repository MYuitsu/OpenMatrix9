> Historical WIP evidence, superseded by [docs/validation/2026-10-07-3dm-hatch-loop-ui.md](2026-10-07-3dm-hatch-loop-ui.md). Deleted-item driver root cause, corrected model-based UI104 and full GUI1344/native20 are recorded there.

# OM9-FILE-012 — ordinary Hatch loop UI, in progress

This continues step4 of the approved full Hatch plan. It is not a completed UI
or class-wide compatibility claim. The full128 registered classes,16 component
categories and6 document categories remain in progress.

## Source work

`Gui/ThreeDmHatchDialog.{h,cpp}` adds a detached C++/Qt field tree for existing
five-class native loops. Seventeen-digit display keeps untouched JSON numbers
exact. Numeric fields, boundary roles and explicit native circle insertion and
loop removal preserve surviving source indices and curve identities. Native
identities, dimension and class metadata are not numeric editing inputs. The
table rejects more than65536 fields or depth64 explicitly.

`ThreeDmHatch.edit_loops_ui` verifies the active document and source baseline,
stages proposed loops through the existing native topology/child-data checks,
and commits one document transaction only after preflight. Double-click and the
context menu are wired to it. The experimental bridge now returns detached data;
host preflight runs after the native dialog call returns. Native errors reopen
the editor with the proposed values, before any document mutation.

UI support for arbitrary array/segment insertion, rationality/class changes,
CurveOnSurface/PolyEdge/plugin/reference adapters, full pattern editing/rendering
and actual Rhino5 application acceptance is still pending.

## Evidence and unresolved failure

- RED: `three_dm_hatch_loop_ui_smoke.FCMacro` originally failed because the typed
  native editor entrypoint was absent. Artifact46416f3226ea49f6a5a85d66b761d9b9.
- Ordinary non-ASAN isolated module builds finish with exit0, including the
  detached bridge. Existing compiler/SDK sources are not replaced.
- One initial UI run recorded85 passing assertions across mm/cm, including exact
  native reread, source strings, cancel, preflight errors, inner circle addition/
  removal and Undo/Redo. Its process crashed with0xc0000409 at termination. This
  is a failed run, not85/85 acceptance. Artifact6818464e5f3f4fda9c9573738a06a47c.
- Other full UI runs crash with0xc0000374 or terminate with0xc0000409, or remain
  waiting without a final report. Last full run62b83bac96c84a4cabdaef4ce3ff186f
  reached20 checks, stopping during the edited Polyline boundary preflight.
- Cancel-only10 cases, pure tree/circle10 cases, callback-error10 cases,
  native unchanged host40 cases, and native negative-radius10 cases each had
  individual process0 before the later bridge redesign. These isolate behavior;
  they do not establish the full UI contract or current bridge compatibility.
- Non-UI1.125 coordinate/radius edits and repeated native preflight passed4 cases,
  process0. Cancel-then-edit and detached-editor-then-edit probes also passed4
  cases each. Thus the supplied geometry alone does not explain the failure.
- Removing Undo/Redo did not eliminate failure. Moving preflight outside the Qt
  signal also did not eliminate it. No root cause is proven from those probes.
- Reject-NaN-then-edit and floating-environment probes waited without a final
  report. Numeric parsing/floating environment is a hypothesis, not a finding.
- Windows produced crash dumps. `build/read-hatch-ui-dump.py` saved approximate
  module/symbol stack scans; unwinding yielded only the exception frame, so those
  scans must not be represented as a validated call stack or proven root cause.
- MSVC ASAN compilation was available, but diagnostic linking failed: mixed
  annotate_string/vector settings and a missing ASAN runtime thunk library.
  The diagnostic source option was removed and a non-ASAN build restored, exit0.
- Processes78472,28320 and49068 were confirmed against exact private smoke macro
  command lines before intentional cleanup after failure evidence was collected.
  Observation timeouts were not treated as successful or terminal process exits.

The1239/1239 GUI and20/20 native checkpoint from the previous legacy-loop package
is historical evidence for that package, not verification of this changed UI
source/runtime. No primary/public integration or push was performed.

## Next exact work

Instrument numeric rejection before/after parsing and modal transitions so a
missed test-driver callback cannot hide its exception. Compare native preflight
with and without prior nonfinite input; prove or disprove floating-environment
changes with captured values. Preserve original finite inputs and document data.
Fix the proven cause using the full failing UI regression, then rerun all native
and GUI suites before any ordinary-editor completion claim. Finish all remaining
Hatch and full openNURBS work packages under the original specification.

Fresh regression on the changed non-ASAN module: existing typed-loop API132/132,
process0, artifact5a6f9941539e4a3b8c31ff66f5340073. This covers native export,
Undo/Redo, independent copies/blocks and FCStd; it does not verify the new UI.
