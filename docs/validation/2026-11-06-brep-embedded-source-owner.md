# B-rep source owner in the production model — 2026-11-06

The existing `BrepSource` value groups the shared immutable source string, source fingerprint and topology owner without display triangles. `Brep` now contains that existing value directly; `validate_source` reads it without constructing a temporary owner, and tracked captures clone it under the existing lease/accounting rules. Document and Undo snapshots therefore use the same grouped source representation. Entity placement remains owned separately by the document. This is a representation refactor: the display is still strongly owned by `Brep`, and portable source validation does not attest native topology-to-ID provenance.

The B-rep serializer explicitly writes the original four v4 fields in their original order; its regression compares the complete JSON bytes against a legacy-layout fixture. Deserialization keeps the v4 display/topology fallback. No new file version or native bridge was added.

Windows worktree, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-core --lib brep -j 2`: 23 passed, including shared source and display accounting across document, capture and Undo, independent allocations, bounded refusal and v4 bytes.
- Archive tests `brep_source_admission`, `document_snapshot`, `project_codec_acceptance`: 1, 11 and 8 passed respectively, including v4 B-rep roundtrip and failure admission.
- `cargo test -p om9-kernel --lib brep::topology_capture::tests -j 2`: 9 passed, 3 ignored; `cargo test -p om9-app --lib topology_owner_tests -j 2`: 32 passed, 2 ignored. These latter tests still exercise the experimental remesh path.
- `cargo test -p om9-app --test solid solid_sphere -j 2`: 54 passed, covering controller, menu/CMD and geometry fixtures. Public-source audit and `git diff --check` passed; existing warnings remain.

This does not release display from document or history, prove source/topology native binding for v4, integrate the ShapeSet owner into production, measure native peak, or validate desktop interaction. The Sphere source/cache/Undo gates remain open; no open OM9 session was changed.
