# `OM9-L2-01-03` — Be Document Scene Entities: API contract

Milestone `M01` · Stage `E01` · Planning state `SPEC_ONLY_NOT_IMPLEMENTED` (tree not edited)
Audited working tree: `b5a15dae9ece3ba4a01e7c26fe60b051c9837e2a` (`crosswalk.json`)
Depends on: OM9-L2-01-08 (units, tolerance, float round-trip) and OM9-L2-01-02 (`ObserverId`,
`session/hosts.rs`) — both loops run first.

Status: **IMPLEMENTED (2026-10-17)** — all 13 L3 have code, fixtures and runtime tests. See §7 for the
implementation record, the per-L3 evidence and the recorded deviations (ADR-03-5, ADR-03-6). The
pre-implementation audit line "2 DONE_WITH_EVIDENCE (001, 006), 8 PARTIAL, 3 MISSING (007, 009, 011)"
is kept for history.

M00 (only QymCAD code reuse approved): **NEEDS_USER_M00_DECISION** for (a) persisting
`Document.groups` / `Entity.extensions` in `.om9` (archive format) and (b) the BRep kind (OCCT,
OM9-L2-01-05). If `.agents/tasks/m01-l2-01-02-03-backend/m00-decisions.md` already records the
user's answer (asked by the L2-01-08 loop), apply it; otherwise stop and ask before item 3.2.

## 1. Scope and exclusions

**In scope.** Stable IDs, typed entity envelope, attributes, layer records/membership/rules,
groups, block definitions and instances, the jewelry extension slot, the selection contract,
document change notification, document snapshot/restore and import provenance queries.

