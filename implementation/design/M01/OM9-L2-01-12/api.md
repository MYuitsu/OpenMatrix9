# `OM9-L2-01-12` — Be Native Project Codec / STL Export: API contract

Milestone `M01` · Stage `E02` · Target `standalone Rust OM9`
Audited working tree: `b5a15dae9ece3ba4a01e7c26fe60b051c9837e2a` (+ untracked crates; see `crosswalk.json`)
Specification source: `OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export`
Status: **9 active L3 DONE_WITH_EVIDENCE, 1 deferred to M07** (`crosswalk.json`, `l4-matrix.json`).

## 1. Scope and exclusions

**In scope.** A native, versioned project container that saves and reopens document identity,
layers, entities, visibility/lock state, B-Rep source plus display snapshot, mesh/cloud data,
definition/instance links, groups, features, history placeholders, embedded resources and explicit
legacy migration; atomic save with write-failure recovery; validated load that never publishes a
partial document; runtime restoration with unresolved-module diagnostics; and the STL unit/winding
policy used by the M07 export command.

**Out of scope.** A second 3DM reader (retained 3DM import stays in `om9-exchange`); STEP/IGES;
OCCT lifetime/ABI (OM9-L2-01-05); the document-to-mesh STL command and tessellation (M07,
`OM9-L3-01-12-008` is `DEFERRED_TO_M07_NOT_INCLUDED_IN_M01`); the jobbag/project-manager object
model (M01 L2-03-15; this codec only round-trips an opaque jobbag reference resource); UI.

## 2. Ownership (one owner per concept)

| Concept | Owner | Decision |
| --- | --- | --- |
| `Document`, `Entity`, `Layer`, `Group`, `Feature`, `Brep`, `Geometry`, `HistoryGraph`, `Resource` | `om9-core/src/model.rs`, `geometry.rs`, `brep.rs`, `history.rs`, `resource.rs` | REUSE_IN_PLACE |
| Schema-4 container encode/decode | `om9-archive/src/container.rs` | ADD (authored; no upstream copy) |
| JSON admission before typed decode | `om9-archive/src/json_admission.rs` | ADD |
| Atomic save and publication | `om9-archive/src/atomic_file.rs` + `om9-native-owner/src/verified_file.rs` | ADD / REUSE |
| Document snapshot | `om9-archive/src/snapshot.rs` | ADD |
| Runtime restoration diagnostics | `om9-archive/src/restore.rs` | ADD |
| STL scale/winding/watertight policy | `om9-exchange/src/stl.rs` | ADD |
| File lifecycle UI/command wiring | `om9-app/src/session/files.rs` | REUSE (`Archive::save`, `Archive::open_restored`) |

The pinned QymCAD paths (`project_file.rs`, `formats.rs`, `model.rs`, `shape/exchange.rs`) were read
as architecture references only. None of their code, container layout, version policy or error model
was copied; OM9 owns its own schema and migration decisions.

## 3. Public API

```rust
// om9-archive
pub struct Archive;
impl Archive {
    pub fn encode(document: &Document, limits: &Limits) -> Result<Vec<u8>>;
    pub fn decode(bytes: &[u8], limits: &Limits) -> Result<Document>;
    pub fn save(path: &Path, document: &Document, limits: &Limits, cancel: &Cancellation) -> Result<()>;
    pub fn open(path: &Path, limits: &Limits) -> Result<Document>;
    pub fn restore(bytes: &[u8], limits: &Limits, available: &BTreeSet<ReplayKey>) -> Result<RestoredDocument>;
    pub fn open_restored(path: &Path, limits: &Limits, available: &BTreeSet<ReplayKey>) -> Result<RestoredDocument>;
    pub fn migrate_from_v1/v2/v3(bytes: &[u8], limits: &Limits) -> Result<Migration>;
    pub fn upgrade_v1/v2/v3_restored_in_place(...) -> Result<RestoredDocument>;
}

pub struct DocumentSnapshot;   // capture / restore / to_json / from_json
pub struct ReplayKey { pub solver: String, pub solver_version: String, pub schema_version: u32,
                       pub command: String, pub command_version: u32 }
pub struct RestoredDocument { pub document: Document, pub diagnostics: Vec<String> }

// om9-exchange::stl
pub struct TriangleSoup { pub positions: Vec<[f32; 3]>, pub triangles: Vec<[u32; 3]> }
pub struct StlOptions { pub scale: f64, pub flip_winding: bool, pub strict: bool }
pub fn open_edges(soup: &TriangleSoup) -> usize;
pub fn write_binary_stl(soup: &TriangleSoup, options: StlOptions) -> Result<Vec<u8>, StlError>;
```

