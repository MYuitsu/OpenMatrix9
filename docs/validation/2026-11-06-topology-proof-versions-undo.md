# Versioned topology proofs and Undo

The existing reclaimable topology-proof cache can now retain more than one
snapshot for the same root object. Its deduplication key compares document,
session, object generation and source/topology/display fingerprints; a match
is only a cache filter, and remesh still performs the existing current-owner
and proof validation. Entries from older geometry in the same document session
remain optional and can be reclaimed. Lookup returns only a proof matching the
current entity, so an old version cannot authorize a new source.

The existing Sphere placement/Undo fixture was extended with a replacement
from radius 5 to radius 6. Before this change, Undo failed to find the old
proof; after it, Undo found the matching proof and remeshed, while Redo did not
return that proof for the different geometry. A second fixture adopted proofs
for both versions of one Sphere and confirmed that Undo/Redo select the correct
allocation and both versions remesh. With the current proof pinned, reclamation
released the inactive old proof; Undo still restored the original document
geometry but its proof was no longer available. This is cache behavior, not a
guarantee that every history state retains a proof.

With `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`, the app topology-owner group
passed 38 tests with 2 ignored; the strengthened two-version reclamation test
passed again. Kernel topology capture passed 10 tests with 3 manual probes
ignored using `3dm-brep`. No desktop or native peak measurement was made.

This change does not make Sphere's source attestation durable, evict its
document/Undo display, add automatic remesh, or establish v4 provenance.
Entries can remain until pressure reclaims them, and cache lookup is linear in
retained proof versions; latency and memory behavior for long editing histories
are unmeasured. The source/display, managed peak and Sphere acceptance gates
remain open.

Follow-up: an attempted scene-publication pruning rule was withdrawn because
another document allocation can retain an equivalent version; see
`2026-11-06-topology-proof-retired-branch.md`.
