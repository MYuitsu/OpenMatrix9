# OpenMatrix9 M05 — L4 execution package for GPT-6 Luna

**Edition:** M05-L4 planning overlay for OpenMatrix9 source-reviewed v0.12 (not a shipped application).  
**Stage:** `E10`, **owner milestone:** `M05`, **scope:** 5 L2 / 43 canonical L3 / 126 planned L4.  
**Prerequisites for coding:** `G00-01..G00-07 PASS`, `E01..E09 PASS`, source/revision/license sign-off and an actual Rust/OCCT development environment.

This package is a task decomposition only. It has **not** cloned, built or tested application source; it has not validated geometry against an OCCT runtime; no FEBE item has been manually accepted. The pinned QymCAD SHA is a review pointer, not proof of a local checkout.

## File layout

- [`RUNBOOK_GPT6_LUNA.md`](RUNBOOK_GPT6_LUNA.md) — agent prompt, bounded execution, context recovery, exact reporting protocol.
- [`M05_L4_EXECUTION_MANIFEST.csv`](M05_L4_EXECUTION_MANIFEST.csv) — filter/sort all L4; JSON variant is machine-readable.
- [`M05_L4_EXECUTION_WAVES.json`](M05_L4_EXECUTION_WAVES.json) — topological task waves; no inference about safe concurrent editing.
- [`M05_L4_COVERAGE_MATRIX.csv`](M05_L4_COVERAGE_MATRIX.csv) — canonical L3 -> L4 + primary manual refs.
- [`M05_L4_REVIEW.md`](M05_L4_REVIEW.md) — independent static audit, scoring, open risks.
- `<owner-module>/<canonical-L3-slug>/<NN-phase>/TASK.md|l4.json` — ONE phase per Luna turn / PR.
- `tools/generate_m05_l4.py` and `tools/validate_m05_l4.py` — reproducible generation and static verification.

## Task types (126)

| Type | Quantity | Meaning | Exit |
|---|---:|---|---|
| `SOURCE_AUDIT_CONTRACT` | 43 | Audit actual source, determine REUSE/ADAPT/ADD, freeze API seam/ownership | Locally verified paths and signed contract, no fabricated code |
| `DOMAIN_IMPLEMENTATION` | 43 | Implement only the owning geometry/domain operation *or* a dispatch adapter, include automated tests | Actual build + validity/undo/history/reference evidence |
| `UI_INTEGRATION_AND_ACCEPTANCE` | 40 | Real command/menu/F6 binding, preview/cancel/accept and guided human test | Runtime UI test, then human signoff |

There are 3 backend L3 and 40 FEBE L3. A frontend-labeled L3 may need Rust domain logic, but that logic must remain within its unique capability owner or dispatch to a previously implemented core service. Forbid parallel duplicate implementation in widgets.

## Safe ordering and responsibilities

Only select tasks with `depends_on_task_ids` all passed **and** stage gates satisfied. The graph includes dependencies within M05; upstream stages are represented as explicit **gate dependencies** and must be checked separately. `M05_L4_EXECUTION_WAVES.json` is *not* permission to execute same-wave tasks touching shared files concurrently. Review one L4 at a time for a human-led workflow.

A task has a small well-defined phase, inputs, outputs, typed error conditions, test oracle, source references, fixture IDs, traceable evidence fields and pass gates. If the implementation needs two independent algorithms, stop and split the affected phase into smaller child tasks **without changing L3 owner/capability IDs**; record the split in an approved manifest revision.

**Source reference:** `matrix_8_manual_book2.pdf` Chapter 27 (`Placing Gems`) and associated pages; individual L3 references carry both printed and PDF page indices. Some L3 has only a parent-L2 range, not command-level support. The manual gives **functional behavior**, not Rust ABI, numerical error bars, copyright permissions or exact QymCAD API paths. See `manual_interpretation_note` in each task.

## Never confuse status layers

The baseline v0.12 registry and `EXECUTION_MANIFEST` are frozen historical plans and continue to say `NOT_CREATED` / `L3_SPEC_ONLY_L4_NOT_CREATED` so that legacy validation remains usable. **The M05 L4 overlay manifest is the authoritative index for the newly created M05 planning tasks.** Its task status is `PLANNED_BLOCKED_G00_E09`, not implemented. Do not rewrite 772 baseline rows to `DONE` or change ownership just to make counters appear updated.

Before any release decision, reconcile planning-overlay schema/status into a subsequent canonical version with an approved migration, then update validators together. This package intentionally avoids an inconsistent unreviewed migration.

## Static checks

From project root (Python 3):

```sh
python tools/validate_l3.py
python tools/validate_v010_plan.py
python tools/validate_m05_l4.py
```

All three validate **plans**. They do not mean QymCAD, OCCT, the UI or M05 passes tests.
