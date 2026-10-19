# OM9-L3-01-02-001 — Command IDs And Alias Registry (typed API)

Owner module `OM9-L2-01-02` (backend framework, stage E01, gate **AUTO_ONLY**:
no manual test). Implementation owner: `standalone/crates/om9-core/src/capability.rs`.
Reuse decision: **ADAPT** the stable-key idea from `qymcad-shell`; **ADD** the
alias table, the ambiguity rules and the diagnostics (see `source_crosswalk.json`).

Oracle — positive: *Registry maps stable command ID from menu/name/alias to one
descriptor.* Negative: *Duplicate alias ambiguous or unknown ID refused with
explicit diagnostic.*

## 1. Typed API

```rust
pub struct Operation {
    pub id: String,              // the stable command id, never localised
    pub feature: String,         // the plan feature id (RCORE-00, …)
    pub aliases: Vec<String>,    // the other names it answers to (menu, Rhino/Matrix)
    pub input_types: Vec<GeometryKind>,
    pub options: Vec<String>,
    pub backend: Backend,
    pub prepare: bool, pub preview: bool, pub commit: bool,
    pub receipts: Vec<Receipt>,
}

pub enum RegistryError {
    UnknownName { name: String },
    DuplicateName { name: String, first: String, second: String },
    EmptyName { operation: String },
    DuplicateId { id: String },
    InvalidOperation { id: String, why: &'static str },
}

impl CapabilityRegistry {
    /// Register one operation and every name it answers to.
    pub fn try_register(&mut self, operation: Operation) -> Result<(), RegistryError>;
    /// The one descriptor a name means: the id or any alias, ignoring case.
    pub fn resolve(&self, name: &str) -> Result<&Operation, RegistryError>;
    /// The pre-existing total entry point; refuses with Error::Invalid.
    pub fn register(&mut self, operation: Operation) -> Result<()>;
    pub fn availability(&self, id: &str) -> Availability;
}
```

## 2. Rules

| Name typed | Result |
| --- | --- |
| the id, any case (`curve-evaluate`, `CURVE-EVALUATE`) | that operation |
| any alias, any case (`curveeval`, `EvaluateCurve`) | that operation |
| a name no operation claims | `UnknownName`, message `unknown command '<name>'` |
| a name two operations claim (alias/alias, alias/id, either case) | `DuplicateName`, message `command name '<name>' is ambiguous: '<first>' and '<second>' both claim it`, and **the first registration survives** |
| one operation spelling its own name twice under different case (`id = line`, alias `Line`) | accepted: the built-in menu name beside the stable id, still one descriptor |
| an empty or whitespace-only alias | `EmptyName` |
| an empty id, an empty feature, no input types, an empty or repeated option | `InvalidOperation` with the reason |
| the id already registered | `DuplicateId` |
| `register()` (the old API) on any of the above | `Err(Error::Invalid("operation registration"))`, unchanged |

The **plan feature id is not a command name**: two commands may share one, and
`resolve` does not answer to it. Names are compared after trimming and
lower-casing; the registry never renames what the user typed in its diagnostic.

## 3. Ownership

* The **registry** owns command identity: ids, aliases and the single descriptor
  each name means. It is pure data: no IO, no document, no UI.
* The **command line** (OM9-L2-01-02 later items) asks the registry; it does not
  keep a second name table.
* Availability is a separate concern (`availability(id)`) and is not decided here.
