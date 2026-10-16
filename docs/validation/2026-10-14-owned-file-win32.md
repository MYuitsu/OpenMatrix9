# Selected Win32 File adapter and replay guards — 2026-10-14

[Current source-bound receipt](2026-10-14-owned-file-win32.json) extends the
[File UI checkpoint](2026-10-14-owned-file-ui.md) and the complete
[File lifecycle plan](../superpowers/plans/2026-10-14-owned-file-lifecycle.md).
Previous turn was verified progress; all683 recorded source hashes matched at
this turn's start. The full RCORE-00/six-screenshot objective remains active;
all full File/Layers/M9IMG/M01–M12/607-feature/148-capability gates remain open.

The authored safe caller now uses selected Win32 common-dialog APIs through
the existing pinned windows0.62.2 binding. rfd is removed from Cargo dependency
closure; its prior receipt/provenance/notice are retained as history. Windows
COM unsafe code lives in om9-native-owner/file_dialog.rs. The adapter validates
same process/current UI-thread HWND before COM, accepts bounded borrowed paths,
returns only an owned optional PathBuf or typed stage/HRESULT error, and never
receives document/transaction/native-shape authority. Setters reject NUL or
oversized UTF-16; returned paths preserve Windows Unicode and free native task
memory through RAII. Native NUL-terminated allocation validity is an explicit
FFI contract, not a native memory-safety certificate.

A thread-bound !Send/!Sync guard balances successful COM initialization,
including S_FALSE. Failed initialization constructs no guard. Path allocation,
shell item and dialog interfaces drop before CoUninitialize. An actual fresh
MTA thread test verifies STA conflict remains an error and leaves the caller's
MTA initialization intact. Only Show's HRESULT_FROM_WIN32(ERROR_CANCELLED)
becomes Cancel; other initialization/setup/Show/result errors are visible for
explicit UI retry. A real invalid-owner adapter error test preserves pending
File token, Line/numeric draft, model/Undo and disables automatic picker loops.
See [Show](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-imodalwindow-show)
and [COM balancing](https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-couninitialize).

Save options remove FOS_OVERWRITEPROMPT and retain filesystem/path restrictions;
OM9's existing fresh owned Replace confirmation remains the output authority.
This resolves the selected API's duplicate-prompt policy without treating a
returned chooser path as overwrite approval. Unit options and real dialog
cancellation are measured; successful OS selection of an existing destination
and the complete single-confirmation save interaction still require evidence.
See [Win32 options](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/ne-shobjidl_core-_fileopendialogoptions).

The retained historical Layers replay failure is not reclassified as a product
fix. Its original first missed-click cause cannot be established from the old
trace. New4 audit tests reproduce a long Advanced gesture exceeding ordinary
egui click duration, prevent absent-modal Ctrl+A/text from reaching CMD, prove
ordinary accepted clicks can be instrumented without recursive context locking,
and bound startup focus waiting without fabricating raw.focused. Replay now
stops on missing Advanced acceptance, expected modal/editor focus or native
focus before editing events. Trace records native time/focus/event kinds,
Advanced rect/enabled/sizing/hover/down/click and top layer. Text event payloads
are not dumped. Native startup briefly lacked real focus in a measured run;
the audit requests focus for its own viewport once and waits at most3 seconds.
It does not alter production click thresholds or inject desktop input/control
another app. A new diagnostic clicked() nested-input-lock bug was caught and
fixed; the ordinary-click test covers the actual successful path.

Measured: **483 workspace tests / 24 ignored SDK
tests**, **104 focused checks are a workspace subset**.8 new unit checks
join the existing16 File widgets,17 File service tests and broader regressions.
Format, all-target warning-strict Clippy, full workspace and release build pass
sequentially with Cargo `-j2`. Seven bounded cold OM9 children exit with no active
processes. Release native replay passes8 File checks/3 real Windows cancels,
10 Layers modal,13 other Layers and6 viewport-menu checks, with source/binary/
per-replay PNG hashes, observed root windows/modules and zero direct children.
The own-modal cancellation probe remains a measured polling/WM_CLOSE boundary;
fully hung native calls/modal deadlines remain part of complete cancellation
acceptance. No new native geometry SDK job runs.

Independent read-only review found no introduced Important/Critical defect in
the selected adapter, RAII, error preservation or audit guards. Reviewer ran no
tests/builds/GUI; all measured results come from the main verifier. Microsoft
windows0.62.2 is already in the locked dependency set; selected COM/Shell features
are enabled, default features off. Its chosen MIT notice is
[preserved](../../Resources/licenses/windows-0.62.2.txt), SHA256
`c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383`;
Cargo checksum `527fadee13e0c05939a6a05d5bd6eec6cd2e3dbd648b9f8e447c6518133d8580`. No backend code from rfd is copied.

Next required work: actual native successful path selection/save/open/overwrite
and --ui-path startup diagnostics; complete UI/native/worker budget ownership;
dirty close, asynchronous/cancellable I/O and hung-modal behavior; versioned
view/CPlane/camera/material/annotation persistence and supported .3dm/losses;
full130core/RCORE/Matrix9 acceptance. Current controllers still save only the
supported schema. No new feature/capability status is promoted; total future
measurement batches remain unknown.
