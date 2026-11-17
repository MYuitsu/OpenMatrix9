# OCCT History source signatures

`OcctTopology` now reads OCCT source geometry for B-rep face area/centroid,
edge endpoints/length and vertex position. The Ring preview worker obtains
the same class of properties from the native snapshot callback, while that
snapshot's source and topology are live. Its output allocation and name
construction are admitted and cancellation is polled between subshapes.
These are numerical OCCT measurements, not exact arithmetic.

Source-derived roles use `source:face:`, `source:edge:` and `source:vertex:`
to avoid silently treating old mesh-derived roles as source identity. An old
`edge:0` reference was explicitly refused by the OCCT Sweep test. Automatic
migration is **not implemented**; the existing relink API changes a fingerprint
only, not the role. Existing projects with those references may therefore need
an eventual explicit migration before recomputation can use them. Source index
names and geometric tolerances do not prove stable semantic identity across
every topology change or symmetric geometry.

On Windows, OCCT 8 SDK, debug target `L:\om9-target`, `-j 2`:

- `cargo test -p om9-kernel --features 3dm-brep -j 2 --test ring_family_brep -- --test-threads=2`: 10 passed. The Signet fixture compares worker and synchronous source names/measures and confirms equality after a remesh; the retained allocation and refusal checks pass.
- `cargo test -p om9-kernel --features 3dm-brep -j 2 --test ring_family_brep all_ring_source_signatures_match_the_synchronous_registry_policy -- --test-threads=1`: 1 passed, comparing the four default ring kinds' worker and synchronous names and measures (no edit or remesh in this test).
- `cargo test -p om9-kernel --features 3dm-brep -j 2 --test associative_history -- --test-threads=2`: 3 passed. Sweep checks source names after remesh, a source reference after profile edit, and refusal of a legacy role while retaining the previous output.
- `cargo test -p om9-app -j 2 --lib ring_builder_ -- --test-threads=1`: 3 passed, covering simulated Signet CMD, queued Apply, Undo/Redo, and Raised tab/cancel.
- `cargo test -p om9-app -j 2 --lib ring_preview_worker::tests -- --test-threads=2`: 15 passed, including the reopened Signet History update, lifetime, stale and cancellation checks.
- `cargo test -p om9-app -j 2 --test ring_family_panel -- --test-threads=1`: 5 passed for the synchronous controller/panel path.
- `cargo check --workspace -j 2` and `cargo check -p om9-app --no-default-features -j 2`: passed, with 2 and 21 existing app warnings respectively.

The synchronous registry's B-rep path also uses source measurements; non-B-rep
geometry retains its portable snapshot policy. The tests do not establish every
ring kind's naming correspondence after edits, desktop latency, arbitrary B-rep
topology, persistent topology proofs, or full UI migration. `NativeBrep` is
not converted by this change. The native source parser, integrator and snapshot
internal allocations are not fully bounded or interruptible. Document/Undo
still retain display and independent cache, cold restore and v5 remain open.

## Ambiguous source references

For `source:` roles, resolution and pre-publication rebuild now refuse when a
second source subshape of the same kind also falls within the reference's
fingerprint tolerance. A simulated graph fixture checks identical and nearby
signatures, preserves the old output and references on a refused rebuild, and
accepts a distinct candidate. An OCCT Sweep fixture rejects a deliberately
overwide tolerance and resolves after a small unambiguous profile edit.

Windows debug, `L:\om9-target`, `-j 2`: the core associative suite passed 25
tests and its focused ambiguity test passed after the nearby-candidate follow-up;
the OCCT associative suite passed 4 tests. This is a conservative check, not a
proof that source ordinals persist across topology changes. It uses the existing
signature metric and tolerance, including its face-area dimensional limitation;
reversed endpoints, near-symmetric cases outside that metric, manual legacy-role
migration, real ambiguous CAD geometry, UI/Undo and desktop remain open.
