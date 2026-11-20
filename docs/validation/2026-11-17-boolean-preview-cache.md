# Boolean Builder mesh preview cache — 2026-11-17

The add-on mesh preview cache now uses SHA-256 over the exact ordered object and cutter IDs, world-millimetre vertex bit patterns, triangle indices and Boolean mode. Previously it used only object IDs and cutter ID, bounds and vertex count, so different inputs could reuse a wire preview. The key does not contain a document ID or source revision: the current add-on event supplies neither, and the first object meshes remain captured in session state. Thus the digest prevents collisions caused by the former incomplete key but does not validate that captured sources are still current. It is not a BRep Boolean implementation or proof of mesh validity.

A focused add-on fixture changes a cutter vertex while preserving its ID, bounding box and vertex count, and compares the new preview with a freshly computed Boolean. It checks changed object vertices, cutter topology and Boolean mode alter the digest. This is in-process event/mesh coverage, not a real installed DLL, OS interaction, source-revision test, or proof of the whole T10 acceptance matrix. Hashing now scans all input mesh data on preview events; performance for large selections is unmeasured.

Validation with `L:/om9-target`, `-j 2`: `om9-addon-setting-tools` 8 passed; `om9-app` addon_session 18 passed, 2 ignored. Skill consent requested.


## Invalid mesh refusal

The mesh-only Boolean Builder path now refuses malformed indices, non-finite coordinates, degenerate triangles, open or non-manifold edges, inconsistent edge orientation and zero/non-finite signed volume. It validates objects before recording them and validates objects and cutters before preview or commit. Selections that are not actual mesh entities, including BRep display tessellation, are refused on this path; the BRep kernel route is still unimplemented. Invalid selection leaves the current stage available for correction.

Add-on event fixtures cover an open mesh, flipped triangle, out-of-range indices, degenerate triangle, non-finite vertex and BRep type as both object and cutter; an in-process Matrix host fixture checks open-object and open-cutter refusal without document or Undo changes. These checks do not establish self-intersection freedom, shell nesting, valid output topology, tolerance-scaled volume, real native DLL behavior or OS interaction. Basic index-based manifold checks may reject a mesh with unwelded seams.

Validation: setting-tools and Matrix library suites 10 passed; default addon_session 19 passed, 2 ignored; no-default addon_session 17 passed, 1 ignored. `L:/om9-target`, `-j 2`. Skill consent requested.


## Captured object source stamps

The host now captures document/object source stamps when Boolean Builder accepts its first group. On later session steps it checks each captured stamp, clears the old preview and refuses with a restart message if any captured source is stale or missing. The existing add-on cache still hashes content rather than document IDs; the host guard supplies owner, document, context, generation and revision checks for this captured group.

An in-process Matrix host fixture accepts a valid mesh object, previews a Boolean with a cutter, moves the captured object through `center_selected`, then verifies stale refusal, cleared preview and no additional document or Undo change. It also restarts, completes a mesh Boolean with unchanged sources and restores the original document with one Undo. Full default addon_session: 19 passed, 2 ignored; no-default: 17 passed, 1 ignored. After adding the final successful-commit assertion, the focused fixture passed again under both feature configurations. `L:/om9-target`, `-j 2`.

This does not establish BRep routing, native DLL/desktop operation, simultaneous external mutation during the synchronous add-on call, source changes before the next session event, self-intersection validation or output correctness across the Boolean matrix. Skill consent requested.


## Refresh after source changes

Boolean Builder tracks stamps of the selection used for its last preview. During the application builder's per-frame refresh, changed selected-cutter stamps cause a new selection event and preview calculation even when selection itself is unchanged. Changed or missing captured first-group stamps clear the preview and require restart. If refresh cannot recalculate, the old preview is cleared first.

The in-process host fixture previews a valid cutter, changes its placement without changing selection, and observes a different preview after refresh. It then changes the captured object and verifies refresh clears the preview; a later command step refuses without changing document or Undo. The same fixture checks a fresh Boolean and one Undo. Default addon_session: 19 passed, 2 ignored; no-default: 17 passed, 1 ignored, using `L:/om9-target` and `-j 2`. Skill consent requested.

