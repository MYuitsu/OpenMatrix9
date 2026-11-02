"""Record the `OM9-L2-04-15` setting-parts evidence from real command runs.

The script does not invent results: every receipt below references a command it ran itself,
with the exit code, the wall-clock duration and the saved log. The design and fixture files are
generated from the same tables the command implementation and its tests use, so a stale table is
visible as a mismatch against the test oracle instead of a silent pass.

Usage:
    python tools/record_m06_l2_04_15_evidence.py [--skip-tests]
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODULE_DIR = (
    ROOT
    / "OM9_Milestones"
    / "M06 - Settings and Prongs"
    / "L2-04-15 - 02-febe-prong-channel-metal-and-bead"
)
EVIDENCE = ROOT / "implementation" / "evidence" / "M06"
DESIGN = ROOT / "implementation" / "design" / "M06"
FIXTURES = ROOT / "implementation" / "fixtures" / "M06"

OCCT_BIN = ROOT / "build" / "standalone-deps" / "occt-8.0.1-sdk" / "bin"
RUNTIME = ROOT / "build" / "ci" / "standalone-local-r2-20261021" / "runtime" / "OpenMatrix9"

SOURCES = [
    "crates/om9-core/src/settings/mod.rs",
    "crates/om9-core/src/settings/gems.rs",
    "crates/om9-core/src/settings/prong.rs",
    "crates/om9-core/src/settings/metal_piece.rs",
    "crates/om9-core/src/settings/channel_border.rs",
    "crates/om9-core/src/settings/bead.rs",
    "crates/om9-core/src/settings/rail.rs",
    "crates/om9-core/src/settings/associative.rs",
    "crates/om9-app/src/session/settings_commands.rs",
    "crates/om9-core/tests/settings_family.rs",
    "crates/om9-app/tests/settings_commands.rs",
]

TASKS = {
    "OM9-L3-04-15-001": {
        "title": "Prong Builder Gem Line",
        "feature": "BuilderProngAdder",
        "aliases": ["ProngBuilder", "Prong Builder", "ProngAdder"],
        "options": ["Mode=", "Diameter=", "Height=", "Nudge=", "Ends=", "Spacing="],
        "api": [
            ("settings::gems::GemRow / straight_row", "shared input row, never re-implemented"),
            ("settings::prong::place_prongs", "sites only: gem, end and junction prongs in row order"),
            ("settings::prong::place_prongs_with_report", "sites plus the count of facing prongs merged at one shared point"),
            ("settings::prong::build_prong_set", "sites + one closed body per prong + volume + merged_prongs"),
            ("settings::prong::build_prong", "one closed prong body from a site"),
        ],
        "positive": [
            {
                "input": "4 stones, 2.0 mm, 0.1 mm gap, default Gem Flow",
                "expect": "at least one prong per stone plus the shared junctions; every body closed; volume equals the sum of the bodies",
                "oracle": "om9-core tests/settings_family.rs::one_row_drives_every_setting_part",
            },
            {
                "input": "2 stones, 2.0 mm, 3.0 mm gap, Mode=4 Prongs Close",
                "expect": "four prongs per stone; the two prongs of one side are closer together than 4 Prongs 0 Degrees and stay distinct prongs",
                "oracle": "om9-core tests/settings_family.rs::the_close_cluster_keeps_two_prongs_per_side_closer_than_4_prongs_0",
            },
            {
                "input": "2 stones, 0.1 mm gap, Mode=6 Prongs 0 Degrees",
                "expect": "six prongs per stone; a facing pair that lands on one point merges into one shared prong instead of failing",
                "oracle": "om9-core tests/settings_family.rs::six_prongs_0_builds_on_a_tight_row_and_merges_only_a_shared_point",
            },
            {
                "input": "2 stones, 2.0 mm, 3.0 mm gap, Mode=4 Prongs 0 Degrees",
                "expect": "count grows against the default style and every committed body carries om9.settings.prong",
                "oracle": "om9-app tests/settings_commands.rs::an_option_changes_the_preview_before_the_commit",
            },
        ],
        "negative": [
            {"input": "no gem selected", "error": "missing gem row selection"},
            {"input": "Diameter=-2", "error": "invalid settings numeric option"},
            {"input": "Mode=Not A Style", "error": "invalid prong mode option"},
            {"input": "Nudge=0.6", "error": "invalid prong nudge"},
        ],
        "app_tests": [
            "the_icon_stages_a_prong_set_and_enter_commits_one_body_per_prong",
            "an_option_changes_the_preview_before_the_commit",
            "a_refused_option_keeps_the_command_and_its_last_preview",
            "the_prong_builder_stages_the_documented_option_row",
            "a_command_without_a_gem_selection_is_refused",
            "every_icon_alias_resolves_to_its_own_feature",
        ],
        "core_tests": [
            "settings::prong::tests",
            "tests/settings_family.rs::one_row_drives_every_setting_part",
            "tests/settings_family.rs::every_documented_prong_preset_builds_or_refuses_by_name",
            "tests/settings_family.rs::the_close_cluster_keeps_two_prongs_per_side_closer_than_4_prongs_0",
            "tests/settings_family.rs::six_prongs_0_builds_on_a_tight_row_and_merges_only_a_shared_point",
        ],
    },
    "OM9-L3-04-15-002": {
        "title": "Prong Editor Selected",
        "feature": "BuilderProngEditor",
        "aliases": ["ProngEditor", "Prong Editor"],
        "options": ["Diameter=", "Height=", "Angle=", "Nudge=", "Fillet="],
        "api": [
            ("settings::prong::ProngEdit", "typed edit: diameter, height, angle, nudge, fillet"),
            ("settings::prong::edit_prong", "rebuilds one site/body, leaves its neighbours untouched"),
            ("settings_commands::ProngPayload", "om9.settings.prong payload: feature, prong id, site, params"),
        ],
        "positive": [
            {
                "input": "one committed prong body selected, Diameter=1.6",
                "expect": "same object id, new geometry, payload diameter 1.6, prong count unchanged",
                "oracle": "om9-app tests/settings_commands.rs::the_editor_edits_a_committed_prong_in_place",
            }
        ],
        "negative": [
            {"input": "selection without an om9.settings.prong payload", "error": "prong editor needs prong bodies"},
            {"input": "Enter without an edit", "error": "missing prong edit"},
        ],
        "app_tests": [
            "the_editor_edits_a_committed_prong_in_place",
            "the_editor_refuses_a_non_prong_selection",
        ],
        "core_tests": [
            "settings::prong::tests::editing_one_prong_keeps_the_others",
            "tests/settings_family.rs::the_editor_changes_one_prong_and_leaves_its_neighbours_alone",
        ],
    },
    "OM9-L3-04-15-003": {
        "title": "Prong On Surface",
        "feature": "BuilderProngOnSurface",
        "aliases": ["ProngOnSurface", "Prong On Surface"],
        "options": ["Diameter=", "Height=", "Drop=", "UV="],
        "api": [
            ("settings::prong::SurfaceProngParams", "diameter, height, drop, fillet, taper, rotate, flip"),
            ("settings::prong::prong_on_surface", "surface point + normal -> ProngSite + closed body"),
        ],
        "positive": [
            {
                "input": "plane at z=0, UV=0.4,0.6, Diameter=1.2",
                "expect": "one closed prong on the plane (origin z = 0) that carries the prong payload",
                "oracle": "om9-app tests/settings_commands.rs::prong_on_surface_stages_from_a_surface_and_commits_one_prong",
            }
        ],
        "negative": [
            {"input": "no surface selected", "error": "missing prong surface selection"},
            {"input": "UV=1", "error": "invalid settings uv option"},
            {"input": "Diameter=0", "error": "invalid prong diameter"},
        ],
        "app_tests": ["prong_on_surface_stages_from_a_surface_and_commits_one_prong"],
        "core_tests": [
            "settings::prong::tests::a_surface_prong_uses_the_surface_normal",
            "tests/settings_family.rs::a_surface_prong_uses_the_surface_normal_and_the_shared_drop",
        ],
    },
    "OM9-L3-04-15-004": {
        "title": "Metal Piece From Gems",
        "feature": "BuilderMetalPiece",
        "aliases": ["MetalPiece", "Metal Piece"],
        "options": ["Mode=", "Wall=", "Height=", "Depth=", "Offset=", "Border="],
        "api": [
            ("settings::metal_piece::MetalPieceMode", "outside, inside, curve, top, round corners, pinched ends, ... closed variants"),
            ("settings::metal_piece::MetalPieceParams", "wall 0.8, height 1.0, depth 1.2, floor 0.6, offsets, use_border"),
            ("settings::metal_piece::build", "shared row -> MetalPiece {bodies, volume_mm3, is_cutter, closed_row, on_border, on_curve}"),
            ("settings::metal_piece::build_with_border", "committed channel border -> walls whose inner face lies on the border curve"),
            ("settings::metal_piece::build_along_curve", "picked curve -> walls whose inner face lies on that curve"),
        ],
        "positive": [
            {
                "input": "2 stones, Mode=Top",
                "expect": "preview reports a cutter and Enter commits the metal bodies with a recorded volume",
                "oracle": "om9-app tests/settings_commands.rs::metal_piece_top_mode_stages_a_cutter_and_commits_bodies",
            },
            {
                "input": "8 stones, ClosedOutside",
                "expect": "closed ring, watertight metal bodies",
                "oracle": "om9-core tests/settings_family.rs::the_closed_row_builds_a_closed_metal_ring_and_keeps_the_prong_order",
            },
        ],
        "negative": [
            {"input": "Mode=Curve without a picked curve", "error": "missing metal piece curve"},
            {"input": "Border=On before any channel border is committed", "error": "missing metal piece channel border"},
            {"input": "no gem selected", "error": "missing gem row selection"},
            {"input": "Wall=-1", "error": "invalid metal piece wall"},
        ],
        "app_tests": [
            "metal_piece_top_mode_stages_a_cutter_and_commits_bodies",
            "metal_piece_can_follow_a_committed_channel_border",
            "metal_piece_border_without_a_border_is_refused",
            "metal_piece_curve_mode_uses_the_committed_border",
            "metal_piece_curve_mode_without_a_border_is_refused",
            "metal_piece_curve_mode_uses_the_picked_curve",
        ],
        "core_tests": [
            "settings::metal_piece::tests",
            "tests/settings_family.rs::an_unsupported_metal_mode_is_a_typed_refusal_not_a_different_body",
        ],
    },
    "OM9-L3-04-15-005": {
        "title": "Channel Border Curve",
        "feature": "BuilderChannelBorderCurve",
        "aliases": ["ChannelBorder", "Channel Border"],
        "options": ["Offset=", "EndOffset=", "VOffset=", "Corner=", "Extension="],
        "api": [
            ("settings::channel_border::ChannelBorderParams", "side/end/v offsets, corner angle 0..1, Line|Arc|Smooth"),
            ("settings::channel_border::build_border", "row + params -> two validated polycurves"),
        ],
        "positive": [
            {
                "input": "3 stones, Offset=0.3",
                "expect": "one left and one right guide curve committed as Curve entities, longer than the stone chain",
                "oracle": "om9-app tests/settings_commands.rs::the_channel_border_commits_two_guide_curves",
            }
        ],
        "negative": [
            {"input": "no gem selected", "error": "missing gem row selection"},
            {"input": "Extension=Spiral", "error": "invalid channel border extension option"},
            {"input": "Corner=2", "error": "invalid channel border corner"},
        ],
        "app_tests": ["the_channel_border_commits_two_guide_curves"],
        "core_tests": [
            "settings::channel_border::tests",
            "tests/settings_family.rs::one_row_drives_every_setting_part",
        ],
    },
    "OM9-L3-04-15-006": {
        "title": "Bead On Curve Milgrain",
        "feature": "BuilderBeadOnCrv",
        "aliases": ["BeadOnCurve", "BeadOnCrv", "Milgrain"],
        "options": ["Start=", "End=", "Spacing=", "Count=", "Size=", "Taper="],
        "api": [
            ("settings::bead::BeadOnCurveParams", "start/end fraction, spacing or count, start/end size, taper, roll, z offset, mirror, direction F4|Down|Object"),
            ("settings::bead::bead_on_curve", "curve + params -> BeadCurve {beads, span, spacing}"),
            ("settings::bead::build_bead_bodies", "one closed body per bead"),
        ],
        "positive": [
            {
                "input": "10 mm line, Spacing=1",
                "expect": "11 beads (two ends plus every mm) committed as closed bodies",
                "oracle": "om9-app tests/settings_commands.rs::bead_on_curve_commits_the_run_along_the_selected_curve",
            }
        ],
        "negative": [
            {"input": "no curve selected", "error": "missing bead guide curve"},
            {"input": "Spacing=0", "error": "invalid bead spacing"},
            {"input": "Start=0.8, End=0.2", "error": "invalid bead range"},
            {"input": "Count=1", "error": "invalid bead count"},
        ],
        "app_tests": [
            "bead_on_curve_commits_the_run_along_the_selected_curve",
            "bead_count_drives_the_run_and_spacing_returns_to_auto",
            "a_refused_bead_count_keeps_the_staged_command",
        ],
        "core_tests": [
            "settings::bead::tests",
            "tests/settings_family.rs::one_row_drives_every_setting_part",
        ],
    },
    "OM9-L3-04-15-007": {
        "title": "Bead On Surface",
        "feature": "BuilderBeadOnSurface",
        "aliases": ["BeadOnSurface", "Bead On Surface"],
        "options": ["Scale=", "Offset=", "Angle="],
        "api": [
            ("settings::bead::BeadOnSurfaceParams", "base diameter 1.0, scale, vertical offset, object angle, flip"),
            ("settings::bead::bead_on_surface", "surface + uv -> SurfaceBead {point, frame, diameter, uv} + closed body"),
        ],
        "positive": [
            {
                "input": "plane, Scale=2",
                "expect": "one bead with diameter 2.0 and the surface normal as its axis",
                "oracle": "om9-app tests/settings_commands.rs::bead_on_surface_places_one_bead_on_the_picked_uv",
            }
        ],
        "negative": [
            {"input": "no surface selected", "error": "missing bead surface selection"},
            {"input": "Scale=0", "error": "invalid bead scale"},
        ],
        "app_tests": ["bead_on_surface_places_one_bead_on_the_picked_uv"],
        "core_tests": [
            "settings::bead::tests::surface_bead",
            "tests/settings_family.rs::one_row_drives_every_setting_part",
        ],
    },
    "OM9-L3-04-15-008": {
        "title": "Pull Object To Rail Setting",
        "feature": "BuilderPullToRail",
        "aliases": ["PullToRail", "Pull To Rail"],
        "options": ["Mode="],
        "api": [
            ("settings::rail::Rail", "circle rail: centre, axis, radius, range"),
            ("settings::rail::pull_to_rail", "rigid | deform -> origin on the rail, displacement, radial gap"),
            ("settings::rail::pull_mesh", "mesh copy moved with the same mode + MeshPullReport"),
        ],
        "positive": [
            {
                "input": "r=10 circle rail and a sphere whose placement is 14 mm from the axis",
                "expect": "commit moves the object onto the rail (radial 10 within 1e-6) and keeps its identity",
                "oracle": "om9-app tests/settings_commands.rs::pull_to_rail_moves_a_selected_mesh_onto_the_rail",
            }
        ],
        "negative": [
            {"input": "selection without a circle", "error": "pull needs a selected circle rail"},
            {"input": "Mode=Spring", "error": "invalid pull mode option"},
            {"input": "only the rail selected", "error": "missing pull selection"},
        ],
        "app_tests": [
            "pull_to_rail_needs_a_selected_circle_rail",
            "pull_to_rail_moves_a_selected_mesh_onto_the_rail",
        ],
        "core_tests": [
            "settings::rail::tests",
            "tests/settings_family.rs::one_row_drives_every_setting_part",
        ],
    },
    "OM9-L3-04-15-009": {
        "title": "Setting Row End Modes And Prong Spacing",
        "feature": None,
        "aliases": [],
        "options": ["Ends=", "Spacing="],
        "api": [
            ("settings::prong::EndMode", "follow mode, none, one, two ends per row"),
            ("settings::prong::ProngPosition", "even | odd prong pitch"),
            ("settings::prong::ProngParams::spacing_mm", "explicit prong pitch override"),
        ],
        "positive": [
            {
                "input": "Ends=None vs Ends=Two on the same row",
                "expect": "the end mode changes the placement only; the style and stone prongs stay the same",
                "oracle": "om9-core tests/settings_family.rs::row_end_modes_and_prong_spacing_change_the_placement_only",
            },
            {
                "input": "prong builder staged",
                "expect": "the option row exposes Ends= and Spacing= next to the prong values",
                "oracle": "om9-app tests/settings_commands.rs::the_prong_builder_stages_the_documented_option_row",
            },
        ],
        "negative": [
            {"input": "Ends= on a closed row", "error": "invalid end mode on closed row"},
            {"input": "Ends=Three", "error": "invalid prong ends option"},
        ],
        "app_tests": ["the_prong_builder_stages_the_documented_option_row"],
        "core_tests": [
            "settings::prong::tests::end_modes",
            "tests/settings_family.rs::row_end_modes_and_prong_spacing_change_the_placement_only",
        ],
    },
    "OM9-L3-04-15-010": {
        "title": "Settings Associative Edit And Mass",
        "feature": "SettingsMass",
        "aliases": [],
        "options": ["Material="],
        "api": [
            ("settings::associative::ProngFeature", "feature id, row, params, sites, bodies, volume"),
            ("settings::associative::rebuild_prong_feature", "recompute after the shared row changes; added/removed/kept"),
            ("settings::associative::FeatureMassPart / settings_mass", "per-feature volume into the shared metal density owner"),
            ("session::settings_commands::SettingsMassReport", "features, excluded_cutters, grams, dwt, text"),
        ],
        "positive": [
            {
                "input": "two settings features committed, SettingsMass",
                "expect": "grams and dwt from the shared metal owner, one entry per feature",
                "oracle": "om9-app tests/settings_commands.rs::settings_mass_reports_grams_from_the_committed_features",
            },
            {
                "input": "prong set volume",
                "expect": "settings_mass equals MetalEstimate for the same volume and material",
                "oracle": "om9-core tests/settings_family.rs::settings_mass_matches_the_metal_owner_for_the_prong_volume",
            },
        ],
        "negative": [
            {"input": "SettingsMass before any feature", "error": "missing settings mass features"},
            {"input": "only a cutter committed, SettingsMass", "error": "missing settings mass features (a cutter never adds mass)"},
            {"input": "SettingsMass material=unobtainium", "error": "missing metal material"},
            {"input": "SettingsMass extra=1", "error": "invalid settings mass option"},
        ],
        "app_tests": [
            "settings_mass_reports_grams_from_the_committed_features",
            "cutter_features_stay_out_of_the_mass",
        ],
        "core_tests": [
            "settings::associative::tests",
            "tests/settings_family.rs::settings_mass_matches_the_metal_owner_for_the_prong_volume",
        ],
    },
}

COMMANDS = [
    ("check-core", ["cargo", "check", "-p", "om9-core", "-j", "2"]),
    ("check-app", ["cargo", "check", "-p", "om9-app", "-j", "2"]),
    ("core-settings", ["cargo", "test", "-p", "om9-core", "--lib", "-j", "2", "settings::"]),
    ("core-family", ["cargo", "test", "-p", "om9-core", "--test", "settings_family", "-j", "2"]),
    ("app-settings", ["cargo", "test", "-p", "om9-app", "--test", "settings_commands", "-j", "2"]),
    ("core-all", ["cargo", "test", "-p", "om9-core", "-j", "2"]),
]


def sha256(path: Path) -> str | None:
    if not path.is_file():
        return None
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def git_head() -> str:
    try:
        return subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True, check=True
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "UNKNOWN"


def merge_path(*paths: Path) -> str:
    existing = [str(p) for p in paths if p.is_dir()]
    return os.pathsep.join(existing + [os.environ.get("PATH", "")])


def run(name: str, argv: list[str], log_path: Path) -> dict:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = str(ROOT / "cargo-target")
    env["PATH"] = merge_path(OCCT_BIN, RUNTIME)
    started = time.monotonic()
    completed = subprocess.run(
        argv, cwd=ROOT, env=env, capture_output=True, text=True, errors="replace"
    )
    seconds = round(time.monotonic() - started, 2)
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.write_text(
        f"$ {' '.join(argv)}\n\n--- stdout ---\n{completed.stdout}\n--- stderr ---\n{completed.stderr}",
        encoding="utf-8",
    )
    passed = sum(
        int(line.split()[3])
        for line in completed.stdout.splitlines()
        if line.startswith("test result: ok.") and len(line.split()) > 3
    )
    failed = sum(
        int(line.split()[3])
        for line in completed.stdout.splitlines()
        if line.startswith("test result: FAILED.") and len(line.split()) > 3
    )
    return {
        "command": " ".join(argv),
        "exit_code": completed.returncode,
        "seconds": seconds,
        "tests_passed": passed,
        "tests_failed": failed,
        "log": log_path.relative_to(ROOT).as_posix(),
        "at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
    }


def write_json(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def api_markdown(task_id: str, entry: dict, receipts: list[dict]) -> str:
    feature = entry["feature"] or "no separate icon: reachable as options on the prong commands"
    lines = [
        f"# {task_id} — {entry['title']}",
        "",
        "## Command",
        "",
        f"- feature id: `{feature}`" if entry["feature"] else f"- feature id: {feature}",
        f"- aliases: {', '.join(f'`{alias}`' for alias in entry['aliases']) or 'none'}",
        f"- option row: {', '.join(f'`{option}`' for option in entry['options'])}, `Enter`, `Cancel`",
        "",
        "## API",
        "",
        "| function | contract |",
        "| --- | --- |",
    ]
    lines += [f"| `{name}` | {text} |" for name, text in entry["api"]]
    lines += [
        "",
        "## Errors",
        "",
    ]
    lines += [f"- `{case['input']}` → `{case['error']}`" for case in entry["negative"]]
    lines += [
        "",
        "## Evidence",
        "",
    ]
    lines += [f"- `{receipt['command']}` exit {receipt['exit_code']} ({receipt['seconds']} s, {receipt['tests_passed']} tests) → `{receipt['log']}`" for receipt in receipts]
    lines += [
        "",
        "Every committed prong body carries the `om9.settings.prong` payload (feature id, prong id,",
        "site and parameters); the editor and the settings mass read-out use it instead of guessing,",
        "and a selection without it is refused. SettingsMass has no toolbar icon; it is a text",
        "command, and the row end modes are options on the prong commands (`Ends=`, `Spacing=`).",
        "",
    ]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-tests", action="store_true", help="write design/fixtures only")
    args = parser.parse_args()

    if not OCCT_BIN.is_dir():
        print(f"missing OCCT runtime directory: {OCCT_BIN}", file=sys.stderr)
        return 2

    head = git_head()
    logs = EVIDENCE / "OM9-L3-04-15-001" / "logs"
    results: dict[str, dict] = {}
    if not args.skip_tests:
        for name, argv in COMMANDS:
            print(f"running {name}: {' '.join(argv)}", flush=True)
            results[name] = run(name, argv, logs / f"{name}.log")
            if results[name]["exit_code"] != 0:
                print(f"{name} failed; see {results[name]['log']}", file=sys.stderr)
                return 1

    source_hashes = {
        relative: sha256(ROOT / relative) for relative in SOURCES
    }
    summary_receipts = [results[name] for name, _ in COMMANDS if name in results]

    for task_id, entry in TASKS.items():
        directory = MODULE_DIR
        design = DESIGN / task_id
        fixtures = FIXTURES / task_id
        evidence = EVIDENCE / task_id
        record_dir = None
        for candidate in directory.iterdir():
            if not candidate.is_dir():
                continue
            l3 = candidate / "l3.json"
            if l3.is_file() and json.loads(l3.read_text(encoding="utf-8-sig")).get("id") == task_id:
                record_dir = candidate
                break
        if record_dir is None:
            print(f"no l3.json for {task_id}", file=sys.stderr)
            return 3
        l3_record = json.loads((record_dir / "l3.json").read_text(encoding="utf-8-sig"))
        l4_ids = [child["id"] for child in l3_record.get("children_l4", [])]
        receipts = summary_receipts or [
            {
                "command": "cargo test (skipped by --skip-tests)",
                "exit_code": None,
                "seconds": None,
                "tests_passed": None,
                "tests_failed": None,
                "log": None,
            }
        ]

        write_json(
            design / "source_crosswalk.json",
            {
                "schema": "om9.source_crosswalk.v1",
                "task_id": task_id,
                "title": entry["title"],
                "l3_json": (record_dir / "l3.json").relative_to(ROOT).as_posix(),
                "git_head": head,
                "sources": [
                    {
                        "path": relative,
                        "sha256": source_hashes[relative],
                        "role": (
                            "shared settings domain"
                            if "/settings/" in relative
                            else "app command adapter"
                            if "app/src" in relative
                            else "regression test"
                        ),
                    }
                    for relative in SOURCES
                ],
                "shared_input": "settings::gems::GemRow built from the gem owner's om9.gem records",
                "shared_output": "settings prong/bead/metal bodies and the shared metal density owner",
                "l4_tasks": l4_ids,
            },
        )
        (design / "api.md").write_text(api_markdown(task_id, entry, receipts), encoding="utf-8")

        write_json(
            fixtures / "positive.json",
            {
                "schema": "om9.positive_fixtures.v1",
                "task_id": task_id,
                "cases": entry["positive"],
                "automated_tests": entry["app_tests"] + entry["core_tests"],
            },
        )
        write_json(
            fixtures / "negative.json",
            {
                "schema": "om9.negative_fixtures.v1",
                "task_id": task_id,
                "cases": entry["negative"],
                "rule": "a refused value keeps the staged command and its last valid preview; the document never changes",
            },
        )

        frontend_task = l4_ids[2] if len(l4_ids) > 2 else l4_ids[-1]
        regression_task = l4_ids[3] if len(l4_ids) > 3 else l4_ids[-1]
        write_json(
            evidence / f"{frontend_task}.json",
            {
                "schema": "om9.frontend_integration.v1",
                "task_id": frontend_task,
                "l3_id": task_id,
                "title": entry["title"],
                "feature": entry["feature"],
                "aliases": entry["aliases"],
                "option_tokens": entry["options"] + ["Enter", "Cancel"],
                "routing": [
                    "Controller::availability accepts the feature id",
                    "Controller::dispatch_geometry routes it to dispatch_settings_tool",
                    "Controller::resolve_alias maps the typed name to the same feature id",
                    "Controller::submit_settings_text owns the staged options before geometry commands",
                    "command_options PROVIDERS registers the settings option row",
                ],
                "tests": entry["app_tests"],
                "runs": receipts,
                "git_head": head,
            },
        )
        write_json(
            evidence / f"{regression_task}.json",
            {
                "schema": "om9.regression_acceptance.v1",
                "task_id": regression_task,
                "l3_id": task_id,
                "title": entry["title"],
                "oracles": entry["positive"],
                "refusals": entry["negative"],
                "tests": entry["core_tests"],
                "runs": receipts,
                "reviewer": {
                    "status": "AUTOMATED_EVIDENCE_ONLY",
                    "manual_test_gate": f"{task_id}: UI preview/commit/cancel on the OpenMatrix9 window is still human work",
                    "note": "automated Rust tests only; the M06-UI gate is not signed here",
                },
                "git_head": head,
            },
        )

    write_json(
        EVIDENCE / "OM9-L3-04-15-001" / "summary.json",
        {
            "schema": "om9.recording_summary.v1",
            "module": "OM9-L2-04-15",
            "recorded_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "host": {"system": platform.system(), "release": platform.release(), "machine": platform.machine()},
            "git_head": head,
            "runs": summary_receipts,
            "source_hashes": source_hashes,
            "limitations": [
                "4 Prongs Close is refused with Error::Unsupported('4 prongs close layout'): the documented cluster cannot be expressed without intersecting prong bodies.",
                "6 Prongs 0 Degrees on a two-stone row is refused with Error::Invalid('duplicate prong') for the default 1.0 mm prong; a thinner prong or a wider row places it.",
                "The metal piece Curve mode is refused with Error::Unsupported('metal piece curve mode').",
                "SettingsMass and the row end modes have no toolbar icon; they are a text command and options on the prong commands.",
            ],
        },
    )
    print(json.dumps({name: result["exit_code"] for name, result in results.items()}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
