# Surface Snap worker: owned NURBS source

For a batch whose candidates refer to one NURBS surface occurrence,
`SurfaceSnapJob` now copies only the NURBS geometry and its optional trim
extension. It keeps the document identity weak, validates every candidate
before capture, and retains the original document/scene/camera/selection checks
before accepting a result. Mixed-source batches still capture a tracked
document. OCCT B-rep batches retain their previously implemented source lease;
exact NativeBrep surface snapping is still unsupported.

The source copy uses fallible allocations and a conservative capacity budget
in the existing kernel reservation. The worker refuses a reservation whose
pending portion is already spent; the copy is marked allocated after creation.
Source copying occurs before spawning the worker; exact intersections run in
the worker. The UI's regular surface snap path still performs synchronous
refinement: the scheduler is not wired to UI gestures by this change.

On Windows with `L:\om9-target`, `-j 2` and the configured OCCT SDK:

* `cargo test -p om9-app --lib -j 2 surface_snap`: 19 passed, one existing timing
  diagnostic ignored. The trimmed-cylinder fixture includes a hole; it proves
  the expected rear hit, low/spent-reservation refusal, and stale result
  rejection after New. A test gate held the source-only worker before native
  work while New released the old document, then let it finish.
* `cargo check -p om9-app --lib --no-default-features -j 2`: passed with existing
  warnings.
* `cargo test -p om9-app --test brep_picking_precision -j 2`: 20 passed and one
  failed. `indexed_face_refinement_respects_a_trimmed_through_hole` also failed
  when rerun alone: `verify_face_display` rejected a triangle with `triangle
  crosses trim boundary` before the worker call. Its cause is not established;
  this is not evidence that the complete B-rep regression suite passed.

No desktop gesture, UI scheduling, OS OOM, native peak allocation or general
multi-source worker ownership is established. Active scene and canonical B-rep
display ownership, cold restore and v5 remain open. Next investigate the trim
verification failure without relaxing topology checks, and integrate a bounded
worker lifecycle into the actual surface-snap UI before claiming background
interaction.

## Multiple NURBS sources

When every candidate is a NURBS surface, a multi-occurrence batch can now copy
its admitted source geometries and trim extensions without a document owner.
Copies are budgeted before construction and their actual vector capacity is
marked allocated. The current implementation copies per candidate; repeated
sources in a mixed batch are not deduplicated. If the existing reservation
cannot cover those copies, or the batch contains other geometry types, the
worker retains the previous tracked-document path. This is a transitional
fallback, not the C1 end state.

A gated fixture with two independently owned overlapping planes checked that
the worker chose the front surface, that New released the document while the
source-only worker was held, and that the eventual receipt was stale. The same
fixture also tested the low-copy-budget tracked-document fallback and obtained
the same front point. `cargo test -p om9-app --lib -j 2 surface_snap` passed
20 tests with one timing diagnostic ignored; the two `surface_worker` tests in
`brep_picking_precision` passed, and `cargo check -p om9-app --lib
--no-default-features -j 2` passed with warnings. The separate full-face
triangle/trim integration failure documented above remains open; these
selected worker tests do not close it. UI surface refinement remains synchronous.

## Mixed NURBS and OCCT B-rep sources

An admitted mixed batch containing NURBS surfaces and OCCT B-reps now captures
each occurrence separately: copied surface and trim data for NURBS, and tracked
canonical B-rep source leases without display for OCCT. Face groups still use
the verified source-ray path. The existing candidate, scene, view, selection
and publication guards remain in force. NativeBrep exact surface snapping is
not implemented. Unsupported geometry or insufficient copy budget still uses
the tracked-document fallback; sources are currently captured per candidate,
so repeated sources are not deduplicated. No UI gesture was moved to this worker.

The gated fixture uses a plane and a box with two candidates on the same box
face. The source-only path selected the front box point, released both old
document and B-rep display after New while the worker was held, and rejected
its stale receipt. The low-copy-budget fallback returned the same front hit;
an admission arithmetic overflow was rejected. `cargo test -p om9-app --lib
-j 2 surface_snap` passed 21 with one timing diagnostic ignored; the two
`surface_worker` tests in `brep_picking_precision` passed. The final mixed
fixture was rerun after adding overflow handling. `cargo check -p om9-app --lib
--no-default-features -j 2` passed with warnings. The known full-face
triangle/trim failure was not reclassified; desktop, UI scheduling, native peak
and cold restore are unverified.

