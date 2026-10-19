# `OM9-L2-01-02` — Be Command Dispatch Undo Redo: API contract

Milestone `M01` · Stage `E01` · Planning state `SPEC_ONLY_NOT_IMPLEMENTED` (tree not edited)
Audited working tree: `b5a15dae9ece3ba4a01e7c26fe60b051c9837e2a` + uncommitted UI edits (`crosswalk.json`)
Depends on: OM9-L2-01-08 (typed `LengthMm`/`AngleDeg`, `OptionDomain::check`) — implemented first.
L4 IMPLEMENT tasks also list `OM9-L4-01-03-001-05` (Document Entity IDs): that L3 is
DONE_WITH_EVIDENCE today (see `../OM9-L2-01-03/crosswalk.json`), so the order 08 → 02 → 03 is valid.

Status: 3 DONE_WITH_EVIDENCE (006, 007, 008), 7 PARTIAL, 1 MISSING (010). Oracles: `oracles.json`.
M00: nothing in this module needs OCCT/UI/3DM/Associative/License/Reproducible approval.

## 1. Scope, exclusions, single engine

**In scope.** One command registry (IDs, Matrix names, aliases), typed descriptors, command context,
the point-input state machine, the CMD option grammar, repeat, availability, command events, and
the real CMD-host state query that `OM9-L3-01-01-002/-003` need.

**Exclusions.** No second command engine (L2 exclusion). `om9_app::session::Controller` remains the
only dispatcher; menu, mouse, F6 and CMD keep calling the same functions. No UI file is edited
(`ui.rs`, `ui/**`, `dock_layout.rs`, `workbench_regions.rs` belong to a concurrent workflow); the UI
stub swap is described in `.agents/tasks/m01-l2-01-02-03-backend/stub-integration-note.md`.

**Already done (reuse, do not rewrite).** Atomic transactions, Undo/Redo journal and Cancel live in
`om9-core::transaction::TransactionCoordinator` and are proved by the tests listed in
`crosswalk.json`. ADR-02-1: the proposed `Document::apply(Command)`, `UndoJournal::undo` and
`CommandSession::cancel` are realised by `TransactionCoordinator::{prepare, commit, undo, redo}` and
`Controller::{cancel, undo, redo}`; no wrapper with a second name is added.

## 2. Files (all non-UI, inside the existing owner)

| File | Content |
| --- | --- |
| `om9-app/src/session/registry.rs` (new, `pub mod registry` in `session.rs`) | `CommandId`, `CommandDescriptor`, `CommandRegistry`, `RegistryError` |
| `om9-app/src/session/options.rs` (new, `pub mod options`) | descriptor-driven option grammar |
| `om9-app/src/session/events.rs` (new, `pub mod events`) | `CommandEvent`, `CommandObserver` |
| `om9-app/src/session/hosts.rs` (new, `pub mod hosts`) | `CommandStep`, `CommandPhase`, `CommandContext`, `CommandHostState`, `Unavailable`, `impl CommandInputHost for Controller` |
| `om9-app/src/session.rs` (edit) | Controller fields + dispatch through the registry |
| `om9-app/src/session/files.rs` (edit, small) | carry observers/event sequence across New/Open replacement |
| `om9-core/src/request.rs` | `OptionDomain` gains `Serialize` (done by L2-01-08) |

`lib.rs` is NOT edited (all modules hang under `session`).

## 3. Registry and descriptor (L3-001, L3-002)

