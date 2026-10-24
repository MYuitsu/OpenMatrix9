# M01 OM9-L2-03-15 — Febe Matrix File Project And Jobbags

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Target:** standalone Rust OM9
**Owner:** `OM9-L2-03-15` (M01)

## Delivered

### File lifecycle

- File > New / Open / Save / Save As remains the existing native `.om9` lifecycle in
  `crates/om9-app/src/file_lifecycle.rs` and `session/files.rs`; cancel/failure leaves the open
  document and Undo/Redo intact.
- File > Import/Export 3DM remains the shared native host adapter (`three_dm_host.rs`); the importer
  is transactional and the exporter is read-only.

### Project Manager and Job Bags

- `crates/om9-app/src/project_manager.rs` owns typed project rows with stable `ProjectId`s,
  duplicate labels allowed (ID is identity), rename/select, Unicode notes and customer metadata.
- Three Job Bag slots per project. A capture stores cloned entities with a snapshot ID, source
  document, source revision, captured object IDs and a data thumbnail (count/kind/revision); the
  capture never mutates document geometry.
- Empty selection is refused explicitly and does not create a ghost slot.
- A deleted captured object makes the slot unavailable; the thumbnail is not shown from stale data.
- Restore mints fresh `ObjectId`s, offsets placement and sends all restored objects through one
  Controller transaction for one Undo entry. Repeated restores never collide.
- Master Job Bag deep-clones the whole `Document` (entities, layers, instances, history) and exposes
  its layer names/count for inspection.
- Project rows/slots/notes are embedded in the `.om9` file as the opaque
  `om9.project-manager.v1` resource and restored on Open. A plain document with the untouched
  default project does not gain an empty resource.
- Restore Master is live: it validates/builds a replacement Controller, then swaps the document in
  one atomic step; failure leaves the current document untouched.
- `PROJECTS` remains above the reserved Builder slot and is a normal dock section, so existing
  collapse/reorder/float/resize behaviour applies without touching Job Bag state.

## UI

`PROJECTS` in the left dock shows:

- selectable project rows and `+` create;
- editable Name, Customer and Project notes;
- three job-bag slots with Capture 1/2/3 and Restore when available;
- Capture Master and a Master object/layer/revision summary.

The manual M01-UI gate is still pending; no human sign-off is claimed.

## Verification — 2026-10-22

| Test target | Result |
|---|---:|
| `om9-app --lib project_manager` | **8 passed** |
| `project_manager_ui` | **3 passed** |
| `file_lifecycle` | **20 passed** |
| `project_manager_persistence` | **2 passed** |
| `file_widgets` | **21 passed** |
| `panel_ops` | **33 passed** |
| `shell_docking` | **10 passed** |

Log:
`build/standalone-validation/ui-artifacts/m01-l2-03-15-20261022/acceptance.log`

Screenshot:
`build/ui-audit-l2-03-15/workspace.png` (Projects panel visible above Builders).

## Remaining

- Manual visual review of the real Windows window/pickers is still required.
- No human M01-UI sign-off is claimed.
