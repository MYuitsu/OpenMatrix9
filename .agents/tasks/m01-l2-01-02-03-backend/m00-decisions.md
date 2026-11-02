# M00 decisions for the M01 backend trio (OM9-L2-01-08 / -02 / -03)

Scope recorded from the user (2026-10-16, relayed by the orchestrator): M00 approval covers ONLY
QymCAD code reuse. OCCT, UI, 3DM, Associative, License and Reproducible are NOT approved. A coder
that reaches a NEEDS_USER_M00_DECISION item asks with `send_message` severity `warning` and records
the answer here; later loops apply a recorded answer without asking again.

| # | Decision | Raised by | Status | Answer (verbatim) |
| --- | --- | --- | --- | --- |
| D1 | `.om9` persistence of new document data (`Document.tolerance`, `Document.groups`, `Entity.extensions`): A additive optional keys in schema 4 + legacy-key guard (recommended) / B schema 5 + explicit migration / C in memory only, saving refuses explicitly | checklist 8.3 (first loop) | PENDING | |
| D2 | Kernel (OCCT) pass-through of the document tolerance | checklist 8.3 | NOT ASKED - deferred: OCCT not approved, OM9-L2-01-05 absent; only `Document::effective_tolerance()` is delivered | |
| D3 | BRep entity kind | checklist 3.1 | NOT ASKED - deferred with OM9-L2-01-05 (OCCT) | |
| D4 | Swapping the UI stub adapters in `ui.rs` (UI scope) | stub-integration-note.md | COVERED by the user's request "Làm cho xong 2 cái kia để không bị block nữa" (message 3): a no-visible-change adapter swap, not a new UI decision. Orchestrator applies it after the UI workflow finishes; ask first only if the swap would change anything visible | |

Not M00 items (decided in the design, reasons in the crosswalks): enabling serde_json
`float_roundtrip` (reader-only change, bytes/schema/lockfile unchanged; stop and ask only if a golden
byte test changes), document snapshots (reuse the existing codec), provenance queries (read-only over
already-retained import data).