This is an explicit controller refresh fixture, not an OS/GPU interaction test. Per-frame stamp comparison and full-mesh digest cost with large selections remain unmeasured. The existing BRep worker is coupled to Solid command pending state and publication semantics; Boolean Builder integration must preserve its own object/cutter stages, mode, KeepCutters and stale/cancel behavior. BRep remains unsupported on the add-on mesh path.


## Mesh Boolean output refusal

Before final publication, the mesh add-on checks the combined cutter and each output for invalid indices, degenerate or non-finite triangles, open/non-manifold edges, inconsistent orientation and zero/non-finite volume. Failure returns a refusal with no replacements or deletions. The existing approximate preview is still shown before commit and is not certified by these checks. Validation reads mesh slices without copying the full vertex or index arrays.

The existing intersecting-cuboid difference fixture confirms a closed output and volume 56. A non-axis-aligned cutter fixture produces an output with open edges from the current mesh CSG and now refuses rather than committing it. A disjoint intersection also refuses an empty result without deleting sources. These tests expose an implementation limitation, not successful support of those cases; self-intersections, shell nesting, tolerance policy and robust output repair are still open.

Setting-tools and Matrix library suites: 11 passed. Default addon_session: 19 passed, 2 ignored; no-default: 17 passed, 1 ignored. In-process fixtures only; `L:/om9-target`, `-j 2`. BRep Builder routing remains unimplemented. Skill consent requested.


## Bounded core mesh Boolean service

Boolean Builder's mesh path now calls the existing Rust `om9_core::mesh_csg::mesh_boolean` service instead of the add-on BSP routine. It uses the core service's T-junction conformation, output reconstruction and default triangle/fragment/work limits. A triangle-count preflight runs before conversion; result and combined-cutter structural checks remain. This supersedes the earlier refusal for the non-axis-aligned cutter fixture: it now returns a closed, oriented mesh. Other CSG consumers still use their existing routines.

Add-on fixtures verify difference/union/intersection for overlapping 4 mm cubes with respective volumes 56, 120 and 8 mm³; KeepCutters; two disjoint cutters with difference volume 206 mm³; the previously open-edge fixture; empty-intersection refusal; and oversized-input preflight. The host fixture exercises a mesh Boolean with one Undo. Setting-tools and Matrix library suites: 13 passed; default addon_session: 19 passed, 2 ignored; no-default: 17 passed, 1 ignored. `L:/om9-target`, `-j 2`.

The service is linked into the add-on and still runs synchronously on the caller with copied world-mesh input, a fixed default tolerance, and per-operation core work limits; it does not yet use host document tolerance, host memory admission, source-independent mesh extraction, worker cancellation or BRep routing. Multiple-object, complex topology, self-intersection, desktop/native DLL and large-mesh performance acceptance remain open. Skill consent requested.


## Authoritative document mesh input

For Boolean Builder, the host now builds mesh input from each directly selected document mesh and its placement in world millimetres, not `RenderBuffers`. It triangulates planar, simple convex polygon faces by fan; concave or nonplanar polygons, malformed faces and excessive triangle counts refuse explicitly. BRep remains unsupported by this mesh path. Other add-on commands retain their existing scene-derived input.

An in-process Matrix host runtime records a translated source quad's exact vertices, triangles and bounds; the default build checks the same input under Light and High display detail. The fixture also checks refusal of a concave face. Default addon_session: 20 passed, 2 ignored; no-default addon_session: 18 passed, 1 ignored (the High transition is OCCT-feature-gated); setting-tools and Matrix libraries: 13 passed. All used `L:/om9-target`, `-j 2`. The separate app `--lib` test target cannot compile because hundreds of existing internal BRep fixtures use the old source/display API; it was not counted as passed.

