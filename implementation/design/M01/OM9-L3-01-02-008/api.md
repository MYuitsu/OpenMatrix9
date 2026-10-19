# OM9-L3-01-02-008 — Command Cancel Rollback (typed API)

Owner module `OM9-L2-01-02` (backend framework, stage E01, gate **AUTO_ONLY**).
Implementation owner: `standalone/crates/om9-app/src/session.rs`. Reuse decision: **ADAPT the abandonable step of `gui.rs`; ADD the rule that a cancel leaves nothing** (see
`source_crosswalk.json`).

Oracle — positive: *Preview geometry canceled with Escape restores document and no undo entry.* Negative: *Cancel after partial state or C++ error must not leave orphan preview entity.*

## 1. Typed API

```rust
controller.preview(point) -> Result<Option<(Point3, Point3)>>;  // &self: draws, never writes
controller.cancel();                                            // Escape
```

## 2. Rules

| Situation | Result |
| --- | --- |
| a line command with its first point taken | `preview(cursor)` returns the rubber band from the start to the cursor |
| Escape at that moment | the session is dropped: `preview(…)` returns `None`, the entity count and the journal are unchanged, phase `Idle` |
| a click after the cancel | refused with `Missing("active command")` — the old preview cannot be committed |
| a **refused** command (zero-length line) | is **not** a cancel: the session and its preview stay, and the next valid point commits exactly one entity and one journal entry |
| two cancels in a row | harmless: nothing to drop, nothing written |
| the preview itself | never becomes an entity; there is no code path that inserts a preview, so no orphan can survive a cancel |

## 3. Ownership

* The **session** owns the preview and the cancel; the **document** is only
  touched by the commit inside `accept_world_point`.
* Because `preview` takes `&self`, a preview cannot write even by mistake: the
  type system carries the rule.