```rust
// om9-app/src/session/registry.rs
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct CommandId(String);          // ^OM9-[A-Z0-9]+(-[A-Z0-9]+)+$, <= 64 bytes; all 607 catalog IDs conform
impl CommandId { pub fn parse(s: &str) -> Result<Self, RegistryError>; pub fn as_str(&self) -> &str; }

pub const UNDO: &str = "OM9-EDIT-UNDO";  // OM9-owned IDs: Undo/Redo have no catalog ID today.
pub const REDO: &str = "OM9-EDIT-REDO";  // CMD names "Undo"/"Redo" are unchanged.

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum CommandKind { Line, Point, Undo, Redo, File(FileKind /* + Serialize derive */), Catalog }
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum InputSpec { Points { count: usize } }                 // Line 2, Point 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum OptionPhase { AnyStep, BeforeFirstPoint }
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OptionSpec {
    pub name: &'static str,             // "BothSides"
    pub initial: char,                  // 'B' (the underlined initial)
    pub domain: om9_core::request::OptionDomain,
    pub phase: OptionPhase,             // Line BothSides: AnyStep (current behavior kept)
    pub invalid_value: &'static str,    // "BothSides Yes/No" -> keeps today's message text
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CommandDescriptor {
    pub id: CommandId,
    pub name: String,                   // catalog display name ("Line", "Save As", "PolyLine")
    pub command: Option<String>,        // catalog command name ("Line", "Polyline")
    pub aliases: Vec<String>,           // builtins only, e.g. "SaveAs"
    pub module: String,
    pub kind: CommandKind,
    pub inputs: Vec<InputSpec>,
    pub options: Vec<OptionSpec>,
    pub output: Option<om9_core::geometry::GeometryKind>,
    pub repeatable: bool,               // Line, Point: true; Undo, Redo, File: false
    pub implemented: bool,              // false for every catalog-only entry
}
#[derive(Debug, Clone, Deserialize)]
pub struct CatalogEntry { pub id: String, pub name: String, pub command: Option<String>, pub module: String }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    InvalidId(String),
    DuplicateId(CommandId),
    DuplicateAlias { alias: String, first: CommandId, second: CommandId }, // between IMPLEMENTED entries
    Unknown,                                                              // no ID / name / alias
    NotImplemented { candidates: Vec<CommandId> },                        // known, none implemented
    Ambiguous { name: String, candidates: Vec<CommandId> },               // >= 2 implemented match
}
impl From<RegistryError> for om9_core::Error;
// Unknown and NotImplemented from a TYPED NAME -> Unsupported("unknown/unavailable command") (today's text)
// Unknown/NotImplemented from an ID       -> Unsupported("command has no accepted standalone operation")
// Ambiguous -> Invalid("ambiguous command name"); build errors -> Invalid("command registry")

pub struct CommandRegistry { /* BTreeMap<CommandId, CommandDescriptor>, BTreeMap<String, Vec<CommandId>> */ }
impl CommandRegistry {
    pub fn matrix9() -> Result<&'static CommandRegistry, RegistryError>;
        // OnceLock; catalog = include_str!("../../assets/features.json") (the same asset ui.rs reads);
        // builtins (Line, Point, Undo, Redo, New, Open, Save, SaveAs) REPLACE the catalog entry with the
        // same ID and keep its catalog name/module; Undo/Redo are added with the IDs above.
    pub fn build(catalog: &[CatalogEntry], builtins: Vec<CommandDescriptor>) -> Result<Self, RegistryError>;
    pub fn by_id(&self, id: &str) -> Result<&CommandDescriptor, RegistryError>;
    pub fn resolve_name(&self, typed: &str) -> Result<&CommandDescriptor, RegistryError>;
        // ASCII case-insensitive exact match on name, command and aliases (today's matching);
        // one implemented candidate -> it; >= 2 -> Ambiguous; 0 implemented -> NotImplemented; none -> Unknown
    pub fn descriptors(&self) -> impl Iterator<Item = &CommandDescriptor>; // ascending CommandId
    pub fn to_json(&self) -> om9_core::Result<String>;                     // deterministic
}
```

The registry is fixed process-lifetime catalog data (like the menu catalog in `ui.rs`), not document
state, so it is not charged to document `Limits`.

## 4. Context, state machine, CMD host (L3-003, L3-004, unblock seam)

