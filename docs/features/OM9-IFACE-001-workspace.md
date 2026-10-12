# OM9-IFACE-001 — functional workspace slice

The user selected workspace selection, deletion, Undo/Redo and camera controls on2026-10-04. This is a bounded extension of the existing Rust-command/Qt-sidebar/native-FreeCAD flow.

## Evidence and scope

`specs/01-core/om9-iface-001-matrix-interface.md` identifies the reference interface sections. Object selection/deletion here are OpenMatrix9 host choices; no recovered Matrix90 deletion or SubD selection behavior is claimed. The raw catalog and decompiled references are preserved.

Native source evidence in the matching local checkout:

- `src/Gui/CommandDoc.cpp`: `Std_SelectAll`, `Std_Delete`, transactions, dependency confirmation and cancellation.
- `src/Gui/Selection/Selection.h`: document-specific selection, complete selection and clearSelection.
- `src/Gui/CommandView.cpp`: seven standard camera orientations and FitAll/FitSelection.

## OpenMatrix9 decisions

Rust owns the workspace command catalog, captions, supported host mappings and success/history policy. Qt presents All, None, Delete, Fit all, Fit sel and a Views menu under the existing quick row. The OpenMatrix9 menu bar also has a Workspace submenu. Existing menu groups, icons and quick positions remain intact.

- All: use a document-scoped adapter calling FreeCAD's `clearSelection(docName)` and `setSelection(docName, doc->getObjects())`. Do not use `Std_SelectAll`, which may select only the current tree group on its first invocation. Empty document/edit mode disables this command. An unchanged selection is not a new history entry.
- None: clear only active-document selection through the native selection API. Other documents' selected objects are preserved. Empty selection/edit mode disables this command.
- Delete: selected active-document objects only, outside edit mode. Reject foreign or mixed document selections. Use `Std_Delete`, retaining host dependency prompts, cleanup, transaction and Undo/Redo. Cancellation and no deletion do not enter OpenMatrix9 history.
- Undo/Redo: existing native commands and history buttons; document transactions remain FreeCAD-owned. Success requires a change in undo/redo availability.
- Views: Isometric, Top, Front, Right, Left, Rear, Bottom on the active native3D view. Camera axis meanings follow FreeCAD, without claiming Matrix's four-view camera/C-plane layout.
- Fit all/selected: use OM9-VIEW-006/007 adapters; selected fit requires an active-document selection and is disabled during edit mode.

No dimensions, geometry parameters, previews, builder dependencies or material changes are added. Ordinary viewport clicks/tree selection continue using FreeCAD. Selection and cameras are not document geometry transactions; a dispatched, available view command is recorded even if its destination is already current. Rust accepts only the effect appropriate to the command, rejecting no-op selection, canceled deletion and unrelated effects.

Native host snapshots copy stable selection strings before commands. Documents are reacquired by name after modal operations. Qt refreshes availability for buttons and camera actions, rechecking at invocation. Workspace operations become unavailable after documents close.

Rust also declares permission categories for document/view/selection changes. Native availability respects `Gui::Control` task-dialog permissions even when there is no edited object. Menu aliases use matching FreeCAD command flags, so a task permitting view changes while denying document/selection changes permits cameras and blocks selection/deletion consistently in both sidebar and menu. Undo remains subject to FreeCAD's transaction/undo configuration.

File and Undo/Redo commands retain their special native availability policies, without adding ordinary document alteration gates. OM9 command wrappers use `NoTransaction`; the dispatched native command owns transactions. Native regression checks exercise Undo/Redo and file-button availability while a restricted task remains open.

Delete/Undo/Redo use native transactions. Surviving/restored shapes are saved in FCStd through FreeCAD. Session icon history is bounded20 and is separate from geometric dependency history.

## Validation and continuation

Rust semantic tests cover workspace registration, unsupported SubD commands, effect-specific success, permission categories and bounded ABI. Qt tests cover dispatch, disabled controls/actions and refresh. `tests/workspace_smoke.FCMacro` verifies real selections including grouped objects, a restricted task without getInEdit, sidebar/menu camera consistency, deletion/restored volume192mm³, redo, canceled dependency deletion, foreign/mixed-document guards, cameras, fit-selected zoom, packaged images/hashes, document closure/save/reload, compact geometry and workbench switches. Camera assertions wait for the native transition rather than sampling intermediate animation frames.

Final measured results are recorded in `docs/validation/2026-10-04-workspace.md` and `workspace-runtime.json`. Four simultaneous viewports, construction planes, selection inversion, advanced filters and jewelry geometry are outside this slice. Next read: the four-viewports spec and native FreeCAD view APIs.
