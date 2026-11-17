# ShapeSet source identity — 2026-11-06

The source-only ShapeSet restore previously returned a newly serialized OCCT B-rep. In a radius-3 sphere fixture, its bytes differed from the retained canonical source even though the existing volume, area and sampled face checks passed. This matters for source fingerprints and versioned ownership; geometric equivalence alone does not preserve source identity.

The private, internally paired packet path now independently rereads the retained source, checks that its bytes survive that read, and matches its topology descriptors and oriented incidence to the packet's verified references before returning a display bound to that source. The packet pairing permits this comparison even when OCCT writes different bytes for the packet root. Ordinary cross-source matching remains strict. Unsupported normalization, ambiguous descriptors, mismatched topology or foreign source is rejected rather than guessed. This path is still experimental and is not a document/Undo owner or archive format.

Windows worktree, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- The strengthened source-identity assertion failed before the correction. Afterward, the existing reread radius-3 sphere and a radius-5 primitive-constructor sphere both retained exact source bytes and topology ID sets across restore. A foreign radius-4 source was refused.
- `cargo test -p om9-kernel --lib brep::topology_capture::tests -j 2`: 9 passed, 3 ignored (timing and cross-process probes). Existing box/sphere tests include source-only display release, sampled face hits, measurements, corrupt packet and budget refusal.
- `cargo test -p om9-core --lib brep::topology_mapping::tests -j 2`: 3 passed, including ambiguity, foreign sources, orientation and under-budget refusal.
- `python tools/public_source_audit.py` and `git diff --check`: passed.

These are in-process test fixtures, not a production Sphere command, desktop, native peak measurement, universal topology certificate, or full-domain error guarantee. Document and Undo still retain the display; v4 fallback and the source/cache/Undo integration remain open. The source is the uncommitted worktree.
