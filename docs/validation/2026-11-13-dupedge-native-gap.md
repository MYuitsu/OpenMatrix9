# DupEdge NativeBrep source view

## Implemented scope

The product now prepares a temporary edge view from a captured NativeBrep payload on a cancellable worker. A preselected NativeBrep starts preparation when DupEdge begins; otherwise an unambiguous click on its mesh requests preparation and the user selects the edge again when ready. The generated edge segment is checked against a unique source edge before commit. The temporary scene is bound to the source, document, session and publication; cancellation restores the preceding scene. The existing managed reservations cover the converted source and admitted view allocations, but are not a bound on all OCCT native allocations.

In simulated egui, one saved/reopened box with nonuniform scale and shear produced the expected world-space edge through typed command and menu dispatch, including selection before or after starting the command and one Undo/Redo. The worker fixture verified malformed source and a deliberately tiny managed budget refuse without changing document, scene or Redo. Another fixture verified cancellation after publication, invalidation after session change, and cancellation immediately after starting the worker; exact execution timing inside a native call is not established.

The source-view worker group passed 21 tests; the DupEdge UI group passed 28, with three imported-3DM fixtures ignored; the wireframe DupEdge group passed 13, with two diagnostics ignored. The no-default-feature check and public source audit passed. Physical desktop, independent imported files, simultaneous overlapping roots, physical OOM and native peak remain unverified. Do not treat this as completion for all NativeBrep geometry.

## Native cylinder edge

An authored NativeBrep cylinder saved and reopened in OM9 was selected, and DupEdge extracted its upper circular source edge through the temporary view. Five sampled points had the expected 2 mm radius and z=5 mm, and document snapshots matched through Undo/Redo. This is one simulated UI fixture, not an independent imported file or physical desktop test.

## Multiple selected native edges

In a saved/reopened NativeBrep box fixture, simulated viewport clicks selected two distinct source edges, toggled one off and on, and created both expected lines in one command. Their endpoints matched the box oracle, and one Undo and Redo restored the complete document snapshots. A prepared Redo existed before commit. This is one box fixture; it does not establish arbitrary multiple sources, blocks, malformed native topology or desktop behavior.

## Authored 3DM trimmed arc import

Batch `native-dupedge-3dm-trimmed-arc`, package 3: three executions of one authored openNURBS quarter-cylinder 3DM fixture, all passed. The admitted exchange host imported it with `BrepMode::RetainOpaque`; the complete document was saved and reopened as OM9. Simulated viewport selection extracted the top quarter-circle, with five samples checked against radius, height and quadrant, both endpoints and their order checked against the fixture generator, and full document snapshots checked through Undo/Redo. The second execution added the endpoint-order assertion; the third checked the source-point screen guard. This checks the fixture's direction, not all orientation parity, desktop interaction, a third-party file, all arc geometry or native peak memory.

Fixture SHA-256 `bacf2611787e0b652ea3fc9f776b627aa6f779537a4b5c048c0c7ecb5a516125`; admitted exchange executable SHA-256 `8bdd11bcb529b9e55b7eeed5e4e80f5741cb2a4d1e4e1c0bd3040347d4817a6c`. Both files were already present in `L:/om9-target`. The current exchange hash differs from the previously recorded hash in the ExtractWireframe evidence; this execution admitted and used the hash measured at test runtime.

One prepared batch remains: `modeling-public-rhino5-generated-exchange` (four fixtures, seven objects); packages 2–8 remain open and the total future batch count is unknown. Next verify representative independent 3DM files and physical desktop once available, while continuing Plan A.

## Native source-view memory recovery

The worker's memory refusal previously reported a stale selection if view preparation began before an edge had been picked. It now retains the command, selection and scene context independently of edge picks, and reports memory refusal if that context still matches. The recovery target retains the root: after consent the app retries view preparation, discarding the oldest Undo only as needed. A simulated worker refusal retried without deleting Undo after pressure was released; under sustained simulated pressure a separate fixture removed one oldest Undo and then prepared the view. Changing selection while the prompt was open stopped retry before removal. Document and Redo were preserved in these fixtures. Initial admission refusal for a preselected NativeBrep also offered retry through CMD and menu in a simulated test. These checks do not cover every allocator or an actual OS out-of-memory event. The user approved recording this tested scope in the feature guidance.

Three focused recovery tests, the 21-test source-view group, the existing B-rep source-view recovery regression and the 28-test DupEdge UI group passed. The no-default-feature check passed. The debug executable was rebuilt in the existing target directory; no physical desktop check was performed.

## Earlier reproduction

The `reopened_native_box_dupedge_currently_cannot_pick_source_edge` simulated egui fixture saves and reopens the same box geometry as ordinary B-rep and NativeBrep. From the same camera, clicking the same top edge and confirming DupEdge succeeds for the ordinary B-rep. For NativeBrep the click leaves the edge selection empty and the command pending; the source document and a prepared Redo entry remain unchanged. This demonstrates one missing user workflow, not general import coverage or a physical desktop result.

The ordinary B-rep display has topology edge identities and sampled segments; NativeBrep's current scene uses its display mesh and does not expose those identities. DupEdge also requires `Geometry::Brep` in its source-pick and worker paths. ExtractWireframe and DupBorder can convert the native source but extracting every edge is not a substitute for identifying the specific clicked edge.

Next implementation needs an admitted, cancellable temporary source-derived edge view for NativeBrep, bound to the exact captured native payload, source stamp, scene publication and command session. Verify the chosen display segment against a unique source edge before one atomic commit; preserve original document, old view and Redo on cancel, stale result, memory refusal and worker failure. Reuse the existing temporary DupEdge view lifecycle where its B-rep-only proof can be safely generalized; do not assign NativeBrep mesh indices as source topology IDs. The positive fixture must cover save/open, source geometry and placement, command/menu paths, and Undo/Redo before calling this gap fixed. Native peak and physical desktop remain unverified.

The original focused simulated egui test passed, including the ordinary B-rep control and the NativeBrep refusal. This records the behavior before the implementation above. Plan A stays open; B1–B3 remain deferred.

## Source-edge adapter groundwork

An admitted kernel entry point now resolves a picked segment directly against a canonical OCCT source string without constructing another display mesh. On a NativeBrep box converted from 3DM, the tested top-edge segment resolves to a curve with the expected endpoints and picked point; a segment joining different edges and a zero-length segment are refused. A simulated admission refusal also returns an error. The source string must remain alive and be charged by the caller; the adapter does not itself bind the result to a document, scene or session.

The focused test and seven existing source-segment regressions passed; the no-default-feature app check and public source audit passed. This describes the initial adapter groundwork, before the product connection documented above.
