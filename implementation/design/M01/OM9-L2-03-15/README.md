# M01 OM9-L2-03-15 — Febe Matrix File Project And Jobbags

**Status:** `IN_PROGRESS_PARTIALLY_VALIDATED` · **Target:** standalone Rust OM9
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
- Numbered Job Bags start with ten slots and grow subject to resource limits. A new capture owns an
  OM9 archive, snapshot identity, provenance and selected-root metadata. Thumbnails are
  rendered asynchronously; capture does not modify the document geometry.
- Empty selection is refused explicitly and does not create a ghost slot.
- Captured content remains available after the original objects are deleted. Insertion preserves
  placement, remaps supported identities and references, and uses one Controller transaction/Undo;
  unsupported dependencies are refused rather than silently discarded.
- Master, Creation, Parts, Render and Output store versioned whole-document snapshots. Opening a
  named snapshot uses the document lifecycle's dirty Save/Discard/Cancel guard before replacement.
- The workspace project catalog persists independently of the open document. Existing embedded
  `om9.project-manager.v1` state is preserved and can be explicitly imported to workspace rows.
  A plain document with untouched default state does not gain an empty resource.
- `PROJECTS` remains above the reserved Builder slot and is a normal dock section, so existing
  collapse/reorder/float/resize behaviour applies without touching Job Bag state.

## UI

`PROJECTS` in the left dock shows:

- selectable project rows, `+` create, import and Mngr;
- a horizontally scrolling numbered Job Bag strip with capture, load, save and clear;
- an expandable, independently scrolling strip for the five named snapshot groups.

Mngr provides a catalog overview, search filters, Info/Files views and project import/export and
folder controls. The actual native window, pickers and complete workflow still need acceptance.

The manual M01-UI gate is still pending; no human sign-off is claimed.

## Historical verification — 2026-10-22

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

- BRep v5 archive support remains guarded pending durable topology binding and native payload
  disposition; snapshot save/restore for BRep is not established.
- Full native-window capture/save/reopen/restore, pickers, desktop DPI and manual visual review
  remain open. No human M01-UI sign-off is claimed.
- The existing P07 evidence is scoped to the listed fixtures; it does not establish all geometry,
  dependency types, resource peaks or failure modes.

## 2026-11-17 Project Manager follow-up

The image-guided Projects layout and Manager overview have headless interaction coverage; the UI and persistence integration suites passed 14 tests. A separate SDK-authored point survived Job Bag capture, source-file removal, Undo/Redo and a native 3DM publish/reimport. These results and their limits are recorded in [P07 validation](../../../../docs/validation/2026-11-17-project-manager-p07.md). BRep v5 archive support, the broader native matrix and manual desktop/DPI review remain open.

When a workspace catalog and document-embedded Project Manager coexist, the embedded manager is preserved on Save. The Manager offers explicit import into separate workspace rows with remapped local IDs; legacy inline bags are retained. Integration checks cover cancel, stale document, invalid later row and dirty-open Save without partial replacement. Complex legacy dependencies and desktop behavior remain open.

The ordinary Controller file lifecycle now has a headless-tested background v6 Save path for BRep sources with current topology proofs and a configured pinned worker. Reopened v6 BRep can reuse verified packets for subsequent saves, including alongside newly attested sources; numbered Job Bag BRep capture and Load have scoped headless coverage. All five named snapshot groups have a headless Box capture/open test, including stale-project refusal and Save As/Open after restoration. The catalog loading and project import workers validate v6 assets; imported numbered Box bags, explicit embedded import and a malformed batch have headless coverage. Broader geometry, production build and desktop acceptance remain open; see the P07 record for limits.

Named BRep Open validates its captured manager assets, restores the document and builds the candidate scene on the worker. Before publication it checks that the project, document, selection and scene are still current. Final controller construction and publication still occur on the UI thread; large-scene responsiveness remains unverified. The full application unit-test target currently has stale BRep fixture compilation errors.

Numbered BRep Job Bag Load now classifies, projects and remaps dependencies, prepares its transaction and builds a candidate scene in a worker. It checks the project, source archive and live scene before an atomic paste. Archive validation admits the selected roots' exact dependency closure and rejects unrelated extra objects; clean-History and BRep cases have headless coverage. Final commit remains on the UI thread; broad responsiveness is not yet established.

The per-bag Save worker validates v6 BRep assets before writing them. A numbered Box bag and a Parts snapshot have been saved and reopened headlessly. With a configured native adapter, all 24 clipboard integration cases and seven SDK-authored native Job Bag publication/reimport cases passed; these do not establish arbitrary BRep or desktop acceptance.

The thumbnail worker can now render verified v6 BRep scenes. An offscreen Box fixture covers a numbered bag and Parts after source deletion, plus rejection of a corrupted archive; other geometry and desktop appearance remain open.

After catalog reload, a synthetic pointer fixture opens Manager > Files, previews the numbered BRep thumbnail without editing, and double-clicks it to insert one Undoable copy. Physical desktop behavior and DPI remain open.

A separately pinned native 3DM payload display worker and Rust host have focused tests for source identity, cancellation, stale results and scene adoption. Job Bag archive paths do not yet use it, and opaque native BRep persistence remains open. See the P07 record for evidence and limits.