This covers direct mesh entities, not instance expansion, general concave/nonplanar polygon triangulation or all display-sensitive add-on commands. Source extraction and mesh CSG still run synchronously, use allocations not individually admitted by the host worker, and the CSG service uses its fixed default tolerance; real DLL/desktop and complex-source acceptance remain open. Skill consent requested.


## Concave source mesh faces

The host now uses a bounded Rust core ear-clipping routine for simple planar polygon faces in Boolean Builder document meshes. It preserves source winding and indices, checks planarity, rejects intersecting or touching non-adjacent edges and degenerate/numerically ambiguous faces, and shares a two-million-step work limit across each mesh. This supersedes the earlier concave-face refusal; nonplanar and invalid polygons still refuse.

Core fixture checks a six-vertex concave face in both orientations against signed area, a self-intersecting face, nonplanarity and work-budget refusal. An in-process Matrix host fixture captures a concave quad's two triangles and checks their signed area; it also checks a crossed polygon is refused. Default addon_session: 20 passed, 2 ignored; no-default: 18 passed, 1 ignored; setting-tools/Matrix library suites: 13 passed; core integration fixture: 1 passed. `L:/om9-target`, `-j 2`. Core and app library-test targets have existing unrelated stale BRep fixture compile errors; integration targets were used.

This establishes planar simple-face triangulation for these fixtures, not all polygon configurations or holes. Source extraction and Boolean computation still run on the caller with allocations not separately admitted by the host worker, fixed CSG tolerance and no instance/BRep support. Native DLL and OS/GPU acceptance remain open. Skill consent requested.


## Document tolerance in mesh Boolean

The host passes the validated document absolute tolerance in millimetres in the Boolean Builder session settings. The add-on supplies that value to core mesh CSG and includes its exact bit pattern in the preview digest. Standalone add-on events without the host value retain the previous core default. The core normalized tolerance admission upper bound changed from 1e-4 to 1e-3; values outside the supported range still refuse rather than being silently clamped.

The host capture fixture verifies a non-default 0.02 mm document tolerance reaches the event. A focused add-on fixture shows the same small inputs accepted at 1e-8 mm and refused at 0.1 mm, and checks the cache key changes with tolerance. Core fixtures check 4 mm overlapping cubes with 0.001 mm tolerance for all three modes using volume and closed-topology oracles, plus coarse-tolerance refusal. Core mesh_csg: 16 passed, 2 ignored; core triangulation: 1 passed; setting-tools/Matrix libraries: 14 passed; default addon_session: 20 passed, 2 ignored; no-default: 18 passed, 1 ignored. `L:/om9-target`, `-j 2`.

The enlarged normalized range is proved only for tested fixtures; near-tangent and complex topology at its upper boundary remain open. Source extraction and mesh CSG still execute synchronously in the add-on without host worker admission or cancellation. Instances, BRep Builder routing and desktop/native DLL tests remain open. Skill consent requested.


## Single-member mesh instances

The host resolves a direct mesh or a chain of single-member instance definitions from the document, composes every placement, and sends the resulting mesh in world millimetres. Cycles, excessive depth, non-mesh leaves and multi-member definitions refuse. Accepted object groups record definition revisions as well as root source stamps; selected cutter previews also track their definition revisions. A changed document tolerance after object acceptance is treated as stale because the stored triangulation was made with the previous tolerance. These revision/tolerance checks run on subsequent events or refresh.

An in-process Matrix host fixture captures a two-level nested instance and verifies composed world vertices, and checks that a multi-member instance refuses. Another fixture performs a mesh Boolean on a single-member instance and verifies it becomes a mesh and Undo restores the original instance and document. It also uses an instance as cutter with KeepCutters and verifies that instance remains unchanged and Undo restores the source. Default addon_session: 20 passed, 2 ignored; no-default: 18 passed, 1 ignored; the expanded instance Boolean/KeepCutters fixture passed separately in both configurations. Core mesh CSG and triangulation integration suites: 17 passed, 2 ignored. `L:/om9-target`, `-j 2`.

