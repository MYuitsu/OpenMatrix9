# Reclaimable previous remesh representation checkpoint

Follow-up: the public display-topology Arc escape described below was addressed after this checkpoint; see [display topology lease boundary](2026-11-13-display-topology-lease.md). Other limitations remain.

2026-11-13. Scoped C1/C2 transition; the plan's source-only document, history and cold restore are not complete.

The explicitly requested, attested root remesh path retains one previous profile in an optional Rust cache without a strong document reference. Its representation owns the existing display and buffer pins and their reservations, not a document or a native mutable shape. The key is the exact source stamp, profile bits and in-process topology proof. The current-profile and previous-profile hit paths also verify that the scene display's private remesh owner has the matching proof: matching profile or source alone is insufficient. For a previous-profile hit, the controller reserves scene metadata, validates document owner, proof and scene publication, and republishes existing buffers without a worker or a CAD transaction. It retains the formerly current representation as the next optional entry when possible. Cancellation, scene replacement and admission failure preserve the current document/history and do not publish the requested cached scene.

The optional cache participates in the existing recency/reclamation policy. It refuses to reclaim while the display, buffers or topology have strong or weak readers. It will not replace an entry while its topology has an independently held Arc or Weak; it skips retention of the newer optional entry instead. In the tested unpinned case, reported reclaimed bytes equal the reduction in the kernel plus CPU-cache reservations. Replacing the document while retaining a cache handle did not retain the old document in the fixture. Replacing a profile by the same profile does not retain a redundant old copy.

**Open ownership limitation:** `BrepDisplay.topology` is still a public independently clonable `Arc`. A detached topology can outlive the cache/controller itself, at which point the present representation reservation may be released without following that external Arc. The cache's pin checks cover reclamation and replacement while the cache lives, but are not a full lifetime solution for teardown or concurrent detached clones. Hide or redesign that API so topology cannot escape its accounting owner before treating this as a complete representation store. The document and Undo also still own their existing BRep displays. Do not infer full C1/C2, general eviction, multi-object cache or cold Undo support from this checkpoint.

## Validation

Default features, OCCT SDK runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app -j 2 --lib remesh_cache_ -- --nocapture`: 3 passed. Includes rejecting a fake same-profile scene owner and preserving an entry when topology is detached.
- `cargo test -p om9-app -j 2 --lib previous_remesh_representation_ -- --nocapture`: 1 passed. Switches profiles and confirms buffer identity, current scene, unchanged document/Undo/Redo, pinned and weak-topology refusal, and exact reservation decrease after final release.
- `cargo test -p om9-app -j 2 --lib stored_remesh_ -- --nocapture`: 2 passed. Covers budget refusal, cancellation, scene replacement and absence of an old-document strong reference.
- `cargo test -p om9-app -j 2 --lib remesh -- --nocapture`: 73 passed, 1 ignored; includes the above cases. `cargo test -p om9-scene -j 2 --lib`: 47 passed.
- `cargo check -p om9-app -j 2 --no-default-features`: passed with 21 warnings; it does not exercise OCCT. Public source audit: 15,990 source files, 594 icon bindings, 593 SVGs, no errors. `git diff --check` passed with CRLF notices; artifact pruning removed zero runs and retained 169 older cited runs.

No physical desktop/GPU, physical OOM, native peak, concurrency stress, imported legacy topology, NativeBrep, block/instance, or cold restore acceptance was established. Tests exercise the controller's explicit remesh API rather than a production UI quality control. Next: close the topology lease escape, then continue C0 dispositions and replace document/history display ownership with the planned canonical representation and guarded restore.

SHA-256: `brep.rs` `581a9a6bec969161350f6719982046827443046633a72c42d6cc1ba535c90418`; `scene.rs` `907344123e5a9620a4aaaca341df17311b6293d37ced92b8bf222439ed7da70a`; `session.rs` `4dcadd8823ade69174c71618192b810007345b57afb484493a7855eadd105127`; `topology_remesh.rs` `cb5fea96227e0f48ef78d222f3bb526fe6a926017a258b5b0f672f01b3dfc247`.
