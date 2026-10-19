# OM9-L3-01-02-006 — Atomic Command Transactions (typed API)

Owner module `OM9-L2-01-02` (backend framework, stage E01, gate **AUTO_ONLY**).
Implementation owner: `standalone/crates/om9-app/src/session.rs` with `om9-core/src/transaction.rs`. Reuse decision: **ADAPT the single doorway of `gui.rs`; ADD the prepared/committed transaction** (see
`source_crosswalk.json`).

Oracle — positive: *One UI action creates one transaction and one revision even with multiple objects.* Negative: *Mid-operation failure rolls back every change and preserves undo journal.*

## 1. Typed API

```rust
// om9-core/src/transaction.rs — unchanged, the contract this item pins
TransactionRequest::new(label, vec![Mutation::Insert { document, entity }])
coordinator.prepare_with_held_usage(request, held) -> PreparedTransaction
coordinator.commit(prepared) -> Result<Receipt>       // Receipt { published }
coordinator.undo() / redo() / undo_len() / redo_len() / epoch()

// om9-app/src/session.rs — one request per FINISHED command
accept_world_point(point) -> Result<()>   // prepare, then commit, then advance the session
```

## 2. Rules

| Situation | Result |
| --- | --- |
| one completed command (LINE, two points) | one entity, **one** journal entry, redo side empty |
| a second completed command | a second entity and a second journal entry — never half of one |
| a command refused at any step (zero-length line, `Busy`, stale context) | **nothing** written: entity count and journal unchanged, and the command stays open where the plan says it should |
| Escape mid-command | nothing written, phase `Idle` |
| a multi-mutation request | the coordinator applies all of it or none of it; the session only ever sends one request for the whole user action |

The per-action counter is the **journal length / publication epoch**. The
document's `context_revision` is an *identity* token: it does **not** move when
the document is edited (the probe in the evidence log shows it unchanged after a
committed line), and it is what marks a session stale when the document is
replaced.

## 3. Ownership

* The **session** decides *when* the user's action is finished and sends exactly
  one request then; the **coordinator** decides whether it can be published.
* A refusal is not a rollback of a partial write: nothing was written yet, so the
  refusal is total by construction.
* The session advances its own state (`repeat`, `recent`) only on
  `receipt.published`.
