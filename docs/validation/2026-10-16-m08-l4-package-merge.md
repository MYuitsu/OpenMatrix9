# M08 / E13 L4 package merge (2026-10-16)

## What was merged

`OpenMatrix9_v014_M08_L4_Patch.zip` (SHA256
`926225e29743feff6397165081d4819374554279d5285df517986a159cac700e`, 536 entries)
was extracted to `build/m08-incoming/OpenMatrix9` and merged into the milestone
tree with an extended `tools/merge_l4_overlay.py --package m08`.

| Item | Count |
|---|---:|
| L4 records merged | 221 |
| L3 capabilities covered | 73 |
| L2 modules touched | 9 |
| L4 placed in the M08 tree | 177 |
| L4 placed in the M03 tree | 44 |
| Package documents copied to `reference/m08-l4` | 8 |

Every task id in the package already uses the milestone form
(`OM9-L4-aabbccc-nn`), so no id had to be rewritten. Each record gained `level`,
`task_role`, `depends_on_l4_ids` and `reference_policy` from the merger; the
`path`-referenced `TASK.md` was placed beside its `l4.json`.

## Cross-milestone owner retention

22 of the 73 L3 are `OM9-L2-04-01` transform-toolbar bridges owned by M03. The
planning handoff states these execute in E13 while keeping their canonical owner
milestone, and the M03 tree already recorded them as
`DEFERRED_TO_M08_NOT_PLANNED_HERE` with `planning_state`
`SPEC_ONLY_NOT_IMPLEMENTED`. The merge therefore places their 44 adapter L4 inside
the M03 tree instead of relocating the specs; `M08/L1.json` records
`l4_executed_in_milestones: ["M08", "M03"]`, `l3_spec_tree_by_milestone` and
`l4_placed_by_milestone` so the split is auditable from one place.

## Parent records updated

- 73 `l3.json`: `children_l4`, `l4_status=PLANNED_NOT_EXECUTED`,
  `planning_state=SPECIFIED_L4_PLANNED_NOT_IMPLEMENTED`, and the
  `specification.l4_status` / `specification.execution_status` markers.
- 9 `L2.json`: `l4_status`, `planned_l4_count_m08` (44 for M03's `OM9-L2-04-01`)
  and `definition.l3_l4_status`.
- `M08/L1.json`: `l4_count` 221, `l4_count_in_tree` 177, the per-milestone
  placement summary and an `execution_readiness` block (stage E13, gate
  `G-FULL requires G-MVP PASS and all E13 approved`, source audit status
  `REMOTE_REVIEW_ONLY_NO_LOCAL_BUILD`, pinned revision
  `c116f8f224ebffe0ada049c91c0d15df3634f00d`).
- `OM9_Milestones/M08 - Advanced Jewelry Modeling/reference/m08-l4/`: the manifest
  (JSON + CSV), packaging report, self review, release gate, Luna handoff, task
  prompt and README.

## Implementation samples

The 221 records were then given the authored `implementation_samples` skeleton and
`implementation_sample_guide` warning by `tools/apply_implementation_samples.py`
(M08 added to its milestone map, `MENU_BINDING_ACCEPTANCE → MENU_BINDING →
INTEGRATE`). Role split: `CONTRACT_TEST` 73, `ACCEPTANCE` 73, `IMPLEMENT` 51,
`REGRESSION_STRESS` 24. Every block keeps `status=author_skeleton_untrusted`,
`sketch_is_untested`, `not_runtime_evidence` and
`must_not_be_reported_as_implemented`.

## Tooling changes

- `tools/merge_l4_overlay.py`: new `m08` package definition; `extra_milestone_dirs`
  so a package may write into a second milestone's tree; L3 records that already
  hold the merged children are no longer rewritten, which keeps re-runs
  idempotent.
- `tools/merge_milestone_package.py`: the `m03` spec now knows about
  `l4_status`, `planned_l4_count_m08` and `definition.l4_count_planned_m08`.
- `tools/verify_merge.py`: the 22 L3 the M03 package itself defers to M08 are no
  longer required to still read `DEFERRED_TO_M08_NOT_PLANNED_HERE`; instead the
  proof checks that their `children_l4` equals the M08 package contribution, that
  the owning L2 declares the M08 planned count, and that no non-planning content
  drifted.
- `tools/verify_m08_l4_merge.py` (new): re-proves all 221 records against the
  package manifest, including id/role/parent/dependency agreement, placement per
  `milestone_id`, the reference documents, and that no record claims execution.

## Verification

```shell
python tools/merge_l4_overlay.py --package m08 --apply   # 221 L4, 0 conflicts
python tools/verify_m08_l4_merge.py                      # OK: 221 L4 over 73 L3
python tools/verify_merge.py m03                         # 854 L4, 2790 kept, 0 errors
python tools/verify_merge.py m04                         # 0 errors
python tools/verify_l4_overlay_merge.py m05|m06|m07      # 0 errors each
python tools/apply_implementation_samples.py --apply     # 221 applied
python tools/validate_implementation_samples.py          # 0 errors
python tools/run_reference_checks.py                     # ALL CHECKS PASS (12/12)
```

## What this is not

No code was produced, no build or runtime test was executed, and no status was
promoted: all 221 L4 stay `PLANNED_NOT_EXECUTED` with
`readiness=BLOCKED_BY_G00_E12_G_MVP_AND_LOCAL_SOURCE`, and the package itself
declares `implementation_count: 0`. The planning numbers in the records
(manual pages, tolerances, options) remain inputs to verify at execution, not
verified behaviour.

## Remaining uncertainty

- The 73 M08 L3 still have no `reference_supplements` (Rhino 5 chapter / Matrix9
  source / manual sheet) and no entry in the applier's `DOMAIN_NOTE_MAP`; both need
  a crosswalk for the E13 L3 before they can be attached.
- The staged copy lives under `build/`, which is gitignored: the zip itself is not
  part of the repository, only its hash and the merged records.
- Nothing can be executed until G00, E01–E12 and G-MVP carry real PASS evidence
  and a local QymCAD checkout exists.
- The M09 release-gate plan merged later the same day adds 30 further `l4.json`
  records under `M09/reference/m09-l4/`; they own no L2/L3 and are counted by
  `tools/verify_m08_l4_merge.py` under `workspace_l4_total`, not under the M08
  package figures above (see [2026-10-16-m09-l4-package-merge.md](2026-10-16-m09-l4-package-merge.md)).
