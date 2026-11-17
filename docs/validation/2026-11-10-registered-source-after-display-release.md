# Registered source after display release

An existing production remesh path uses a registered source-only lease and its
native topology proof. The expanded Sphere fixture captures that lease while
the document is current, performs Undo, discards Redo, and reclaims optional
cache. It confirms the old display allocation is gone while the tracked
source and proof remain alive. It then builds an OCCT candidate from that
source and maps it with `into_rebound_for_source`; the resulting Sphere
topology IDs and adjacency match the original fixture.

The fixture was subsequently strengthened to use coordinator kernel
reservations and the worker's live/pending admission pattern: a one-byte
reservation is refused, then a 16 MiB reservation permits the source-only
capture and rebound. This is a managed-reservation check, not a measurement
of native OCCT or process peak.

The app topology-owner group passed 48 tests, with two ignored. Public-source
audit and diff check passed. Builds used `L:/om9-target` and `-j 2`.

This corrects the broader prior inference that proof always requires the old
display: creating or rebinding the lease still checks the original document
and display, whereas an already captured lease can be used without them.
History does not yet retain this lease, and the test does not publish the
candidate as an Undo result, demonstrate eviction from a retained document,
measure native/process peak, or validate desktop behavior. M0 remains open.
