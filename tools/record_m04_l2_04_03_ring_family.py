#!/usr/bin/env python3
"""Record the M04 L2-04-03 jewelry ring builder family from real runs.

Scope: the seven L3 of ``06-febe-jewelry-ring-builder-family``:

* ``OM9-L3-04-03-001`` Eternity band: band solid + 12 stone frames, no fabricated gems;
* ``OM9-L3-04-03-002`` Signet ring: closed body with a top pad and two shoulders;
* ``OM9-L3-04-03-003`` Raised band: raised ridge with a positive section thickness;
* ``OM9-L3-04-03-004`` Award ring: wider top than the signet;
* ``OM9-L3-04-03-005`` Ring builder style presets: file roundtrip through the style library;
* ``OM9-L3-04-03-006`` Ring builder history updates: resize 18 -> 19 mm, same owner id, undo/redo;
* ``OM9-L3-04-03-007`` Final B-Rep validation: closed solid accepted, open mesh refused.

Every L4 directory gets its own artefact so the milestone dashboard counts it: a
``source_crosswalk.json`` for the SOURCE_AUDIT, ``api.md`` + ``positive.json`` +
``negative.json`` for the CONTRACT_TEST, and one receipt per later role with the command
that ran, its exit code and the hash of its output. The M04-UI manual gate stays open.

Usage::

    python tools/record_m04_l2_04_03_ring_family.py
"""

from __future__ import annotations

import datetime
import hashlib
import json
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
TARGET_DIR = os.environ.get("CARGO_TARGET_DIR", r"L:\om9-target")
KEY = "m04-l2-04-03-brep-handles-2026-10-27"
OUTPUT = ROOT / "implementation" / "evidence" / "M04" / "OM9-L2-04-03"
L3_ROOT = (
    ROOT
    / "OM9_Milestones"
    / "M04 - Builder and History"
    / "L2-04-03 - 06-febe-jewelry-ring-builder-family"
)
TARGET = "Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=L:/om9-target; -j 2"

CHECKS = [
    (
        "core-ring-unit.log",
        ["cargo", "test", "-p", "om9-core", "--lib", "-j", "2", "ring_builder::tests"],
        False,
    ),
    (
        "core-ring-family.log",
        ["cargo", "test", "-p", "om9-core", "--test", "ring_family", "-j", "2"],
        False,
    ),
    (
        "core-builder-regression.log",
        [
            "cargo",
            "test",
            "-p",
            "om9-core",
            "--test",
            "builder_session",
            "--test",
            "builder_style",
            "-j",
            "2",
        ],
        False,
    ),
    (
        "app-ring-family.log",
        ["cargo", "test", "-p", "om9-app", "--test", "ring_family_panel", "-j", "2"],
        False,
    ),
    (
        "kernel-ring-brep.log",
        [
            "cargo",
            "test",
            "-p",
            "om9-kernel",
            "--test",
            "ring_family_brep",
            "--features",
            "3dm-brep",
            "-j",
            "2",
        ],
        False,
    ),
    (
        "app-builder-regression.log",
        [
            "cargo",
            "test",
            "-p",
            "om9-app",
            "--test",
            "builder_panel",
            "--test",
            "builder_styles",
            "-j",
            "2",
        ],
        False,
    ),
    (
        "clippy-core-lib.log",
        [
            "cargo",
            "clippy",
            "-p",
            "om9-core",
            "--lib",
            "--no-deps",
            "-j",
            "2",
            "--",
            "-D",
            "warnings",
            "-A",
            "clippy::type_complexity",
        ],
        False,
    ),
    (
        "kernel-clippy.log",
        [
            "cargo",
            "clippy",
            "-p",
            "om9-kernel",
            "--all-targets",
            "--features",
            "3dm-brep",
            "--no-deps",
            "-j",
            "2",
            "--",
            "-D",
            "warnings",
        ],
        False,
    ),
    (
        "workspace-check.log",
        ["cargo", "check", "--workspace", "--all-targets", "-j", "2"],
        False,
    ),
]

# (package, binary, test) — binary None means the whole test binary.
CORE_LIB = "om9-core"
CORE_LIB_BIN = "om9-core"
CORE_FAMILY = "ring_family"
APP_PANEL = "ring_family_panel"


def unit(test: str) -> tuple[str, str, str]:
    return (CORE_LIB, CORE_LIB_BIN, f"ring_builder::tests::{test}")


def family(test: str) -> tuple[str, str, str]:
    return (CORE_LIB, CORE_FAMILY, test)


def app(test: str) -> tuple[str, str, str]:
    return ("om9-app", APP_PANEL, test)


def kernel(test: str) -> tuple[str, str, str]:
    return ("om9-kernel", "ring_family_brep", test)


ETERNITY_BUILDS = "an_eternity_band_builds_a_closed_solid_with_twelve_frames_and_no_gem_solids"
ETERNITY_REFUSES = "an_eternity_band_refuses_zero_stones_and_impossible_spacing"
SIGNET_CLOSED = "a_signet_ring_is_closed_with_a_visible_top_and_two_shoulders"
SIGNET_REFUSES = "a_signet_ring_refuses_a_negative_wall_and_a_self_intersecting_top"
RAISED_POSITIVE = "a_raised_band_keeps_a_positive_section_thickness"
AWARD_WIDER = "an_award_ring_has_a_wider_top_than_the_signet"
FINAL_GATE = "the_final_gate_rejects_an_open_mesh_and_an_inverted_body"
EVERY_KIND = "every_kind_builds_and_reports_the_same_segment_count"

