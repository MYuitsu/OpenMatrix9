# History output checksum and derived display

The History output checksum now uses the immutable source and topology-binding
fingerprints for an OCCT B-rep, and payload bytes plus CRC for a NativeBrep;
neither path hashes its separate viewport mesh. An authored `Geometry::Mesh`
still hashes its vertices and faces. Other geometry keeps the prior JSON hash
path. OCCT source/topology fingerprints are portable integrity hashes, not
proof that a legacy topology ID corresponds to a native subshape. Topology
binding order remains significant. Opaque NativeBrep payloads have not been
proven free of embedded display data.

Cache entries have a hash version. An entry deserialized without that version
is a cache miss and must rebuild before publishing a checksum in the new format.
Entries also retain each output's generation and geometry revision; changing
either makes the cache miss. This relies on mutations correctly advancing those
document stamps; cache lookup does not rehash arbitrary in-place modifications.
This is a cache compatibility rule, not a `.om9` v5 migration or a change to
the saved B-rep format.

Windows debug, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-core -j 2 --test history_source_hash --test associative_rebuild -- --test-threads=2`: 2 checksum tests and 24 associative regressions passed. Fixtures check display edits versus source, topology, native payload, CRC and authored mesh changes. The focused `a_successful_rebuild_updates_geometry_published_result_and_cache` rerun also passed with legacy/missing version and output generation/revision refusal.
- `cargo test -p om9-kernel --features 3dm-brep -j 2 --test associative_history -- --test-threads=2`: 4 passed, including an attested real OCCT Sphere remesh with different triangle counts and unchanged output hash, plus the Sweep legacy-reference refusal retaining the old output.
- `cargo test -p om9-app -j 2 --lib leased_candidate_updates_a_reopened_history_feature -- --test-threads=1`: 1 passed for a reopened Signet History feature with the new cache entry structure.
- `cargo check --workspace -j 2` passed with two existing app warnings. `python tools/public_source_audit.py` found no errors; artifact pruning removed no runs and preserved 169 older cited runs.

These are focused checksum and cache checks. They do not prove fully independent
Document/Undo ownership, eviction, cold restore, all native payload semantics,
or all M01–M09 consumers. Other geometry hashing can still allocate a JSON
buffer, and the checksum itself is not an independent native topology proof.
