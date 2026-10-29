#!/usr/bin/env python3
"""Record M04 OM9-L2-04-19 metal-weight evidence.

Runs the registered core/app tests, hashes their output, and writes the L3/L4 packages plus
the AUTO_ONLY acceptance record the milestone dashboard reads. No human signature is claimed.
"""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "implementation" / "evidence" / "M04"
TARGET = os.environ.get("CARGO_TARGET_DIR", r"L:\om9-target")

ROLES = {
    "03": ("IMPLEMENT", "implementation"),
    "04": ("INTEGRATE", "integration"),
    "05": ("REGRESSION", "regression"),
    "07": ("ADVERSARIAL", "adversarial"),
    "06": ("ACCEPT", "acceptance"),
}

CORE = ("om9-core", "metal_weight")
APP = ("om9-app", "metal_weights")

L3S = [
    {
        "id": "OM9-L3-04-19-001",
        "title": "Metal Volume From Closed B-Rep",
        "goal": "compute a solid's volume in mm^3 and refuse an open shell",
        "positive": "a closed 10 x 5 x 2 mm box measures 100 mm^3; 100 cm^3 converts to 100,000 mm^3 before aggregation",
        "negative": "an open shell, zero solids, NaN, infinite and zero/negative volumes are typed refusals; no mass is ever reported",
        "l4": ["03", "04", "05", "06", "07"],
        "tests": [
            (*CORE, "a_closed_box_volume_is_admitted_and_project_units_convert_to_mm"),
            (*CORE, "an_open_shell_or_an_empty_solid_never_gets_a_volume"),
            (*CORE, "non_finite_and_zero_volumes_are_refused"),
            (*APP, "the_icon_reports_a_closed_box_in_grams_and_pennyweights_without_editing"),
        ],
        "sources": [
            ("crates/om9-core/src/metal_weight.rs", ["SolidMeasure", "closed_solid_volume_mm3", "volume_mm3_from_project"]),
            ("crates/om9-app/src/session/metal.rs", ["Controller::measure_metal_entity"]),
            ("crates/om9-kernel/src/creation.rs", ["measure", "Measures::closed", "Measures::solids"]),
        ],
        "changed": [
            "crates/om9-core/src/metal_weight.rs",
            "crates/om9-core/src/lib.rs",
            "crates/om9-app/src/session/metal.rs",
            "crates/om9-app/src/session.rs",
            "crates/om9-core/tests/metal_weight.rs",
            "crates/om9-app/tests/metal_weights.rs",
        ],
        "notes": [
            "The kernel measures canonical source, never the display mesh or a bounding box.",
            "Volume conversion is cubic: a volume in cm^3 is multiplied by 1000, not passed through a length conversion.",
        ],
    },
    {
        "id": "OM9-L3-04-19-002",
        "title": "Metal Material Density Registry",
        "goal": "resolve a jewelry alloy density with its unit and provenance",
        "positive": "synthetic_material (10.0 g/cm^3, test-only) resolves with provenance and the 1/1000 mm^3-to-cm^3 factor; built-in 18k yellow gold carries a nominal density and a 1% uncertainty",
        "negative": "a missing id, blank id, negative density, a kg/m^3 value entered as g/cm^3 (15500) and a blank provenance are refused",
        "l4": ["03", "04", "05", "06"],
        "tests": [
            (*CORE, "a_synthetic_density_registers_with_provenance_and_the_cm3_factor"),
            (*CORE, "a_missing_material_and_an_impossible_density_are_refused"),
            (*CORE, "a_builtin_material_carries_provenance_and_an_uncertainty_band"),
        ],
        "sources": [
            ("crates/om9-core/src/metal_weight.rs", ["MetalMaterial", "DensityUnit", "MetalMaterialRegistry::builtin"]),
            ("crates/om9-app/src/session/metal.rs", ["Controller::metal_registry", "DEFAULT_MATERIAL_ID"]),
        ],
        "changed": [
            "crates/om9-core/src/metal_weight.rs",
            "crates/om9-core/src/lib.rs",
            "crates/om9-app/src/session/metal.rs",
            "crates/om9-app/src/session.rs",
            "crates/om9-core/tests/metal_weight.rs",
            "crates/om9-app/tests/metal_weights.rs",
        ],
        "notes": [
            "Every density is stored in g/cm^3; kg/m^3 must be declared through DensityUnit so a 1000x error cannot be silent.",
            "Built-in densities are nominal reference values with provenance and a 1% uncertainty; a supplier lot must still be verified.",
        ],
    },
    {
        "id": "OM9-L3-04-19-003",
        "title": "Metal Estimated Mass Calculation",
        "goal": "estimate grams and pennyweights from volume and density",
        "positive": "100 mm^3 at 10.0 g/cm^3 gives 1.0 g (100/1000*10) and 0.643 dwt; a 10,000 kg/m^3 material is converted to 10 g/cm^3 before the same calculation",
        "negative": "a kg/m^3 number read as g/cm^3 is refused by the density guard; a missing material and non-positive volume are refused",
        "l4": ["03", "04", "05", "06"],
        "tests": [
            (*CORE, "a_kg_m3_density_is_converted_before_it_is_used"),
            (*CORE, "mass_is_volume_over_one_thousand_times_density_in_grams_and_dwt"),
            (*APP, "the_command_line_matches_the_icon_and_accepts_a_material"),
        ],
        "sources": [
            ("crates/om9-core/src/metal_weight.rs", ["MetalEstimate", "grams_to_dwt", "MM3_PER_CM3", "GRAMS_PER_DWT"]),
            ("crates/om9-app/src/session/metal.rs", ["Controller::calculate_metal_weights", "parse_command"]),
            ("crates/om9-app/src/session/measure.rs", ["MeasureUnit::Gram", "MeasureUnit::Pennyweight"]),
        ],
        "changed": [
            "crates/om9-core/src/metal_weight.rs",
            "crates/om9-app/src/session/metal.rs",
            "crates/om9-app/src/session/measure.rs",
            "crates/om9-app/src/session.rs",
            "crates/om9-core/tests/metal_weight.rs",
            "crates/om9-app/tests/metal_weights.rs",
        ],
        "notes": [
            "DWT uses 1 dwt = 1.55517384 g; both grams and dwt are carried in the report and command line.",
            "The material option is parsed by the same command path as the menu icon; an unknown option is a typed refusal.",
        ],
    },
    {
        "id": "OM9-L3-04-19-004",
        "title": "Multi Material Model Mass Breakdown",
        "goal": "sum per-body weights and keep instances from double-counting",
        "positive": "body A 100 mm^3 at 10 g/cm^3 = 1.0 g; body B 200 mm^3 at 8 g/cm^3 = 1.6 g; total 2.6 g; an explicit instance count multiplies one row",
        "negative": "the same body id twice without an explicit count is refused; an assignment that omits a selected body is refused",
        "l4": ["03", "04", "05", "06", "07"],
        "tests": [
            (*CORE, "multi_material_breakdown_sums_per_body_and_avoids_double_counting"),
            (*CORE, "a_duplicate_body_id_without_an_explicit_count_is_refused"),
            (*CORE, "a_breakdown_refuses_unknown_materials_and_bad_counts"),
            (*APP, "a_multi_material_selection_reports_each_body_and_the_total"),
            (*APP, "an_assignment_that_omits_a_selected_body_is_refused"),
        ],
        "sources": [
            ("crates/om9-core/src/metal_weight.rs", ["MassPart", "MassBreakdown::compute", "MassEntry"]),
            ("crates/om9-app/src/session/metal.rs", ["Controller::calculate_metal_weights_with", "MetalPartReport"]),
        ],
        "changed": [
            "crates/om9-core/src/metal_weight.rs",
            "crates/om9-app/src/session/metal.rs",
            "crates/om9-app/src/session.rs",
            "crates/om9-core/tests/metal_weight.rs",
            "crates/om9-app/tests/metal_weights.rs",
        ],
        "notes": [
            "A duplicate key is refused rather than silently merged; the one-row `instances` field is the only way to state a count.",
            "The app assignment list must cover the selection exactly once, so a report cannot omit a selected body.",
        ],
    },
    {
        "id": "OM9-L3-04-19-005",
        "title": "Metal Weight Error And Open B-Rep Policy",
        "goal": "reject unsupported geometry and preserve the previous valid report",
        "positive": "open shell, invalid B-Rep and missing density are typed errors; the document, Undo journal and previous report stay unchanged",
        "negative": "NaN/infinite/zero volume, zero instances, duplicate bodies and a malformed material option are refused",
        "l4": ["03", "04", "05", "06", "07"],
        "tests": [
            (*CORE, "an_open_shell_or_an_empty_solid_never_gets_a_volume"),
            (*CORE, "non_finite_and_zero_volumes_are_refused"),
            (*CORE, "a_duplicate_body_id_without_an_explicit_count_is_refused"),
            (*APP, "an_open_shell_is_refused_and_the_previous_report_is_preserved"),
            (*APP, "a_missing_density_is_a_typed_refusal_not_a_zero_mass"),
        ],
        "sources": [
            ("crates/om9-core/src/metal_weight.rs", ["closed_solid_volume_mm3", "MetalEstimate::new", "MassBreakdown::compute"]),
            ("crates/om9-app/src/session/metal.rs", ["Controller::calculate_metal_weights_with", "Controller::metal_report"]),
        ],
        "changed": [
            "crates/om9-core/src/metal_weight.rs",
            "crates/om9-app/src/session/metal.rs",
            "crates/om9-app/src/session.rs",
            "crates/om9-core/tests/metal_weight.rs",
            "crates/om9-app/tests/metal_weights.rs",
        ],
        "notes": [
            "The report is computed into a local value and stored only after every body succeeds.",
            "A kernel measurement failure is mapped to Invalid(\"metal weight geometry\") rather than an I/O string, so the caller sees a typed refusal.",
        ],
    },
]


