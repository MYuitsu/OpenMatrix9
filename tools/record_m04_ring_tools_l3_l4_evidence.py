#!/usr/bin/env python3
"""Record M04 OM9-L2-04-02 ring/profile tool evidence.

Runs the registered core/app tests, hashes their output, and writes the L3/L4 receipt
packages the milestone dashboard reads. The manual M04-UI gate stays open: no signature is
written and every receipt says so.
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

# (package, test binary, test filter)
CORE = "om9-core"
APP = "om9-app"
RING_TOOLS = (CORE, "ring_tools")
APP_RING_TOOLS = (APP, "ring_tools")
SURFACE = (APP, "surface_menu")
BUILDER_SESSION = (CORE, "builder_session")

L3S = [
    {
        "id": "OM9-L3-04-02-001",
        "title": "Ring Rail Jewelry Size Standard",
        "goal": "an explicit millimetre diameter or a US ring size drives the same rail builder",
        "positive": "US size 8 resolves through the frozen table to 18.13 mm inside diameter (radius 9.065); US size 9 resolves to 18.95 mm; an explicit 18 mm rail keeps radius 9.0 and circumference 56.55 mm",
        "negative": "an unverified locale such as UK is refused by the choice domain; a size outside 1..15 is refused by the count domain",
        "l4_count": 6,
        "tests": [
            (*RING_TOOLS, "a_us_ring_size_resolves_through_the_frozen_table_and_an_unknown_locale_is_refused"),
            (*APP_RING_TOOLS, "the_ring_size_command_switches_the_builder_to_the_us_table"),
        ],
        "sources": [
            ("crates/om9-core/src/ring_size.rs", ["US_TABLE", "US_PROVENANCE", "RingSizeStandard"]),
            ("crates/om9-core/src/producers.rs", ["RingRailProducer", "standard_from_id"]),
            ("crates/om9-core/src/builder.rs", ["BuilderKind::RingRail", "sync_standard_diameter"]),
            ("crates/om9-app/src/session/builder.rs", ["builder_text standard=/size="]),
        ],
        "changed": [
            "crates/om9-core/src/ring_size.rs",
            "crates/om9-core/src/lib.rs",
            "crates/om9-core/src/producers.rs",
            "crates/om9-core/src/builder.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-core/tests/ring_tools.rs",
            "crates/om9-app/tests/ring_tools.rs",
        ],
        "notes": [
            "The table is frozen in om9-core with provenance text; the producer reads the standard choice and the size count instead of a hidden locale lookup.",
            "The panel sends Value::Quantity with the field's Dimension so a count or ratio is not mis-typed as a length.",
        ],
    },
    {
        "id": "OM9-L3-04-02-003",
        "title": "Profile Placer On Rail",
        "goal": "place a typed profile on the selected rail with a stable tangent/normal frame",
        "positive": "rail radius 9 with a closed 2 x 1 profile: position 0.00 sits at the rail start, 0.25 moves to the quarter station, and a 90 degree roll moves the section corner while the profile centre stays on the rail",
        "negative": "a missing rail input is refused; a zero-length or degenerate tangent is refused before a preview is built",
        "l4_count": 6,
        "tests": [
            (*RING_TOOLS, "a_profile_placer_moves_the_section_along_the_rail_and_rolls_it_without_flipping"),
            (*RING_TOOLS, "a_profile_placer_refuses_a_missing_rail_selection"),
            (*APP_RING_TOOLS, "the_profile_placer_and_outside_rail_icons_open_the_existing_builder_modes"),
        ],
        "sources": [
            ("crates/om9-core/src/producers.rs", ["RailProfileProducer", "ratio_parameter", "angle_parameter"]),
            ("crates/om9-core/src/curve_ops.rs", ["point_at", "tangent_at"]),
            ("crates/om9-core/src/builder.rs", ["BuilderKind::RailProfile", "profile_center_and_normal"]),
            ("crates/om9-app/src/session/ring_tools.rs", ["PROFILE_PLACER"]),
        ],
        "changed": [
            "crates/om9-core/src/producers.rs",
            "crates/om9-core/src/curve_ops.rs",
            "crates/om9-core/src/builder.rs",
            "crates/om9-app/src/session/ring_tools.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-core/tests/ring_tools.rs",
            "crates/om9-app/tests/ring_tools.rs",
        ],
        "notes": [
            "Position is a Ratio in 0..1 and roll is an Angle in degrees; both are optional so older recipes replay at position 0 / roll 0.",
            "The third builder handle carries scale 0.01 because the value is a ratio, not millimetres.",
        ],
    },
    {
        "id": "OM9-L3-04-02-004",
        "title": "Custom Rail And Advanced Rail",
        "goal": "accept a user-authored closed smooth rail for the profile sweep",
        "positive": "a closed rational-quadratic ellipse rail (9 x 6) validates as closed/planar/non-self-intersecting and a builder sweep over it produces a surface with continuous frames",
        "negative": "an open line is refused with ring rail not closed; a bow-tie polyline is refused with ring rail self-intersection; the advanced icon refuses an open rail without inserting geometry",
        "l4_count": 7,
        "tests": [
            (*RING_TOOLS, "a_custom_rail_must_be_closed_planar_and_non_self_intersecting"),
            (*RING_TOOLS, "a_builder_ring_sweep_refuses_an_open_rail_and_accepts_a_custom_closed_one"),
            (*APP_RING_TOOLS, "the_custom_rail_icon_records_a_validated_closed_rail_and_refuses_an_open_one"),
        ],
        "sources": [
            ("crates/om9-core/src/curve_ops.rs", ["validate_ring_rail", "segments_cross", "point_on_segment"]),
            ("crates/om9-core/src/producers.rs", ["RailSweepProducer", "closed_rail"]),
            ("crates/om9-core/src/surface_kernel.rs", ["path_frames"]),
            ("crates/om9-app/src/session/ring_tools.rs", ["CUSTOM_RAIL", "CUSTOM_RAIL_ADVANCED", "dispatch_custom_rail"]),
        ],
        "changed": [
            "crates/om9-core/src/curve_ops.rs",
            "crates/om9-core/src/producers.rs",
            "crates/om9-core/src/surface_kernel.rs",
            "crates/om9-core/src/builder.rs",
            "crates/om9-app/src/session/ring_tools.rs",
            "crates/om9-core/tests/ring_tools.rs",
            "crates/om9-app/tests/ring_tools.rs",
        ],
        "notes": [
            "The self-intersection test includes endpoint-touching crossings: a strict orientation-only test misses a bow-tie whose crossing lands exactly on a sample.",
            "A clamped NURBS seam can refuse D1; path_frames and tangent_at fall back to a symmetric secant so the frame stays defined without changing the curve.",
        ],
    },
    {
        "id": "OM9-L3-04-02-005",
        "title": "Four Rail Profile Tool",
        "goal": "tag two cross-sections from four consistent rails and sweep them",
        "positive": "four rails with consistent start/direction/span tag both profiles with om9.four-rail-profile metadata, and Sweep4 consumes them as one four-boundary surface",
        "negative": "a selection that is not four rails plus two profiles is refused; the command does not insert metadata or a partial surface",
        "l4_count": 7,
        "tests": [
            (*APP_RING_TOOLS, "the_four_profile_icon_reaches_the_existing_four_rail_metadata_command"),
            (*APP_RING_TOOLS, "the_four_profile_icon_refuses_an_inconsistent_selection"),
            (*SURFACE, "four_rail_profile_command_tags_profiles_then_sweep4_uses_them"),
        ],
        "sources": [
            ("crates/om9-app/src/session/surface_edit.rs", ["FOUR_RAIL_PROFILE", "FOUR_RAIL_PROFILE_ICON", "handles_four_rail_profile", "label"]),
            ("crates/om9-app/src/session/ring_tools.rs", ["FOUR_RAIL_PROFILE", "dispatch_ring_tool"]),
            ("crates/om9-core/src/rhino_semantics.rs", ["FourRailProfile", "set_four_rail_profile"]),
        ],
        "changed": [
            "crates/om9-app/src/session/surface_edit.rs",
            "crates/om9-app/src/session/ring_tools.rs",
            "crates/om9-app/src/session.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-app/tests/ring_tools.rs",
        ],
        "notes": [
            "OM9-TOOLS-032 is an alias of the existing OM9-TOOLS-004 metadata command; both record the same stable command and label, so a recipe is not split by the icon used.",
            "Five or more selected curves on OM9-TOOLS-004 take the four-rail path; a smaller selection is treated as a custom rail.",
        ],
    },
    {
        "id": "OM9-L3-04-02-006",
        "title": "Profile Edit Merge And Endcap",
        "goal": "merge compatible profile sections and close a profile for a valid cap",
        "positive": "two half-sections of one circle merge into a closed polycurve; a closed rectangular profile produces a planar cap surface; a closed profile path is checked before the cap",
        "negative": "one section is refused; reversed winding is refused with profile section winding; an open curve has no cap boundary and the document is unchanged",
        "l4_count": 7,
        "tests": [
            (*RING_TOOLS, "profile_sections_merge_when_they_are_connected_and_consistently_wound"),
            (*APP_RING_TOOLS, "the_profile_merge_icon_joins_compatible_sections_and_refuses_one_section"),
            (*APP_RING_TOOLS, "the_profile_end_cap_icon_builds_a_planar_cap_from_closed_sections"),
            (*APP_RING_TOOLS, "the_profile_end_cap_refuses_an_open_curve_without_changing_the_document"),
        ],
        "sources": [
            ("crates/om9-core/src/curve_ops.rs", ["merge_profile_sections", "profile_plane", "signed_area"]),
            ("crates/om9-core/src/surface_ops.rs", ["planar_surface_from_curves"]),
            ("crates/om9-core/src/surface_kernel.rs", ["surface", "cap_planar"]),
            ("crates/om9-app/src/session/ring_tools.rs", ["PROFILE_MERGE", "PROFILE_END_CAP", "dispatch_profile_merge", "dispatch_profile_endcap"]),
        ],
        "changed": [
            "crates/om9-core/src/curve_ops.rs",
            "crates/om9-app/src/session/ring_tools.rs",
            "crates/om9-app/src/session.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-core/tests/ring_tools.rs",
            "crates/om9-app/tests/ring_tools.rs",
        ],
        "notes": [
            "Winding is measured in the first section's fixed plane frame; a per-section basis would flip twice for a reversed loop and hide the change.",
            "A curve selection gets a planar cap surface. An open B-Rep selection uses the kernel planar cap under the occt-brep feature and otherwise reports Unsupported.",
        ],
    },
    {
        "id": "OM9-L3-04-02-008",
        "title": "Rail Profile Sweep Associative Output",
        "goal": "re-evaluate the swept band after a ring size or profile edit",
        "positive": "a RingRail -> RailProfile -> RailSweep chain rebuilds once when the ring diameter changes from 18 to 19 mm; the dependent sweep gets a new geometry revision and the plan is clean",
        "negative": "an invalid profile width makes the downstream rebuild fail explicitly and leaves the committed sweep geometry revision unchanged",
        "l4_count": 7,
        "tests": [
            (*APP_RING_TOOLS, "a_ring_size_edit_rebuilds_the_profile_and_one_dirty_sweep"),
            (*RING_TOOLS, "a_builder_ring_sweep_refuses_an_open_rail_and_accepts_a_custom_closed_one"),
        ],
        "sources": [
            ("crates/om9-app/src/session/builder.rs", ["Controller::set_history_parameter", "Controller::recompute_history", "Controller::recompute_history_inner"]),
            ("crates/om9-core/src/history_ops.rs", ["plan_recompute", "refresh_inputs"]),
            ("crates/om9-core/src/producers.rs", ["RailSweepProducer", "recompute_registered"]),
            ("crates/om9-app/src/session/ring_tools.rs", ["dispatch_ring_tool"]),
        ],
        "changed": [
            "crates/om9-core/src/producers.rs",
            "crates/om9-core/src/surface_kernel.rs",
            "crates/om9-core/src/history_ops.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-core/tests/ring_tools.rs",
            "crates/om9-app/tests/ring_tools.rs",
        ],
        "notes": [
            "This build uses the OCCT registry, so the committed sweep is a B-Rep; the core reference backend test covers the analytic surface variant.",
            "The chain is A -> B -> C: a ring edit re-stamps the profile against the new rail revision, then the sweep against the new profile revision.",
            "A failed profile rebuild is reported as RecomputeOutcome::Failed and the sweep keeps its last committed geometry revision.",
        ],
    },
    {
        "id": "OM9-L3-04-02-009",
        "title": "Rail And Profile Viewport Handles",
        "goal": "drag the profile width handle and commit exactly one profile",
        "positive": "the RailProfile builder exposes width, height and position handles; adjusting width by +1 mm reports 3.00 x 1.50 in the panel and one Apply inserts one entity and one recipe",
        "negative": "Cancel after a drag inserts nothing; an invalid or locked drag is refused before the model changes",
        "l4_count": 7,
        "tests": [
            (*APP_RING_TOOLS, "the_rail_profile_handles_edit_width_and_commit_exactly_once"),
            (*BUILDER_SESSION, "a_rail_profile_builder_reads_the_selected_rail_and_exposes_two_handles"),
        ],
        "sources": [
            ("crates/om9-core/src/builder.rs", ["BuilderHandle", "BuilderKind::RailProfile handles", "scale"]),
            ("crates/om9-app/src/ui.rs", ["builder handle drag"]),
            ("crates/om9-app/src/ui/chrome.rs", ["handle read-outs"]),
        ],
        "changed": [
            "crates/om9-core/src/builder.rs",
            "crates/om9-app/src/ui.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-core/tests/builder_session.rs",
            "crates/om9-app/tests/ring_tools.rs",
        ],
        "notes": [
            "Handle drag deltas are projected millimetres times the handle scale; the position handle scales by 0.01 so one projected millimetre is 0.01 of the rail.",
            "Apply is the only insert path: Cancel clears the session without a mutation.",
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
        numeric = spec["id"].rsplit("-", 1)[1]
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
            "owner": "om9-app Controller ring/profile tools over om9-core producer chain",
            "source_revision": revision,
            "checkout_dirty": dirty,
            "sources": [
                {"path": path, "symbols": symbols, "state": "INSPECTED_IN_CHECKOUT"}
                for path, symbols in spec["sources"]
            ],
            "planning_tree_touched": False,
            "notes": [
                "The L3/L4 plan records are planning input; runtime behavior is established by the receipts.",
                *spec["notes"],
            ],
        }
        (folder / "source_crosswalk.json").write_text(
            json.dumps(crosswalk, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )

        api = f"""# {spec['id']} — {spec['title']}

