# L4 reference supplements - Matrix9 / Rhino 5 / manual sheets (2026-10-15)

## What was added

Every L4 task record in M00, M01, M02, M03, M04, M05, M06 and M07 now carries a
`reference_supplements` block plus a `reference_source_policy` block. M00-M02
records additionally end their `agent_runbook_markdown` with a mandatory
reference section; the M03/M04/M05/M06/M07 packages use their own task schemas, so
there
the anchors live in the JSON block only. The blocks route three
reference families that the task must verify against, none of which replaces
runtime evidence:

- `rhino5_core_chapters`: `RCORE-01..RCORE-12` chapter id, title, Guide PDF
  range, official Help URLs and the chapter capability/fixture IDs, taken from
  `ref/rhino5/CORE_REFERENCE_INDEX.json` (the user-approved public documentation
  exception).
- `matrix9_reference_sources`: recovered Matrix9 artifacts - form, class,
  control and procedure identifiers plus export provenance - located through
  `ref/matrix` (private archive; `docs/private-reference-policy.md`).
- `manual_reference_sheets`: the authored reference sheets already shipped inside
  `OM9_Milestones/**/reference/text/*.md`. Each sheet transcribes one Matrix
  manual section with its command path, option list and stated defaults, so this
  is where a Matrix command/option/default question is actually answered.

Counts: M00 73, M01 516, M02 306, M03 854, M04 340, M05 126, M06 76 = 2291 L4
records over 551 L3 capabilities. 542 L3 carry at least one Rhino 5 core chapter,
522 carry at least one Matrix9 source, 271 carry at least one manual sheet, and
every one of the 551 carries a written focus line. The anchors resolve to 109
Matrix9 artifacts, 230 distinct manual sheets and the 12 Rhino 5 core chapters.

## Merge provenance

M03 and M04 L4 records come from the execution-hardened packages and were merged
with `tools/merge_milestone_package.py`: local L1/L2/L3 records win, only
L4-planning members are adopted from the package, and every package L4 is added.
`tools/verify_merge.py` re-proves afterwards that each merged L4 body equals the
package record apart from the two anchor members, and that no local-only file was
lost (M03 2702 kept, M04 329 kept).

The M05 overlay ships a flat `planning/M05` layout plus an execution manifest, so
`tools/merge_m05_overlay.py` places each L4 at
`<L2>/<L3>/<L4-nn - Role>/l4.json` beside its `TASK.md`, rewrites the overlay id
`OM9-M05-L4-09-001-01` to the milestone form `OM9-L4-09-001-01` while keeping the
overlay id on the record, and fills `children_l4`/`l4_status`/planning state on
the parents. The M06 patch uses the same shape with its own work-kind labels, and
is merged by `tools/merge_l4_overlay.py --package m06`. `tools/verify_l4_overlay_merge.py`
re-proves every overlay body (M05 126/126, M06 76/76) against its manifest.

## Locator caveat (read before using an anchor)

`CORE_REFERENCE_INDEX.json` addresses its companion specs as
`../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/*.md` and
`.../CAPABILITY_LOOKUP.json`. On 2026-10-15 this checkout has neither package:
the v1 directory is deleted in the working tree and `OpenMatrix9_Codex_Spec_v2`
is not present, so `docs/AGENTS.md`'s `../OpenMatrix9_Codex_Spec_v2` router
cannot be resolved locally. The Guide PDF ranges, official Help URLs and the
capability/fixture id sets live inside the index itself, which is why the
records cite `ref/rhino5/CORE_REFERENCE_INDEX.json` rather than the absent
chapter markdown. Resolve the chapter spec through the reference registry
(`docs/openmatrix9-reference-locations.json`) before implementing.

`ref/matrix` is `.gitignore`d and holds the private Matrix9 exports; a record's
`root` + `path` locator is workspace-local and must not be published.

## What was deliberately not done

- No reference source body, decompiler listing, native resource blob, recovered
  image or manual text was copied into the repository.
- No `planning_state`, `execution_policy.status` or gate status was promoted;
  every record keeps `requirements_only_not_runtime_evidence`.
- No private archive path was added to public documentation or release metadata.
- Captions, identifiers and sheet option names are recorded as documentation
  evidence only, never as verified defaults or proof of behaviour.

## Verification

```shell
python tools/matrix_ref_inventory.py
python tools/matrix_ref_sources.py
python tools/gen_manual_sheets.py
python tools/gen_m03_anchors.py
python tools/validate_matrix_ref_crosswalk.py
python tools/apply_matrix_ref_supplements.py            # dry run, must not change files
python tools/validate_matrix_ref_supplements_applied.py
python tools/verify_merge.py m03
python tools/verify_merge.py m04
python tools/verify_l4_overlay_merge.py m05
python tools/verify_l4_overlay_merge.py m06
python tools/verify_l4_overlay_merge.py m07
python tools/validate_implementation_samples.py
```

The crosswalk validator fails closed on an unrouted L3, an unknown RCORE
chapter, Matrix9 anchor or manual sheet, an anchor whose locator does not
resolve, or any drift between the crosswalk and `CORE_REFERENCE_INDEX.json`. The
applied validator fails closed on invalid JSON, duplicated members, a missing
runbook section where the schema has one, crosswalk drift, or any promoted
status.

## Remaining uncertainty

Matrix9 `observed_identifiers` are names and paths extracted from the exports;
they are locators, not verified behaviour. Manual-sheet options and defaults are
transcribed documentation for Matrix 8 / 9.0-addendum material and are not proof
that the installed Matrix9 build behaves identically. Symbol-level verification
at a checkout, numerical defaults, measurement tolerances and Matrix9 parity
remain unproven until the corresponding L4 task runs with real evidence.
