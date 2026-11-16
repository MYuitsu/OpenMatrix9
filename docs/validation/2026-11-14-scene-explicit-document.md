# Scene entity lookup: explicit document

`Scene::entity_for` now borrows the entity from an explicitly supplied document,
not from the lifetime of the scene. It checks that the supplied document is
the same allocation used by this scene and retains the existing root, path,
revision and display-receipt checks. The source-capture functions retain their
existing separate coordinator validation and do not require its document to be
the same allocation. Actual picking, snap, UI and command consumers were
adapted; the controller-owned document is supplied directly in the Isocurve
and source-view paths.

The scene test checks an unrelated allocation is rejected and that an entity
borrowed from a separately retained document remains usable after dropping the
scene. This is a prerequisite for removing document ownership from the scene:
the scene still holds `Arc<Document>`, B-rep geometry still holds display, and
no cache eviction, cold Undo/Redo or v5 file claim follows from this change.

Validation on Windows with `L:\om9-target`, `-j 2` and the configured OCCT SDK:

* `cargo test -p om9-scene -j 2`: passed all scene unit and integration tests;
  the new post-drop lifetime assertion was additionally run with the targeted
  `entity_lookup_rejects_stale_source_and_wrong_leaf` test, passing 1/1.
* `cargo test -p om9-app --lib -j 2 source_view_`: 26 passed.
* `cargo test -p om9-app --lib -j 2 isocurve`: 21 passed.
* `cargo check -p om9-app --all-targets -j 2`: failed in the existing
  `tests/curve_shape_audit.rs`, which references undefined `CURVE_MENU` and
  `controller`; the application library and its unit-test target compiled.
* `python tools/public_source_audit.py`: 0 errors. Artifact pruning kept 169
  older runs cited by records and removed none.

No desktop interaction, native peak memory or final C1–C5 gate is established.
Next remove scene's strong document owner while preserving explicit tracked
document leases for worker consumers and scene publication/stale checks.

## Hover worker follow-up

The picking implementation now receives an explicit document. The background
hover caller supplies its existing accounted `CapturedDocument`; ordinary pick
entry points retain their previous behavior. A different document allocation
is rejected before traversing candidates, including for an empty scene. This
still does not remove the scene's own strong document reference.

`cargo test -p om9-scene --test world_picking -j 2` passed 8/8, including
matching the ordinary pick with a tracked-document pick and rejecting a cloned
document allocation. `cargo test -p om9-app --lib -j 2 hover` passed 14 tests,
including stale result suppression after document replacement. An additional
targeted test passed after the hover snapshot picked through its tracked lease
across New; this establishes that the private old result can be computed, not
that it may be published into the new document. Desktop behavior remains
unverified.

## Detached hover snapshot

The product hover path now converts its admitted scene copy to `PickSnapshot`.
This retains scene objects, immutable display buffers, picking-cache owner and
a weak document identity, but no strong `Document`. The existing separately
accounted `CapturedDocument` supplies the source during pick. Spatial index
creation remains lazy and occurs when the worker picks. Scene and detached
snapshot use the same cache implementation and receipt validation.

The hover reservation includes a conservative charge for the weak document
allocation and, per native receipt, the weak payload and mesh owner allocations;
shared receipts can consequently be counted more than once. The charge remains
until the snapshot is dropped. This does not certify allocator overhead, native
peak memory, GPU memory, or accounting for unrelated snapshot consumers.

`cargo test -p om9-scene -j 2` passed, including a NativeBrep fixture in which
document, payload and display strong owners die while the detached picking
snapshot and its buffers remain. `cargo test -p om9-scene --test
managed_picking_cache -j 2` passed 15/15 after extracting the shared cache path.
`cargo test -p om9-app --lib -j 2 hover` passed 16/16, including a real worker
thread using its tracked source after New, stale-result suppression, quota
release and an independently retained snapshot whose native source and display
die after the tracked lease is dropped. These are headless checks, not desktop
performance evidence. The active scene, other snapshots and document B-reps
still retain their original strong owners; C1–C5 remain open.
