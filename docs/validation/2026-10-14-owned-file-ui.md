# Owned File UI and native cancellation — 2026-10-14

[Source-bound receipt](2026-10-14-owned-file-ui.json) extends the historical
[controller service checkpoint](2026-10-14-owned-file-service.md) toward the full
[File lifecycle plan](../superpowers/plans/2026-10-14-owned-file-lifecycle.md).
The original RCORE-00 plus screenshot goal remains active. No complete File,
Layers, M9IMG, M01–M12,607-feature or148-capability gate is promoted.

Native `.om9` New/Open/Save/Save As now share controller effects across the File
menu, CMD, F6 and Standard New/Open/Save icons. Ctrl+N/O/S and Ctrl+Shift+S use
the same operations. Unsupported `.3dm` Import/Export stays visible and disabled.
Typed File literals are consumed as commands; toolbar/menu/shortcuts preserve
an unrelated numeric Line draft. Save/Discard/Cancel and owned overwrite prompts
bind buttons to the current authority/response serial. Cancel/error keeps the
old document/scene/draft and executable Redo; accepted replacement clears old
command/transcript editor state, selection/Undo/popups and refreshes title/path.
`--ui path` now constructs the same guarded from-file controller and exposes
missing-module restoration diagnostics instead of losing active path/diagnostics.
That startup constructor is tested; actual native `--ui path` launch remains
unmeasured in this checkpoint.

**475 workspace tests / 24 ignored SDK tests**, 96 focused
checks are a workspace subset.16 actual egui File widget checks include menu,
toolbar, keyboard shortcuts, Save/Discard/Cancel/Replace, scripted owned path
delivery, failed suffix retry without automatic picker looping, stale replies,
old pointer releases, preserved draft/Redo, missing-module startup and30 prompt
cycles with stable area/scroll owners and retained CMD editor state. Scripted
path delivery exercises real archive I/O and controller ownership; it does not
claim a successful selection inside the Windows chooser.

Three input defects were witnessed RED→GREEN: IME Enter could activate Save;
holding the CMD opening Enter could repeat into Save/New on a known active path;
CMD composition lost its guard when focus moved into a File prompt. File IME
now follows OS events across focus changes, Commit/Enter waits for a later key,
and repeated Enter/Space cannot affirm. An independent read-only reviewer
identified the last two cases and reviewed the fixes; no remaining demonstrated
Important/Critical finding was reported. The reviewer did not run tests; passing
test logs belong to the main verifier.

The release native-window replay verifies8 File checks across toolbar, File
menu, F6 and CMD. It actually opens3 Windows common dialogs (Save, Open, Save As)
through the synchronous adapter. An explicit audit probe enumerates only the
verified OM9 root UI thread, requires same PID/thread, direct owner, `#32770`
class and exact application-supplied title, then posts `WM_CLOSE`. Each dialog
is observed, returns Cancel, is destroyed within5 seconds and its probe is
joined. No desktop input or other application is controlled. Loaded modules,
root window and zero child processes are observed; source/binary/PNG hashes
are bound in the receipt. This is native cancellation evidence, not OS path
selection, OS save/open success or broad OS/DPI/IME acceptance.

Images use per-replay checkpoint directories. Earlier generic PNG cache paths
were mutable; earlier receipts retain historical hashes, while this receipt is
the current source/binary/image authority.

The same release binary also passes10 Layers modal,13 earlier Layers and6
viewport-menu regressions. Seven existing bounded cold CLI children exit with
zero active processes. Format, warning-strict all-target Clippy, full workspace
and release checks run sequentially with Cargo `-j2`. No fresh SDK geometry job
runs; historical openNURBS/OCCT evidence retains its own source checkpoints.

One earlier Layers modal replay missed its initial Advanced click and then sent
editing events into CMD because the audit advanced without confirming modal
readiness. The [failed trace](2026-10-14-owned-file-ui-native-attempts.json) is
retained. An isolated same-binary retry and the final full verifier pass10 checks;
no production fix was inferred. Independent review confirmed the cascade but
could not establish the first missed-click cause. Native timestamps, raw OS
focus/events, hit-test responses and stage readiness still need diagnosis;
full Layers/native input and performance acceptance remain open.

Windows native exception: pinned MIT `rfd 0.17.2`, Cargo checksum
`20dafead71c16a34e1ff357ddefc8afc11e7d51d6d2b9fbd07eaa48e3e540220`, default features disabled, no Linux dialog backend selected.
Its closure uses existing log/raw-window-handle/windows-sys dependencies;
raw-window-handle0.6.2 is also pinned directly for the audit parent handle.
The unchanged MIT notice is [preserved](../../Resources/licenses/rfd-0.17.2.txt),
SHA256 `65fc947f4bac882a2cd104ef73c246b1b037059df9f2b3a58b7f77f0b9e56071`.
Safe OM9 Rust owns document/transaction authority and owned paths; rfd invokes
unsafe Win32 COM. Narrow audit Win32 handle code lives in om9-native-owner;
application/core/archive model logic remains safe Rust. No third-party/native
memory-safety certificate is implied. See [rfd API](https://docs.rs/rfd/0.17.2/rfd/struct.FileDialog.html).

Known adapter limitations remain explicit: rfd collapses COM/build/Show/result
HRESULT errors into `None`, so the current wrapper cannot distinguish those
failures from Cancel (catch_unwind only reports panic). Windows Save defaults
to overwrite confirmation; OM9 retains its own fresh owned confirmation because
a chooser result must not blindly approve a destination created after selection.
An existing destination can therefore prompt twice. This preserves confirmation
but remains a UX/API continuation. See [Win32 dialog options](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/ne-shobjidl_core-_fileopendialogoptions).

Controller aggregate admission covers retained model/Undo/scene/workspace/file
owners. MatrixApp Transcript/recall, TextEdit/egui/GPU caches and native dialog
memory are not covered by that controller peak claim. Full input-owner/worker
admission, cancellable/background I/O, actionable native backend failure,
single overwrite UX, actual OS successful select/save/open and native loaded-file
startup remain required. So do versioned view/CPlane/camera/material/annotation
persistence, `.3dm` interoperability/losses, dirty window close and every complete
original File/RCORE/Matrix9 requirement. Future full batch count remains unknown.