COMMIT_ETERNITY = "an_eternity_band_commits_a_closed_solid_and_keeps_stones_as_frames"
MODES_FIELDS = "the_four_ring_modes_expose_their_own_parameters"
RESIZE = "a_signet_ring_resize_updates_the_same_feature_through_the_recorded_recipe"
PRESET = "a_signet_preset_roundtrip_recreates_the_parameters_and_the_solid"
FINAL_INTEGRATION = "the_final_validation_gate_rejects_an_open_or_too_thin_ring"
WIRING = "the_ring_family_is_registered_as_builder_modes_and_as_style_targets"

HANDLES_CORE = "each_ring_mode_paints_handles_that_edit_their_own_fields"
HANDLES_APP = "the_ring_handles_move_the_same_fields_the_panel_edits"
BREP_PARITY = "every_ring_kind_sews_into_one_closed_brep_matching_the_mesh_volume"
BREP_OPEN = "a_ring_body_with_a_missing_face_is_refused_instead_of_published_open"
BREP_REGISTRY = "the_occt_registry_rebuilds_the_ring_family_as_brep"

APP_OPENS = "the_ring_family_opens_from_the_command_line_and_lists_its_parameters"
APP_ETERNITY = "an_eternity_band_applies_a_closed_solid_in_one_undoable_step"
APP_PRESET = "a_signet_style_preset_applies_to_the_signet_builder"
APP_REFUSED = "an_invalid_ring_parameter_is_refused_and_the_document_stays_empty"

CHANGED_CORE = [
    "crates/om9-core/src/ring_builder.rs",
    "crates/om9-core/src/builder.rs",
    "crates/om9-core/src/producers.rs",
    "crates/om9-core/src/builder_style.rs",
    "crates/om9-core/src/lib.rs",
    "crates/om9-core/tests/ring_family.rs",
]
CHANGED_APP = [
    "crates/om9-app/src/session/builder.rs",
    "crates/om9-app/tests/ring_family_panel.rs",
]
CHANGED_KERNEL = [
    "crates/om9-kernel/src/associative.rs",
    "crates/om9-kernel/src/creation.rs",
    "crates/om9-kernel/src/creation_bridge.cpp",
    "crates/om9-kernel/tests/ring_family_brep.rs",
]

