# M09 release-gate L4 package merge (2026-10-16)

## What the package is

`OpenMatrix9_M09_L4_Patch.zip` (SHA256
`ff962d452f8e081321b3ff25d9d9b382d71665fe9f911420c44968cd86094249`, 77 entries)
is not an implementation plan. M09 owns the two release gates and, by the source
plan's own rule, **no L2 and no L3**: the package ships 30 release-verification
work orders that only read and verify canonical L3 owned by M01–M08.

| Item | Count |
|---|---:|
| Release L4 merged | 30 |
| G-MVP L4 (after G00 + E01–E12) | 18 |
| G-FULL L4 (after signed G-MVP + E13) | 12 |
| Identified test cases | 177 |
| Distinct canonical L3 traced | 59 |
| Canonical L2/L3 added | 0 |
| Files merged | 77 |

Because there is no L2/L3 to hang an `L4-nn` directory under — and inventing one
would contradict `M09/L1.json` and the package's `canonical_ownership_policy`
(`READ_ONLY_TRACE_TO_EXISTING_L3; NO_L2_OR_L3_CREATED; NO_GEOMETRY_OWNER`) — the
merge follows the M05/M06/M07 overlay convention: the package keeps its own layout
under a `reference/` tag, and the milestone root gains the plan summary.

## Merged layout

`OM9_Milestones/M09 - Acceptance and Release Gates/`

- `L1.json` — unchanged "no L2/L3 owner" contract plus `release_gate_plan`:
  manifest path, schema, owner type, gate list, gate dependency graph
  (`G-MVP ← G00,E01..E12`; `G-FULL ← G-MVP,E13`), `l4_by_gate` with per-gate work
  kinds, 177 test cases, 59 canonical references, the document index, the archive
  SHA256, and `gate_status {G-MVP: NOT_TESTED, G-FULL: NOT_TESTED}` with
  `release_decision: UNDECIDED`, `evidence_state: NOT_CREATED`.
- `reference/m09-l4/` — `g-mvp/01..18` and `g-full/01..12` (`l4.json` + `TASK.md`),
  `M09_EXECUTION_MANIFEST.json|csv`, `M09_EXECUTION_TREE.txt`,
  `M09_TEST_CASES.csv`, `M09_REVIEW_REPORT.md`, the three templates,
  `G_MVP_CHECKLIST.md`, `G_FULL_CHECKLIST.md`, `GPT6_LUNA_EXECUTION_PROMPT.md`,
  `README.md`, and the release-gate contract documents
  (`M09_RELEASE_GATES.md`, `PLANNING_README.md`).
- `reference/m09-l4/tools/` — the package's own helpers, kept where they load
  without touching `sys.path` (`validate_m09_l4.py`, `m09_next_task.py`); working
  copies live in `tools/` (see below).

The manifest's `path`/`task_path` members were re-pointed from `planning/M09/...`
to `reference/m09-l4/...` (60 rewrites) so every listed file resolves inside this
checkout, and `M09_EXECUTION_MANIFEST.csv` is regenerated from the manifest so it
cannot drift. No record was edited beyond that: all 30 keep
`status=PLANNED_NOT_EXECUTED`, `gate_status=NOT_TESTED`,
`release_decision=UNDECIDED`, `evidence_state=NOT_CREATED`,
`source_revision=null`, `toolchain_verified=false`.

## Tooling

The package's two scripts assume a `planning/` root, `planning/EXECUTION_MANIFEST.json`
(772 L3) and `specs/_meta/l3-registry.json`; none of the three exists in this
checkout. They were therefore ported into `tools/`:

- `tools/validate_m09_l4.py` — the canonical L3 inventory now comes from the
  milestone trees (`OM9_Milestones/**/l3.json`). Every assertion of the source
  validator is preserved: 30 L4, 18/12 per gate, manifest/CSV agreement, the
  per-record fields (owner type, `NOT_TESTED`, `UNDECIDED`, no L2/L3 owner, no
  source revision, no effort estimate), `canonical_traceability` resolving to real
  L3 with matching module/capability/stage and mainline-only stages, ≥3 execution
  steps, ≥2 negative cases, case-count identity, unique case ids, gate-local
  evidence targets, same-gate dependencies, the 177-case CSV, the templates, the
  `NOT_TESTED` checklists, acyclicity, and the two pinned dependencies
  (`GMVP-018 ← GMVP-001,GMVP-017`; `GFULL-010 ∈ GFULL-012`).
  Two differences are deliberate and documented in the file: the duplicate L3
  declaration `OM9-L3-01-12-008` (M01 and M07 carry it identically) is allowed
  when both copies agree, and the source plan's assertion that
  `implementation/evidence/M09` must not exist is dropped, because that directory
  is exactly what real release verification will legitimately create.
  The archive's `l3_path` spelling (slug folders) is checked for shape only —
  this checkout names the same folders by title — while module, capability and
  stage are compared against the local records.
- `tools/m09_next_task.py` — same scheduler, reading the manifest from the merged
  reference root so printed task paths resolve. It still refuses a `template_only`
  state file.

## Verification

```shell
python tools/merge_m09_package.py --apply          # 77 files, 60 path rewrites, 0 conflicts
python tools/verify_m09_l4_merge.py                # byte-identical, no promoted status
python tools/validate_m09_l4.py                    # 30 L4 / 177 cases / 59 refs / acyclic
python tools/m09_next_task.py --state <blank template> --gate G-MVP   # BLOCKED_STATE, exit 2
python tools/m09_next_task.py --state <synthetic complete state> --gate G-MVP  # GMVP-001
python tools/m09_next_task.py --state <synthetic MVP-only state> --gate G-FULL # BLOCKED_GATE
python tools/run_reference_checks.py               # ALL CHECKS PASS (12/12)
```

`tools/verify_m09_l4_merge.py` re-proves byte equality for every copied package file
(the manifest and its CSV are regenerated, so they are checked structurally
instead), that every manifest path resolves and was re-pointed under
`reference/m09-l4/`, that the records and the CSVs agree, that the L1 summary
matches the manifest, that the recorded archive hash matches the delivered zip,
and that no evidence directory was fabricated.

## What this is not

No gate was tested and nothing was signed. `gate_status` is `NOT_TESTED` for both
gates, the release decision is `UNDECIDED`, there is no build, no Rust/OCCT
artifact, no `implementation/evidence/M09` content, and no claim of G-MVP or
G-FULL readiness. The plan's own words apply: passing these validators only proves
the planning metadata is consistent.

## Remaining uncertainty

- The package's canonical L3 reference paths (`specs/.../NNN-slug/l3.json`) do not
  exist in this checkout, which names those folders by title; the L3 records
  themselves were verified by id, module, capability and stage.
- `planning/EXECUTION_MANIFEST.json`, `planning/GATES_AND_EVIDENCE.md` and
  `planning/DEPENDENCY_AND_RELEASE_POLICY.md` cited by the plan are not present, so
  the stage counts were re-derived from the trees (E01–E12 529, E13 73) instead of
  read from that manifest.
- The two gate folders keep the package's lowercase `g-mvp`/`g-full` names so the
  manifest paths stay recognisable; the gate ids themselves are `G-MVP`/`G-FULL`.
