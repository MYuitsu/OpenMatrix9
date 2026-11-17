# Production Sphere topology proof and opt-in remesh

`solid_sphere_points`, used by the normal Sphere completion paths, now attempts
to capture topology descriptors with the same admitted native snapshot as the
Sphere. It transfers the proof reservation separately and adopts it into the
existing reclaimable cache after a successful commit. If optional construction
or a qualifying pre-publication admission/commit attempt fails, the original
admitted Sphere path is retried; failure to adopt the optional proof after
commit does not report a failed command. Box is unchanged. This does not
request remesh automatically or expose a new user-facing quality control.

The proof cache keeps an idle proof for an object temporarily absent from the
same document/session, permitting Undo/Redo reuse when it survives eviction.
Lookup filters absent or changed objects. The existing full revalidation and
stale-worker checks still apply before use; an idle proof remains reclaimable.

The new production-Sphere test failed before the change because no proof was
available. It now creates a translated Sphere through the production method,
requests remesh, checks publication and unchanged document, undoes creation,
redoes it, then requests a second remesh. A forced optional-proof refusal
falls back to a single Sphere transaction with Undo/Redo. A separate two-root
fixture confirms absent proofs are not returned, another live proof remains
findable, and an absent proof can be reclaimed before Undo. Existing pressure
fixtures exercise snapshot refusal and retry. The `sphere` unit filter passed
13 tests, the topology-owner group passed 36 with 2 ignored, and the separate
absent-proof reclamation fixture passed. The Solid integration suite and
no-default-feature check are reported in the live ledger.

This is only a partial M0 integration. The proof is optional and may be
reclaimed or fail to be adopted, so a later remesh request can be refused;
there is no proof restoration from saved v4 files or after cache eviction.
Document and Undo still own the original display and cannot evict it. The test
does not prove desktop presentation, automatic scheduling, native OCCT peak,
large-scene latency, or the full Sphere acceptance gate. Earlier small release
measurements of a different experimental constructor are not a current
production performance measurement.