```rust
// om9-app/src/session/hosts.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStep { AwaitPoint { index: usize, count: usize } }   // Line 0/2 then 1/2; Point 0/1
#[derive(Debug, Clone, PartialEq)]
pub enum CommandPhase { Idle, Running { command: CommandId, step: CommandStep } }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalOwner { FileOperation, LayerDialog }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandHostUnavailable { SceneUnavailable }   // Controller.scene is None (refresh_scene failure)
#[derive(Debug, Clone, PartialEq)]
pub enum CommandHostState {
    Ready { phase: CommandPhase, modal: Option<ModalOwner> },
    Unavailable(CommandHostUnavailable),
}
#[derive(Debug, Clone, PartialEq)]
pub struct CommandContext {                     // captured once at dispatch, read-only afterwards
    pub document: DocumentId, pub owner_session: SessionId,
    pub publication_epoch: u64, pub context_revision: u64,
    pub view: ViewId, pub plane: CPlane, pub unit: Unit,
    pub selection: Vec<ObjectId>,               // sorted copy of Controller.selection at dispatch
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unavailable { UnknownCommand, NotImplemented, CommandRunning, ModalOwner(ModalOwner),
                       NothingToUndo, NothingToRedo, SceneUnavailable }
impl From<Unavailable> for om9_core::Error;
// Unknown/NotImplemented -> Unsupported("command has no accepted standalone operation"),
// CommandRunning/ModalOwner -> Busy, NothingToUndo -> Missing("Undo entry"),
// NothingToRedo -> Missing("Redo entry"), SceneUnavailable -> Missing("command scene")  (today's texts)

impl Controller {
    pub fn command_phase(&self) -> CommandPhase;
    pub fn command_context(&self) -> Option<&CommandContext>;            // Some only while running
    pub fn command_host_state(&self) -> CommandHostState;
    pub fn command_input_ready(&self) -> bool;                            // !Unavailable
    pub fn command_availability(&self, id: &str) -> Result<(), Unavailable>;
    pub fn availability(feature: &str) -> om9_core::Result<()>;           // EXISTING static, signature kept;
                                                                          // now registry.by_id(..).implemented
}
impl crate::workbench_regions::CommandInputHost for Controller {
    fn command_input_ready(&self) -> bool { Controller::command_input_ready(self) }
}
```

`CommandSession` (session.rs:30-38) is rewritten around `command: &'static CommandDescriptor`,
`context: CommandContext`, `start: Option<Point3>`, `both_sides: bool`; `validate_session` compares
`context.{document, owner_session, publication_epoch, context_revision}` with the live document and
coordinator (same `Stale("command document/context")` text). Prompts are unchanged.

`command_input_ready()` is `false` only when the controller cannot publish any command result
(`scene == None`, the state `refresh_scene` leaves after a scene-cache failure, where
`accept_world_point` already fails `Missing("command scene")`). Modal owners keep it `true`: the CMD
input stays placed and the region plan must not raise a layout error during file prompts or dialogs.

**Availability rules** (`command_availability`, also enforced inside dispatch so a refused call never
mutates): unknown/unimplemented ID → refused; Line/Point refused while running, while a file
operation or layer dialog is open (NEW guard; `dispatch_geometry` checked only `is_running`), or when
the scene is unavailable; Undo/Redo refused while running or with an empty journal; File kinds keep
their existing `begin_file_intent` checks.

## 5. Option grammar (L3-005)

```rust
// om9-app/src/session/options.rs
#[derive(Debug, Clone, PartialEq)]
pub enum OptionInput { Set { option: usize, value: Value }, Toggle { option: usize } }
pub fn parse_option(d: &CommandDescriptor, text: &str, unit: Unit, first_point_taken: bool)
    -> om9_core::Result<Option<OptionInput>>;
```