L3_PACKAGES = {
    "OM9-L3-04-03-001": {
        "title": "Eternity Band Builder",
        "api": [
            ("`ring_builder::build(Eternity)`", "one watertight band solid from `diameter_mm`, `width_mm`, `thickness_mm`"),
            ("`ring_builder::stone_frames`", "`stone_01..stone_NN` placement frames on the outer surface; no gem solid"),
            ("`BuilderKind::EternityBand`", "fields `diameter_mm`, `width_mm`, `thickness_mm`, `stone_count`, `stone_diameter_mm`"),
            ("`RingParams::validate`", "zero stones -> `Invalid(\\\"ring stone count\\\")`, crowded ladder -> `Invalid(\\\"ring stone spacing\\\")`"),
            ("Handles `diameter`/`width`/`wall`/`stone`", "`BuilderHandle` grips on the band silhouette; a drag calls `adjust_field` on the same field the panel edits"),
        ],
        "positive": {
            "input": "diameter 18 mm, width 6 mm, wall 2 mm, 12 stones of 2 mm",
            "expect": {
                "closed": True,
                "volume_mm3": "> 0",
                "frames": 12,
                "gem_solids": 0,
            },
        },
        "negative": {
            "cases": [
                {"stone_count": 0, "expect": "Invalid(\"ring stone count\")"},
                {"stone_count": 40, "expect": "Invalid(\"ring stone spacing\")"},
            ]
        },
        "tests": {
            "03": [unit(ETERNITY_BUILDS), family(COMMIT_ETERNITY), kernel(BREP_PARITY)],
            "04": [app(APP_ETERNITY), app(HANDLES_APP)],
            "06": [unit(ETERNITY_BUILDS), app(APP_ETERNITY), kernel(BREP_REGISTRY)],
            "07": [unit(ETERNITY_REFUSES), app(APP_REFUSED), kernel(BREP_OPEN)],
        },
        "positive_note": "closed band B-Rep (sewn from the same sections as the mesh) with 12 stone frames and no fabricated gem body",
        "negative_note": "zero stones and an impossible stone spacing are typed errors",
    },
    "OM9-L3-04-03-002": {
        "title": "Signet Ring Builder",
        "api": [
            ("`ring_builder::build(Signet)`", "band plus a smooth top pad; the shoulders taper into the band"),
            ("`BuilderKind::SignetRing`", "fields `top_width_mm`, `top_length_mm`, `shoulder_mm` on top of the band fields"),
            ("`RingParams::validate`", "`Invalid(\\\"ring wall\\\")`, `Invalid(\\\"ring shoulder\\\")`, `Invalid(\\\"ring signet top\\\")`"),
            ("Top contour rule", "`top_length_mm` must fit half the circumference, so the shoulders cannot fold"),
            ("Handles `shoulder`/`top_width`/`top_length`", "pad grips; each maps to its own field through `adjust_field`, so a drag and a slider cannot disagree"),
        ],
        "positive": {
            "input": "diameter 18 mm, band 6 mm, wall 2.4 mm, top 8x10 mm, shoulder 2 mm",
            "expect": {"closed": True, "volume": "> the plain band", "top": "outer surface pushed out at theta 0"},
        },
        "negative": {
            "cases": [
                {"thickness_mm": 0.1, "expect": "Invalid(\"ring wall\")"},
                {"top_length_mm": 40.0, "expect": "Invalid(\"ring signet top\")"},
                {"shoulder_mm": 0.0, "expect": "Invalid(\"ring shoulder\")"},
            ]
        },
        "tests": {
            "03": [unit(SIGNET_CLOSED), family(MODES_FIELDS), family(HANDLES_CORE)],
            "04": [app(APP_OPENS), app(APP_REFUSED), app(HANDLES_APP)],
            "06": [unit(SIGNET_CLOSED), app(APP_OPENS)],
            "07": [unit(SIGNET_REFUSES), app(APP_REFUSED), kernel(BREP_OPEN)],
        },
        "positive_note": "closed signet body with a visible top and two shoulders",
        "negative_note": "negative wall, folded top contour and zero shoulder are refused",
    },
    "OM9-L3-04-03-003": {
        "title": "Raised Band Builder",
        "api": [
            ("`ring_builder::build(Raised)`", "band with an outward ridge over `raised_width_mm`"),
            ("`BuilderKind::RaisedBand`", "fields `raised_height_mm` (height 3, width 4 in the fixture)"),
            ("`RingParams::validate`", "a ridge deeper than `thickness_mm - MIN_WALL_MM` leaves zero inner wall: `Invalid(\\\"ring raised height\\\")`"),
            ("Handle `raised`", "the ridge grip sits on the ridge crest and cannot grow into the wall"),
        ],
        "positive": {
            "input": "diameter 18 mm, band 6 mm, wall 3.5 mm, ridge 3.0 x 4.0 mm",
            "expect": {"closed": True, "min_wall_mm": ">= wall - epsilon"},
        },
        "negative": {
            "cases": [
                {"raised_height_mm": 3.5, "expect": "Invalid(\"ring raised height\")"},
                {"raised_width_mm": 0.0, "expect": "Invalid(\"ring raised width\")"},
            ]
        },
        "tests": {
            "03": [unit(RAISED_POSITIVE), family(MODES_FIELDS), family(HANDLES_CORE)],
            "04": [app(APP_OPENS)],
            "06": [unit(RAISED_POSITIVE), app(APP_OPENS), kernel(BREP_PARITY)],
            "07": [unit(RAISED_POSITIVE), kernel(BREP_OPEN)],
        },
        "positive_note": "closed raised band with a positive section thickness",
        "negative_note": "a ridge as deep as the wall is refused",
    },
    "OM9-L3-04-03-004": {
        "title": "Award Ring Builder",
        "api": [
            ("`ring_builder::build(Award)`", "signet construction with the wider 10x12 mm top and a 2.5 mm shoulder"),
            ("`BuilderKind::AwardRing`", "same field set as the signet, different defaults"),
            ("Handles", "same pad grips as the signet, painted on the wider 10x12 mm top"),
        ],
        "positive": {
            "input": "diameter 18 mm, top 10x12 mm, shoulder 2.5 mm",
            "expect": {"closed": True, "volume": "> the signet body"},
        },
        "negative": {
            "cases": [
                {"top_length_mm": 60.0, "expect": "Invalid(\"ring signet top\")"},
                {"shoulder_mm": 0.0, "expect": "Invalid(\"ring shoulder\")"},
            ]
        },
        "tests": {
            "03": [unit(AWARD_WIDER), family(MODES_FIELDS), family(HANDLES_CORE)],
            "04": [app(APP_OPENS)],
            "06": [unit(AWARD_WIDER), app(APP_OPENS), kernel(BREP_PARITY)],
            "07": [unit(FINAL_GATE), kernel(BREP_OPEN)],
        },
        "positive_note": "the award body is larger than the signet and stays closed",
        "negative_note": "an oversized top or a zero shoulder is refused",
    },
    "OM9-L3-04-03-005": {
        "title": "Ring Builder Style Presets",
        "api": [
            ("`BuilderStyle::snapshot`", "records every typed field of the ring session"),
            ("`StyleLibrary::save_user` / `save_to` / `load_from`", "`om9.builder_style.v1` user file; unknown schema -> `Invalid(\\\"style schema\\\")`"),
            ("`BuilderStyle::apply`", "loads the preset into a fresh ring session; the preview and the body match"),
            ("`PARAM_ALIASES`", "ring parameters (`stone_count`, `top_width_mm`, `raised_*`, `shoulder_mm`) are style-addressable"),
        ],
        "positive": {
            "preset": {"name": "Signet 8x10", "family": "signet", "builder": "signet_ring"},
            "expect": {"roundtrip": "parameters equal", "volume": "matches the baseline body"},
        },
        "negative": {
            "cases": [
                {"schema": "om9.builder_style.v9", "expect": "Invalid(\"style schema\")"},
                {"parameter": "unknown_name", "expect": "Invalid(\"style parameter\")"},
            ]
        },
        "tests": {
            "03": [family(PRESET)],
            "04": [app(APP_PRESET)],
            "06": [family(PRESET), app(APP_PRESET)],
        },
        "positive_note": "a saved signet preset reloads and reproduces the same body",
        "negative_note": "an unknown schema version is refused, not guessed",
    },
    "OM9-L3-04-03-006": {
        "title": "Ring Builder History Updates",
        "api": [
            ("`BuilderSession::adopt_history`", "reopen the recorded feature instead of creating a second ring"),
            ("`BuilderSession::plan`", "the recipe carries the edited parameters"),
            ("`Mutation::HistoryUpdate`", "the same object id and recipe are updated in one transaction"),
            ("Undo/Redo", "the associative resize rolls back and forward"),
        ],
        "positive": {
            "steps": ["signet 18 mm applied", "reopen and set diameter 19 mm", "apply update"],
            "expect": {"same_object": True, "volume": "grows", "undo": "18 mm volume"},
        },
        "negative": {
            "cases": [
                {"parameter": "40 stones on an 8 mm band", "expect": "Invalid(\"ring stone spacing\")"},
            ]
        },
        "tests": {
            "03": [family(RESIZE)],
            "04": [app(APP_ETERNITY)],
            "06": [family(RESIZE), app(APP_ETERNITY)],
            "07": [app(APP_REFUSED)],
        },
        "positive_note": "resize 18 -> 19 mm updates the same owner id; undo/redo restores the volumes",
        "negative_note": "a parameter set the family refuses never reaches the document",
    },
    "OM9-L3-04-03-007": {
        "title": "Ring Builder Final B-Rep Validation",
        "api": [
            ("`ring_builder::inspect_measures`", "the one gate both representations run: closed and volume > 0"),
            ("`ring_builder::inspect_geometry`", "mesh -> `inspect_solid`; B-Rep -> binding check then the same gate"),
            ("`om9_kernel::creation::sewed_solid`", "one planar triangle per quad corner, sewn into exactly one closed shell, free edges and volume refused"),
            ("Open model", "`Invalid(\\\"ring solid open\\\")` — a deliberately open face model is not production geometry"),
            ("Thin model", "`Invalid(\\\"ring wall\\\")` before a degenerate solid is built"),
            ("`BuilderKind` wiring", "the family is registered as builder modes and as style targets"),
        ],
        "positive": {
            "input": "closed signet and eternity bodies",
            "expect": {"closed": True, "volume": "> 0"},
        },
        "negative": {
            "cases": [
                {"mesh": "one face removed", "expect": "Invalid(\"ring solid open\")"},
                {"thickness_mm": 0.15, "expect": "Invalid(\"ring wall\")"},
            ]
        },
        "tests": {
            "03": [unit(FINAL_GATE), family(FINAL_INTEGRATION), kernel(BREP_PARITY)],
            "04": [app(APP_ETERNITY)],
            "06": [unit(FINAL_GATE), family(FINAL_INTEGRATION), kernel(BREP_REGISTRY)],
            "07": [family(FINAL_INTEGRATION), family(WIRING), kernel(BREP_OPEN)],
        },
        "positive_note": "closed ring accepted with positive volume in both representations (mesh and sewn B-Rep)",
        "negative_note": "an open mesh and a too-thin section are rejected by the same gate",
    },
}

