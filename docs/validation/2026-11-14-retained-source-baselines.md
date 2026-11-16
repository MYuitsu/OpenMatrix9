# Retained import baselines and display independence

For newly bound root or definition-member B-rep and NativeBrep entities,
retained-import field baselines and the whole-entity binding use a versioned
source fingerprint. The separate display mesh is excluded; B-rep source and
topology, or NativeBrep payload and CRC, remain included. Placement,
attributes, label, revisions and extensions remain accounted. Authored mesh
geometry is still part of its own baseline.

Existing baselines with no version marker retain their old display-sensitive
interpretation; unknown versions are rejected. They are not silently upgraded.
New baseline records carry an extra serialized version field. Current code
reads both forms, but forward reading by older binaries that reject unknown
fields is not established; this is not the planned v5 migration. Retained
definition-level fingerprints were display-sensitive at this stage; see the
scoped follow-up below.

Windows debug, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-core -j 2 --test retained_entity_baselines -- --test-threads=2`: 17 passed. Authored B-rep and NativeBrep fixtures check display-only changes, source/payload edits, old-version interpretation, unknown-version refusal and serialized marker preservation.
- `cargo test -p om9-exchange -j 2 --test current_overlay --test current_overlay_review -- --test-threads=2`: 16 and 8 passed. An authored B-rep fixture survives a document snapshot restore after a display change; the source is retained, and a subsequent label edit requests only a label patch.
- `cargo test -p om9-archive -j 2 --test document_snapshot -- --test-threads=2`: 11 passed. `cargo test -p om9-core -j 2 --test retained_imports -- --test-threads=2`: 17 passed. `cargo test -p om9-exchange -j 2 --test import_capture -- --test-threads=2`: 12 passed.
- With `build/native-source-fixture/Release/om9-source-fixture.exe` and the existing `L:/om9-target/debug/om9-exchange-native.exe`, the ignored `real_source_import_keeps_native_geometry_shared_members_original_bytes_and_fresh_namespaces` test passed: one authored native fixture was imported twice, retaining original bytes and passing Undo/Redo. A separate attempted display test could not run its intended assertion: this fixture's B-rep is retained-only and has no displayed native B-rep root. No actual 3DM export/remesh acceptance is claimed.
- `cargo check --workspace -j 2` passed with two existing app warnings.

This change does not separate document or Undo display ownership, provide
legacy baseline migration, prove native payloads contain no display, or certify
arbitrary source topology. Native export, desktop,
cold representation restore and v5 remain open.

## Definition member follow-up

New imported definitions with a directly owned B-rep or NativeBrep member use
a separately versioned, streamed definition fingerprint. It includes ordered
member identities and source fingerprints, member placement/attributes and
metadata, plus definition identity, name and revision, without the members'
separate display meshes. Definitions without such a member retain the prior
format. A legacy unversioned definition remains display-sensitive and is not
silently upgraded. The new field can be rejected by older binaries; forward
compatibility and automatic legacy migration are not established.

`cargo test -p om9-core -j 2 --test retained_entity_baselines --test retained_imports --test definition_units -- --test-threads=2`: 21, 17 and 8 passed. Authored fixtures check a B-rep member's display change versus source, placement, order and definition-name changes; old and unknown versions are checked. The focused definition-content test also passed after adding an authored-mesh vertex change. `cargo test -p om9-exchange -j 2 --test current_overlay_review -- --test-threads=2`: 9 passed, including a shared-definition fixture that survives a document snapshot restore and retains the definition and member after a display-only change.

The named native import fixture test above was rerun and passed after the
catalog change; its B-rep is retained-only, so it does not establish actual
import/export behavior for a displayed B-rep member. `cargo check --workspace
-j 2` passed with two existing app warnings. This scope does not prove every
nested dependency, every native format, actual export, desktop, eviction or v5.