## Worker receipt lifetime and quota

After a successful source refinement, the worker now drops source copies and
leases before shrinking its kernel reservation to a conservative bound for the
retained candidates, result, weak document/native owner headers, job and
receipt. The job and receipt share the reservation: a retained job can still
cancel a delivered receipt, so its candidate and weak-identity storage remains
charged until the last owner drops. A source/reservation input owner preserves
drop order on thread-spawn failure; cancellation and error paths also drop the
source before releasing the charge. Weak native headers and shared candidates
may be counted more than once, and native allocator peak is not certified.

`cargo test -p om9-app --lib -j 2 surface_snap` passed 21 with one timing
diagnostic ignored. The gated trimmed-cylinder fixture checked source release,
quota shrink, both job/receipt drop orders, cancellation and stale receipt.
The two `surface_worker` and the `surface_scheduler` tests in
`brep_picking_precision` passed, including concurrent STL use and coalescing.
The no-default-features library check passed with warnings. The separate
full-face trim integration failure, UI scheduling and C1–C5 remain open.

## Admission and revalidation for UI integration

The job now exposes its request-specific Rust storage floor and, when the
selected geometry supports it, the floor for the source-only path. These do
not bound OCCT internal allocations and do not include independently registered
B-rep source lease charges. A completed, accounted receipt can be resolved
more than once, with the current document, scene, camera, cursor, selection and
preview checked on every use. The consuming `accept` path retains the same
checks. The scheduler can identify an unchanged in-flight request before a
caller reserves another job; it rejects a different scene publication even
when the document allocation and epoch are unchanged.

The mixed plane/box fixture successfully ran with exactly its calculated
source-only Rust floor, resolved the same receipt, and rejected a changed
cursor. A pending scheduler fixture rejected a replacement scene at the same
document and epoch. `cargo test -p om9-app --lib -j 2 surface_snap` passed 21
with one timing diagnostic ignored; the `surface_scheduler` and two
`surface_worker` integration tests passed. The no-default-features library
check passed with warnings.

These are worker/scheduler capabilities, not UI integration. Normal hover,
click and Gumball refinement remains synchronous; the UI must still reserve
native headroom, validate displayed-frame occlusion, and defer or cancel a
click safely if command/input changes while work is pending. Desktop, native
peak, the full-face trim failure and C1–C5 remain open.

## Line viewport integration

With OCCT enabled, an active Line command with an empty command draft submits
Surface Snap refinement to the existing source-only scheduler. While it waits,
the viewport does not substitute the construction-plane point. A click is
deferred and checked against command run, phase, options, plane, projection and
snap settings; new input cancels it. The result is revalidated for the current
source, scene, camera, cursor and selection, and shaded display occlusion is
checked before a point is accepted. Non-surface snaps retain their existing path.
The worker reserves the calculated Rust floor plus 8 MiB of managed headroom;
this is not a certified bound on native OCCT allocations.

An egui headless fixture deliberately occupies the worker permit, clicks an
OCCT sphere during Line, then releases the permit: the command advances with
the exact sphere point and clears the one-shot snap. Its second case introduces
text input while waiting and verifies the click is canceled and the command
phase is unchanged. Further cases change the pointer or camera while waiting
and likewise preserve the command phase. `cargo test -p om9-app --lib -j 2 surface_snap` passed 21
tests, with one timing diagnostic ignored; the new UI test passed separately.
The no-default-features library check and public source audit passed.

This is scoped integration, not completion of background snapping: other
commands and Gumball still use synchronous refinement; shaded occlusion may
also perform source checks on the UI thread. NativeBrep exact snap, native peak,
physical desktop/GPU, broader changed-view and real memory-pressure coverage remain
open. The previously recorded full-face curved-trim integration failure is
not fixed. Canonical Document still owns display and C1–C5 remain open.

## Deferred click display-mode guard

The deferred Line click now records its viewport and Wireframe/Shaded mode.
Changing the mode while the source worker is pending cancels the queued click;
completion also rechecks both fields before accepting a point. The headless
sphere fixture holds the worker permit, changes the mode, asserts the queued
click is immediately removed, then releases the permit and verifies that Line
does not advance. Its existing input, pointer and camera cancellation cases
also assert immediate removal. The focused test passed with OCCT enabled and
`-j 2`; the public source audit passed. This does not establish physical desktop
interaction or complete background occlusion processing.