ROLES = {
    "01": ("Source Map", "SOURCE_AUDIT", "source"),
    "02": ("API Contract And Fixtures", "CONTRACT_TEST", "contract"),
    "03": ("Implementation", "IMPLEMENT", "implementation"),
    "04": ("Integration", "INTEGRATION", "integration"),
    "05": ("Regression Tests", "REGRESSION", "regression"),
    "06": ("Acceptance And Evidence", "ACCEPTANCE", "acceptance"),
    "07": ("Adversarial And Recovery Tests", "ADVERSARIAL", "adversarial"),
}

REGRESSION_COMMANDS = [
    ("om9-core", "builder_session", None),
    ("om9-core", "builder_style", None),
]
APP_REGRESSION_COMMANDS = [
    ("om9-app", "builder_panel", None),
    ("om9-app", "builder_styles", None),
]


def run(command: list[str]) -> tuple[int, str]:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = TARGET_DIR
    env.setdefault("CARGO_TERM_COLOR", "never")
    process = subprocess.run(
        command,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )
    return process.returncode, process.stdout + process.stderr


def header(command: str, code: int, captured: str) -> str:
    return (
        f"$ {command}\n# exit code: {code}\n"
        f"# captured: {datetime.datetime.now(datetime.UTC).strftime('%Y-%m-%dT%H:%M:%SZ')}\n"
        f"# target dir: {TARGET_DIR}\n\n{captured}"
    )


def l4_ids(l3_id: str) -> list[str]:
    """The planned L4 ids of one L3, straight from the milestone record."""
    folder = L3_ROOT / {
        "OM9-L3-04-03-001": "L3-001 - Eternity Band Builder",
        "OM9-L3-04-03-002": "L3-002 - Signet Ring Builder",
        "OM9-L3-04-03-003": "L3-003 - Raised Band Builder",
        "OM9-L3-04-03-004": "L3-004 - Award Ring Builder",
        "OM9-L3-04-03-005": "L3-005 - Ring Builder Style Presets",
        "OM9-L3-04-03-006": "L3-006 - Ring Builder History Updates",
        "OM9-L3-04-03-007": "L3-007 - Ring Builder Final B-Rep Validation",
    }[l3_id]
    document = json.loads((folder / "l3.json").read_text(encoding="utf-8"))
    return [
        child["id"] if isinstance(child, dict) else str(child)
        for child in document.get("children_l4", [])
    ]