`decode` returns a *validated* document; `restore` additionally establishes runtime ownership and
reports missing replay modules. A caller that only reads a project uses `open`; only an interactive
session that will recompute history needs `open_restored`.

## 4. Container format (schema 4)

```
offset 0   8   magic "OM9\0\r\n\x1a\n"
offset 8   4   schema version, little-endian u32 (4)
offset 12  4   chunk count (metadata + resource chunks)
offset 16  4   chunk 0 kind = 1 (metadata)         [entry 0 begins at offset 16]
entry      72  kind u32 | flags u32 | id [16] | start u64 | length u64 | sha256 [32]
...            entry 0 is the metadata chunk; entries 1..n are resource chunks (kind 2)
table_end     JSON document metadata (UTF-8)
...           resource payload bytes, in ascending entry order, contiguous, no gaps
```

Rules enforced on write and read:

- Exactly one metadata chunk, zero-uuid identity, non-empty payload; its hash is recorded in the
  header.
- Resource chunks use the typed `ResourceId` bytes as their identity and the resource SHA-256.
  Payloads are embedded; the codec never resolves an external path and never opens a second file.
- Chunks are sorted by `(start, end)`, must start exactly at the table end and be contiguous; gaps,
  overlaps and unreferenced trailing bytes are `Invalid`.
- Flags/compression are `Unsupported` today; there is exactly one schema-4 chunk kind pair.
- The document is serialized deterministically for the same document (`encode(decode(bytes)) == bytes`).

`schema 1/2/3` are explicit migration inputs only (`migrate_from_*`, `upgrade_*_in_place`): an
unknown version is `Unsupported("OM9 schema version; no implicit migration")`, never guessed.

## 5. Save pipeline and failure semantics

`Archive::save`:

1. `document.validate(limits)` (typed model invariants, budgets).
2. `Archive::encode` into a bounded buffer.
3. `Archive::decode` of that same buffer and compare to the live document **before any file is
   created**. This proves the file a caller is about to write is exactly re-openable.
4. Reject a symlink/non-file/readonly destination.
5. Create a unique `.<name>.<uuid>.om9-tmp` staging file with `create_new`, `write_all`, `flush`,
   `sync_all`, then drop the handle.
6. Re-read the staging file, compare bytes and decoded document again.
7. Publish atomically: `fs::rename` on non-Windows, `om9_native_owner::VerifiedFile` fingerprint +
   rename on Windows.
8. Cancellation is checked before encoding, before file creation, before publication and inside
   `VerifiedFile`; a failure or cancellation drops the staging name and leaves the destination
   byte-identical.

`sync_all` covers file data, not the directory entry; the rename itself is the atomic publication
step and a post-rename cleanup error is never treated as a rollback.

## 6. Load pipeline and compatibility

`Archive::decode` validates in this order, before a document exists:

1. magic, exact schema version, input byte budget;
2. chunk count, table bounds, per-entry start/length overflow, contiguity/overlap/trailing bytes;
3. duplicate chunk IDs, entry flags, per-chunk SHA-256;
4. chunk kind/identity rules (one metadata chunk; resource IDs are typed UUIDs);
5. JSON structural admission (`json_admission`): duplicate keys, string/depth/node/allocation
   estimates before a DOM or typed value is built;
6. typed serde decode with explicit `Unsupported` for an unknown geometry/document variant (never a
   silent downgrade to a neighbouring kind);
7. legacy ownership guards (`schema < 2` has no resources; `schema < 3` has no native source
   ownership);
8. document manifest admission plus the full hydrated document admission;
9. resource payload hydration only when descriptor ID, length and SHA-256 all match;
10. final `document.validate(limits)`.

