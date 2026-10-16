# Owned File lifecycle service — 2026-10-14

[Source-bound receipt](2026-10-14-owned-file-service.json) proves Task1 of
[the complete File lifecycle plan](../superpowers/plans/2026-10-14-owned-file-lifecycle.md).
The previous goal turn was progress: the modal source checkpoint was still
hash-current when this turn began. This checkpoint extends implementation
toward the original RCORE-00/screenshot objective; it does not redefine it.

**File menu/CMD/toolbar/picker UI is still disabled and unaccepted.** This is an
owned controller/I/O service prerequisite. All New/Open/Save/Save As feature
statuses and complete RCORE/M9IMG acceptance gates stay open. The29 native UI
checks below are regressions of existing Layers/viewport routes, not File UI tests.

Safe Rust owns FileKind/phase, active path, saved fingerprint, restoration
diagnostics and opaque response tokens. New/Open on dirty state enters explicit
Save/Discard/Cancel; failed/canceled preparation preserves the old document,
scene, pending Line, current/selection and executable Redo. Candidate archive
restoration, fresh runtime authority and scene are prepared before replacing
the controller. New/Open acceptance clears the prior command/selection/Undo
owner. Same-file reopen preserves durable IDs but renews owner session and
coordinator authority, so old layer/viewport/file callbacks cannot revive.

Dirty comparison streams current supported document serialization into SHA256
without a whole JSON output buffer, caches it by coordinator epoch and compares
the saved content fingerprint. Revision alone is insufficient: Undo followed by
a different branch can reuse the revision. Initial controller input is the clean
baseline; edited content is dirty, Save becomes clean only after atomic output,
Undo/Redo back to saved content becomes clean again, and a different equal-
revision branch remains dirty. Missing replay modules remain diagnostics and
produce a dirty unresolved restore rather than a false clean/pass.

First Save requests a native `.om9` path; known-path Save writes its active path.
Save As requires selection and explicit overwrite confirmation. Paths are owned,
absolute and bounded to32,768 encoded bytes. Unsupported suffixes, missing parent,
corrupt/missing input, readonly destinations and budget failures preserve model,
Undo, active file identity and a good destination. A failed known-path Save closes
that attempt instead of becoming a Save As picker. The existing archive verifies
private bytes/staging and atomically publishes through the selected OS owner.
Successful Save has no geometry Undo effect. Save/New/Open/SaveAs cover the complete
currently supported Document, including Hidden/off/locked Layers, explicit object
preferences, current and durable identities; missing view/material schema is still
a required continuation, not silently considered saved.

Every asynchronous response boundary has a fresh token serial, with coordinator
authority/context and document identity/session/epoch/generation/revisions. An
independent review demonstrated that reusing one token across SavePath→OpenPath
accepted a duplicate Save picker response as Open. Three actual service tests
reproduced and fixed this class: consumed Save replies cannot select an Open file,
old overwrite approval cannot authorize a reselected destination, and a rejected
missing path reply cannot become valid after another actor creates the file.
Phase transitions and rejected path/overwrite replies consume the old token;
foreign/stale/consumed cancel cannot close a newer owner. Nonce overflow is checked
before I/O. Accepted intermediate Save returns SavedPending/SavedAndReplaced and
records an owned acceptance receipt before any follow-up replacement preparation.
The reviewer's initial max-name7 post-save failure hypothesis was disproved by
archive admission and was removed; it is not counted as RED/GREEN evidence.

Old retained world/Undo/scene/selection/dialog/file owners coexist with parse/
restore/candidate scene. `TransactionCoordinator::new_with_held` reserves external
owners during startup validation and identity capture while preserving the future
coordinator's full configured limits. File metadata/diagnostics/path storage joins
auxiliary usage; existing Workspace BTree/clock, recent command buffers and repeat
string are also charged, including prospective Workspace startup before allocation.
A10.5-MB test admits either old1,000 long-label roots or a65,537-point candidate
alone but refuses their Open coexistence without changing the old state. The
existing forced-moving allocation probes were rerun within their declared bounds.
These checks are not global GUI/GPU/worker/RAM or file-latency certification.

Measured: **458 workspace tests / 24 ignored SDK tests**;
**80 focused checks are a workspace subset**. Seventeen File controller
tests, one held-startup test and one File cold fixture are included. The cold
fixture uses actual controller Save/New/Open/SaveAs, then two bounded independent
OM9 CLI children compare geometry/attributes/Layers and confirm the original file
bytes survive Save As. Together with the earlier cold fixtures, seven children
exit cleanly. Format, warning-strict all-target Clippy and release build passed.

Three actual release native window regressions pass10 Layers dialogs,13 earlier
Layers and6 viewport-menu checks, with observed windows/modules/no children and
source/binary/screenshot hashes including compiled SVG inputs. No new SDK/native
geometry jobs ran. Historical SDK receipts retain their prior source checkpoints.
The first unrestricted workspace compile exhausted Windows commit/pagefile and
failed; a later overlapping targeted build also locked a test executable. These
were failed infrastructure attempts, not product passes. The final verifier runs
Cargo sequentially with `-j2`, preserves all tests and records the passing logs.

Pending tasks in the complete plan: actual common menu/CMD/F6/toolbar entry and
owned dirty/overwrite prompts; selected native OS chooser and late-callback/close
lifecycle; cancellable/background I/O and worker admission; complete view/CPlane/
camera/material/annotation persistence and explicit migration; supported `.3dm`
codec/UI interoperability and losses; complete resource/native/structural families,
OS input/DPI/global caches/latency and every full File/Layers/RCORE gate. A save
destination's external concurrent replacement policy also needs its full output
publication evidence. No feature/catalog implementation status is promoted.
