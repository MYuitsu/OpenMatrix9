# Display chord and exact hole ray

For the existing OCCT 2 mm radius through-hole fixture at display deflection
0.2 mm, a temporary diagnostic found a top-face triangle edge whose minimum
radius is approximately 1.994407594362 mm: its chord enters the exact hole by
about 0.00559 mm. The full-face verifier currently cuts each potentially
overlapping triangle against the trimmed face and compares outside area using
the much smaller vertex tolerance of 0.000001 mm. The observed chord overlap
explains why a normal tessellation can conflict with that strict policy; it
does not establish that every reported outside-area failure has the same cause.

## Same-source remesh probe

The authored drilled block was generated at deflections 0.2 and 0.1 mm,
saved to OM9, reopened, and independently re-tessellated from its reopened
source at the same deflection and angular setting 0.3. In both fixture runs,
the vertex arrays, face-indexed triangles and face/edge incidence arrays
matched exactly. Adding three boundary points and a forged triangle across
the hole changed both arrays; changing one face index also failed equality.
The targeted `hole_face_source_remesh_reconstructs_display_probe` test passed.

This is evidence for one authored geometry and pinned OCCT build, not a
general proof of deterministic meshing, persistent topology identity or
correctness of every source. At the probe stage no acceptance path had changed. A safe
product use would still need a typed source/binding proof, exact per-point
source testing, bounded admission and cancellation, all affected face-command
consumers, and a conservative path for display meshes that do not match.
The existing `indexed_face_refinement_respects_a_trimmed_through_hole` test was
rerun and still fails with `triangle crosses trim boundary` before snapping;
no full-face guard was relaxed.

## Captured-source worker exact-remesh binding

When strict face-display verification reports a triangle crossing a trim
boundary, the captured-source worker may now retry by tessellating the same
source at the display deflection and angular setting 0.3. It requires matching
source digest and canonical topology, exact vertices and face-indexed triangle
arrays, matching face/edge incidence and topology counts, and a witness that
is an actual triangle of the selected face. It then resolves that witness
uniquely against the source. Mismatch is refused; display triangles are not
treated as exact source points. The Rust capture uses the worker's admission
callback and checks cancellation, but a single in-flight native call cannot be
interrupted and native allocation peaks are not proven bounded.

On an authored, saved/reopened block with a round hole, the captured-source
API and `FaceVerificationJob` passed at deflections 0.1 and 0.2. A forged
triangle across the hole, altered face label, unrelated witness, canceled
request and admission denial during remeshing were rejected. Receipt reuse
after changing the selection was rejected. The separate false-point fixture
still rejects a ray inside the hole. A five-hole saved/reopened block exercises
the actual `ExtractIsocurve` worker: the strict synchronous verifier fails,
while CMD and menu each produce two center curves ending at -5, -2, 2 and 5,
with Undo/Redo. The full Isocurve suite passed 56 tests, four ignored; three
face-worker library tests passed.

The plain synchronous `verify_face_display` path is unchanged: the full
`brep_picking_precision` suite has 22 passed and one known failure at its
strict verifier. The new fallback is limited to the captured-source worker;
other profiles, imported geometry, all face consumers, desktop/GPU, real OOM
and native peak remain unverified. C1–C5 are not complete.

## Selected B-rep Isocurve verification always uses a worker

The selected-root B-rep face path without a viewport-ray receipt no longer
uses the 512-triangle threshold: it always starts the guarded face verification
worker. Tests now wait for worker completion and still reject forged display
triangles and mixed face bindings. The existing viewport-ray and NativeBrep
routes have separate behavior; at this stage extracting parameter bounds and
committing curves still included synchronous work.

The authored one-hole and five-hole blocks, respectively below and above the
former threshold, were saved and reopened. In both, the old strict verifier
refuses the selected top face, while CMD and menu each complete through the
worker and produce two center isocurves with endpoints -5, -2, 2 and 5;
Undo/Redo passes. Changing selection while the worker is pending rejects its
receipt and preserves the document and Redo. The Isocurve integration suite
passed 56 tests with four ignored; 21 matching app library/UI tests passed,
including managed-memory recovery. Native peak, physical OOM, desktop/GPU and
arbitrary inputs remain unverified. The plain synchronous kernel verifier's
existing test still fails, and C1–C5 remain open.

## Selected B-rep Isocurve bounds on the face worker

After verifying the selected face, the same worker now obtains its source UV
bounds and places the four values in the guarded face receipt. The UI accepts
them only after revalidating the same command, source, scene publication and
selection. The source lease and display pin are released before receipt
publication; the resumed path no longer repeats source validation or the
native bounds query for this selected-face route. The worker admits source
storage and a separate scratch allowance; the allowance is not a measured
upper bound on native OCCT allocations, and one native call cannot be
interrupted once started.

A test confirms the bounds query runs on a different thread, rejects the
receipt after a selection change, and injects an admission refusal at the
bounds phase without changing the transaction snapshot. The Isocurve suite
passed 56 tests with four ignored, and 22 matching library/UI tests passed;
the no-default-feature library check passed with warnings. These are headless
tests, not desktop/GPU or physical OOM. Viewport-ray and NativeBrep routes,
the later isocurve construction/commit, plain synchronous kernel verification,
native peak and C1–C5 remain open.

The new `displayed_hole_chord_does_not_authorize_a_source_surface_hit` fixture
finds an actual interior point of a displayed top-face triangle within the
exact circular hole. A fresh viewport candidate exists there, but synchronous
source refinement returns the specific missing-exact-intersection result;
the source-only `SurfaceSnapJob` also returns no result. This test exercises
one authored fixture and its headless candidate/worker path, not the desktop.

On Windows with `L:\om9-target`, `-j 2` and the configured OCCT SDK, the new
test passed. The full `brep_picking_precision` suite reported **21 passed, one
failed**: `indexed_face_refinement_respects_a_trimmed_through_hole` still
expects full-face verification to succeed and gets `triangle crosses trim
boundary`. The existing test was not weakened and the native guard was not
relaxed. A separate previous experiment had already established that this
failure persisted without the newer hole-centre witness; see
`2026-11-13-small-trim-hole-witness.md`.

The next implementation must distinguish a certified face binding from
per-point exact hit testing, or produce a conservatively bounded representation,
and audit every consumer before permitting approximate triangles. A loose
deflection-based threshold alone can authorize a fabricated triangle across a
small or irregular hole. C1–C5 and the full integration suite remain open.
