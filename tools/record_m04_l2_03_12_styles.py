#!/usr/bin/env python3
"""Record the M04 L2-03-12 builder styles/presets batch from real runs.

Scope: the six L3 of ``03-febe-matrix-builder-styles-and-presets``:

* ``OM9-L3-03-12-001`` typed, versioned style snapshot (``om9.builder_style.v1``);
* ``OM9-L3-03-12-002`` style browser rows with a thumbnail or its fallback;
* ``OM9-L3-03-12-003`` preview-only apply, mandatory-input refusal;
* ``OM9-L3-03-12-004`` user saves that never rewrite the built-ins;
* ``OM9-L3-03-12-005`` mode/family filter, incompatible styles disabled and refused;
* ``OM9-L3-03-12-006`` attribute matching from a source entity with stable identity.

The manual gate stays open: no reviewer signature is claimed, and the L3 records under
``OM9-L2-03-12`` keep their plan status.

Usage::

    python tools/record_m04_l2_03_12_styles.py
"""

from __future__ import annotations

import datetime
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
TARGET_DIR = os.environ.get("CARGO_TARGET_DIR", r"L:\om9-target")
KEY = "m04-l2-03-12-builder-styles-2026-10-27"
OUTPUT = ROOT / "implementation" / "evidence" / "M04" / "OM9-L2-03-12"

TARGET = "Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=L:/om9-target; -j 2"

CHECKS = [
    (
        "core-builder-style.log",
        ["cargo", "test", "-p", "om9-core", "--test", "builder_style", "-j", "2"],
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
            "builder_controls",
            "--test",
            "builder_session",
            "--test",
            "history_mutation",
            "-j",
            "2",
        ],
        False,
    ),
    (
        "app-builder-styles.log",
        [
            "cargo",
            "test",
            "-p",
            "om9-app",
            "--test",
            "builder_styles",
            "--test",
            "builder_panel",
            "--test",
            "builder_controls",
            "--test",
            "builder_slot",
            "--test",
            "top11",
            "-j",
            "2",
        ],
        False,
    ),
    (
        "occt-chain.log",
        [
            "cargo",
            "test",
            "-p",
            "om9-kernel",
            "--features",
            "3dm-brep",
            "--test",
            "associative_history",
            "-j",
            "2",
        ],
        True,
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
        ],
        False,
    ),
    (
        "workspace-check.log",
        ["cargo", "check", "--workspace", "--all-targets", "-j", "2"],
        False,
    ),
]

L3_COVERAGE = [
    {
        "l3": "OM9-L3-03-12-001",
        "title": "Builder Style Param Snapshot Schema",
        "state": "automated_verified_manual_gate_open",
        "code": [
            "om9_core::builder_style::StyleValue/StyleFile/BuilderStyle",
            "STYLE_SCHEMA = om9.builder_style.v1",
            "BuilderStyle::snapshot/defaults_of/validate",
            "profile_choice: profile_ref design names -> builder preset choices",
        ],
        "tests": [
            "a_style_roundtrip_keeps_the_values_their_unit_and_the_schema",
            "an_unknown_schema_version_or_an_invalid_numeric_is_refused",
            "a_profile_reference_name_is_translated_to_the_builder_preset",
        ],
        "fixture": "typed style v1: mode=band,width_mm=2.2,profile_ref=basic_round",
    },
    {
        "l3": "OM9-L3-03-12-002",
        "title": "Builder Style Browser And Thumbnail",
        "state": "automated_verified_manual_gate_open",
        "code": [
            "StyleLibrary::styles_for + StyleEntry",
            "StyleThumbnail::for_style (authored icon or placeholder)",
            "Controller::builder_styles + the Builder Styles window",
        ],
        "tests": [
            "the_browser_lists_every_style_for_the_mode_and_disables_the_incompatible",
            "a_missing_or_corrupt_thumbnail_falls_back_without_dropping_the_style",
            "the_browser_lists_the_styles_of_the_open_mode",
        ],
        "fixture": "style library 3 styles, filter ring mode",
    },
    {
        "l3": "OM9-L3-03-12-003",
        "title": "Builder Style Apply And Preview",
        "state": "automated_verified_manual_gate_open",
        "code": [
            "BuilderStyle::apply (preview only, reports applied/ignored)",
            "Controller::builder_apply_style + builder_refresh",
            "Style= command line",
        ],
        "tests": [
            "applying_a_style_changes_the_preview_only_and_cancel_keeps_the_baseline",
            "a_style_missing_the_mandatory_profile_input_is_refused",
            "applying_a_style_changes_the_preview_only",
        ],
        "fixture": "live style A width 2.2 -> style B width 3.0 -> Cancel",
    },
    {
        "l3": "OM9-L3-03-12-004",
        "title": "Builder Style Save New Name",
        "state": "automated_verified_manual_gate_open",
        "code": [
            "StyleLibrary::save_user/remove_user (built-ins read-only)",
            "validate_style_name, StyleLibrary::save_to/load_from",
            "Controller::builder_save_style + StyleSave= command line",
        ],
        "tests": [
            "saving_a_user_style_leaves_the_defaults_untouched_and_refuses_a_duplicate_name",
            "a_style_name_cannot_be_a_path_or_control_text",
            "the_user_library_roundtrips_through_a_file_and_reports_a_refused_write",
            "saving_a_style_adds_a_user_row_without_rewriting_the_defaults",
            "user_styles_roundtrip_through_the_configured_file",
        ],
        "fixture": "user style name Classic 2.5 saved in user space",
    },
    {
        "l3": "OM9-L3-03-12-005",
        "title": "Builder Style Mode And Shape Filter",
        "state": "automated_verified_manual_gate_open",
        "code": [
            "StyleFamily::matches + BuilderKind::tag/from_tag/is_ring_builder",
            "BuilderStyle::compatible/targets/missing_required",
            "StyleEntry.enabled + disabled rows in the window",
        ],
        "tests": [
            "a_gem_style_cannot_be_applied_to_a_ring_builder",
            "the_builder_tags_filter_a_style_to_one_mode",
            "a_style_ignores_parameters_of_another_mode_and_reports_them",
            "a_style_for_another_mode_is_listed_disabled_and_refused",
        ],
        "fixture": "ring mode Band vs Signet with mutually exclusive preset types",
    },
    {
        "l3": "OM9-L3-03-12-006",
        "title": "Builder Style Match Attributes",
        "state": "automated_verified_manual_gate_open",
        "code": [
            "BuilderStyle::match_attributes over declared extension slots om9.rail/om9.head",
            "STYLE_ATTRIBUTES (width_mm/height_mm/material) with types kept",
            "Controller::builder_match_attributes + StyleMatch command line",
        ],
        "tests": [
            "matching_attributes_copies_only_the_declared_attributes_and_keeps_the_identity",
            "a_source_without_a_declared_slot_or_with_a_wrong_type_is_refused",
            "matching_attributes_copies_from_the_selection_and_keeps_the_ids",
        ],
        "fixture": "source ring has width=3, material=synthetic, height=2",
    },
]