Fixtures do not mutate a definition revision or document tolerance during a live session, so those new stale checks are implemented but not runtime-proved. Multi-member/mixed instances, general BRep Builder routing, concurrent external mutation, worker cancellation and OS/native DLL remain open. Boolean still runs synchronously with allocations not separately admitted by the host. Skill consent requested.


## Cancellable core mesh CSG foundation

Core mesh CSG now offers a cancellable entry point. It checks cancellation at work-budget checkpoints used by the spatial index, splitting, classification and output reconstruction, and between major preflight/final validation stages. The existing Solid Boolean Union approximate mesh-preview worker uses that entry point for both its first attempt and retry. The ordinary non-cancellable API retains its behavior.

Core fixtures compare cancellable and ordinary results, verify pre-cancel refusal without changing inputs, and cancel a scheduled 700-component operation. The latter does not identify the precise stage at which cancellation was observed or establish a latency bound. Input validation, topology checks and some allocation loops still run between cancellation checks. Core mesh CSG and triangulation: 19 passed, 2 ignored; default app boolean_speculative and addon_session: 23 passed, 2 ignored; no-default addon_session: 18 passed, 1 ignored. `L:/om9-target`, `-j 2`.

This is a worker prerequisite, not asynchronous Boolean Builder: the Builder mesh path still calls the non-cancellable service synchronously. Host admission, native Builder routing, cancellation latency, desktop behavior and full acceptance remain open.


## Shared cancellable mesh batch service

The Rust core now owns a document-independent mesh Boolean batch operation: union the cutter group, then compute one result per object. It uses the cancellable core CSG routine, carries one work budget across the batch, limits aggregate output faces, and returns no partial result when any item fails, produces an empty result, exhausts the budget or is cancelled. The add-on mesh path now calls this same service, retaining object/result correspondence and existing source publication and Undo handling.

Core fixtures use two objects and two cutters and check each 56 mm³ result with closed topology, plus empty-result, budget and pre-cancel refusal with unchanged inputs. An add-on fixture checks that the two results remain associated with the correct source IDs. Core mesh CSG/triangulation: 20 passed, 2 ignored; setting-tools/Matrix libraries: 15 passed; default addon_session: 20 passed, 2 ignored; no-default: 18 passed, 1 ignored. `L:/om9-target`, `-j 2`. Skill consent requested.

This establishes a shared cancellable computation service, not host-worker execution: the add-on currently supplies a fresh uncancelled token and invokes it synchronously on copied inputs. Aggregate work/face counts are limits, not a proof of peak memory admission; cancellation timing and atomic publication of a multi-object batch through the real native DLL/desktop remain open. BRep routing and OneByOne/retry remain unimplemented.


## Host worker for final mesh calculation

In MatrixApp, the final Enter for the built-in Matrix or standalone Boolean Builder mesh session now schedules the shared core batch on a host-owned thread. The add-on still owns object/cutter selection and mesh preview. The worker carries a cancellation token and an admitted memory allowance; before publication the host checks authority, epoch, scene publication, selection and builder revisions, object and cutter source stamps, instance definition revisions and captured document tolerance. A stale context refuses, and the complete result batch is applied in one existing transaction with one Undo. Escape or new command input cancels; the worker permit remains held until its thread exits. Direct Controller use without the MatrixApp background setting remains synchronous.

Headless MatrixApp/add-on fixtures exercise icon-target and typed starts, all three modes with tetrahedron volume oracles, two objects and cutters, KeepCutters, one Undo/Redo, empty-result refusal, Escape and typed-input cancellation, changed selection/options, and retry after cancellation. An opt-in low-limit fixture verifies admission refusal before document mutation. Full default addon_session, boolean_speculative and join_history_ui suites: 69 passed, 2 ignored; no-default addon_session: 19 passed, 1 ignored; core mesh_csg/triangulation: 20 passed, 2 ignored; setting-tools/Matrix libraries: 15 passed. After additional mode and retry assertions, the focused worker fixture passed again in both default and no-default configurations. The test-fixtures admission case passed separately. `L:/om9-target`, `-j 2`.

