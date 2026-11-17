# Replaced scene buffer lease lifetime — 2026-11-06

The experimental remesh owner retained the previous scene's aggregate buffer
lease even when the replacement scene no longer used any original shared
buffers. For a one-root scene, the old lease consequently remained reserved
after the last old scene snapshot was dropped. The new regression reproduced
that retention before the fix.

When the replacement scene has zero original shared-buffer bytes, its owner
now releases its reference to the original lease. The old scene and its
snapshots still retain their own buffer leases; the charge is released only
after their last reference goes away. When another root still uses original
buffers, the replacement keeps the aggregate original lease. This is
deliberately conservative: for a partially replaced scene that aggregate may
still charge buffers no longer in use; no per-buffer accounting is claimed.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -p om9-app --lib topology_owner_tests -j 2`: 36 passed, two
  ignored (3DM exchange and release timing). The one-root regression failed
  before the fix and passed afterward: an old render snapshot pins the lease,
  and the weak lease expires after dropping that snapshot. The two-root test
  confirms the remaining original root keeps the lease until its controller
  drops. The group also exercises stale jobs, pressure, Undo and remesh.
- `cargo test -p om9-app --lib scene_buffer -j 2`: three passed, including
  one of the above regressions and existing detached-buffer/lease-chain tests.

The remesh implementation at this run had SHA-256
`d1d731c430eb5b65f018d8724893b30ff1e7c5d566390c13513ac98bd27b287c`.
These tests observe managed scene-buffer lease lifetime, not process RSS,
native OCCT peak or GPU fence lifetime. Remesh remains an experimental route;
the document and Undo still own the B-rep display mesh, and this change does
not enable production Sphere eviction or source-only history. The already open
OM9 session and drawing were not touched.

## Follow-up: two consecutive replacements

A stronger two-root fixture found that the first remeshed buffer's owner still
retained the aggregate original lease after the second root was also replaced.
That assertion failed with the earlier fix. The original lease now belongs to
the prepared/current scene's ordinary-buffer owner rather than to each remeshed
buffer's owner. Once both roots have been replaced, an old scene snapshot keeps
the lease alive; dropping that last snapshot releases it even while both new
remeshed buffers remain live. A still-unreplaced root continues to keep it.

`cargo test -p om9-app --lib topology_owner_tests -j 2`: 36 passed and two
ignored; the strengthened two-root snapshot test passed again separately.
`cargo test -p om9-app --lib incremental_scene_memory_tests -j 2`: 12 passed.
The updated `topology_remesh.rs` SHA-256 is
`c553842ac41282cb86bdf097e1276b0cdf4387840ce90307bbc7462edd1b527a`.
The original hash above identifies only the preceding version. While even one
original root remains, its aggregate lease can still overcount already replaced
buffers. This does not establish production Sphere eviction, per-buffer
accounting, GPU lifetime or native memory peak.