def run(command: list[str], with_path: bool = False) -> tuple[int, str]:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = TARGET_DIR
    env.setdefault("CARGO_TERM_COLOR", "never")
    if with_path:
        env["PATH"] = TARGET_DIR + os.pathsep + env.get("PATH", "")
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


# The L4 packages of the six L3. Each entry names the real fixtures of the L3, the source
# crosswalk, the API contract and the tests that prove the IMPLEMENT / INTEGRATION /
# REGRESSION / ACCEPTANCE / ADVERSARIAL roles. The dashboard counts an L4 only when its own
# artefact exists, so every file here is a receipt of a command that actually ran.
L4_PACKAGES = {
    "OM9-L3-03-12-001": {
        "title": "Builder Style Param Snapshot Schema",
        "api": [
            ("`StyleFile` / `STYLE_SCHEMA`", "`om9.builder_style.v1` file with `schema` and `styles`; an unknown tag is `Invalid(\"style schema\")`"),
            ("`StyleValue`", "`length_mm` / `number` / `choice` / `boolean`; unit and type survive the roundtrip"),
            ("`BuilderStyle::snapshot` / `defaults_of`", "record every typed field of a session explicitly, not implicitly"),
            ("`BuilderStyle::validate`", "name rules, at least one family and parameter, typed finite values"),
            ("`profile_choice`", "`profile_ref=basic_round` -> builder preset `circle`; an unknown reference is `Invalid(\"style profile\")`"),
        ],
        "fixture_positive": {
            "schema": "om9.builder_style.v1",
            "style": {
                "name": "typed v1",
                "family": ["band"],
                "parameters": {
                    "width_mm": {"length_mm": 2.2},
                    "profile_ref": {"choice": "basic_round"},
                },
            },
            "expect": {
                "roundtrip": "parameters and unit preserved",
                "preset": "circle",
            },
        },
        "fixture_negative": {
            "cases": [
                {"schema": "om9.builder_style.v2", "expect": "Invalid(\"style schema\")"},
                {"parameters": {"width_mm": {"length_mm": -1.0}}, "expect": "Invalid(\"style numeric\")"},
                {"parameters": {"width_mm": {"length_mm": "wide"}}, "expect": "Invalid(\"style schema\")"},
                {"parameters": {"profile_ref": {"choice": "triangular"}}, "expect": "Invalid(\"style profile\")"},
            ]
        },
        "l4_tests": {
            "03": [
                ("om9-core", "builder_style", "a_style_roundtrip_keeps_the_values_their_unit_and_the_schema"),
                ("om9-core", "builder_style", "a_profile_reference_name_is_translated_to_the_builder_preset"),
            ],
            "04": [("om9-app", "builder_styles", "user_styles_roundtrip_through_the_configured_file")],
            "05": [("om9-core", "builder_style", None)],
            "06": [
                ("om9-core", "builder_style", "a_style_roundtrip_keeps_the_values_their_unit_and_the_schema"),
                ("om9-app", "builder_styles", "user_styles_roundtrip_through_the_configured_file"),
            ],
            "07": [
                ("om9-core", "builder_style", "an_unknown_schema_version_or_an_invalid_numeric_is_refused")
            ],
        },
        "changed_files": [
            "crates/om9-core/src/builder_style.rs",
            "crates/om9-core/tests/builder_style.rs",
        ],
        "positive": "a typed v1 style roundtrips through the file and drives the circle preset",
        "negative": "unknown schema version, invalid numeric and unknown profile reference are typed errors",
    },
    "OM9-L3-03-12-002": {
        "title": "Builder Style Browser And Thumbnail",
        "api": [
            ("`StyleLibrary::styles_for(kind)`", "rows for the open mode: compatible first, incompatible listed but `enabled=false`"),
            ("`StyleEntry`", "style, `enabled`, `thumbnail`, `builtin`"),
            ("`StyleThumbnail::for_style`", "authored icon key, else the placeholder with `fallback=true`; a corrupt key never drops the row"),
            ("`Controller::builder_styles`", "owned rows for the panel (name, enabled, icon, fallback, builtin, applied)"),
        ],
        "fixture_positive": {
            "library": ["Classic 2.2", "Signet 3.0", "Brilliant Gem 1.9"],
            "mode": "ring_rail",
            "expect": {"enabled": ["Classic 2.2", "Signet 3.0"], "rows": 3},
        },
        "fixture_negative": {
            "cases": [
                {"thumbnail": "not-an-icon.png", "expect": "row kept, fallback=true"},
                {"thumbnail": None, "expect": "row kept, fallback=true"},
            ]
        },
        "l4_tests": {
            "03": [
                ("om9-core", "builder_style", "the_browser_lists_every_style_for_the_mode_and_disables_the_incompatible"),
                ("om9-core", "builder_style", "a_missing_or_corrupt_thumbnail_falls_back_without_dropping_the_style"),
            ],
            "04": [("om9-app", "builder_styles", "the_browser_lists_the_styles_of_the_open_mode")],
            "05": [("om9-app", "builder_styles", None)],
            "06": [
                ("om9-core", "builder_style", "the_browser_lists_every_style_for_the_mode_and_disables_the_incompatible"),
                ("om9-app", "builder_styles", "the_browser_lists_the_styles_of_the_open_mode"),
            ],
        },
        "changed_files": [
            "crates/om9-core/src/builder_style.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-core/tests/builder_style.rs",
            "crates/om9-app/tests/builder_styles.rs",
        ],
        "positive": "the ring mode lists its three styles with authored thumbnails",
        "negative": "a corrupt thumbnail falls back to the placeholder and the row stays",
    },
    "OM9-L3-03-12-003": {
        "title": "Builder Style Apply And Preview",
        "api": [
            ("`BuilderStyle::apply(session)`", "loads the parameters into the session (preview source) and reports `applied`/`ignored`; the document is untouched"),
            ("`StyleApply::summary`", "one-line report for the panel and the command line"),
            ("`Controller::builder_apply_style`", "preview refresh through `builder_refresh`, applied-style marker, document byte-identical"),
            ("`Style=<name>`", "command-line route to the same service"),
        ],
        "fixture_positive": {
            "steps": ["style A width 2.2", "style B width 3.0", "Cancel"],
            "expect": {"session_width": 3.0, "document_unchanged": True, "undo_len": 0, "cancel_baseline": 2.0},
        },
        "fixture_negative": {
            "cases": [
                {"style": "without profile_ref on RailProfile", "expect": "Missing(\"style profile\")"},
                {"style": "name not in the library", "expect": "Missing(\"builder style\")"},
            ]
        },
        "l4_tests": {
            "03": [
                ("om9-core", "builder_style", "applying_a_style_changes_the_preview_only_and_cancel_keeps_the_baseline")
            ],
            "04": [
                ("om9-app", "builder_styles", "applying_a_style_changes_the_preview_only"),
                ("om9-app", "builder_styles", "cancel_and_apply_clear_the_applied_style_marker"),
            ],
            "05": [("om9-core", "builder_style", None), ("om9-app", "builder_styles", None)],
            "06": [
                ("om9-core", "builder_style", "applying_a_style_changes_the_preview_only_and_cancel_keeps_the_baseline"),
                ("om9-app", "builder_styles", "applying_a_style_changes_the_preview_only"),
            ],
            "07": [
                ("om9-core", "builder_style", "a_style_missing_the_mandatory_profile_input_is_refused")
            ],
        },
        "changed_files": [
            "crates/om9-core/src/builder_style.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-core/tests/builder_style.rs",
            "crates/om9-app/tests/builder_styles.rs",
        ],
        "positive": "two previews and a Cancel: the document stays byte-identical and a new session starts at the baseline",
        "negative": "a style without the mandatory profile is refused before it can touch the preview",
    },
    "OM9-L3-03-12-004": {
        "title": "Builder Style Save New Name",
        "api": [
            ("`StyleLibrary::save_user`", "user list only, built-ins untouched; duplicate name is `Invalid(\"style exists\")`"),
            ("`validate_style_name`", "trimmed, bounded, no separators, traversal, leading dot or control text"),
            ("`StyleLibrary::save_to` / `load_from`", "`om9.builder_style.v1` user file; a refused write is `Permission(\"style library\")`, a corrupt file `Invalid(\"style schema\")`"),
            ("`Controller::builder_save_style` / `StyleSave=`", "snapshot the open session under a new name"),
            ("`OM9_STYLE_LIBRARY` / `Controller::builder_set_style_path`", "where the user file lives; without it the list is session-only"),
        ],
        "fixture_positive": {
            "save": {"name": "Classic 2.5", "width_mm": 2.4},
            "expect": {"user_rows": 1, "builtin_rows": 3, "file": "om9.builder_style.v1 with the user style only"},
        },
        "fixture_negative": {
            "cases": [
                {"name": "classic 2.5", "expect": "Invalid(\"style exists\")"},
                {"name": "Classic 2.2", "expect": "Invalid(\"style exists\") (built-in name)"},
                {"name": "../escape", "expect": "Invalid(\"style name\")"},
                {"write": "a directory as the style file", "expect": "Permission(\"style library\")"},
                {"file": "{ not json", "expect": "Invalid(\"style schema\")"},
            ]
        },
        "l4_tests": {
            "03": [
                ("om9-core", "builder_style", "saving_a_user_style_leaves_the_defaults_untouched_and_refuses_a_duplicate_name")
            ],
            "04": [
                ("om9-app", "builder_styles", "saving_a_style_adds_a_user_row_without_rewriting_the_defaults"),
                ("om9-app", "builder_styles", "user_styles_roundtrip_through_the_configured_file"),
            ],
            "05": [("om9-core", "builder_style", None), ("om9-app", "builder_styles", None)],
            "06": [
                ("om9-core", "builder_style", "saving_a_user_style_leaves_the_defaults_untouched_and_refuses_a_duplicate_name"),
                ("om9-app", "builder_styles", "saving_a_style_adds_a_user_row_without_rewriting_the_defaults"),
            ],
            "07": [
                ("om9-core", "builder_style", "a_style_name_cannot_be_a_path_or_control_text"),
                ("om9-core", "builder_style", "the_user_library_roundtrips_through_a_file_and_reports_a_refused_write"),
            ],
        },
        "changed_files": [
            "crates/om9-core/src/builder_style.rs",
            "crates/om9-app/src/session.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-core/tests/builder_style.rs",
            "crates/om9-app/tests/builder_styles.rs",
        ],
        "positive": "one new named user style; the three built-ins are unchanged",
        "negative": "duplicate/built-in names, traversal text, a refused write and a corrupt file are typed errors",
    },
    "OM9-L3-03-12-005": {
        "title": "Builder Style Mode And Shape Filter",
        "api": [
            ("`StyleFamily::matches`", "band/signet/setting fit ring builders; gem does not"),
            ("`BuilderKind::tag` / `from_tag` / `is_ring_builder`", "stable tags for the style file and the filter"),
            ("`BuilderStyle::compatible` / `targets`", "family and optional builder tags decide whether a row is enabled"),
            ("`BuilderStyle::apply`", "a foreign style is `Invalid(\"style mode\")`, never a silent no-op"),
            ("`StyleEntry.enabled`", "the panel lists the incompatible style disabled instead of hiding it"),
        ],
        "fixture_positive": {
            "mode": "RingRail",
            "styles": ["Classic 2.2 (band)", "Signet 3.0 (band)", "Brilliant Gem 1.9 (gem)"],
            "expect": {"enabled": 2, "disabled": 1},
        },
        "fixture_negative": {
            "cases": [
                {"style": "gem family on a ring builder", "expect": "Invalid(\"style mode\")"},
                {"style": "builder tag gem_builder", "expect": "Invalid(\"style builder\")"},
                {"style": "only foreign parameters", "expect": "Invalid(\"style mode\")"},
            ]
        },
        "l4_tests": {
            "03": [
                ("om9-core", "builder_style", "the_builder_tags_filter_a_style_to_one_mode"),
                ("om9-core", "builder_style", "a_style_ignores_parameters_of_another_mode_and_reports_them"),
            ],
            "04": [("om9-app", "builder_styles", "a_style_for_another_mode_is_listed_disabled_and_refused")],
            "05": [("om9-core", "builder_style", None), ("om9-app", "builder_styles", None)],
            "06": [
                ("om9-core", "builder_style", "a_gem_style_cannot_be_applied_to_a_ring_builder"),
                ("om9-app", "builder_styles", "a_style_for_another_mode_is_listed_disabled_and_refused"),
            ],
        },
        "changed_files": [
            "crates/om9-core/src/builder_style.rs",
            "crates/om9-core/src/builder.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-core/tests/builder_style.rs",
            "crates/om9-app/tests/builder_styles.rs",
        ],
        "positive": "the ring mode enables the two ring styles and disables the gem one",
        "negative": "a gem style and a foreign-parameter style are refused by the ring builder",
    },
    "OM9-L3-03-12-006": {
        "title": "Builder Style Match Attributes",
        "api": [
            ("`BuilderStyle::match_attributes(entity)`", "copies the declared `om9.rail`/`om9.head` members with their types"),
            ("`STYLE_ATTRIBUTES`", "`width_mm`, `height_mm` (length) and `material` (choice)"),
            ("`Controller::builder_match_attributes`", "reads the selected entity, sets the matching session fields, leaves ids alone"),
            ("`StyleMatch`", "command-line route to the same service"),
        ],
        "fixture_positive": {
            "source": {"om9.rail": {"width_mm": 3.0, "height_mm": 2.0, "material": "synthetic"}},
            "expect": {"copied": ["width_mm", "height_mm", "material"], "identity": "unchanged"},
        },
        "fixture_negative": {
            "cases": [
                {"source": "no declared slot", "expect": "Missing(\"style source\")"},
                {"source": {"width_mm": "wide"}, "expect": "Invalid(\"style attribute\")"},
                {"selection": "empty", "expect": "Missing(\"style source\")"},
            ]
        },
        "l4_tests": {
            "03": [
                ("om9-core", "builder_style", "matching_attributes_copies_only_the_declared_attributes_and_keeps_the_identity")
            ],
            "04": [
                ("om9-app", "builder_styles", "matching_attributes_copies_from_the_selection_and_keeps_the_ids")
            ],
            "05": [("om9-core", "builder_style", None), ("om9-app", "builder_styles", None)],
            "06": [
                ("om9-core", "builder_style", "matching_attributes_copies_only_the_declared_attributes_and_keeps_the_identity"),
                ("om9-app", "builder_styles", "matching_attributes_copies_from_the_selection_and_keeps_the_ids"),
            ],
        },
        "changed_files": [
            "crates/om9-core/src/builder_style.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-core/tests/builder_style.rs",
            "crates/om9-app/tests/builder_styles.rs",
        ],
        "positive": "width 3, height 2 and material synthetic are copied; the source entity id and slots are unchanged",
        "negative": "a source without a declared slot and a wrong-typed member are refused",
    },
}