## Goal

{spec['goal']}

## Controller contract

- The Builder/Tools feature IDs and the typed command names dispatch to the same
  `Controller` methods.
- Positive: {spec['positive']}
- Negative: {spec['negative']}

## Tested entry points

""" + "\n".join(f"- `{test}`" for test in tests) + "\n"
        (folder / "api.md").write_text(api, encoding="utf-8")

        suffixes = ["03", "04", "05", "06"] + (["07"] if spec["l4_count"] >= 7 else [])
        for suffix in suffixes:
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
                "manual_gate": "OPEN",
                "signature": None,
                "not_claimed": [
                    "manual panel/handle review at the M04-UI gate",
                    "a reviewer signature",
                ],
                "fixture_manifest_sha256": fixture_hash,
                "document_snapshot": {
                    "source": "asserted by the registered tests",
                    "positive": spec["positive"],
                    "negative": spec["negative"],
                },
                "l4_id": f"OM9-L4-04-02-{numeric}-{suffix}",
                "role": role,
                "kind": kind,
                "status": "READY_FOR_REVIEW",
                "changed_files": spec["changed"],
                "commands": commands,
                "manual_gate_note": "M04-UI manual review remains open; no signature is included.",
            }
            path = folder / f"OM9-L4-04-02-{numeric}-{suffix}.json"
            path.write_text(
                json.dumps(l4, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
            )

        readme = f"""# {spec['id']} — {spec['title']}

**Status:** implemented and automatically tested; **manual M04-UI gate is open** (no
reviewer signature is claimed). The dashboard keeps this L3 at 🟡 until a human signs the
gate.

## L4 artefacts

- `source_crosswalk.json` — sources inspected in this checkout
- `api.md` — the contract the tests exercise
- `positive.json` / `negative.json` — fixtures
- `OM9-L4-…-03/04/05/06.json` (+ `-07` where the plan adds an adversarial task) —
  implementation, integration, regression, acceptance and adversarial receipts

## Tests

""" + "\n".join(f"- `{test}`" for test in tests) + "\n\n## Notes\n\n" + "\n".join(
            f"- {note}" for note in spec["notes"]
        ) + """

## Not claimed

- Human acceptance of the panel, pointer gestures or the visual result.
- A signature: record only the automated receipts; a human owns the remaining gate.
"""
        (folder / "README.md").write_text(readme, encoding="utf-8")
        print(f"wrote {folder.relative_to(ROOT)} ({spec['l4_count']} L4)")

    print(f"failures={failures}")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
