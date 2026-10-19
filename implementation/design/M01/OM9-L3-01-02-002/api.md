# OM9-L3-01-02-002 — Typed Command Descriptor (typed API)

Owner module `OM9-L2-01-02` (backend framework, stage E01, gate **AUTO_ONLY**).
Implementation owner: `standalone/crates/om9-core/src/capability.rs`. Reuse
decision: **ADAPT** the keyed parameter table of `qymcad-ui-state`; **ADD** the
declared signature, the prompt/output and the refusal of an undeclared option
(see `source_crosswalk.json`).

Oracle — positive: *Descriptor declares typed inputs/options/output and is
serializable/deterministic.* Negative: *Missing mandatory input or unregistered
option rejected.*

## 1. Typed API

```rust
pub struct Operation {
    pub id: String,
    pub feature: String,
    pub aliases: Vec<String>,
    pub prompt: String,               // the question the command asks, "" = none
    pub input_types: Vec<GeometryKind>,   // what the user must supply
    pub requires_input: bool,         // true ⇒ input_types may not be empty
    pub output: Vec<GeometryKind>,    // what it produces, [] = modifies/consumes
    pub options: Vec<String>,         // the option names the user may type
    pub preconditions: Preconditions, // OM9-L3-01-02-011
    pub backend: Backend,
    pub prepare: bool, pub preview: bool, pub commit: bool,
    pub receipts: Vec<Receipt>,
}

impl Operation {
    /// The declared spelling of an option, read without case, or a refusal that
    /// names both the option and the command.
    pub fn option(&self, name: &str) -> Result<&str, RegistryError>;
}
```

The whole struct is `Serialize + Deserialize + PartialEq` and the registry is a
`BTreeMap`, so serializing the same descriptor twice yields the same bytes and a
round trip compares equal.

## 2. Rules

| Descriptor | Result |
| --- | --- |
| input types, output and a prompt declared | registered; the signature is inspectable |
| `requires_input: true` with no input type | refused: `a mandatory input has no declared type` |
| no input, no output **and** no prompt | refused: `the descriptor declares no input, no output and no prompt` |
| an option that is empty or only whitespace | refused: `an option is empty or repeated` |
| two options that differ only by case (`Length`, `length`) | refused: the parser reads options without case, so they are one name |
| `option("Length")` / `option("length")` / `option("  Length  ")` | the declared spelling `Length` |
| `option("Angle")` on a command that does not declare it | `UnknownOption`: `command 'line' has no option 'Angle'` |
| a command with a prompt and no input/output/options (for example `fit`) | registered: it is still a descriptor |
| the same descriptor serialized twice | byte-identical |

`option()` exists so the option parser (OM9-L3-01-02-005) has one place to ask
"is this option declared", instead of a second name table.

## 3. Ownership

* The **descriptor** declares the contract of one command: its inputs, its
  options, its output, its prompt and what it needs from the session.
* It carries **no** geometry, no document and no runtime state; a run of the
  command holds those.
* Whether a command may be offered right now is decided by availability
  (`availability_with`), not here.
