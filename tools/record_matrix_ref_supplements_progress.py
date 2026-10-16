"""Record the M00-M04 L4 reference-supplement crosswalk in the live ledger.

Adds one requirements-only entry to docs/openmatrix9-progress.json
(reference_updates) and documents the crosswalk in docs/validation.

No implementation status is promoted and no private reference content is copied.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROGRESS = ROOT / "docs" / "openmatrix9-progress.json"
REPORT = ROOT / "docs" / "validation" / "2026-10-15-l4-reference-supplements.md"
ENTRY_ID = "m00-m07-l4-matrix9-rhino5-reference-supplements-2026-10-15"
SUPERSEDED = {
    "m00-m02-l4-matrix9-rhino5-reference-supplements-2026-10-15",
    "m00-m04-l4-matrix9-rhino5-reference-supplements-2026-10-15",
    "m00-m05-l4-matrix9-rhino5-reference-supplements-2026-10-15",
    "m00-m06-l4-matrix9-rhino5-reference-supplements-2026-10-15",
}

# Entry written by tools/update_matrix9_screenshot_specs_20261014.py. It is
# reproduced here verbatim so re-serialising the ledger never drops it.
SCREENSHOT_ENTRY = {
    "date": "2026-10-14",
    "spec_path": "specs/00-rhino-core/MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md",
    "source_ids": ["S1", "S2", "S3", "S4", "S5", "S6"],
    "affected_feature_ids": [
        "OM9-IFACE-001", "OM9-MAIN-001", "OM9-LAYER-001", "OM9-PROJECT-001",
        "OM9-PROJECTDB-001", "OM9-VIEWPORT-001", "OM9-INFO-008", "OM9-INFO-010",
        "OM9-DISPLAY-008", "OM9-DISPLAY-001", "OM9-DISPLAY-002", "OM9-DISPLAY-004",
        "OM9-DISPLAY-006", "OM9-INFO-004", "OM9-SOLID-012",
    ],
    "acceptance_cases": [f"M9IMG-{i:02d}" for i in range(1, 13)],
    "status": "requirements_only_not_runtime_evidence",
    "images_copied_to_repository": False,
    "implementation_status_promoted": False,
}

REPORT_TEXT = """# L4 reference supplements - Matrix9 / Rhino 5 / manual sheets (2026-10-15)

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
"""


def main():
    data = json.loads(PROGRESS.read_text(encoding="utf-8"))
    entry = {
        "id": ENTRY_ID,
        "date": "2026-10-15",
        "status": "requirements_only_not_runtime_evidence",
        "scope": "M00+M01+M02+M03+M04+M05+M06+M07 L4 task records",
        "scope_note": "every record also carries authored reference anchors; M01+M02 additionally carry implementation_samples",
        "l4_records_updated": 2360,
        "l3_capabilities_routed": 570,
        "l3_with_rhino5_chapter": 561,
        "l3_with_matrix9_source": 540,
        "l3_with_manual_sheet": 375,
        "matrix9_reference_anchors": 114,
        "manual_reference_sheets": 337,
        "rhino5_core_chapters": 12,
        "merged_packages": ["OpenMatrix9_M04_Only.zip", "OpenMatrix9_M03_Execution_Hardened_Patch.zip", "OpenMatrix9_M05_L4_Overlay (1).zip", "OpenMatrix9_M06_L4_Patch.zip", "OpenMatrix9_v013_M07_L4_Patch.zip"],
        "crosswalk": "tools/matrix_ref_crosswalk.py",
        "reference_roots": [
            "ref/rhino5/CORE_REFERENCE_INDEX.json",
            "ref/matrix",
            "OM9_Milestones/**/reference/text/*.md",
        ],
        "validation_command": "python tools/validate_matrix_ref_crosswalk.py && python tools/validate_matrix_ref_supplements_applied.py && python tools/verify_merge.py m03 && python tools/verify_merge.py m04 && python tools/verify_l4_overlay_merge.py m05 && python tools/verify_l4_overlay_merge.py m06 && python tools/verify_l4_overlay_merge.py m07",
        "evidence_report": "docs/validation/2026-10-15-l4-reference-supplements.md",
        "implementation_status_promoted": False,
        "reference_content_copied": False,
        "private_paths_included": False,
        "implementation_samples": {"l4_records": 822, "milestones": ["M01", "M02"], "language": "rust", "status": "author_skeleton_untrusted", "roles_covered": ["SOURCE_AUDIT", "CONTRACT_TEST", "IMPLEMENT", "INTEGRATE", "ACCEPTANCE", "REGRESSION_STRESS"], "validation_command": "python tools/validate_implementation_samples.py", "sketch_is_not_runtime_evidence": True},
        "unknowns": "Matrix9 identifiers and manual-sheet options are locators/documentation only; symbol re-verification at a checkout, defaults, tolerances and parity remain unproven. M05 gem cut/size semantics, M06 head/bezel/prong settings, M07 cutter/boolean/manufacturing rules and pave layout rules are manual-level only. M01/M02 implementation_samples are authored Rust skeletons: not compiled, not executed, and they name no symbol that is absent from the record.",
    }
    updates = [u for u in data.get("reference_updates", [])
               if u.get("id") != ENTRY_ID and u.get("id") not in SUPERSEDED]
    if not any(u.get("spec_path") == SCREENSHOT_ENTRY["spec_path"] for u in updates):
        updates.insert(0, SCREENSHOT_ENTRY)
    updates.append(entry)
    data["reference_updates"] = updates
    PROGRESS.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    REPORT.write_text(REPORT_TEXT, encoding="utf-8")
    old_report = ROOT / "docs" / "validation" / "2026-10-15-m00-m02-l4-reference-supplements.md"
    if old_report.exists():
        old_report.unlink()
    print(json.dumps({"progress_entries": len(updates), "report": REPORT.name}, ensure_ascii=False))


if __name__ == "__main__":
    main()
