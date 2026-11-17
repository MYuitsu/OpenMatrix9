# Unattested v4 Sphere fallback — 2026-11-06

An integration fixture creates a radius-5 Sphere through the product command,
substitutes a distinct but structurally valid face UUID while preserving its
source and display, and round-trips the document through the v4 archive. The
new controller retains that UUID and the original decoded display allocation.
An explicit request to the experimental remesh API is refused for missing
topology proof; document, CPU scene publication, Undo and Redo remain unchanged.
CPU face picking still returns the v4 face UUID from the retained display.
This checks that portable structural/source integrity does not itself become a
native topology attestation.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -p om9-app --test sphere_gumball_source -j 2`: two passed.
  After adding the CPU-pick assertion, the v4 fixture passed again separately.
- `cargo test -p om9-archive --test project_codec_acceptance existing_v4_brep_keeps_source_and_topology_after_roundtrip -j 2`:
  one passed for the existing independent v4 archive fixture.

This is an in-memory archive and headless controller/CPU scene test. The test
intentionally requests an API that has no default production caller; it does
not establish safe eviction, a production cache, disk lifecycle, desktop or
GPU behavior. The document and its history still strongly retain display.
The open OM9 session and drawing were not modified.