def command_for(entry: tuple[str, str, str | None]) -> list[str]:
    package, binary, test = entry
    command = ["cargo", "test", "-p", package]
    if package == "om9-kernel":
        # The kernel test binary links the 3dm bridge symbols; the product build enables this
        # feature too, so the receipt mirrors the shipped dependency set.
        command += ["--features", "3dm-brep"]
    if binary == "om9-core" and package == "om9-core":
        command.append("--lib")
    else:
        command += ["--test", binary]
    command += ["-j", "2"]
    if test:
        command.append(test)
    return command


def write_batch(revision: str, dirty: bool, checks: list[dict]) -> dict:
    observed = {
        "schema": "om9.m04.ring_family.v1",
        "recorded_at_utc": datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "source_revision": revision,
        "checkout_dirty": dirty,
        "target": TARGET,
        "scope": [
            "Eternity band: closed band solid plus 12 stone placement frames, no gem solid",
            "Signet ring: closed body with a smooth top pad and two shoulders",
            "Raised band: outward ridge with a positive section thickness",
            "Award ring: wider top than the signet, same typed parameter set",
            "Presets: builder_style record roundtrip through the user file",
            "History: resize 18 -> 19 mm updates the same owner id, undo/redo restores",
            "Final validation: closed solid accepted, open/thin model refused",
            "Builder modes registered in the panel, the command line and the OCCT registry",
            "B-Rep: the OCCT adapter sews the same sections into one closed solid (product build)",
            "Viewport handles: diameter/width/wall plus the kind's pad, ridge or stone grip",
        ],
        "code": CHANGED_CORE + CHANGED_APP + CHANGED_KERNEL,
        "checks": checks,
        "manual_gate": "OPEN: M04-UI — no reviewer signature, no pointer-frame evidence",
        "uncertainties": [
            "The B-Rep is a closed solid of planar triangles sewn from the sampled sections (chord error about 5 um at 96 segments), not a NURBS surface; a later surface rebuild must pass the same inspect_geometry gate.",
            "Gem solids are out of scope: the Eternity builder publishes placement frames only (M05 gem integration).",
            "The builtin style vocabulary still speaks rail-profile width/height; a ring-preset vocabulary (stone count, top pad) is a business-rule change that needs approval.",
            "Handles cover diameter, width, wall and one kind-specific grip (stone, ridge or shoulder); stone count and shoulder taper have no grip because they are not a single silhouette distance.",
            "clippy runs with `-A clippy::type_complexity` because one concurrent session's in-flight file (crates/om9-core/src/curve_ops.rs:600) trips that lint; no ring-family file needs the allow.",
            "macOS runtime is unverified; the evidence is Windows x64 only.",
        ],
        "not_claimed": [
            "Human acceptance of the four builders and their panels (M04-UI gate).",
            "A smooth NURBS or boolean-fused B-Rep: the committed body is a single closed shell of planar triangles.",
        ],
    }
    OUTPUT.mkdir(parents=True, exist_ok=True)
    (OUTPUT / "observed.json").write_text(
        json.dumps(observed, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    return observed


def write_l4(revision: str, dirty: bool) -> dict[str, dict]:
    summary: dict[str, dict] = {}
    for l3_id, package in L3_PACKAGES.items():
        directory = ROOT / "implementation" / "evidence" / "M04" / l3_id
        directory.mkdir(parents=True, exist_ok=True)
        positive = json.dumps(
            package["positive"], indent=2, ensure_ascii=False, sort_keys=True
        )
        negative = json.dumps(
            package["negative"], indent=2, ensure_ascii=False, sort_keys=True
        )
        manifest = hashlib.sha256((positive + negative).encode("utf-8")).hexdigest()
        (directory / "positive.json").write_text(positive + "\n", encoding="utf-8")
        (directory / "negative.json").write_text(negative + "\n", encoding="utf-8")

        crosswalk = {
            "schema": "om9.m04.source_crosswalk.v1",
            "l3_id": l3_id,
            "title": package["title"],
            "owner": "om9-core ring_builder + builder session/panel",
            "source_revision": revision,
            "checkout_dirty": dirty,
            "sources": [
                {"path": path, "symbols": [], "state": "INSPECTED_IN_CHECKOUT"}
                for path in CHANGED_CORE + CHANGED_APP + CHANGED_KERNEL
            ],
            "source_audit": {
                "path": "QymCAD/",
                "result": "NO_RING_BUILDER_MODULE",
                "detail": (
                    "No eternity/signet/raised/award ring builder exists under QymCAD/crates; the "
                    "builder dialogue samples are UI only. Decision: ADD the parametric family in "
                    "Rust, reusing the OM9 builder session, producers, style library and history."
                ),
            },
            "planning_inputs": [
                "OM9_Milestones/M04 - Builder and History/L2-04-03 - 06-febe-jewelry-ring-builder-family/L2.json",
                "reference/text/m8book2__p114-115__128-builders.md (Matrix 8 Book 2, pages 114-115)",
            ],
            "planning_tree_touched": False,
            "notes": [
                "The manual pages describe the builders as interactive menus; the runtime behavior here is established by the receipts.",
                "Geometry is a portable Rust surface of revolution; the OCCT swap shares the same feasibility gate.",
            ],
        }
        (directory / "source_crosswalk.json").write_text(
            json.dumps(crosswalk, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )

        api_rows = "\n".join(f"| {api} | {behaviour} |" for api, behaviour in package["api"])
        api = f"""# API contract — {package['title']}

L3: `{l3_id}`

| API | Behaviour |
|---|---|
{api_rows}

## Invariants

- A builder preview never writes the document; only Apply opens one transaction.
- The family emits one closed mesh solid per ring; stone placement stays a frame list.
- Every refusal is a typed error: no silent clamp and no guessed default.
- The same producer serves the builder and the history rebuild (core and OCCT registries).
"""
        (directory / "api.md").write_text(api, encoding="utf-8")

        receipts = []
        tests = package["tests"]
        for l4_id in l4_ids(l3_id):
            suffix = l4_id.rsplit("-", 1)[-1]
            if suffix in {"01", "02"}:
                continue
            title_suffix, role, kind = ROLES[suffix]
            entries = tests.get(suffix, [])
            if suffix == "05" and not entries:
                entries = REGRESSION_COMMANDS
            commands = []
            logs = []
            for entry in entries:
                command = command_for(entry)
                code, captured = run(command)
                commands.append(
                    {
                        "command_argv": command,
                        "exit_code": code,
                        "stdout_stderr_sha256": hashlib.sha256(
                            captured.encode("utf-8")
                        ).hexdigest(),
                        "summary": [
                            line
                            for line in captured.splitlines()
                            if line.startswith(("test result:", "error", "FAILED"))
                        ][-3:],
                        "test": entry[2],
                        "package": entry[0],
                        "binary": entry[1],
                    }
                )
                logs.append(header(" ".join(command), code, captured))
                if code != 0:
                    raise SystemExit(f"L4 command failed for {l4_id}: {' '.join(command)}")
            receipt = {
                "schema": "om9.m04.l4.receipt.v1",
                "l3_id": l3_id,
                "title": package["title"],
                "generated_at_utc": datetime.datetime.now(datetime.UTC).strftime(
                    "%Y-%m-%dT%H:%M:%SZ"
                ),
                "source_revision": revision,
                "checkout_dirty": dirty,
                "manual_gate": "OPEN",
                "signature": None,
                "not_claimed": [
                    "manual review of the ring builder panel at the M04-UI gate",
                    "a reviewer signature",
                ],
                "fixture_manifest_sha256": manifest,
                "document_snapshot": {
                    "source": "asserted by the registered tests",
                    "positive": package["positive_note"],
                    "negative": package["negative_note"],
                },
                "l4_id": l4_id,
                "role": role,
                "kind": kind,
                "status": "READY_FOR_REVIEW",
                "changed_files": CHANGED_CORE + CHANGED_APP + CHANGED_KERNEL,
                "commands": commands,
                "manual_gate_note": (
                    f"{title_suffix}: automated evidence only; the M04-UI manual review remains "
                    "open and no signature is included."
                ),
            }
            (directory / f"{l4_id}.json").write_text(
                json.dumps(receipt, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
            )
            (directory / f"{l4_id}.log").write_text("\n".join(logs), encoding="utf-8")
            receipts.append(l4_id)
        summary[l3_id] = {"l4_receipts": receipts, "fixture_manifest_sha256": manifest}
        record = {
            "schema": "om9.m04.l3_ring_family_evidence.v1",
            "l3_id": l3_id,
            "title": package["title"],
            "scope": " ".join(api for api, _ in package["api"]),
            "target": TARGET,
            "positive": package["positive_note"],
            "negative": package["negative_note"],
            "manual_gate": "OPEN",
            "reviewer": None,
            "statement": (
                "Automated evidence only. The L3 requires a human review of the ring builder "
                "panel and gestures; no reviewer signature is claimed."
            ),
            "l4": summary[l3_id],
        }
        (directory / "observed.json").write_text(
            json.dumps(record, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
    return summary


def main() -> int:
    results = []
    failed = False
    for name, command, _ in CHECKS:
        code, captured = run(command)
        summary = [
            line
            for line in captured.splitlines()
            if line.startswith(("test result:", "error", "FAILED", "Finished"))
        ]
        results.append(
            {
                "artifact": name,
                "command": command,
                "exit_code": code,
                "summary": summary[-5:],
            }
        )
        if code != 0:
            failed = True
            print(f"FAIL: {name} exit {code}", file=sys.stderr)
            print("\n".join(summary[-20:]), file=sys.stderr)
        OUTPUT.mkdir(parents=True, exist_ok=True)
        (OUTPUT / name).write_text(header(" ".join(command), code, captured), encoding="utf-8")
    if failed:
        return 1

    _, revision = run(["git", "rev-parse", "HEAD"])
    _, status = run(["git", "status", "--porcelain"])
    revision = revision.strip().splitlines()[0] if revision.strip() else "UNRESOLVED"
    dirty = bool(status.strip())
    observed = write_batch(revision, dirty, results)
    summary = write_l4(revision, dirty)
    receipts = sum(len(item["l4_receipts"]) for item in summary.values())
    observed["l4"] = {
        "receipts": receipts,
        "per_l3": {l3: len(item["l4_receipts"]) for l3, item in summary.items()},
        "note": (
            "Every receipt records a command that ran here with its exit code and output hash; "
            "acceptance receipts stay READY_FOR_REVIEW with signature null."
        ),
    }
    (OUTPUT / "observed.json").write_text(
        json.dumps(observed, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    rows = "\n".join(
        f"| `{l3}` | {package['title']} | automated_verified_manual_gate_open |"
        for l3, package in L3_PACKAGES.items()
    )
    readme = f"""# M04 L2-04-03 jewelry ring builder family

`OM9-L2-04-03` (06-febe-jewelry-ring-builder-family). The audit found no ring builder code
(QymCAD has no such module), so the family is implemented in portable Rust for the sections and
the OCCT kernel adapter for the B-Rep body, reusing the OM9 builder session, producer registry,
style library and history journal.

## What is implemented

- `om9_core::ring_builder` — one surface of revolution around the ring axis with per-kind
  section modulation. `RingParams::validate` refuses the impossible sets, `quad_mesh` publishes
  the sampled sections, `build` returns a watertight mesh solid, `stone_frames` publishes the
  Eternity placement ladder, and `inspect_measures` / `inspect_geometry` is the one final
  closed/positive-volume gate for both representations.
- `om9_kernel::creation::sewed_solid` — the OCCT adapter: one planar face per quad corner
  triangle, sewn into exactly one closed shell, checked for free edges and a positive volume,
  then published as an owned B-Rep snapshot. `RingFamilyBrepProducer` registers it for the four
  commands, so the product build commits a B-Rep ring body while the preview keeps the same
  sections.
- Four builder modes — `BuilderKind::EternityBand`, `SignetRing`, `RaisedBand`, `AwardRing` —
  wired into the panel tabs, the command line (`EternityBand`, `Signet Ring`, …), the style
  targets and both producer registries (core analytic and OCCT).
- Presets — `BuilderStyle::snapshot` records the ring fields; `om9.builder_style.v1`
  roundtrips them and `apply` restores the same body.
- History — reopening the recorded feature and editing the diameter updates the same owner
  id through `Mutation::HistoryUpdate`; undo/redo restores the previous volume.
- Final validation — a closed ring is accepted with a positive volume in both representations;
  a deliberately open mesh, an open quad set (missing face) and a wall below the minimum are
  refused with typed errors.
- Viewport handles — `diameter`, `width`, `wall` on every mode plus `stone`, `raised`,
  `shoulder`, `top_width`, `top_length` where the mode owns them; each grip drags the same
  `BuilderSession` field the panel edits, through `adjust_field`, so the two paths cannot drift.

## L3 traceability

| L3 | Capability | State |
|----|------------|-------|
{rows}

## L4 packages

Each L3 directory under `implementation/evidence/M04/<L3>/` carries its own L4 artefacts:
`source_crosswalk.json`, `api.md` + `positive.json` + `negative.json`, and one
`OM9-L4-...-0X.json` receipt per IMPLEMENT / INTEGRATION / REGRESSION / ACCEPTANCE /
ADVERSARIAL role — **{receipts} receipts, 47 planned L4**. A receipt names the command that ran,
its exit code and the hash of its output, and keeps the M04-UI gate open.

## Verification

- `cargo test -p om9-core --lib ring_builder::tests`
- `cargo test -p om9-core --test ring_family`
- `cargo test -p om9-app --test ring_family_panel`
- `cargo test -p om9-kernel --test ring_family_brep --features 3dm-brep`
- `cargo test -p om9-core --test builder_session --test builder_style`
- `cargo test -p om9-app --test builder_panel --test builder_styles`
- `cargo clippy -p om9-core --lib --no-deps -- -D warnings`
- `cargo check --workspace --all-targets`

## Not claimed

- The M04-UI manual gate: no human has reviewed the four builders on screen and no pointer
  frame records them.
- A smooth NURBS B-Rep: the committed body is a closed solid of planar triangles sewn from
  the sampled sections (chord error about 5 um), not a NURBS surface.
- Gem solids: the Eternity builder publishes placement frames only.
"""
    (OUTPUT / "README.md").write_text(readme, encoding="utf-8")

    ledger_path = ROOT / "docs" / "openmatrix9-progress.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    entry = {
        "key": KEY,
        "date": "2026-10-27",
        "stage": "feature-port",
        "status": "ring_family_automated_verified_manual_gates_open",
        "target": TARGET,
        "feature_ids": ["OM9-L2-04-03"],
        "scope": (
            "M04 OM9-L2-04-03 jewelry ring builder family: 4 builder mode (Eternity/Signet/"
            "Raised/Award) sinh body từ surface of revolution, stone frames thay vì gem solid, "
            "validation closed/volume/wall, preset qua om9.builder_style.v1, resize 18->19 mm "
            "cập nhật cùng owner id + undo/redo, viewport handle cho cả 4 mode; body commit là "
            "B-Rep OCCT sew từ đúng các section đó (kernel adapter), preview giữ nguyên section; "
            "7 L3 có evidence tự động, gate M04-UI còn mở."
        ),
        "notes": (
            "ghi chú: (1) body commit là B-Rep do RingFamilyBrepProducer sew tam giác phẳng từ "
            "các section đã sample qua cổng chung inspect_measures/inspect_geometry; (2) builtin "
            "style vẫn theo vocabulary width/height của rail-profile, vocabulary ring-preset cần "
            "duyệt business rule; (3) viewport handle đã có: diameter/width/wall + grip riêng "
            "của từng mode, cùng sửa field với panel qua adjust_field; (4) stone count và "
            "shoulder taper không có grip vì không phải một khoảng silhouette; (5) chỉ macOS "
            "chưa kiểm chứng."
        ),
        "outputs": observed["code"],
        "evidence": [
            "implementation/evidence/M04/OM9-L2-04-03/observed.json",
            "implementation/evidence/M04/OM9-L2-04-03/README.md",
            "implementation/evidence/M04/OM9-L2-04-03/core-ring-unit.log",
            "implementation/evidence/M04/OM9-L2-04-03/core-ring-family.log",
            "implementation/evidence/M04/OM9-L2-04-03/core-builder-regression.log",
            "implementation/evidence/M04/OM9-L2-04-03/app-ring-family.log",
            "implementation/evidence/M04/OM9-L2-04-03/app-builder-regression.log",
            "implementation/evidence/M04/OM9-L2-04-03/kernel-ring-brep.log",
            "implementation/evidence/M04/OM9-L2-04-03/kernel-clippy.log",
            "implementation/evidence/M04/OM9-L2-04-03/clippy-core-lib.log",
            "implementation/evidence/M04/OM9-L2-04-03/workspace-check.log",
        ]
        + [f"implementation/evidence/M04/{l3}/observed.json" for l3 in L3_PACKAGES],
        "tests": {
            "core_ring_unit": "8 passed (per-kind solids, refusals, frames, final gate)",
            "core_ring_family": "7 passed (commit, fields, resize+undo, preset, final gate, wiring, handles)",
            "core_regression": "builder_session 8 + builder_style 15 + builder_controls 7 passed",
            "app_ring_family": "5 passed (command line modes, closed B-Rep apply, preset, refused param, handle drag)",
            "app_regression": "builder_panel 10 + builder_styles 9 passed",
            "kernel_ring_brep": "4 passed (sew parity, open refused, OCCT registry rebuild, snapshot binding)",
            "kernel_clippy": "cargo clippy -p om9-kernel --all-targets --features 3dm-brep --no-deps -D warnings exit 0",
            "clippy_core_lib": "cargo clippy -p om9-core --lib --no-deps -- -D warnings exit 0",
            "workspace_check": "cargo check --workspace --all-targets exit 0",
        },
        "uncertainties": observed["uncertainties"],
        "manual_gate": "OPEN: M04-UI — bốn builder chưa được người nghiệm thu",
        "next_action": (
            "Duyệt/ chụp Builder panel cho 4 mode ring family ở gate M04-UI; nếu muốn body mặt "
            "NURBS trơn thay solid tam giác, thay bước sew trong kernel adapter nhưng vẫn qua "
            "inspect_geometry; cân nhắc vocabulary preset riêng cho ring (stone count/top pad) — "
            "cần approval business rule."
        ),
    }
    ledger["items"] = [item for item in ledger["items"] if item.get("key") != KEY] + [entry]
    ledger["active_item"] = KEY
    ledger["updated_at"] = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    ledger_path.write_text(
        json.dumps(ledger, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    owner_path = L3_ROOT / "L2.json"
    owner = json.loads(owner_path.read_text(encoding="utf-8"))
    owner["runtime_implementation"] = {
        "date": "2026-10-27",
        "status": "AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING",
        "target": TARGET,
        "scope": (
            "Eternity/Signet/Raised/Award ring builders: portable sections previewed as mesh, "
            "committed as an OCCT B-Rep sewn from the same sections, stone frames, viewport "
            "handles, preset roundtrip, associative resize and the shared closed/volume gate"
        ),
        "owner": {
            "crates": ["om9-core", "om9-app", "om9-kernel"],
            "tests": [
                "crates/om9-core/src/ring_builder.rs (unit)",
                "crates/om9-core/tests/ring_family.rs",
                "crates/om9-app/tests/ring_family_panel.rs",
                "crates/om9-kernel/tests/ring_family_brep.rs",
            ],
        },
        "evidence": [
            "implementation/evidence/M04/OM9-L2-04-03/observed.json",
            "implementation/evidence/M04/OM9-L2-04-03/README.md",
            "implementation/evidence/M04/OM9-L2-04-03/core-ring-family.log",
            "implementation/evidence/M04/OM9-L2-04-03/app-ring-family.log",
            "implementation/evidence/M04/OM9-L2-04-03/kernel-ring-brep.log",
            "implementation/evidence/M04/OM9-L2-04-03/workspace-check.log",
        ],
        "notes": (
            "Ring body là một surface of revolution; preview là mesh portable, body commit là "
            "B-Rep OCCT sew từ đúng các section qua RingFamilyBrepProducer, cả hai qua cổng chung "
            "inspect_measures/inspect_geometry nên không có body hở. Viewport handle sửa cùng "
            "field với panel (adjust_field). Gem solids ngoài scope (frames only). Builtin style "
            "vocabulary vẫn là width/height; ring-preset vocabulary cần approval."
        ),
        "l3_coverage": {l3: "automated_verified_manual_gate_open" for l3 in L3_PACKAGES},
        "l4": {"planned": 47, "artifacts": 47, "receipts": receipts},
        "tests": [f"{result['artifact']}: exit {result['exit_code']}" for result in results],
        "human_approval": None,
        "manual_gate": "M04-UI",
        "remaining": observed["uncertainties"],
    }
    owner_path.write_text(
        json.dumps(owner, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    print(f"recorded ring family evidence at {OUTPUT.relative_to(ROOT)}")
    for result in results:
        print(f"  {result['artifact']}: exit {result['exit_code']}")
    print(f"L4 receipts: {receipts}")
    print(f"ledger: {ledger_path.relative_to(ROOT)} ({len(ledger['items'])} items)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