With its fingerprint embedded, asynchronous legacy v4 Open now uses that worker when a native display is missing. One authored Box was reopened with its source payload and pick geometry intact. Job Bag persistence and history remeshing remain open; see the P07 follow-up.

History remeshing now routes its existing shared cancellation flag to that worker when the fingerprint is embedded. This compiles but still needs a runtime Undo/Redo cache-miss fixture in the final validation batch.

An explicit v7 mixed-BRep structural archive candidate compiles; it is not routed into Save, Open or Job Bag and is not yet native-verified. Its publication, scene ownership and final tests remain open.

The detached v7 host now has worker validation for canonical packets and each native payload, retaining the resulting display owners. This compiles but has no runtime evidence or scene/publication integration yet.

Detached mixed documents can now register source-matched native display owners and bind them for scene preparation. This has compile evidence only; product routing and runtime validation remain open.

The detached mixed-source host also prepares a staged v7 file with worker validation before and after readback. Product routing and runtime verification remain open.

The read-side routes now recognize v7 for Open, project validation, named/numbered restoration and thumbnails. They have compile evidence only; native capture/save, cache reuse and runtime validation remain open.

Capture/save now selects v7 for native sources and can reuse canonical packets from v6 or v7; native-only snapshots are accepted by the cache planner. This does not establish the P07 gate.

One generated native Box passed headless numbered Job Bag capture, source deletion, insertion, Undo/Redo and v7 Save/Open with its payload retained. Mixed sources, other native geometry, failure cases and desktop acceptance remain open; see the P07 receipt.

The same fixture also reloaded a persisted catalog in a fresh app and verified numbered insertion and Parts Open with the native payload intact. Coverage remains limited to one headless Box.

One native v7 Box was subsequently captured and opened through all five named groups, with its payload and pick geometry retained. This is headless coverage for one native shape.

A generated native Sphere also survived v7 Job Bag capture, deletion, insertion and Undo/Redo with source payload and pick geometry intact. Broader curved models remain unverified.

The archive library and three focused integration targets passed after v7 changes. A targeted core library test could not compile because its lib-test target has 107 BRep fixture/API errors; see P07 for scope.

A mixed native/canonical Box Job Bag passed headless insert, Undo/Redo, Save, Open and resave after retaining its verified packet archive on successful insertion. A second distinct canonical Box bag was also inserted and both packet caches survived Save/Open/resave. For one native Box, post-preparation cancellation and a stale project preserved the document, scene and CPU-cache baseline. Broader failure and desktop cases remain open; see P07.

For one native Box bag, corrupted archive export and immediate cancellation preserved an existing destination. A structurally valid v7 container with an invalid native payload was also refused without changing the destination or scene. Broader corruption and active native-call cancellation remain open.

The shared process-owner suite passed running-process cancellation and cleanup tests. It does not identify the point within a native BRep call at which cancellation occurs.

Repeated insertion of the same shared Job Bag archive now reuses its cache entry; a native Box fixture verified no extra entry and successful Undo. Separately allocated duplicate archives and long-term retention still need assessment.

The project capture worker suite passed 23 tests after replacing an obsolete BRep-refusal assertion; seven GPU/child-fixture cases were ignored. This is not desktop acceptance.

Six GPU thumbnail cases were subsequently run and passed; a current offscreen Projects panel render was also visually inspected at one scale. The separate child fixture and physical desktop/DPI review remain open; see P07.

Focused interaction checks passed at egui scales 1.0, 1.25 and 1.5; GPU offscreen panel images at 1.25 and 1.5 were also inspected. Physical Windows DPI and desktop acceptance remain open.

Manager's focused group passed 12 UI cases plus the offscreen GPU case for a non-active project; a native v7 Box also produced a nonuniform thumbnail without changing the target document. Physical desktop and broader geometry remain open.

Current persistence tests passed 11 cases; catalog integration passed 10 and a separate 257 MiB resource workflow passed capture, reload, export, root change and import. These are headless checks; see P07.

The full current project_manager_ui executable passed 40 default cases, and its two ignored GPU cases passed separately. This supersedes the earlier interrupted run, not physical desktop acceptance.

The rebuilt clipboard integration suite passed all 24 cases with the configured native adapter, including retained sources, clean History, gems, resources and Undo/Redo. This remains headless coverage.

Native replacement representation sharing now has a focused Box check for account ownership, cancellation, exact source identity and selectable scene geometry. Broader native and desktop behavior remain open.

The existing release profile now has a paired application and workers with matching embedded digests. Local runtime DLL copies, CLI fixture create/inspect and contained Box restores were checked; release packaging and desktop acceptance remain open. See P07.

The rebuilt native/SDK Job Bag integration suite passed its seven representative 3DM publication/reimport cases. This remains headless evidence, not broad native table or desktop acceptance.

The actual paired release executable opened a native Box v7 fixture through its headless Controller path and rendered it in four offscreen viewports. Its separate synthetic om9_ui replay refused that document because it clones the source without the representation store; Job Bag UI and physical desktop behavior remain unverified.

A separate default-feature integration test sent synthetic pointer events through the Job Bag sidebar for one native Box v7, saved the workspace catalog and verified insertion after source deletion, catalog reload and Undo/Redo. A separate test process also loaded the catalog and inserted the bag. Physical desktop and release UI acceptance remain open.
