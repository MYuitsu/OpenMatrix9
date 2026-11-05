# B-rep display sharing and managed inventory — 2026-11-04

Editable B-rep clones now share the immutable, source-bound display snapshot using `Arc`, in addition to sharing canonical source. Document and retained-world inventories count a shared display allocation once, including vector capacities; independent equal snapshots remain separate. Undo attribution excludes a display also owned by the live document or a tracked capture. Clone estimates exclude this shared allocation. Copy-on-write modification does not alter another clone and fails existing fingerprint validation unless the snapshot is rebuilt through the normal binding boundary. The serialized `display` remains an ordinary JSON object; this change does not introduce v5 or remove display data from v4.

Default-feature debug tests used `E:\om9-surface-snap-validation`, `-j 2`, OCCT 8.0.1/TBB/jemalloc runtime paths, `CARGO_PROFILE_DEV_DEBUG=0`, and `CARGO_INCREMENTAL=0`:

- 89 core retained-owner tests passed. The added 50,000-point fixture checks same-document aliases versus separately allocated equal snapshots, a placement edit with capture and Undo/Redo, indexed/fallback inventory equality, and exclusion from Undo when the live document owns the snapshot. The expanded fixture was rerun successfully after the suite.
- One core B-rep validation test passed, including JSON shape and roundtrip, shared pointer identity, copy-on-write isolation, and fingerprint rejection of a mutated snapshot.
- 13 kernel unit tests, one archive B-rep source roundtrip, 19 B-rep picking precision tests, seven Gumball interaction tests and six object-info tests passed. `stress_instances` compiled; it was not run. An initial `om9-kernel --lib brep::tests` filter selected zero tests; the unfiltered 13-test kernel suite was then run.
- 27 serial offscreen GPU publication tests passed with `--nocapture`; no adapter skip was reported. They verify their existing fixtures, not desktop presentation, physical VRAM, or general performance.
- Public-source audit reported 15,145 source files and no errors after recording evidence. Artifact pruning removed zero runs, preserving cited runs. The initial `python` shim lacked an active version; both commands succeeded using the installed Python 3.12.10 executable.

These checks do not prove complete RAM owner accounting, process RSS or OCCT native peak limits. The display remains embedded in `Brep` and in the current archive, not yet a separately evictable display cache. Source/version/purpose meshing keys, v5 migration, large representative project and release/desktop validation remain open. GPU budget enforcement and complete batching remain separate open work.

## Snapshot handoff follow-up

`EdgeTopologySnapshot::into_display` now moves the existing buffers only when uniquely owned; it returns `Busy` when another strong owner exists instead of silently cloning a potentially large, unreserved mesh. The STL export's locally created snapshot propagates this error. The 14-test kernel suite passed with a new real OCCT box fixture checking pointer identity on a successful move and preservation of a shared snapshot after refusal. This closes the specific unreserved clone identified above; it does not bound native OCCT allocations or the remaining Rust export buffers.

The default-feature `om9-app --test stl_export` suite passed 60 tests in 117.98 seconds, including OCCT spheres, independent display/export profiles, NURBS, topology, captured source, cancellation and destination preservation. The specific `Busy` branch was tested at the kernel handoff, not injected into an actual file export. Public-source audit reported no errors; artifact pruning removed zero runs. No desktop or release-memory benchmark was performed.
