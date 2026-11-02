"""Record the `OM9-L2-04-14` head/bezel evidence from real command runs.

The script does not invent results: every receipt references a command it ran itself, with the
exit code, the wall-clock duration and the saved log. It never creates a signed UI review, so the
dashboard keeps the nine L3 at WIP until a human signs the M06-UI gate.

Usage:
    python tools/record_m06_l2_04_14_evidence.py [--skip-tests]
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
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
    / "L2-04-14 - 01-febe-head-and-bezel-brep-settings"
)
EVIDENCE = ROOT / "implementation" / "evidence" / "M06"
DESIGN = ROOT / "implementation" / "design" / "M06"
FIXTURES = ROOT / "implementation" / "fixtures" / "M06"

OCCT_BIN = ROOT / "build" / "standalone-deps" / "occt-8.0.1-sdk" / "bin"
RUNTIME = ROOT / "build" / "ci" / "standalone-local-r2-20261021" / "runtime" / "OpenMatrix9"

SOURCES = [
    "crates/om9-core/src/settings/mod.rs",
    "crates/om9-core/src/settings/head.rs",
    "crates/om9-core/src/settings/bezel.rs",
    "crates/om9-core/src/model.rs",
    "crates/om9-core/src/gem.rs",
    "crates/om9-app/src/session/head_commands.rs",
    "crates/om9-app/src/session.rs",
    "crates/om9-app/src/command_options.rs",
    "crates/om9-app/tests/head_commands.rs",
]

TASKS = {
    "OM9-L3-04-14-001": {
        "title": "Head Builder Basket",
        "feature": "BuilderHead",
        "aliases": ["HeadBuilder", "Head Builder", "gvHeadBuilder"],
        "options": ["Mode=", "Prongs=", "Profile=", "ProfileSave=", "Rails=", "Height=", "Drop=", "SeatDepth=", "Level="],
        "api": [
            ("settings::head::GemGirdle::from_spec", "the gem owner's spec and placement become one girdle spine"),
            ("settings::head::build", "basket head: N closed prongs + gallery rails on the gem frame"),
            ("settings::head::HeadParams", "typed level-1 controls, validated before any geometry"),
            ("settings::head::ProfileLibrary", "prong/rail/edge profile library entry point"),
            ("settings::head::validate_clearance", "no metal vertex inside the gem surface"),
        ],
        "positive": [
            {
                "input": "round 6 mm gem, Prongs=4, Rails=2, default Height/Drop",
                "expect": "4 prongs and 2 rails, every body closed with positive volume, prong ids gem:<id>:prong:NN",
                "oracle": "om9-core src/settings/head.rs::a_basket_head_builds_closed_prongs_and_rails",
            },
            {
                "input": "oval 8x6 gem, Prongs=8",
                "expect": "8 prongs standing on the oval outline (nudge offset), long and short axes both represented, all bodies closed",
                "oracle": "om9-core src/settings/head.rs::non_round_girdles_build_and_keep_prong_symmetry",
            },
            {
                "input": "BuilderHead icon with a selected gem",
                "expect": "preview message reports prong/rail counts and volume; Enter commits one body per prong and rail with the om9.settings.head payload; Undo restores the document",
                "oracle": "om9-app tests/head_commands.rs::the_head_icon_stages_a_basket_and_enter_commits_prongs_and_rails",
            },
        ],
        "negative": [
            {"input": "Head drop below the culet", "error": "invalid head prong below pavilion"},
            {"input": "Metal body inside the pavilion", "error": "invalid head metal inside pavilion"},
            {"input": "Selection without a gem", "error": "missing head gem selection"},
        ],
        "app_tests": ["the_head_icon_stages_a_basket_and_enter_commits_prongs_and_rails"],
        "core_tests": [
            "a_basket_head_builds_closed_prongs_and_rails",
            "non_round_girdles_build_and_keep_prong_symmetry",
        ],
    },
    "OM9-L3-04-14-002": {
        "title": "Head Builder Half Bezel Mode",
        "feature": "BuilderHead",
        "aliases": ["HeadBuilder", "Head Builder", "gvHeadBuilder"],
        "options": ["Mode=HalfBezel", "Span=", "Position=", "Thickness=", "BezelHeight=", "SeatAngle="],
        "api": [
            ("settings::head::HeadMode", "Basket or Half Bezel, the manual's shape toggle"),
            ("settings::head::HalfBezelParams", "position, span, thickness, height and seat angle of the band"),
            ("settings::head::build_half_bezel", "one closed band following the girdle arc; the prong profile is ignored"),
        ],
        "positive": [
            {
                "input": "Mode=HalfBezel, Span=120",
                "expect": "no prongs, prong_profile_ignored true, one closed band with the 120 degree span, rails kept",
                "oracle": "om9-core src/settings/head.rs::the_half_bezel_band_replaces_the_prongs_and_ignores_the_profile",
            },
            {
                "input": "toggle the head icon to half bezel",
                "expect": "the preview reports the half-bezel span; Enter commits a Half Bezel body and no Prong body",
                "oracle": "om9-app tests/head_commands.rs::the_half_bezel_toggle_replaces_the_prongs_and_keeps_rails",
            },
        ],
        "negative": [
            {"input": "Half bezel span 0", "error": "invalid half bezel span"},
            {"input": "Half bezel thickness below 0.05 mm", "error": "invalid half bezel thickness"},
        ],
        "app_tests": ["the_half_bezel_toggle_replaces_the_prongs_and_keeps_rails"],
        "core_tests": ["the_half_bezel_band_replaces_the_prongs_and_ignores_the_profile"],
    },
    "OM9-L3-04-14-003": {
        "title": "Head Builder Prong Rail Level Edit",
        "feature": "BuilderHead",
        "aliases": ["HeadBuilder", "Head Builder", "gvHeadBuilder"],
        "options": ["Level=", "Prong=", "Rail=", "Size=", "RailDrop=", "RailSize=", "RailOffset="],
        "api": [
            ("settings::head::HeadLevel", "the manual's four levels: overall, grouped, ends, individual"),
            ("settings::head::HeadEdit", "one field plus the edited element"),
            ("settings::head::apply_edit", "rebuilds only the permitted element set; the rest keeps ids, frames and vertices"),
        ],
        "positive": [
            {
                "input": "Level=4, Prong=2, Size=1.3",
                "expect": "only prong 1 is rebuilt; prong 0 and rail 0 keep their exact vertices",
                "oracle": "om9-core src/settings/head.rs::a_level_four_edit_touches_only_the_selected_prong",
            },
            {
                "input": "Level=1, Height=2.2",
                "expect": "every prong takes the new height",
                "oracle": "om9-core src/settings/head.rs::an_overall_edit_rebuilds_every_prong",
            },
            {
                "input": "Level=4 Prong=2 Size=1.3 in the app preview",
                "expect": "the staged preview shows prong 1 at 1.3 mm and prong 0 at 0.9 mm; commit keeps four prongs",
                "oracle": "om9-app tests/head_commands.rs::a_level_four_edit_changes_only_its_prong_in_the_preview",
            },
        ],
        "negative": [
            {"input": "Grouped level with an overall field", "error": "invalid head edit level field"},
            {"input": "Prong index 0", "error": "invalid head prong index"},
        ],
        "app_tests": ["a_level_four_edit_changes_only_its_prong_in_the_preview"],
        "core_tests": [
            "a_level_four_edit_touches_only_the_selected_prong",
            "an_overall_edit_rebuilds_every_prong",
        ],
    },
    "OM9-L3-04-14-004": {
        "title": "Bezel Builder Full",
        "feature": "BuilderBezel",
        "aliases": ["BezelBuilder", "Bezel Builder", "gvBezelBuilder"],
        "options": ["Mode=", "Profile=", "ProfileSave=", "Height=", "Top=", "Base=", "Dome=", "Angle=", "Seat=", "Placement=", "Scale=", "ChamferX=", "ChamferZ=", "Shear=", "Boolean="],
        "api": [
            ("settings::bezel::BezelParams", "typed edge profile, height, top/base, dome, angles, placement, scale, chamfer, shear, boolean and the finger rail"),
            ("settings::bezel::section", "closed (radial, axial) section, sheared, upright or with the Boolean groove cut out"),
            ("settings::bezel::finger_rail", "the rail ring the Boolean subtracts; committed as its own Finger Rail body"),
            ("settings::bezel::build", "one closed ring swept along the girdle outline, plus the removed volume of the Boolean"),
            ("settings::bezel::BezelMode", "Bezel, Under Bezel and Gallery Rail"),
        ],
        "positive": [
            {
                "input": "each of the three modes with the default section (Boolean=On)",
                "expect": "one closed notched ring with positive volume, a Finger Rail body and a measured removed volume",
                "oracle": "om9-core src/settings/bezel.rs::the_three_modes_build_a_closed_ring",
            },
            {
                "input": "Edge profile names Chamfer, Round, Step, Knife",
                "expect": "every library entry builds; an unknown name is a typed edge profile error",
                "oracle": "om9-core src/settings/bezel.rs::the_edge_profile_is_read_from_the_library",
            },
            {
                "input": "Boolean=On vs Boolean=Off on the same ring",
                "expect": "the Boolean ring loses the finger rail volume and stays closed; Boolean=Off keeps the solid ring",
                "oracle": "om9-core src/settings/bezel.rs::the_boolean_cuts_the_finger_rail_groove_out_of_the_ring",
            },
            {
                "input": "Bezel Builder icon, Mode=Bezel",
                "expect": "one notched Bezel body with the om9.settings.bezel payload and one Finger Rail body",
                "oracle": "om9-app tests/head_commands.rs::the_three_bezel_modes_build_and_the_boolean_cuts_a_finger_rail",
            },
        ],
        "negative": [
            {"input": "Finger rail size 0", "error": "invalid finger rail size"},
            {"input": "Height below 0.1 mm", "error": "invalid bezel height"},
            {"input": "Seat length above half the base thickness", "error": "invalid bezel seat length"},
        ],
        "app_tests": ["the_three_bezel_modes_build_and_the_boolean_cuts_a_finger_rail"],
        "core_tests": [
            "the_three_modes_build_a_closed_ring",
            "the_edge_profile_is_read_from_the_library",
            "the_boolean_cuts_the_finger_rail_groove_out_of_the_ring",
        ],
    },
    "OM9-L3-04-14-005": {
        "title": "Bezel Builder Under Bezel And Gallery",
        "feature": "BuilderBezel",
        "aliases": ["BezelBuilder", "Bezel Builder", "gvBezelBuilder"],
        "options": ["Mode=UnderBezel", "Mode=GalleryRail", "Height=", "Top=", "Base=", "Profile=", "Placement="],
        "api": [
            ("settings::bezel::BezelParams.mode", "Under Bezel and Gallery Rail share the section builder"),
            ("settings::bezel::section", "under-bezel band shifted below the girdle; gallery rail ring below it"),
            ("settings::bezel::build", "same closed-ring and Boolean gates for both modes"),
            ("settings::bezel::clearance", "measures the pavilion cone a lower ring must clear"),
        ],
        "positive": [
            {
                "input": "Mode=UnderBezel and Mode=GalleryRail",
                "expect": "the lowest vertex of both bodies sits below the girdle bottom, both bodies are closed and both Boolean grooves are cut",
                "oracle": "om9-core src/settings/bezel.rs::the_under_bezel_and_gallery_rail_sit_below_the_girdle",
            },
            {
                "input": "the same two modes through the icon",
                "expect": "one Bezel body each, lowest vertex below zero, plus the Finger Rail body",
                "oracle": "om9-app tests/head_commands.rs::the_three_bezel_modes_build_and_the_boolean_cuts_a_finger_rail",
            },
        ],
        "negative": [
            {"input": "Base thickness below 0.1 mm", "error": "invalid bezel base thickness"},
            {"input": "Scale 0", "error": "invalid bezel scale"},
        ],
        "app_tests": ["the_three_bezel_modes_build_and_the_boolean_cuts_a_finger_rail"],
        "core_tests": ["the_under_bezel_and_gallery_rail_sit_below_the_girdle"],
    },
    "OM9-L3-04-14-006": {
        "title": "Scallop Bezel Builder",
        "feature": "BuilderBezelCutter",
        "aliases": ["ScallopBezel", "Scallop Bezel", "BezelCutter", "gvBezelCutter"],
        "options": ["Profile=", "ProfileSave=", "Count=", "Angle=", "Offset=", "Depth="],
        "api": [
            ("settings::bezel::ScallopParams", "profile, count, pattern angle, offset and radial depth"),
            ("settings::bezel::build_scallop", "one closed cutter body per position, extruded along the outline tangent"),
            ("settings::bezel::ScallopSetting::is_cutter", "cutters never enter the settings metal mass"),
            ("session::settings_commands::SettingsMassReport", "publishes the cutter volume and the net metal"),
        ],
        "positive": [
            {
                "input": "12 cutters, Round edge profile, 0.05 mm offset",
                "expect": "12 closed cutter bodies, positive cutter volume, summary reports the count",
                "oracle": "om9-core src/settings/bezel.rs::scallop_builds_one_closed_cutter_per_position",
            },
            {
                "input": "Scallop Bezel icon after a head commit",
                "expect": "12 Scallop Cutter bodies with the om9.settings.scallop payload; SettingsMass counts one excluded cutter and reports the cutter volume and the net metal",
                "oracle": "om9-app tests/head_commands.rs::scallop_cutters_commit_and_stay_out_of_the_settings_mass",
            },
        ],
        "negative": [
            {"input": "Count 200", "error": "invalid scallop count"},
            {"input": "Unknown edge profile", "error": "invalid edge profile"},
            {"input": "Depth below 0.1 mm", "error": "invalid scallop depth"},
        ],
        "app_tests": ["scallop_cutters_commit_and_stay_out_of_the_settings_mass"],
        "core_tests": ["scallop_builds_one_closed_cutter_per_position", "the_scallop_takes_an_explicit_section"],
    },
    "OM9-L3-04-14-007": {
        "title": "Head Bezel Profile Browser",
        "feature": None,
        "aliases": ["Profile=", "RailProfile=", "Edge profile library"],
        "options": ["Profile=", "ProfileSave=", "RailProfile="],
        "api": [
            ("settings::head::ProfileKind", "Prong, Rail and Edge libraries"),
            ("settings::head::Profile", "closed unit section with width/height validation"),
            ("settings::head::Profile::from_section", "a user section normalised to the unit profile frame"),
            ("settings::head::Profile::from_bezier", "closed cubic Bezier sampled into a typed user profile"),
            ("settings::head::ProfileLibrary::builtin", "read-only built-in entries; user records use the same typed list"),
            ("settings::head::ProfileLibrary::find", "name lookup with a typed error per library"),
        ],
        "positive": [
            {
                "input": "list the built-in Prong/Rail/Edge libraries",
                "expect": "5 prong, 4 rail and 4 edge profiles, each a closed unit section with positive width and height",
                "oracle": "om9-core src/settings/head.rs::an_unknown_profile_is_a_typed_error",
            },
            {
                "input": "Profile=Round on the head and Profile=Chamfer on the bezel",
                "expect": "the built profile uses the library section; profiles never insert a scene object",
                "oracle": "om9-app tests/head_commands.rs::the_head_icon_stages_a_basket_and_enter_commits_prongs_and_rails",
            },
            {
                "input": "Profile=Bezier:0,0;0.5,0.1;0.55,0.8;0,1 then ProfileSave=Leaf then Profile=Leaf",
                "expect": "the Bezier section drives the prongs; saving keeps it in the session library and the name resolves it again",
                "oracle": "om9-app tests/head_commands.rs::the_bezier_profile_is_saved_and_resolved_by_name",
            },
        ],
        "negative": [
            {"input": "Profile=Unobtainium on the head", "error": "invalid prong profile"},
            {"input": "Profile=Nonexistent on the bezel", "error": "invalid edge profile"},
            {"input": "Bezier with 2 control points", "error": "invalid profile bezier control"},
        ],
        "app_tests": [
            "the_head_icon_stages_a_basket_and_enter_commits_prongs_and_rails",
            "the_bezier_profile_is_saved_and_resolved_by_name",
        ],
        "core_tests": [
            "an_unknown_profile_is_a_typed_error",
            "the_edge_profile_is_read_from_the_library",
            "a_user_bezier_section_drives_the_prongs",
        ],
    },
    "OM9-L3-04-14-008": {
        "title": "Head Bezel Girdle Seat Attachment",
        "feature": None,
        "aliases": ["Seats=", "seat frames of the head"],
        "options": ["Seats=", "SeatAngle=", "SeatDepth="],
        "api": [
            ("settings::head::SeatParams", "count, seat angle, depth and offset"),
            ("settings::head::seat_frames", "one stable frame per prong on the girdle plane"),
            ("settings::head::seat_cutters", "one closed wedge cutter per seat, angled by the seat angle"),
            ("settings::head::cut_seats", "subtracts the seats from the stone and returns the seated mesh and removed volume"),
            ("settings::head::SeatFrame", "id, gem id, angle, origin, unit normal, tangent, angle and depth"),
        ],
        "positive": [
            {
                "input": "4 seats on a round girdle at 0.15 mm depth",
                "expect": "four frames whose origin sits the seat depth below the girdle plane, unit normal, ids gem:<id>:seat:NN, identical on a rebuild",
                "oracle": "om9-core src/settings/head.rs::seat_frames_are_stable_and_on_the_girdle",
            },
            {
                "input": "Enter on a head with the default seat depth",
                "expect": "the gem body is replaced by the seated mesh (smaller stone), the seat cut payload is stored and Undo restores the whole document",
                "oracle": "om9-app tests/head_commands.rs::the_seat_cut_opens_the_stone_and_undo_restores_it",
            },
            {
                "input": "every head preview",
                "expect": "the message reports the seat count and the seat depth; the frames follow the same girdle as the prongs",
                "oracle": "om9-app tests/head_commands.rs::the_head_icon_stages_a_basket_and_enter_commits_prongs_and_rails",
            },
        ],
        "negative": [
            {"input": "Seat count 2", "error": "invalid seat count"},
            {"input": "Seat angle 90", "error": "invalid seat angle"},
        ],
        "app_tests": ["the_seat_cut_opens_the_stone_and_undo_restores_it"],
        "core_tests": [
            "seat_frames_are_stable_and_on_the_girdle",
            "the_seat_cut_removes_stone_under_the_girdle",
        ],
    },
    "OM9-L3-04-14-009": {
        "title": "Head Bezel Geometry Validity And Update",
        "feature": None,
        "aliases": ["head payload", "rebuild of a committed head"],
        "options": ["Level=", "Enter"],
        "api": [
            ("settings::head::HeadFeature", "feature id, gem id, typed params and revision"),
            ("settings::head::check", "measured closed/positive-volume/deterministic report with the gem dependency"),
            ("settings::head::rebuild", "identical inputs reproduce identical bodies"),
            ("settings::head::update", "same gem preserves ids, a new gem replaces them"),
            ("session::head_commands::HeadPayload", "the committed bodies carry the feature state for the next edit"),
        ],
        "positive": [
            {
                "input": "rebuild the same feature twice, then update it with the same and with a new gem",
                "expect": "two identical settings; same gem keeps prong ids; a new gem replaces them",
                "oracle": "om9-core src/settings/head.rs::a_rebuild_is_deterministic_and_a_new_gem_replaces_the_ids",
            },
            {
                "input": "check a committed basket",
                "expect": "closed, positive volume, deterministic ids and the gem listed as dependency",
                "oracle": "om9-core src/settings/head.rs::a_basket_head_builds_closed_prongs_and_rails",
            },
        ],
        "negative": [
            {"input": "Rebuild a feature on a different gem id", "error": "invalid head gem dependency"},
        ],
        "app_tests": ["the_head_icon_stages_a_basket_and_enter_commits_prongs_and_rails"],
        "core_tests": ["a_rebuild_is_deterministic_and_a_new_gem_replaces_the_ids"],
    },
}

COMMANDS = [
    ("core_head", ["cargo", "test", "-p", "om9-core", "--lib", "settings::head", "-j", "2"]),
    ("core_bezel", ["cargo", "test", "-p", "om9-core", "--lib", "settings::bezel", "-j", "2"]),
    ("app_head", ["cargo", "test", "-p", "om9-app", "--test", "head_commands", "-j", "2"]),
    ("core_lib", ["cargo", "test", "-p", "om9-core", "--lib", "-j", "2"]),
    ("app_check", ["cargo", "check", "-p", "om9-app", "-j", "2"]),
]


def sha256(path: Path) -> str:
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


def run(argv: list[str], log_path: Path) -> dict:
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
    feature = entry["feature"] or "no separate icon: options on the head/bezel commands"
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
    lines += ["", "## Errors", ""]
    lines += [f"- `{case['input']}` → `{case['error']}`" for case in entry["negative"]]
    lines += ["", "## Evidence", ""]
    lines += [
        f"- `{receipt['command']}` exit {receipt['exit_code']} ({receipt['seconds']} s, "
        f"{receipt['tests_passed']} tests) → `{receipt['log']}`"
        for receipt in receipts
        if receipt["log"]
    ]
    lines += ["", "Automated evidence only. The manual M06-UI review is not signed.", ""]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-tests", action="store_true", help="write design/fixtures only")
    args = parser.parse_args()

    if not OCCT_BIN.is_dir():
        print(f"missing OCCT runtime directory: {OCCT_BIN}", file=sys.stderr)
        return 2

    head = git_head()
    logs = EVIDENCE / "OM9-L3-04-14-001" / "logs"
    results: dict[str, dict] = {}
    if not args.skip_tests:
        for name, argv in COMMANDS:
            print(f"running {name}: {' '.join(argv)}", flush=True)
            results[name] = run(argv, logs / f"{name}.log")
            if results[name]["exit_code"] != 0:
                print(f"{name} failed; see {results[name]['log']}", file=sys.stderr)
                return 1

    source_hashes = {relative: sha256(ROOT / relative) for relative in SOURCES}
    receipts = [results[name] for name, _ in COMMANDS if name in results]

    for task_id, entry in TASKS.items():
        record_dir = None
        for candidate in MODULE_DIR.iterdir():
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

        write_json(
            DESIGN / task_id / "source_crosswalk.json",
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
                "shared_input": "the gem owner's om9.gem record: GemSpec + document placement -> GemGirdle",
                "shared_output": "closed head/bezel meshes with the om9.settings.head/bezel/scallop payload",
                "l4_tasks": l4_ids,
            },
        )
        (DESIGN / task_id / "api.md").parent.mkdir(parents=True, exist_ok=True)
        (DESIGN / task_id / "api.md").write_text(
            api_markdown(task_id, entry, receipts), encoding="utf-8"
        )
        write_json(
            FIXTURES / task_id / "positive.json",
            {
                "schema": "om9.positive_fixtures.v1",
                "task_id": task_id,
                "cases": entry["positive"],
                "automated_tests": entry["app_tests"] + entry["core_tests"],
            },
        )
        write_json(
            FIXTURES / task_id / "negative.json",
            {
                "schema": "om9.negative_fixtures.v1",
                "task_id": task_id,
                "cases": entry["negative"],
                "rule": "a refused value keeps the staged command and its last valid preview; the document never changes",
            },
        )

        evidence = EVIDENCE / task_id
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
                    "Controller::dispatch_geometry routes it to dispatch_head_tool",
                    "Controller::resolve_alias maps the typed name to the same feature id",
                    "Controller::submit_head_text owns the staged options before geometry commands",
                    "command_options PROVIDERS registers the head option row",
                ],
                "tests": entry["app_tests"],
                "runs": receipts,
                "reviewer": {
                    "status": "AUTOMATED_EVIDENCE_ONLY",
                    "role": "automation",
                    "note": "no human UI review; M06-UI stays open",
                },
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
                "automated_tests": {"core": entry["core_tests"], "app": entry["app_tests"]},
                "negative_cases": entry["negative"],
                "manual_gate": {"gate": "M06-UI", "signed": False, "required": True},
                "runs": receipts,
                "reviewer": {
                    "status": "AUTOMATED_EVIDENCE_ONLY",
                    "role": "automation",
                    "note": "no human UI review; M06-UI stays open",
                },
                "git_head": head,
            },
        )

    write_json(
        EVIDENCE / "OM9-L3-04-14-001" / "summary.json",
        {
            "schema": "om9.evidence_summary.v1",
            "module": "OM9-L2-04-14",
            "recorded_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "platform": platform.platform(),
            "git_head": head,
            "runs": receipts,
            "sources": source_hashes,
            "manual_gate": {"gate": "M06-UI", "signed": False},
        },
    )

    lines = [
        "# OM9-L2-04-14 — 01-febe-head-and-bezel-brep-settings",
        "",
        "Head Builder (basket, half bezel, four edit levels, seat cut into the stone), Bezel Builder",
        "(Bezel, Under Bezel, Gallery Rail, Boolean finger rail groove), Scallop Bezel cutters, the",
        "profile library with user Bezier sections, the girdle seat frames and the rebuild/validity",
        "report. Automated Rust evidence only — the M06-UI manual review is open.",
        "",
        "## Commands",
        "",
        "| run | exit | tests | seconds | log |",
        "| --- | --- | --- | --- | --- |",
    ]
    for receipt in receipts:
        lines.append(
            f"| `{receipt['command']}` | {receipt['exit_code']} | {receipt['tests_passed']} | "
            f"{receipt['seconds']} | `{receipt['log']}` |"
        )
    lines += [
        "",
        "## Per L3",
        "",
        "| L3 | title | feature | design | fixtures |",
        "| --- | --- | --- | --- | --- |",
    ]
    for task_id, entry in TASKS.items():
        lines.append(
            f"| {task_id} | {entry['title']} | `{entry['feature'] or 'command options only'}` | "
            f"[api](../../../design/M06/{task_id}/api.md) | "
            f"[positive](../../../fixtures/M06/{task_id}/positive.json) / "
            f"[negative](../../../fixtures/M06/{task_id}/negative.json) |"
        )
    lines += [
        "",
        "## Gates",
        "",
        "- `M06-UI` manual review: **not signed**; the dashboard keeps the nine L3 at WIP.",
        "- Occlusion/cancel/undo: the app tests commit through the host transaction and Undo restores.",
        "- `Boolean=On` is a section-plane boolean: the finger rail is a ring on the same girdle, so the",
        "  groove is the rail section swept along the outline (exact for the ring rail; a non-ring target",
        "  would need the 3-D CSG owner). The Finger Rail body is committed in the groove it cut.",
        "- Profiles: `Profile=Bezier:x,y;x,y;...` samples a closed section and `ProfileSave=<name>` keeps it",
        "  in the session library, so the built-in library stays read-only; the Bezier control-point",
        "  editing UI still belongs to the UI owner.",
        "- Scallop cutters never enter the settings metal mass; the mass report adds the cutter volume",
        "  and the net metal (an upper bound on the removed metal, not a measured cut).",
        "- The seat frames are published by this owner; Enter now applies the seat cut to the gem body",
        "  through the transaction, so Undo restores both the stone and the head.",
        "",
    ]
    completion = EVIDENCE / "OM9-L2-04-14-COMPLETION" / "README.md"
    completion.parent.mkdir(parents=True, exist_ok=True)
    completion.write_text("\n".join(lines), encoding="utf-8")
    print(f"recorded {len(TASKS)} L3 evidence entries")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