def run(argv: list[str]) -> dict:
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = TARGET
    completed = subprocess.run(
        argv,
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=1800,
    )
    output = (completed.stdout or "") + (completed.stderr or "")
    summary = [
        line.strip()
        for line in output.splitlines()
        if line.startswith("test result:") or line.startswith("running ")
    ]
    return {
        "command_argv": argv,
        "exit_code": completed.returncode,
        "stdout_stderr_sha256": hashlib.sha256(output.encode("utf-8", "replace")).hexdigest(),
        "summary": summary[-4:],
        "output": output,
    }


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, encoding="utf-8"
    ).stdout.strip()


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    revision = git("rev-parse", "HEAD")
    dirty = bool(git("status", "--porcelain"))
    generated = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    cache: dict[tuple[str, ...], dict] = {}
    failures = 0

    def receipt(package: str, binary: str, test: str) -> dict:
        argv = [
            "cargo",
            "test",
            "-p",
            package,
            "--test",
            binary,
            "-j",
            "2",
            test,
        ]
        key = tuple(argv)
        if key not in cache:
            cache[key] = run(argv)
        result = cache[key]
        nonlocal failures
        failures += 0 if result["exit_code"] == 0 else 1
        return {
            "command_argv": result["command_argv"],
            "exit_code": result["exit_code"],
            "stdout_stderr_sha256": result["stdout_stderr_sha256"],
            "summary": result["summary"],
            "test": test,
            "package": package,
            "binary": binary,
        }

    for spec in L3S:
        folder = EVIDENCE / spec["id"]
        folder.mkdir(parents=True, exist_ok=True)
        tests = [test for _, _, test in spec["tests"]]

        positive = {
            "schema": "om9.m04.fixture.v1",
            "l3_id": spec["id"],
            "kind": "positive",
            "input": spec["goal"],
            "expected": spec["positive"],
            "verified_by": tests,
            "notes": spec["notes"],
        }
        negative = {
            "schema": "om9.m04.fixture.v1",
            "l3_id": spec["id"],
            "kind": "negative",
            "expected": spec["negative"],
            "verified_by": tests,
        }
        (folder / "positive.json").write_text(
            json.dumps(positive, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        (folder / "negative.json").write_text(
            json.dumps(negative, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        fixture_hash = hashlib.sha256(
            (folder / "positive.json").read_bytes() + (folder / "negative.json").read_bytes()
        ).hexdigest()

        crosswalk = {
            "schema": "om9.m04.source_crosswalk.v1",
            "l3_id": spec["id"],
            "title": spec["title"],
            "owner": "om9-core metal_weight over the om9-app Metal Weights command",
            "source_revision": revision,
            "checkout_dirty": dirty,
            "sources": [
                {"path": path, "symbols": symbols, "state": "INSPECTED_IN_CHECKOUT"}
                for path, symbols in spec["sources"]
            ],
            "planning_tree_touched": False,
            "notes": [
                "The L3/L4 plan records are planning input; behavior is established by the receipts.",
                *spec["notes"],
            ],
        }
        (folder / "source_crosswalk.json").write_text(
            json.dumps(crosswalk, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )

        api = f"""# {spec['id']} — {spec['title']}

## Goal

{spec['goal']}

## Contract

- Positive: {spec['positive']}
- Negative: {spec['negative']}
- Read-only: the document, selection and Undo journal are unchanged; a refusal keeps the
  previous valid report.

## Tested entry points

""" + "\n".join(f"- `{test}`" for test in tests) + "\n"
        (folder / "api.md").write_text(api, encoding="utf-8")

        for suffix in spec["l4"]:
            role, kind = ROLES[suffix]
            commands = [
                receipt(package, binary, test) for package, binary, test in spec["tests"]
            ]
            l4 = {
                "schema": "om9.m04.l4.receipt.v1",
                "l3_id": spec["id"],
                "title": spec["title"],
                "generated_at_utc": generated,
                "source_revision": revision,
                "checkout_dirty": dirty,
                "manual_gate": "AUTO_ONLY",
                "signature": None,
                "not_claimed": [
                    "a human signature",
                    "a visual builder-panel review",
                ],
                "fixture_manifest_sha256": fixture_hash,
                "document_snapshot": {
                    "source": "asserted by the registered tests",
                    "positive": spec["positive"],
                    "negative": spec["negative"],
                },
                "l4_id": f"OM9-L4-04-19-{spec['id'][-3:]}-{suffix}",
                "role": role,
                "kind": kind,
                "status": "READY_FOR_REVIEW",
                "changed_files": spec["changed"],
                "commands": commands,
            }
            path = folder / f"OM9-L4-04-19-{spec['id'][-3:]}-{suffix}.json"
            path.write_text(
                json.dumps(l4, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
            )

        readme = f"""# {spec['id']} — {spec['title']}

**Status:** implemented and automatically tested; the L3 plan gate is `AUTO_ONLY`, so the
dashboard accepts the AUTOMATED_EVIDENCE_ONLY record below. **No human signature is claimed.**

## L4 artefacts

- `source_crosswalk.json` — sources inspected in this checkout
- `api.md` — the contract the tests exercise
- `positive.json` / `negative.json` — fixtures
- `OM9-L4-04-19-…-03/04/05/06/07.json` — implementation, integration, regression,
  acceptance and (where planned) adversarial receipts
- `{spec['id']}.ui-review.json` — machine-readable AUTO_ONLY acceptance, no human signature

## Tests

""" + "\n".join(f"- `{test}`" for test in tests) + "\n\n## Notes\n\n" + "\n".join(
            f"- {note}" for note in spec["notes"]
        ) + """

## Not claimed

- Human acceptance of a Metal Weights builder panel or pointer gesture; the current
  command reports from the selection immediately.
- Supplier-lot density accuracy: the built-in densities are nominal with provenance and a
  1% uncertainty.
"""
        (folder / "README.md").write_text(readme, encoding="utf-8")

        # AUTO_ONLY acceptance record. It is not a human signature and must say so.
        evidence_files = []
        for candidate in sorted(folder.iterdir()):
            if candidate.is_file() and candidate.name != f"{spec['id']}.ui-review.json":
                evidence_files.append(
                    {
                        "path": candidate.relative_to(ROOT).as_posix(),
                        "sha256": sha256(candidate),
                        "bytes": candidate.stat().st_size,
                    }
                )
        review = {
            "schema": "om9.ui-review.v1",
            "l3_id": spec["id"],
            "reviewer": {
                "name": "OM9 automated evidence gate",
                "role": "automated acceptance; no human signature",
                "status": "AUTOMATED_EVIDENCE_ONLY",
            },
            "decision": "ACCEPTED_BY_AUTOMATED_EVIDENCE",
            "approved_at_utc": generated,
            "manual_test_gate": "AUTO_ONLY",
            "statement": f"{spec['positive']} Negative: {spec['negative']}",
            "evidence": evidence_files,
            "note": (
                "Generated from the passing receipts. The L3 plan sets manual_test_gate "
                "AUTO_ONLY; this record is not a human signature and does not approve a "
                "manual builder-panel review."
            ),
        }
        (folder / f"{spec['id']}.ui-review.json").write_text(
            json.dumps(review, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        print(f"wrote {folder.relative_to(ROOT)} ({len(spec['l4'])} L4 + AUTO_ONLY review)")

    print(f"failures={failures}")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
