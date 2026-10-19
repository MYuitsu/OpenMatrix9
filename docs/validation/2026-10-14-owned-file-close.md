# Owned window close — 2026-10-14

[Source-bound receipt](2026-10-14-owned-file-close.json) extends
[selected Win32 File evidence](2026-10-14-owned-file-win32.md).
The full foundation/six-screenshot goal remains active and unpromoted.

Safe Rust now represents window shutdown as internal `FileIntent::WindowClose`,
without assigning a new catalog feature ID. Both normal egui UI and eframe's
minimized/occluded logic pass veto a dirty native close request. The same owned
Save/Discard/Cancel prompt controls continuation. A clean document may close
immediately. Existing File or Layers modal ownership vetoes closing; it does not
implicitly cancel that other operation or queue a new shutdown decision.

Discard approves shutdown without replacing the document, clearing its draft,
changing geometry/selection/views or publishing Undo. Save approves only after
the supported `.om9` archive is accepted; first Save uses the native chooser,
known-path Save retains the path. Failed output and canceled chooser keep the
window and prior state. `CloseReady` retains the captured operation token until
native shutdown and is revalidated before every Close command/close event.
Mutation makes old consent stale. UI input is suspended after valid approval.
Close still relies on native application teardown; complete late-job/observer/
resource disposal and global bounded shutdown remain required foundation work.

Measured **494 workspace tests / 24 ignored SDK
tests**, **115 focused checks are a workspace subset**, 86 suites.
New tests cover clean/dirty Discard, first/known-path Save, Cancel, save failure,
stale consent, repeated window requests, other File ownership, active-path
failure and minimized logic. The active-path failure test replaces its own
archive fixture with an empty same-name directory and verifies retained path,
prompt/model/Undo/Line draft/dirty state and unchanged directory. It does not
claim that fixture replacement was performed by the product.

The new native replay passes 11 checks, including two actual root `WM_CLOSE`
events, Cancel preserving the window and Save followed by a real Windows picker
Cancel. Its four chooser cancellations are separate from the existing File
regression's three; the eight original File checks overlap the new replay.
The fixed message is posted only to the supplied same-process/current-thread
root HWND through the selected native-owner adapter. There is no desktop input,
window search or foreign application control. Real native Discard-button and
successful-save shutdown remain unmeasured; controller/widget evidence is not
relabelled as native OS evidence. Audit teardown explicitly uses captured close
intent plus Discard for the throwaway audit document.

The initial native attempt is retained in
[attempts](2026-10-14-owned-file-close-native-attempts.json). Its final assertion
ran before the UI pass serviced SavePath; the resulting receipt subsequently
contained four completed picker cancellations. The harness now waits for owned
prompt/chooser readiness before advancing. No product threshold is relaxed and
no failed attempt is counted as passing.

A first aggregate attempt then stopped the Layers modal regression at step5
because actual native focus was lost. That separate failed receipt is retained
in the same attempts record; its cause is unproven. The guard prevented further
synthetic input. Fresh passing evidence retains its own source/binary hashes.

Format, warning-strict all-target Clippy, workspace, release build and five
native replays pass sequentially with Cargo `-j2`. Layers regressions pass
10 modal plus13 other checks, viewport-menu6 checks; seven bounded cold OM9
children exit with zero active processes. Receipt hashes bind source, executable
and immutable per-replay screenshots. Independent read-only review found no
Important/Critical defect in the product close flow; reviewers ran no build,
test or GUI. The reviewer found a test-only unwind cleanup hole: the active-path
directory fixture could cause Files::drop to panic again after an assertion.
An exact-path, nonrecursive RAII guard now cleans that directory before the
fixture owner, including unwind, and success checks it is gone. No new SDK
geometry jobs. V1 fingerprints remain unchanged.

Required next work: actual native successful chooser selection/save/open and
owned overwrite, actual `--ui path` startup diagnostics, complete view/CPlane/
camera/material/annotation persistence with migrations and `.3dm`, asynchronous
I/O/hung-modal/full UI-native-worker budget ownership, native success/Discard
shutdown and complete owner disposal. Full130core/M01–M12/File/Layers/M9IMG/
607-feature/148-capability gates remain open. Total future measurement batch
count remains unknown; all unclosed packages are retained.
