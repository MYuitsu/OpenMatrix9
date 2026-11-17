# Current remesh profile hit

An explicit controller remesh request can reuse the display already published
for the same root in the current scene. The scene records the exact bits of the
requested linear and angular deflections with the published remesh. A hit
requires one matching root, its current source stamp and scene receipt, an
unchanged coordinator/document allocation and a valid in-process topology
proof. The purpose is the viewport remesh; the proof and scene are scoped to
this running process and its loaded kernel. A malformed profile is rejected.

The hit does not launch a worker or reserve a new kernel/display/scene budget.
It still produces a pending completion for the existing `poll` contract,
returns Busy for a second request until polled, and is canceled by Undo or
scene replacement. `poll` rechecks the proof, document and scene publication.
A different profile or stale document takes the existing validation/worker
path. No persistent or multi-profile cache is introduced.

A production Sphere fixture initially failed because a repeat request started
another worker. It now observes a hit with unchanged scene publication and no
additional kernel reservation even with one-byte requested budgets. A changed
angle starts a worker, invalid input is refused, and Undo removes eligibility.
Additional fixtures cover a document move followed by Undo, pending-hit
cancellation, and rejection of duplicate scene roots. The app topology group
passed 48 tests with 2 ignored; scene unit tests passed 34; one offscreen GPU
remesh/picking test and one unattested-v4 Sphere fallback test passed. Builds
used `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`.

The scene scans its objects for this hit and stores profile metadata per scene
object; large-model latency and memory impact were not measured. This is not a
cache across profiles, documents or processes, nor proof that cache eviction
and regeneration work. Document and Undo still retain the original display;
native/process peak and desktop operation remain unverified. M0/M1 stay open.
