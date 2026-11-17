# Multi-entry cache admission — 2026-11-06

The shared coordinator previously bounded retries by the number of registered cache owners. A single topology cache can contain multiple independently reclaimable proofs, so an admission could fail after two releases even with another idle entry available. The new bound uses the initial total of accounted external reservations and the cache index, and retries after each actual release. It stops as soon as admission succeeds or the initial byte bound is reached; active cache entries still remain pinned by their owners.

On the current Windows worktree, with `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -p om9-core admission_reclaims_multiple_entries_from_one_owner -j 2 --lib` failed before the fix with `Budget("bytes")`, then the two `transaction::cache_pressure::tests` passed after it. Four separately reserved entries under one owner required three releases; an insufficient-cache case preserved an unrelated reservation.
- `cargo test -p om9-scene --test managed_picking_cache -j 2`: 15 passed, including pinned readers, concurrent reservations, recency and one-byte pressure.
- `cargo test -p om9-app --lib topology_remesh_pressure_reclaims_only_idle_other_proofs -j 2`: 1 passed.
- `cargo test -p om9-app --lib layer_and_property_edits_refuse_pressure_without_losing_scene_or_redo -j 2`: 1 passed. The app build emitted an existing unreachable-pattern warning in `addon_session.rs`.
- `python tools/public_source_audit.py`: no errors; `git diff --check`: passed.

This validates managed admission for these fixtures, not native OCCT peak, physical desktop presentation, or a global RSS limit. The concurrent-churn byte bound has not been stress tested. Sphere's document and Undo still retain display, and the production source/cache/remesh gate remains open. The source is the uncommitted working tree; no release executable or full suite is claimed.
