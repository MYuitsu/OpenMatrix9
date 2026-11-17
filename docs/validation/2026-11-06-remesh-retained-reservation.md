# Remesh retained reservation

The topology remesh worker previously held its full requested kernel allowance
until the last result/scene owner was dropped, even after all construction
scratch was gone. The worker now reconciles and shrinks that lease after the
candidate and optional display buffers are built. It retains the candidate's
source, topology, display and descriptor capacities plus a receipt-envelope
allowance; the proof and prepared display buffers retain their own leases.
Scene snapshots continue to hold the result's lease for their lifetime.

Follow-up: after publication the scene now drops the temporary candidate
source and descriptors and retains a display/topology-only charge; see
`2026-11-06-remesh-scene-display-owner.md`.

The new fixture requested 16,777,216 bytes and, before the change, found that
the whole reservation remained held although candidate storage and manifest
were 6,894 bytes. After the change it verifies that the live receipt holds the
candidate capacity plus the receipt-envelope allowance, less than the request,
and releases the charge when dropped. The topology-owner group passed 39 tests
with 2 ignored, including existing two-publication snapshot and retained-lease
cases; 12 incremental scene-memory tests passed. All used
`CARGO_TARGET_DIR=L:/om9-target` and `-j 2`.

This is managed-account evidence for the tested fixture, not a native OCCT,
channel allocator or process-peak bound. The envelope allowance is conservative
and can overlap owner metadata charged elsewhere; strict global peak and exact
allocator overhead remain unproved. The document and Undo still own the
original display, proof remains optional, and no display eviction or desktop
measurement was established.