A failed load is a typed error and no document is returned. Integration keeps the currently open
world untouched while a candidate is read (`om9-app/tests/file_lifecycle.rs:320`, `:425`).

## 7. Restoration semantics

`Archive::restore` distinguishes *validated project data* from *runtime authority*:

- `Document::renew_session` gives each open a fresh session/epoch and rebinds only stamps that are
  still verifiably current; stale/foreign stamps stay stale.
- For each recipe in topological order, a missing `ReplayKey` marks the recipe
  `Unresolved(diagnostic)` and its outputs `Stale`. Geometry is retained and the diagnostic is
  returned; the codec never invents recompute support.
- Restoration admission reserves the diagnostic copies and traversal scratch before it allocates, so
  a budget failure leaves the input bytes intact and publishes nothing.
- Two restores of the same bytes produce distinct owner sessions.

## 8. B-Rep, mesh and instance encoding

- `Geometry::Brep(Brep)` stores the exact native source string plus a disposable display snapshot:
  deflection, points, triangles `[a,b,c,face]`, segments `[a,b,edge]`, topological vertices, face/
  edge/vertex UUIDs and face-edge adjacency.
- `Brep::from_native` binds `source_sha256` into both the source and the display and records a
  display fingerprint. `validate` rejects a mismatched source/cache, out-of-range or degenerate
  topology, nil or duplicate topology UUIDs, and non-finite points.
- The display snapshot is never treated as the shape: reopening keeps `Geometry::Brep`; a real
  `source_volume` measurement still comes from OCCT through `om9-kernel`.
- `Geometry::Mesh` stores vertices, 3/4-gons, optional normals/colors; `Geometry::Cloud` stores
  points/normals/colors/hidden.
- A definition owns member entities; an occurrence stores `Geometry::Instance(DefinitionId)` plus a
  placement. Document validation requires the definition to exist and rejects definition cycles.

## 9. History and feature placeholders

`Feature` (annotation/dimension/construction/semantic) and `Recipe` (command, replay environment,
inputs, options, outputs, policy, state) are persisted as typed records. A recipe whose module is not
registered in the current build is preserved on disk and reported at runtime restoration; an unknown
persistent feature kind is refused with `Unsupported("document variant")` rather than dropped or
reinterpreted.

## 10. STL units and winding policy

STL carries neither a unit nor topology, so both are explicit:

- `StlOptions.scale` is millimetres per model unit. `1.0` writes millimetres; `1.0/25.4` writes
  inches. The 80-byte STL header states the authoring scale.
- `flip_winding` swaps two vertices per triangle; normals are computed from the final winding with a
  right-handed cross product, so the normal always agrees with the visible side.
- `strict` (default) refuses open surfaces (`open_edges`), degenerate triangles, bad indices, bad
  scale and non-finite coordinates before a byte stream is produced. The lenient path still refuses
  non-finite coordinates.
- The real document-to-mesh export command, tessellation tolerance and file selection are M07
  (`OM9-L3-01-12-008` deferred).

## 11. Invariants

- No partial document is returned by `decode`/`open`; no partial file is published by `save`.
- Durable identity (document, layer, object, definition, group, feature, resource, recipe) is never
  regenerated by save, open, migration or restoration.
- Cost is admitted (bytes, objects, nodes, points, history, outputs) before allocation; limits are
  explicit and carried through nested calls.
- The codec is `#![forbid(unsafe_code)]`; native work is limited to the Windows verified-file
  publication and the OCCT adapter, each behind a typed safe API.
- Unknown schema, variants or modules fail closed with a typed diagnostic, never by downgrading data.

## 12. Residual risks / review notes

- No external reviewer has signed this record; automated target evidence is complete.
- `Om9 archive` snapshots are JSON envelopes carrying the container as hex; a future binary envelope
  needs a version decision, not an implicit change.
- Real B-Rep volume depends on the OCCT toolchain; the archive-level fixture proves display volume
  and byte equality, while `om9-app/tests/brep_selection.rs` proves real OCCT volume 6000 mm3.
- M07 STL export still needs tessellation, unit selection UI and file writing; this module only owns
  the policy codec.
- The jobbag reference in the roundtrip fixture is an opaque embedded resource because the jobbag
  object model belongs to M01 L2-03-15.