The 32 MiB plus input-based reservation is an allowance, not a proven CSG allocation peak; core scratch allocations are not individually admitted. Source extraction, interactive mesh preview and final geometry/scene preparation still run on the caller, and no latency or cancellation bound is established. The GUI fixture is headless; real OS/GPU, installed native DLL, BRep Builder route and full plan acceptance remain open. Definition/tolerance mutation during a live job remains untested. Skill consent requested.


The worker fixture additionally verifies a document reopen retains the application worker setting and permit, the standalone Boolean Builder package follows the same host worker route, and a canceled worker releases its permit for retry. Its latest cases cover Difference, Union and Intersection with volume oracles; multiple objects/cutters; KeepCutters; invalid/empty result; command-option and selection staleness; Escape and new-text cancellation; one Undo/Redo; and reopening. The package runtime is in-process through the host JSON boundary, not an installed native DLL. Final validation: default addon_session and file_lifecycle 41 passed, 2 ignored; no-default addon_session 19 passed, 1 ignored; test-fixtures admission refusal 1 passed. The earlier 69-test batch covered boolean_speculative and join_history_ui before these final lifecycle refinements.


## Host worker for mesh preview

MatrixApp now sends Boolean Builder mesh preview computation to the same cancellable host worker as the final calculation. It retains source, scene, selection, tolerance, definition and builder-option guards. Stale preview results are discarded and retried after the previous worker releases its permit. Viewport object selection remains available while a preview is pending. If Enter arrives before preview completes, the in-flight result is promoted to the same guarded one-transaction commit; a selection change after promotion refuses publication. Preview edge transfer uses one line per edge rather than resampling each straight line.

The headless MatrixApp fixture verifies preview, stale selection/option retry, typed and UI Enter promotion (including two objects/cutters), stale selection after promotion, cancellation, all three modes and Undo. A separate headless viewport pointer fixture selects a different cutter while preview is pending. Default addon_session/file_lifecycle/join_history_ui: 87 passed, 2 ignored; no-default addon_session: 20 passed, 1 ignored; core mesh CSG/triangulation: 20 passed, 2 ignored; test-fixtures admission refusal: 1 passed. After the final UI/status and promotion assertions, focused worker and viewport fixtures passed again in default configuration and the worker fixture passed without default features. `L:/om9-target`, `-j 2`.

The preview is the full mesh calculation, not a proved low-cost preview; after a preview has already been displayed, a later Enter recomputes it. Source extraction and final scene preparation still run on the caller. The worker allowance remains an estimate rather than a peak-memory bound, and no preview/commit latency or OS/GPU/native DLL behavior is proved. BRep routing and OneByOne remain open. Skill consent requested.

## Cutter-stage source preparation on the worker

MatrixApp captures a tracked immutable document and selected root IDs before scheduling the Boolean Builder worker. The worker resolves selected meshes or single-member instance chains, applies placement and units, and triangulates source faces before mesh CSG. Source, definition, tolerance, selection and scene freshness guards still apply before preview or commit. Metadata traversal and the input-size admission estimate remain on the caller. Initial object-group extraction and final geometry/scene preparation also remain synchronous. Cancellation is checked between selected roots, but a single face triangulation has no independent cancellation latency bound.

The headless worker fixture was extended with a mesh-instance cutter; it checks preview, commit and Undo along with direct mesh, multi-cutter, promotion, stale and cancellation cases. Default addon_session/file_lifecycle/join_history_ui: 87 passed, 2 ignored; no-default addon_session: 20 passed, 1 ignored; core mesh CSG/triangulation: 20 passed, 2 ignored. `L:/om9-target`, `-j 2`. The allowance is an estimate, not a demonstrated peak allocation bound; OS/GPU, native DLL, BRep and full plan acceptance remain open. The previously proposed preview rule was approved by the user and recorded in the feature-port skill.
