# Native File selection and startup — 2026-10-14

[Current source-bound receipt](2026-10-14-owned-file-selection.json) extends
[owned window close](2026-10-14-owned-file-close.md). The prior turn made
verified progress; all684 recorded source hashes matched at this turn's start.
The complete foundation/six-screenshot goal remains active and unpromoted.

The selected authored adapter retains normal production Open/Save behavior.
An explicit own-app audit variant prefills an authored fixture filename using
`IFileDialog::SetFileName`. It still calls native Show and GetResult; the hint
is never returned as a fabricated selection. A joined probe finds only a modal
on the verified root's PID/UI thread, checks direct owner, class #32770/exact
title and the enabled same-thread IDOK child, then posts fixed WM_COMMAND1.
No desktop input or foreign-app control is used. Native handle validity and
NUL-terminated COM allocation remain explicit unsafe contracts, not a memory
safety certificate. windows-sys0.61.2 selects the Input/KeyboardAndMouse
feature only for IsWindowEnabled query; no desktop input API is called.
See Microsoft [SetFileName](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifiledialog-setfilename)
and [GetResult](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifiledialog-getresult).

The actual native replay authors a Point, invokes Save through CMD and obtains
`selected.om9` through the real Windows dialog. Its archive equals the owned
document. The audit makes an exact owned copy as existing `alternate.om9`, then
authors another Point and invokes SaveAs. Native dialog acceptance returns the
alternate path but leaves both files and the active path unchanged until the
owned OM9 Replace button is clicked. SHA256 captures require both copies equal
and non-null at seeding, both unchanged before Replace, original unchanged
afterward/finally, and replaced alternate bytes different from the seed.
Replace adopts alternate path after archive acceptance without adding geometry
Undo. New then clears runtime/Undo/path through the actual toolbar; Open obtains
alternate from the real Windows chooser and restores two objects/current path.
Save/SaveAs/Open all consume captured operation tokens normally.

Open deliberately renews owner_session, epoch and session_generation. The
comparison requires exactly that renewal and all other fields equal for this
authored recipe-free fixture; it does not infer general graph/replay fidelity.
The initial failed audit compared the entire Document including renewed runtime
fields. Its three accepted OS selections/failure are retained in
[attempts](2026-10-14-owned-file-selection-attempts.json), never counted as a
pass. A unit check verifies that normalization still rejects name/context data
changes; another ensures absent owned overwrite never injects acceptance.

A separate native process runs actual `--ui path`, using the normal guarded
Controller::from_file constructor. An explicit audit-directory environment
variable only enables observation/screenshot/teardown; no startup document or
input is substituted. Active path, filename/title, full supported persistent
report, empty diagnostics/clean state and actual four-view rendering are
checked against a bounded cold CLI inspection of the same unchanged archive.
This measures the clean-startup branch. Native missing-module warnings,
unavailable resources and broad DPI/IME cases remain required continuations.

Measured **498 workspace tests / 24 ignored SDK
tests**, **119 focused checks are a workspace subset**, 86 suites. Four
new unit tests join the existing File20 controller/File21 widgets and broader
regressions. Format, warning-strict all-target Clippy, workspace and release
build pass sequentially with Cargo `-j2`. Seven Job-owned cold children exit
with zero active processes; two additional bounded `--inspect` children return
success and are reaped by the UI harness, without claiming Job accounting for
them. Native success5checks/3 actual selected paths and startup3checks are
separate from overlapping File-close11/File8 and Layers10+13/viewport6 replay
checks. Immutable per-replay PNGs and original/replaced authored archives bind
to source/binary/fixture SHA256. No new geometry SDK jobs; v1 unchanged.

Independent read-only review found no Important/Critical product/native-guard
defects. Reviewer found a P2 proof gap where only object count demonstrated
file preservation; complete file fingerprints now close that gap. Reviewer
ran no build/test/GUI. Prior reports and failed attempts retain their scope.

Required next work: versioned complete view/CPlane/camera/material/annotation
persistence and migrations, .3dm/file codec capability/loss integration;
native missing-replay startup and successful-save/Discard-button shutdown;
asynchronous/cancellable I/O, hung-modal and global UI/native/worker budgets,
complete owner disposal. Full130core/M01–M12/File/Layers/M9IMG/607-feature/
148-capability gates remain open. Total future measurement batch count remains
unknown; all unclosed packages are retained.
