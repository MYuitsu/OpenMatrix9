# OM9-L3-01-02-007 — Undo Redo Journal (typed API)

Owner module `OM9-L2-01-02` (backend framework, stage E01, gate **AUTO_ONLY**).
Implementation owner: `standalone/crates/om9-core/src/transaction.rs` + `om9-app/src/session.rs`. Reuse decision: **ADAPT the document-is-the-truth habit of `gui.rs`; ADD the journal rules as contract tests** (see
`source_crosswalk.json`).

Oracle — positive: *Create object -> undo removes -> redo restores stable ID and layer.* Negative: *Undo after delete/reorder must not replay corrupted references.*

## 1. Typed API

```rust
controller.undo() -> Result<()>;   // refused with Busy while a command runs
controller.redo() -> Result<()>;
controller.coordinator().undo_len() / redo_len() / epoch();
```

## 2. Rules

| Situation | Result |
| --- | --- |
| create then undo | the entity is gone, one entry moved to the redo side (`undo_len` -1, `redo_len` +1) |
| redo | the entity comes back with the **same id** and the **same layer** (verified: `same_id = true`) |
| three commands then three undos | the document is empty and `redo_len == 3` |
| a **new command after an undo** | the redo side is **cleared** (`redo_before = 1`, `redo_after = 0`): forward references that no longer describe the document are dropped, never replayed |
| `redo` with nothing to redo, or `undo` with nothing to undo | refused |
| `undo`/`redo` while a command runs | refused with `Busy` — the running command owns the document |
| after undo/redo | the scene is rebuilt from the document (`refresh_scene`), so no interface state can outlive the journal |

## 3. Ownership

* The **coordinator** owns the journal and the epoch; the **session** only asks.
* Undo and redo are document operations: they are refused while a command is
  alive, so a half-finished command can never be undone underneath.