1. `NAME`, `NAME=VALUE`, `I`, `I=VALUE` (ASCII case-insensitive; `I` = the option's initial).
   Ambiguous initial → `Invalid("ambiguous option initial")`.
2. Text with `=` whose key matches no option → `Unsupported("command option")` (today's text). Text
   without `=` that matches no option → `Ok(None)`; the caller continues with coordinates (today's
   path, so `Normal`/`Invented` keep their current errors).
3. `BeforeFirstPoint` option after the first point → `Invalid("option after first point")`.
4. Values: Boolean `Yes|Y|True|No|N|False`, missing value = `Toggle`; Length →
   `LengthMm::parse(v, unit)`; Angle → `AngleDeg::parse(v)`; Count → `u32`; Ratio → finite `f64`;
   then `OptionDomain::check`. A malformed/NaN/inf value → `Invalid(spec.invalid_value)`.

Line keeps exactly one option, `BothSides` (`B`, Boolean, `AnyStep`): `BothSides=Yes|No` behaves as
today; `B`, `B=Y`, `B=N` follow the approved Line row of
`skills/openmatrix9-workspace-contract/references/command-and-curve.md`; bare `B` toggles (OM9
reading of that row — report it to the user at the end as a CMD behavior to confirm).

## 6. Repeat (L3-009)

`Controller.repeat` stores the ID of the last **completed repeatable** command. Undo, Redo and File
commands never set it, so empty Enter after them repeats the command before them (Rhino 5 manual
p.22 note; matches today's code). Repeat dispatches through the same `dispatch_descriptor` with
`InvocationOrigin::Cmd` and a freshly captured context.

## 7. Events and observers (L3-010)

```rust
// om9-app/src/session/events.rs
#[derive(Debug, Clone, PartialEq)]
pub enum CommandOutcome { Committed { transaction: TransactionId, epoch: u64 }, Completed { epoch: u64 },
                          Canceled }
#[derive(Debug, Clone, PartialEq)]
pub enum CommandEventKind {
    Started { command: CommandId, origin: InvocationOrigin },
    Prompt { text: &'static str },
    OptionChanged { option: &'static str },
    Progress { done: usize, total: usize },          // accepted points of required points
    Warning { message: String },                      // rejected input while running / refused dispatch
    Finished { command: CommandId, outcome: CommandOutcome },
}
#[derive(Debug, Clone, PartialEq)]
pub struct CommandEvent { pub sequence: u64, pub kind: CommandEventKind } // strictly increasing per Controller
pub trait CommandObserver { fn on_command_event(&mut self, event: &CommandEvent); }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct ObserverId(u64);
pub const MAX_COMMAND_EVENTS: usize = 256;     // ring buffer; oldest dropped
pub const MAX_COMMAND_OBSERVERS: usize = 16;

impl Controller {
    pub fn add_command_observer(&mut self, o: Box<dyn CommandObserver>) -> om9_core::Result<ObserverId>; // Budget beyond 16
    pub fn remove_command_observer(&mut self, id: ObserverId) -> bool;
    pub fn command_events(&self) -> impl Iterator<Item = &CommandEvent>;
    pub fn observer_diagnostics(&self) -> &[String];
}
```

Events are emitted only after the state change is complete (after `commit`, after the session is
dropped). Each observer call runs in `catch_unwind(AssertUnwindSafe(..))`; a panic removes that
observer and appends one diagnostic; the document, Undo stacks and scene are untouched because the
observer only receives `&CommandEvent`. Undo/Redo emit `Started` + `Finished{Completed}`. File
lifecycle commands keep their own receipts in M01 and emit no command events (stated limitation).
Observers, the event ring and the sequence survive New/Open replacement (`session/files.rs`).
Event strings and observer slots are added to `Controller::auxiliary_usage` (session/layers.rs:191).

## 8. Test design (RED first; `cargo test -p om9-app --lib session::` and named integration tests)

| Test | Oracle |
| --- | --- |
| `session::registry::tests` | real catalog builds; `Line`/`line`/`OM9-CURVE-002` → same descriptor; `Save As`/`SaveAs`; duplicate implemented alias → `DuplicateAlias`; two implemented `Dim` → `Ambiguous`; `OM9-NOPE-999` → `Unknown`; `to_json` identical twice |
| `tests/command_registry.rs` (new) | menu `dispatch(LINE)` and CMD `submit("LINE")` reach the same session (same prompt, same geometry) |
| `session::hosts::tests` | phase/step progression; `command_context()` holds view+selection; scene `None` → `Unavailable(SceneUnavailable)` and `compute_regions_for(.., &controller)` → `Err(RegionError::CommandInputUnavailable)` |
| `tests/command_state.rs` (new) | extra point after commit refused (`Missing("active command")`), snapshot unchanged; empty Enter mid-session refused; New/discard during Line ends context without old-document mutation |
| `tests/command_options.rs` (new) | `B`, `b=y`, `B=N`, `BothSides=Yes`; synthetic descriptor `length=2.5mm angle=90deg`; `length=NaN`, `length=inf`, `length=2.5yd`, `X=1` errors |
| `tests/command_repeat.rs` (new) | Line → Undo → Enter starts Line; Line → Save → Enter starts Line; nothing → Enter `Missing("previous command")` |
| `tests/command_events.rs` (new) | Line emits Started, Prompt, Progress 1/2, Prompt, Progress 2/2, Finished{Committed} with increasing sequence; panicking observer → commit still published, diagnostic 1, later events still delivered |
| `tests/command_availability.rs` (new) | Undo empty → `NothingToUndo`, after Line → Ok; Line during an open file operation → `ModalOwner` and no session; OM9-SOLID-012 → `NotImplemented` |
| existing `command_sessions`, `ui_events`, `file_lifecycle`, `layer_controller`, `pinned_command` | unchanged behavior |

## 9. Unverified / out of scope

- `OpenMatrix9_Codex_Spec_v2/` absent; `om9.m01.oracle.v1.5` has no local definition.
- The meaning of a bare `B` (toggle) is an OM9 reading; confirm with the user after implementation.
- Menu enable state still uses the static `availability`; switching menus to the state-aware
  `command_availability` would change what the user sees (e.g. Undo greyed) and needs user consent.