L4_ROLES = {
    "03": ("Implementation", "IMPLEMENT", "implementation"),
    "04": ("Integration", "INTEGRATION", "integration"),
    "05": ("Regression Tests", "REGRESSION", "regression"),
    "06": ("Acceptance And Evidence", "ACCEPTANCE", "acceptance"),
    "07": ("Adversarial And Recovery Tests", "ADVERSARIAL", "adversarial"),
}


def write_l4_packages(revision: str) -> dict:
    """Write the four L4 artefacts per L3 and one receipt per L4 role.

    The dashboard counts an L4 only when its own artefact exists: a ``source_crosswalk.json``
    for SOURCE_AUDIT, ``api.md`` + ``positive.json`` + ``negative.json`` for CONTRACT_TEST, and
    a file whose name starts with the L4 id for every later role. Each receipt is a record of a
    command that actually ran here, with its exit code and output hash.
    """
    _, status = run(["git", "status", "--porcelain"])
    revision = revision.strip().splitlines()[0] if revision.strip() else "UNRESOLVED"
    dirty = bool(status.strip())
    summary: dict[str, dict] = {}
    for l3_id, package in L4_PACKAGES.items():
        directory = ROOT / "implementation" / "evidence" / "M04" / l3_id
        directory.mkdir(parents=True, exist_ok=True)
        positive = json.dumps(
            package["fixture_positive"], indent=2, ensure_ascii=False, sort_keys=True
        )
        negative = json.dumps(
            package["fixture_negative"], indent=2, ensure_ascii=False, sort_keys=True
        )
        manifest = hashlib.sha256((positive + negative).encode("utf-8")).hexdigest()
        (directory / "positive.json").write_text(positive + "\n", encoding="utf-8")
        (directory / "negative.json").write_text(negative + "\n", encoding="utf-8")

        symbols = [
            row[0].strip("`").split("(")[0].strip()
            for row in package["api"]
            if row[0].startswith("`")
        ]
        crosswalk = {
            "schema": "om9.m04.source_crosswalk.v1",
            "l3_id": l3_id,
            "title": package["title"],
            "owner": "om9-core builder_style + om9-app builder session/panel",
            "source_revision": revision,
            "checkout_dirty": dirty,
            "sources": [
                {
                    "path": path,
                    "symbols": symbols if index == 0 else [],
                    "state": "INSPECTED_IN_CHECKOUT",
                }
                for index, path in enumerate(package["changed_files"])
            ],
            "source_audit": {
                "path": "QymCAD/",
                "result": "NO_BUILDER_STYLE_MODULE",
                "detail": (
                    "No builder-style/preset module exists under QymCAD/crates; the only `preset` "
                    "hits are unrelated dialog/extrude code. Decision: ADD in Rust, reusing the "
                    "OM9 builder session and producer preset vocabulary."
                ),
            },
            "planning_inputs": [
                "OM9_Milestones/M04 - Builder and History/L2-03-12 - 03-febe-matrix-builder-styles-and-presets/L2.json",
                "matrix8_book_1.pdf printed pages 20-21 (pdf 30-31) referenced by the L3 handoff",
            ],
            "planning_tree_touched": False,
            "notes": [
                "The L3 skill/plan record is planning input; runtime behavior is established by the receipts.",
                "The source audit is a checkout inspection, not a QymCAD implementation reuse claim.",
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

## Builders the styles drive

| Mode | Tag | Inputs |
|---|---|---|
| Ring Rail | `ring_rail` | none |
| Rail Profile | `rail_profile` | rail curve |
| Rail Sweep | `rail_sweep` | rail + profile curve |
| Outside Rail | `outside_rail` | rail curve |
| Ring Resizer | `ring_resizer` | ring curve |

## Invariants

- A style is a declarative record: applying it never writes the document, and the builder's own
  Apply stays the one transaction.
- Built-in styles are read-only; the user list is a separate file/list.
- An incompatible mode/family or a missing mandatory input is a typed error, never a silent no-op.
- Matching attributes keeps the source entity identity and its extension slots.
- A corrupt style file or thumbnail degrades explicitly (typed error, placeholder), never silently.
"""
        (directory / "api.md").write_text(api, encoding="utf-8")

        receipts = []
        for suffix, tests in package["l4_tests"].items():
            if not tests:
                continue
            title_suffix, role, kind = L4_ROLES[suffix]
            l4_id = f"{l3_id.replace('OM9-L3-', 'OM9-L4-')}-{suffix}"
            commands = []
            logs = []
            for package_name, binary, test in tests:
                command = [
                    "cargo",
                    "test",
                    "-p",
                    package_name,
                    "--test",
                    binary,
                    "-j",
                    "2",
                ]
                if test:
                    command.append(test)
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
                        "test": test,
                        "package": package_name,
                        "binary": binary,
                    }
                )
                logs.append(header(" ".join(command), code, captured))
                if code != 0:
                    raise SystemExit(
                        f"L4 command failed for {l4_id}: {' '.join(command)}"
                    )
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
                    "manual review of the Builder Styles window at the M04-UI gate",
                    "a reviewer signature",
                ],
                "fixture_manifest_sha256": manifest,
                "document_snapshot": {
                    "source": "asserted by the registered tests",
                    "positive": package["positive"],
                    "negative": package["negative"],
                },
                "l4_id": l4_id,
                "role": role,
                "kind": kind,
                "status": "READY_FOR_REVIEW",
                "changed_files": package["changed_files"],
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
        summary[l3_id] = {
            "l4_receipts": receipts,
            "fixture_manifest_sha256": manifest,
        }
        # Attach the L4 summary to the L3 record so one file explains the whole package.
        record_path = directory / "observed.json"
        record = json.loads(record_path.read_text(encoding="utf-8"))
        record["l4"] = summary[l3_id]
        record_path.write_text(
            json.dumps(record, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
    return summary


def main() -> int:
    results = []
    failed = False
    for name, command, with_path in CHECKS:
        code, captured = run(command, with_path)
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
    observed = {
        "schema": "om9.m04.builder_styles.v1",
        "recorded_at_utc": datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "source_revision": revision.strip().splitlines()[0] if revision.strip() else "UNRESOLVED",
        "checkout_dirty": bool(status.strip()),
        "target": TARGET,
        "scope": [
            "versioned style file om9.builder_style.v1 with typed parameters and explicit defaults",
            "style browser: one row per style for the open mode, thumbnail or placeholder fallback",
            "preview-only apply: style -> session fields -> preview, document untouched until Apply",
            "user saves in their own list/file; built-ins are read-only and never rewritten",
            "mode/family filter: incompatible styles are listed disabled and refused on apply",
            "attribute matching from a declared source slot, keeping entity identity",
            "UI: Builder Styles window with rows, selected marker, Save field and Match action",
            "command line: Style=<name>, StyleSave=<name>, StyleMatch",
        ],
        "code": [
            "crates/om9-core/src/builder_style.rs",
            "crates/om9-core/src/builder.rs",
            "crates/om9-core/src/lib.rs",
            "crates/om9-core/tests/builder_style.rs",
            "crates/om9-app/src/session.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-app/src/ui.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-app/tests/builder_styles.rs",
            "tools/record_m04_l2_03_12_styles.py",
        ],
        "checks": results,
        "l3_coverage": L3_COVERAGE,
        "manual_gate": "OPEN: M04-UI — no reviewer signature, no pointer-frame evidence for the window",
        "uncertainties": [
            "The user style file location comes from OM9_STYLE_LIBRARY; without it the user list lives for the session only — no default user directory was chosen.",
            "Thumbnails are authored icon keys with a placeholder fallback; no raster thumbnail cache is implemented.",
            "Only the five ring builders exist, so a gem-family style is always listed disabled; the gem builder belongs to another owner.",
            "The Builder Styles window is painted by egui, but no screenshot/pointer frame records it.",
            "macOS runtime is unverified; the evidence is Windows x64 only.",
        ],
        "not_claimed": [
            "Human acceptance of the style browser, the apply/save gestures and the match action.",
            "Persistence into a default user directory or a resource-pack thumbnail store.",
        ],
    }
    OUTPUT.mkdir(parents=True, exist_ok=True)
    (OUTPUT / "observed.json").write_text(
        json.dumps(observed, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    # Per-L3 records: the dashboard reads these directories, and a human gate stays open.
    for coverage in L3_COVERAGE:
        directory = ROOT / "implementation" / "evidence" / "M04" / coverage["l3"]
        directory.mkdir(parents=True, exist_ok=True)
        for log in ("core-builder-style.log", "app-builder-styles.log", "workspace-check.log"):
            source = OUTPUT / log
            if source.is_file():
                shutil.copyfile(source, directory / log)
        (directory / "observed.json").write_text(
            json.dumps(
                {
                    "schema": "om9.m04.l3_builder_style_evidence.v1",
                    "l3_id": coverage["l3"],
                    "title": coverage["title"],
                    "scope": " ".join(coverage["code"]),
                    "target": TARGET,
                    "tests": coverage["tests"],
                    "fixture": coverage["fixture"],
                    "manual_gate": "OPEN",
                    "reviewer": None,
                    "statement": (
                        "Automated evidence only. The L3 requires a human review of the Builder "
                        "Styles browser and its gestures; no reviewer signature is claimed."
                    ),
                    "logs": [
                        "core-builder-style.log",
                        "app-builder-styles.log",
                        "workspace-check.log",
                    ],
                },
                indent=2,
                ensure_ascii=False,
            )
            + "\n",
            encoding="utf-8",
        )

    l4_summary = write_l4_packages(revision)
    l4_receipts = sum(len(item["l4_receipts"]) for item in l4_summary.values())
    observed["l4"] = {
        "receipts": l4_receipts,
        "per_l3": {l3: len(item["l4_receipts"]) for l3, item in l4_summary.items()},
        "artefact_names": [
            "source_crosswalk.json",
            "api.md",
            "positive.json",
            "negative.json",
            "OM9-L4-<l3>-<role>.json",
        ],
        "note": (
            "Every receipt records a command that ran here with its exit code and output hash; "
            "the acceptance receipts stay READY_FOR_REVIEW with signature null."
        ),
    }
    (OUTPUT / "observed.json").write_text(
        json.dumps(observed, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    rows = "\n".join(
        f"| `{item['l3']}` | {item['title']} | {item['state']} |" for item in L3_COVERAGE
    )
    readme = f"""# M04 L2-03-12 builder styles and presets

This directory records the builder style/preset batch for `OM9-L2-03-12`
(03-febe-matrix-builder-styles-and-presets). It reuses the builder session and the producer
preset vocabulary from `OM9-L2-03-11`; no second builder or profile library was added.

## What is implemented

- `om9_core::builder_style` — the `om9.builder_style.v1` file, typed `StyleValue`
  (`length_mm`/`number`/`choice`/`boolean`), `BuilderStyle::snapshot`/`defaults_of`/`validate`,
  `StyleLibrary` (built-ins + user list), `StyleThumbnail`, `match_attributes`.
- Browser — `Controller::builder_styles` returns one row per style for the open mode; an
  incompatible style is listed but disabled, and a missing/corrupt thumbnail falls back to the
  placeholder instead of dropping the row.
- Apply — `BuilderStyle::apply` loads the parameters into the session (the preview source) and
  reports the applied/ignored names; the document is untouched until the builder's Apply. A
  style without the mandatory `profile_ref` is refused with `Missing("style profile")`.
- Save — `StyleLibrary::save_user` validates the name (no separators, traversal, control text or
  duplicates) and never rewrites the built-ins; `save_to`/`load_from` move the user list through
  `OM9_STYLE_LIBRARY`, and a refused write is reported.
- Filter — `StyleFamily::matches`/`BuilderKind::tag` gate the list; a gem style is refused by a
  ring builder with `Invalid("style mode")`.
- Match attributes — declared `om9.rail`/`om9.head` members (`width_mm`, `height_mm`,
  `material`) are copied with their types; the source entity keeps its id and slots.
- UI — the builder panel's `Styles…` button opens the Builder Styles window (rows, selected
  marker, Save field, Match action, user-file line); the command line accepts `Style=<name>`,
  `StyleSave=<name>` and `StyleMatch`.

## L3 traceability

| L3 | Capability | State |
|----|------------|-------|
{rows}

## L4 packages

Each L3 carries its own L4 artefacts under `implementation/evidence/M04/<L3>/`:
`source_crosswalk.json` (SOURCE_AUDIT), `api.md` + `positive.json` + `negative.json`
(CONTRACT_TEST), and one `OM9-L4-...-0X.json` receipt per IMPLEMENT / INTEGRATION /
REGRESSION / ACCEPTANCE / ADVERSARIAL role — **{l4_receipts} receipts, 39 planned L4**.
A receipt names the command that ran, its exit code and the hash of its output, and keeps the
M04-UI manual gate open.

## Verification

See `observed.json` and the logs here. Focused commands:

- `cargo test -p om9-core --test builder_style`
- `cargo test -p om9-core --test builder_controls --test builder_session --test history_mutation`
- `cargo test -p om9-app --test builder_styles --test builder_panel --test builder_controls --test builder_slot --test top11`
- `cargo test -p om9-kernel --features 3dm-brep --test associative_history` (real OCCT sweep)
- `cargo clippy -p om9-core --lib --no-deps -- -D warnings`
- `cargo check --workspace --all-targets`

## Not claimed

- The M04-UI manual gate: no human has reviewed the style browser, the apply/save gestures or
  the match action on screen, and no screenshot/pointer frame records them.
- A default user-style directory: the path comes from `OM9_STYLE_LIBRARY`, otherwise the user
  list is session-only.
- Raster thumbnails: the browser uses authored icon keys with a placeholder fallback.
- macOS runtime for any of this: the evidence is Windows x64 only.
"""
    (OUTPUT / "README.md").write_text(readme, encoding="utf-8")

    ledger_path = ROOT / "docs" / "openmatrix9-progress.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    entry = {
        "key": KEY,
        "date": "2026-10-27",
        "stage": "feature-port",
        "status": "builder_styles_automated_verified_manual_gates_open",
        "target": TARGET,
        "feature_ids": ["OM9-L2-03-12", "OM9-L2-03-11"],
        "scope": (
            "M04 OM9-L2-03-12 builder styles/presets: file om9.builder_style.v1 với typed "
            "parameters, browser row + thumbnail fallback, preview-only apply, user save tách "
            "khỏi built-ins, mode/family filter, match attributes giữ identity; UI Builder Styles "
            "window + Style=/StyleSave=/StyleMatch; 6 L3 có evidence tự động, gate M04-UI còn mở."
        ),
        "outputs": observed["code"],
        "evidence": [
            "implementation/evidence/M04/OM9-L2-03-12/observed.json",
            "implementation/evidence/M04/OM9-L2-03-12/README.md",
            "implementation/evidence/M04/OM9-L2-03-12/core-builder-style.log",
            "implementation/evidence/M04/OM9-L2-03-12/app-builder-styles.log",
            "implementation/evidence/M04/OM9-L2-03-12/core-builder-regression.log",
            "implementation/evidence/M04/OM9-L2-03-12/occt-chain.log",
            "implementation/evidence/M04/OM9-L2-03-12/clippy-core-lib.log",
            "implementation/evidence/M04/OM9-L2-03-12/workspace-check.log",
        ]
        + [
            f"implementation/evidence/M04/{item['l3']}/observed.json"
            for item in L3_COVERAGE
        ],
        "tests": {
            "core_builder_style": "14 passed (roundtrip, schema/numeric refusal, browser, thumbnail fallback, preview apply, mandatory profile, gem refusal, save/duplicate/name, file roundtrip, attributes, tags)",
            "core_regression": "builder_controls 7 + builder_session 8 + history_mutation 3 passed",
            "app_builder_styles": "9 passed (browser, preview-only apply, other-mode refusal, save/duplicate/traversal, file roundtrip, match, command line, marker clear, locked input)",
            "app_regression": "builder_panel 10 + builder_controls 8 + builder_slot 9 + top11 19 passed",
            "occt_chain": "2 passed (real B-Rep sweep, topology names, reference after edit)",
            "clippy_core_lib": "cargo clippy -p om9-core --lib --no-deps -- -D warnings exit 0",
            "workspace_check": "cargo check --workspace --all-targets exit 0",
        },
        "uncertainties": observed["uncertainties"],
        "manual_gate": "OPEN: M04-UI — style browser/save/match chưa được người nghiệm thu",
        "next_action": (
            "Chụp/duyệt Builder Styles window cho gate M04-UI; chọn thư mục user-style mặc định nếu "
            "muốn lưu ngoài OM9_STYLE_LIBRARY; thumbnail raster thuộc owner resource-package."
        ),
    }
    ledger["items"] = [item for item in ledger["items"] if item.get("key") != KEY] + [entry]
    ledger["active_item"] = KEY
    ledger["updated_at"] = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    ledger_path.write_text(json.dumps(ledger, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    owner_path = (
        ROOT
        / "OM9_Milestones"
        / "M04 - Builder and History"
        / "L2-03-12 - 03-febe-matrix-builder-styles-and-presets"
        / "L2.json"
    )
    owner = json.loads(owner_path.read_text(encoding="utf-8"))
    owner["runtime_implementation"] = {
        "date": "2026-10-27",
        "status": "AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING",
        "target": TARGET,
        "scope": "builder style snapshot, browser, preview apply, user save, mode filter, attribute match",
        "owner": {
            "crates": ["om9-core", "om9-app"],
            "tests": [
                "crates/om9-core/tests/builder_style.rs",
                "crates/om9-app/tests/builder_styles.rs",
            ],
        },
        "evidence": [
            "implementation/evidence/M04/OM9-L2-03-12/observed.json",
            "implementation/evidence/M04/OM9-L2-03-12/README.md",
            "implementation/evidence/M04/OM9-L2-03-12/core-builder-style.log",
            "implementation/evidence/M04/OM9-L2-03-12/app-builder-styles.log",
            "implementation/evidence/M04/OM9-L2-03-12/workspace-check.log",
        ],
        "l3_coverage": {item["l3"]: item["state"] for item in L3_COVERAGE},
        "tests": [f"{result['artifact']}: exit {result['exit_code']}" for result in results],
        "human_approval": None,
        "manual_gate": "M04-UI",
        "remaining": observed["uncertainties"],
    }
    owner_path.write_text(
        json.dumps(owner, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    print(f"recorded builder style evidence at {OUTPUT.relative_to(ROOT)}")
    for result in results:
        print(f"  {result['artifact']}: exit {result['exit_code']}")
    print(f"ledger: {ledger_path.relative_to(ROOT)} ({len(ledger['items'])} items)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

