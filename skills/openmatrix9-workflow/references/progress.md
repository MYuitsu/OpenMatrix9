# Persistent implementation progress

Use `docs/openmatrix9-progress.json` in the project checkout. `FEATURES.json` is an input catalog; do not rewrite its `implementation_status` as the live ledger.

Each work item records:

| Field | Meaning |
|---|---|
| `stage` | Route key from stages.json |
| `feature_ids` | Exact OM9 IDs, empty for infrastructure |
| `procedure_va` | Exact VA when native analysis is involved, otherwise null |
| `status` | planned, in_progress, implemented, validated, blocked, or superseded |
| `outputs` | Actual created source/analysis/artifact paths |
| `evidence` | File/line, source fingerprint or runtime artifact supporting the claim |
| `tests` | Command, result and relevant environment; absence is explicit |
| `uncertainties` | Missing manual, inferred type/default, untested behavior |
| `next_document` | Exact file and optional section/function/page |
| `next_action` | Small concrete continuation |

Update the ledger after a meaningful verified step; preserve unrelated work items. Use actual timestamps. If evidence contradicts a prior entry, correct it and explain why in that entry. File existence can show that a plan was written; it cannot show that the planned UI works.

Completion is stage-specific. A mapped procedure can still be behaviorally uncertain. An implemented feature can still lack runtime validation. A skill suite being available does not advance product implementation status.