**Out of scope.** Any Layers management UI (L2 exclusion; the LAYERS panel position in the left
column is the concurrent UI workflow's concern); BRep geometry (OCCT); FeatureRef and gem payload
schemas (M04/M05 attach them through `Entity.extensions` without changing IDs); layer deletion (no
such operation exists; dangling membership is refused by validation — ADR-03-3).

## 2. Ownership

| Concept | Owner | Decision |
| --- | --- | --- |
| IDs, ledger | `om9-core/src/identity.rs`, `transaction.rs:477-583` | REUSE; add `GroupId`, `AllocatedIdentity::Group` |
| Entity, Attributes, Layer, Definition | `om9-core/src/model.rs` | EXTEND (`Entity.extensions`, `Document.groups`) |
| Mutations | `om9-core/src/transaction.rs:158-206` `Mutation` | EXTEND (`GroupCreate`, `GroupDelete`, `DefinitionInsert`, `Extension`) |
| Change diff | `om9-core/src/changes.rs` (new) | ADD (pure) |
| Provenance query | `om9-core/src/retained.rs` | EXTEND (read-only) |
| Snapshot bytes | `om9-archive/src/snapshot.rs` (new) reusing `Archive::encode/restore` | ADD (no second codec) |
| Unknown-kind diagnostic | `om9-archive/src/container.rs:264-272` | EXTEND (error mapping only) |
| Legacy key guard | `om9-archive/src/json_admission.rs:50-55` | EXTEND (only if persistence is approved) |
| Selection, notifications, snapshot restore, scene host | `om9-app/src/session/{selection,notify,snapshots,hosts}.rs` | ADD (non-UI) |

ADR-03-1: `EntityId` = existing `ObjectId`, `FeatureId` = existing `HistoryId`, `BlockDefId` =
`DefinitionId`, `InstanceId` = the `ObjectId` of an `Entity` with `Geometry::Instance`. No aliases.
ADR-03-2: `Document::insert(Entity)` / `insert_instance` / `assign_layer` / `restore` are realised as
`Mutation`s through `TransactionCoordinator` (one Undo entry each), not as direct `&mut Document`
methods, so every change keeps the ledger, permissions and atomic rollback.
ADR-03-3: there is no layer delete in M01; a document whose entity references a missing layer is
refused by `Document::validate` (`Missing("entity layer")`, model.rs:326). That is the explicit
refusal the negative oracle asks for.

## 3. Typed API

```rust
// om9-core/src/identity.rs
identifier!(.., GroupId);                                   // added to the existing list

// om9-core/src/model.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] #[serde(deny_unknown_fields)]
pub struct Group { pub id: GroupId, pub name: String, pub members: Vec<ObjectId>, pub revision: u64 }
impl Group { pub fn new(name: impl Into<String>, members: Vec<ObjectId>) -> crate::Result<Self>; }
    // members: >= 1, strictly ascending after sort, duplicates -> Invalid("duplicate group member")

pub const EXTENSION_GEM: &str = "om9.gem";      pub const EXTENSION_RAIL: &str = "om9.rail";
pub const EXTENSION_HEAD: &str = "om9.head";    pub const EXTENSION_CUTTER: &str = "om9.cutter";
pub const MAX_EXTENSION_PAYLOAD_BYTES: usize = 64 * 1024;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] #[serde(deny_unknown_fields)]
pub struct Extension { pub version: u32, pub payload: String }  // payload = JSON object text, kept verbatim
impl Extension { pub fn new(version: u32, payload: impl Into<String>) -> crate::Result<Self>; }
    // version >= 1; payload <= 64 KiB; trim_start starts with '{' and
    // serde_json::from_str::<serde::de::IgnoredAny>(payload) succeeds (syntax check, no DOM)

pub struct Entity { /* existing */,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, Extension>,           // slot -> record; unknown slots preserved
}
impl Entity {
    pub fn extension(&self, slot: &str) -> Option<&Extension>;
    pub fn unknown_extension_slots(&self) -> impl Iterator<Item = &str>; // not one of the four known
}
pub struct Document { /* existing */,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty",
            deserialize_with = "crate::identity::unique_uuid_map")]
    pub groups: BTreeMap<GroupId, Group>,
}
impl Document {
    pub fn groups_of(&self, object: ObjectId) -> impl Iterator<Item = &Group>;
    pub fn instance(&self, definition: DefinitionId, placement: Transform, layer: Option<LayerId>)
        -> crate::Result<Entity>;                           // Missing("definition") / placement.validate()
}
// validate(): groups (key == id, non-nil, revision > 0, name(), members non-empty, strictly
// ascending, every member is a ROOT entity of this document); extension slot names match
// ^[a-z0-9]+(\.[a-z0-9_-]+)+$ and Extension::new rules. storage_usage_internal AND validate_internal
// both charge the new owned storage (group map/name/members, slot strings, payload capacity) -
// validate_internal ends with debug_assert_eq!(total, storage).

// om9-core/src/transaction.rs
pub enum Mutation { /* existing */,
    GroupCreate { document: DocumentId, group: Group },        // ledger: GroupId never reused
    GroupDelete { document: DocumentId, group: GroupId },      // Ungroup: entities untouched
    DefinitionInsert { document: DocumentId, definition: Definition },
        // ledger: DefinitionId and every member ObjectId unallocated; member geometry/label/placement
        // validated; member layers exist; staged validate refuses Invalid("definition cycle")
    Extension { source: SourceStamp, slot: String, extension: Option<Extension> },
        // source mutation -> editable permission required; bumps attributes_revision; removing an
        // absent slot -> Missing("extension slot"); ObjectId and generation unchanged
}
// Mutation::Delete and Mutation::Move additionally remove the object from every group of the source
// document; a group left without members is removed (ADR-03-4, OM9 choice: an empty group carries
// no meaning). Groups and extensions never change geometry, placement or IDs.
```

```rust
// om9-core/src/changes.rs (new)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct EntityFields { pub geometry: bool, pub placement: bool, pub attributes: bool,
                          pub label: bool, pub extensions: bool, pub ownership: bool }
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Change {                         // declaration order = report order, then ascending ID
    Context, CurrentLayer, LayerOrder,
    LayerAdded(LayerId), LayerRemoved(LayerId), LayerModified(LayerId),
    EntityAdded(ObjectId), EntityRemoved(ObjectId), EntityModified { id: ObjectId, fields: EntityFields },
    DefinitionAdded(DefinitionId), DefinitionRemoved(DefinitionId), DefinitionModified(DefinitionId),
    GroupAdded(GroupId), GroupRemoved(GroupId), GroupModified(GroupId),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum ChangeCause { Commit, Undo, Redo, Replaced }
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeBatch { pub document: DocumentId, pub sequence: u64, pub publication_epoch: u64,
                         pub revision_before: u64, pub revision_after: u64,
                         pub cause: ChangeCause, pub changes: Vec<Change> }
pub fn diff(before: &Document, after: &Document, limits: &Limits) -> crate::Result<Vec<Change>>;
    // two-pass: count, admit Usage::array::<Change>(n) against limits, then fill; sorted output;
    // Context = page_unit, tolerance or context_revision differs; ownership = generation differs
```

```rust
// om9-core/src/retained.rs (read-only queries)
#[derive(Debug, Clone, PartialEq)]
pub struct Provenance<'a> { pub source: SourceId, pub namespace: ImportNamespaceId,
    pub source_name: &'a str /* Resource.name = file name only */, pub source_sha256: [u8; 32],
    pub archive_version: u32, pub native: NativeRecordKey, pub family: &'a str,
    pub disposition: RecordDisposition }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImportSummary { pub records: usize, pub bound_to_host: usize, pub retained_only: usize,
                           pub edited: usize, pub deleted: usize }
impl Document {
    pub fn provenance(&self, object: ObjectId) -> Option<Provenance<'_>>; // None for native-created
    pub fn import_summary(&self, namespace: ImportNamespaceId) -> crate::Result<ImportSummary>;
}
```

```rust
// om9-archive/src/snapshot.rs (new; pub use snapshot::DocumentSnapshot in lib.rs)
pub struct DocumentSnapshot { document: DocumentId, revision: u64, bytes: Vec<u8> }
impl DocumentSnapshot {
    pub fn capture(document: &Document, limits: &Limits) -> Result<Self>;
        // bytes = container::encode; decode(bytes) must equal *document, else Invalid("snapshot verification")
    pub fn from_parts(document: DocumentId, revision: u64, bytes: Vec<u8>) -> Self;
    pub fn document(&self) -> DocumentId; pub fn revision(&self) -> u64; pub fn bytes(&self) -> &[u8];
    pub fn restore(&self, expected: DocumentId, limits: &Limits) -> Result<RestoredDocument>;
        // expected != document -> Invalid("snapshot document"); Archive::restore(bytes, limits, &{})
        // (truncated/corrupt/version errors unchanged); decoded id/revision mismatch -> Invalid("snapshot identity")
}
// om9-archive/src/container.rs:264-272: a serde message starting with "unknown variant" maps to
// Unsupported("document variant") (a newer writer), never to Invalid or a silent downgrade.
```

```rust
// om9-app/src/session/selection.rs (new)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionState { pub document: DocumentId, pub owner_session: SessionId,
                            pub publication_epoch: u64, pub objects: Vec<ObjectId> /* ascending */ }
impl Controller {
    pub fn selection_state(&self) -> SelectionState;
    pub fn validate_selection(&self, s: &SelectionState) -> om9_core::Result<()>;
        // Stale("selection document") / Stale("selection epoch")
}
// Every publication (accept_world_point, apply_layer_actions, undo, redo, restore_snapshot) prunes
// Controller.selection to IDs still present in the document (ADR-03-5: only removed objects are
// dropped; hiding/locking does not deselect - that would be a new business rule).

// om9-app/src/session/notify.rs (new)
pub trait DocumentObserver { fn on_document_change(&mut self, batch: &ChangeBatch); }
pub const MAX_CHANGE_BATCHES: usize = 64;
impl Controller {
    pub fn add_document_observer(&mut self, o: Box<dyn DocumentObserver>) -> om9_core::Result<ObserverId>;
    pub fn remove_document_observer(&mut self, id: ObserverId) -> bool;
    pub fn document_changes(&self) -> impl Iterator<Item = &ChangeBatch>;
}
// Batches are computed from the previous and the published document Arc after the publication has
// completed, numbered by a per-Controller sequence (strictly increasing; publication_epoch also
// increases; revision_after may be lower than revision_before after Undo). Observers get &ChangeBatch
// only, so they cannot mutate the document (no reentrancy by construction); a panic is caught,
// recorded in observer_diagnostics and that observer is removed. Ring + observers survive New/Open.

// om9-app/src/session/snapshots.rs (new)
impl Controller {
    pub fn capture_snapshot(&self) -> om9_core::Result<DocumentSnapshot>;
    pub fn restore_snapshot(&mut self, s: &DocumentSnapshot) -> om9_core::Result<()>;
        // Busy while a command, file operation or layer dialog runs; s.restore(self.document().id)?;
        // candidate = Self::new_with_held(..) like replace_from_file (session/files.rs:426);
        // candidate keeps workspace, file path/saved hash, observers, sequences; *self = candidate;
        // Undo/Redo history is reset (same as File > Open, ADR-03-6); emits ChangeBatch{Replaced}.
        // Any error leaves the current controller untouched.
}

// om9-app/src/session/hosts.rs (extends the L2-01-02 file)
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentSceneState { pub document: DocumentId, pub owner_session: SessionId,
    pub revision: u64, pub publication_epoch: u64, pub scene_epoch: Option<u64>,
    pub scene_objects: usize, pub entities: usize, pub layers: usize, pub views: Vec<ViewId> }
impl Controller { pub fn document_scene_state(&self) -> DocumentSceneState; }
impl crate::workbench_regions::ViewportSceneHost for Controller {
    fn visible_views(&self) -> Vec<ViewId> { self.workspace.visible_views() } // identical to the Workspace impl
}
```

## 4. Persistence gate (NEEDS_USER_M00_DECISION)

With serde `default` + `skip_serializing_if`, documents without groups/extensions (all existing
files) encode byte-identically. Persisting non-empty `groups`/`extensions` is a format decision:

- **A (recommended)**: additive optional keys in schema 4; add `(Document,"groups")`,
  `(Document,"tolerance")`, `(Entity,"extensions")` to the legacy-key guard
  (`json_admission.rs:50-55`) so schema 1-3 files carrying them are refused; older OM9 builds refuse
  new files explicitly (`deny_unknown_fields`). No version bump, no migration.
- **B**: bump to schema 5 with an explicit `migrate_from_v4` (larger change in om9-archive).
- **C**: keep in memory only; `Archive::encode` refuses a document with non-empty groups/extensions/
  tolerance with `Unsupported("<field> is not persisted yet")` (explicit, never silent loss).

The coder implements §3 in memory first, then applies the recorded answer.

## 5. Test design (RED first)

| Test file | Oracle |
| --- | --- |
| `om9-core/tests/document_graph.rs` (new) | dangling instance / missing parent / unknown assign target / dangling membership refused; two instances share one definition; self-cycle DefinitionInsert refused atomically; GroupCreate/Delete keep IDs and geometry; group with deleted or unknown member refused; Delete prunes groups; Extension set/remove keeps ObjectId, unknown slot preserved through serde |
| `om9-core/tests/document_changes.rs` (new) | `diff` order and fields; Layer create+current+Insert in one commit = one ordered batch |
| `om9-core/tests/provenance.rs` (new) | imported entity provenance (sha256, native id/index, source name); native entity None; summary counts |
| `om9-archive/tests/snapshot.rs` (new) | capture/restore keeps IDs, layers, revision and Line geometry bit-exact; other document, truncated bytes, wrong version refused; unknown variant -> Unsupported |
| `om9-scene/tests/instance_admission.rs` (extend) or `shared_instances.rs` (new) | L4-04: scene/picking of two instances of one definition, Undo removes all |
| `om9-app/tests/document_contract.rs` (new) | selection pruned after Undo, stale SelectionState after New/Open, change batches from Line/Layer/Undo, panicking DocumentObserver, restore_snapshot atomic, `document_scene_state`, `ViewportSceneHost for Controller` == Workspace |

## 6. Unverified

- `OpenMatrix9_Codex_Spec_v2/` absent; Matrix group semantics for a group emptied by deletion were not
  verified in a manual (ADR-03-4 is an OM9 choice to report).
- BRep envelope: not designed here (OCCT/OM9-L2-01-05, M00 not approved).

## 7. Implementation record (2026-10-17)

### 7.1 What exists now, per L3

| L3 | Owner symbol | Runtime test |
| --- | --- | --- |
| 001 IDs | `identity::FeatureId`/`GroupId`; `model::{Feature,FeatureKind}`; `Document.features`; `transaction::AllocatedIdentity::{Feature,Group}` | `om9-core/tests/document_graph.rs::l3_001_*` (3), `om9-archive/tests/document_identity_round_trip.rs` |
| 002 typed records | `geometry::GeometryKind::{tag,from_tag}`; `Container` decode maps an unknown variant to `Unsupported("document variant")` and `Brep` to `Unsupported("brep geometry")` | `document_graph.rs::l3_002_*` (2) |
| 003 handle + attributes | `model::GeometryHandle`, `DisplayAttributes`; `Document::{geometry_handle,resolve_handle,display_attributes,geometry_kind}` | `document_graph.rs::l3_003_*` (2) |
| 004 layer records | existing `Layer`; `Document::validate` layer-order/parent/key checks | `document_graph.rs::l3_004_*` (3) |
| 005 membership | existing `Attributes.layer`; `Missing("entity layer")` refusal | `document_graph.rs::l3_005_*` (2) |
| 006 visibility/lock | existing `Document::{layer_permissions,permissions_for_entity}` | `document_graph.rs::l3_006_*` (2) |
| 007 groups | `model::Group`, `Document.{groups,group,groups_of,prune_membership}`, `Mutation::{GroupCreate,GroupDelete}` | `document_graph.rs::l3_007_*` (5) |
| 008 blocks/instances | `Document::instance`, `Mutation::DefinitionInsert`, existing `Definition`/`member_transform` | `document_graph.rs::l3_008_*` (5), `om9-scene/tests/shared_instances.rs` (4) |
| 009 extensions | `model::{Extension,slot_name_valid,extension_slot_known}`, `Entity.extensions`, `Mutation::Extension`, `Document::unknown_extension_slots` | `document_graph.rs::l3_009_*` (6) |
| 010 selection | `selection::{SelectionState,SubshapeRef,SubshapePart}`; `Controller::{selection_state,validate_selection,prune_selection}` | `om9-core/tests/selection_state.rs` (7), `om9-app/tests/document_contract.rs::l3_010_*` (3) |
| 011 change notification | `changes::{Change,ChangeBatch,ChangeCause,diff}`; `session/contract.rs` observers, diagnostics, bounded ring; published on Insert/layer commit, Undo, Redo, New/Open, restore | `om9-core/tests/document_changes.rs` (6), `document_contract.rs::l3_011_*` (4) |
| 012 snapshot/restore | `om9-archive/src/snapshot.rs::DocumentSnapshot`; `Controller::{capture_snapshot,restore_snapshot}` | `om9-archive/tests/document_snapshot.rs`, `document_contract.rs::l3_012_*` (3) |
| 013 provenance | `provenance::{Provenance,ImportSummary}`; `Document::{provenance,has_source_proof,import_summary,namespace_summary}` | `om9-core/tests/provenance.rs` (4) |

Fixtures: `implementation/fixtures/M01/OM9-L3-01-03-00N/{positive,negative}.json` (13 pairs) freeze
the cases; each file names the runtime test that carries the oracle.

### 7.2 Deviations from this document

**ADR-03-5 (replaces ADR-03-1 for features).** The audit proposed `FeatureId = HistoryId` with no new
type. The implementation adds a real typed document feature: `FeatureId`, `FeatureKind` and
`Document.features`, created by `Mutation::FeatureInsert`. Reason: the L3-001 oracle requires a
feature identity whose allocation and lifetime are provable independently of the evaluation graph
(`Recipe`/`HistoryId` identity is owned by `om9-core/src/history.rs` and is exercised through
`Mutation::Publish`, a much heavier path). `Document.features` is additive and optional: a document
without features encodes byte-identically. `EntityId`/`BlockDefId`/`InstanceId` keep ADR-03-1 exactly
(`ObjectId`/`DefinitionId`/`ObjectId` of an `Geometry::Instance` entity). **Owner veto is possible
without touching anything else**: dropping `Document.features` and `Mutation::FeatureInsert` removes
the deviation and leaves the other 12 L3 untouched.

**Owner decision (2026-10-17): KEEP ADR-03-5 as implemented.** `FeatureId`, `FeatureKind`,
`Document.features` and `Mutation::FeatureInsert` stay exactly as delivered, including `Feature.outputs`
(no shrink to identity+kind). Reason recorded by the owner: it satisfies the L3-001 oracle inside this
module and the downstream M04/M05 records name `FeatureId` as a stable identity next to
`EntityId/GemId/LayerId`. Consequence for M04: the history `Recipe` that evaluates a feature must
reference the `FeatureId` it evaluates — the split is *identity/output refs live in `Document.features`,
evaluation lives in `HistoryGraph`* — and the drift guard (a feature's outputs equal its recipe's
outputs) is added when the first real producer exists, not before.

**ADR-03-6 (persistence: option A applied).** The three additive keys are admitted by the archive key
guard — `(Document,"groups")`, `(Document,"features")`, `(Entity,"extensions")` — with no schema bump
and no migration; a document carrying none of them is byte-identical to before, and the archive
golden tests stay green. `(Document,"tolerance")` was NOT added because OM9-L2-01-08 (units and
tolerance) is not implemented in this module's scope; adding the tolerance key without the document
field would be dead admission.

**Scene host.** `Controller::document_scene_state()` and `impl ViewportSceneHost for Controller` are
implemented in `session/contract.rs`; the layout seam is therefore no longer a stub. The command-input
half of the same seam (`CommandInputHost`, item 2.7 of OM9-L2-01-02) is NOT part of this module.

### 7.3 Verification (commands actually run)

```
cargo test -p om9-core           # 30 suites incl. document_graph (30), selection_state (7), document_changes (6), provenance (4)
cargo test -p om9-archive        # container admission, snapshot, identity round trip
cargo test -p om9-scene          # shared_instances (4)
cargo test -p om9-app --test document_contract
```
isolated target dir: `CARGO_TARGET_DIR=$env:TEMP\om9-target` (the workspace `standalone/target` is not
writable in this environment; the shared SDK policy is unaffected).

Evidence logs: `implementation/evidence/M01/OM9-L3-01-03-00N/`.

### 7.4 Known limitations

- `Brep` stays unsupported (`Unsupported("brep geometry")`); OM9-L2-01-05 is a declared dependency of
  this L2 in `L2.json` and is not implemented. Recorded as `BLOCKED_DEPENDENCY OM9-L2-01-05 (OCCT)`.
- OM9-L2-01-08 (typed units/tolerance/float round trip) is the declared dependency of this L2 and is
  NOT implemented here. This module's own scope does not consume unit types; the dependency is
  recorded, not satisfied. The one place it would have been consumed — persisting
  `Document.tolerance` — is deliberately left out (ADR-03-6).
- `crates/om9-scene/tests/tmp_probe.rs` is an empty (0-test) scratch target left by a delegated
  session. It could not be deleted from any session in this environment (`Access is denied` on
  delete/move for that path). It contains no test and no production effect.
