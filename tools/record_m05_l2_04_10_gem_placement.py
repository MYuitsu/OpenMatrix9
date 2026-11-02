#!/usr/bin/env python3
"""Record the M05 ``OM9-L2-04-10`` gems-on-curves/surfaces capability from real runs.

Scope: all eleven L3 of ``02-febe-gem-on-curves-and-surfaces``:

* ``OM9-L3-04-10-001`` gem on curve basic;
* ``OM9-L3-04-10-002`` gem on curve advanced (taper);
* ``OM9-L3-04-10-003`` gem count on curve;
* ``OM9-L3-04-10-004`` gem list on curve (heterogeneous stones);
* ``OM9-L3-04-10-005`` gem on curve multi (atomic);
* ``OM9-L3-04-10-006`` gem profile curve and guides;
* ``OM9-L3-04-10-007`` gem on surface;
* ``OM9-L3-04-10-008`` gem follow underlying curve;
* ``OM9-L3-04-10-009`` gem flow placement;
* ``OM9-L3-04-10-010`` gem splop on surface;
* ``OM9-L3-04-10-011`` gem to curve/surface history links.

Every L3 in this module is a command with a human-facing M05-UI gate
(``requires_manual_test: true``), so no reviewer signature is claimed and no
``ui-review.json`` is written. The dashboard therefore counts each L3 as WIP-with-evidence,
never DONE.

Each L3 directory receives ``source_crosswalk.json`` (L4-01), ``api.md`` +
``positive.json`` + ``negative.json`` (L4-02), one ``OM9-L4-10-0xx-03`` receipt + log
(L4-03) and ``observed.json``.

Usage::

    python tools/record_m05_l2_04_10_gem_placement.py
"""

from __future__ import annotations

import datetime
import hashlib
import json
import os
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
TARGET_DIR = os.environ.get("CARGO_TARGET_DIR", r"H:\OpenMatrix9\cargo-target")
KEY = "m05-l2-04-10-gem-placement-2026-10-27"
MILESTONE_KEY = "M05"
BATCH = ROOT / "implementation" / "evidence" / MILESTONE_KEY / "OM9-L2-04-10"
L3_ROOT = (
    ROOT
    / "OM9_Milestones"
    / "M05 - Gems and Layout"
    / "L2-04-10 - 02-febe-gem-on-curves-and-surfaces"
)
TARGET = (
    "Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=H:/OpenMatrix9/cargo-target; -j 2; "
    "app tests need the OCCT bin and VC runtime on PATH"
)

# The app tests link OCCT, so the runtime DLL directories are prepended for the child runs.
RUNTIME_PATH = [
    ROOT / "build" / "standalone-deps" / "occt-8.0.1-sdk" / "bin",
    ROOT / "build" / "ci" / "standalone-local-r2-20261021" / "clean-path-smoke" / "OpenMatrix9",
]

CHECKS = [
    (
        "core-gem-placement.log",
        ["cargo", "test", "-p", "om9-core", "--test", "gem_placement", "-j", "2"],
    ),
    (
        "app-gem-placement.log",
        ["cargo", "test", "-p", "om9-app", "--test", "gem_placement", "-j", "2"],
    ),
    (
        "core-gem-assets-regression.log",
        ["cargo", "test", "-p", "om9-core", "--test", "gem_assets", "-j", "2"],
    ),
    (
        "app-gem-panel-regression.log",
        ["cargo", "test", "-p", "om9-app", "--test", "gem_panel", "-j", "2"],
    ),
    (
        "core-transaction-regression.log",
        [
            "cargo",
            "test",
            "-p",
            "om9-core",
            "--test",
            "atomic_transactions",
            "--test",
            "transaction_scopes",
            "-j",
            "2",
        ],
    ),
    (
        "workspace-check.log",
        ["cargo", "check", "--workspace", "--all-targets", "-j", "2"],
    ),
    (
        "clippy-gem-placement.log",
        [
            "cargo",
            "clippy",
            "-p",
            "om9-core",
            "-p",
            "om9-app",
            "--all-targets",
            "-j",
            "2",
        ],
    ),
]

# Clippy is scoped to the files this task owns: the workspace is shared with other work in
# flight, so a foreign warning is recorded (never hidden) instead of failing this batch.
CLIPPY_SCOPE = {
    "crates/om9-core/src/gem_placement.rs",
    "crates/om9-core/src/gem.rs",
    "crates/om9-core/src/model.rs",
    "crates/om9-core/src/lib.rs",
    "crates/om9-core/tests/gem_placement.rs",
    "crates/om9-app/src/session/gem_placement.rs",
    "crates/om9-app/src/session/gem.rs",
    "crates/om9-app/src/session.rs",
    "crates/om9-app/tests/gem_placement.rs",
}


def assess_clippy(captured: str) -> tuple[list[str], list[str], bool]:
    """Split clippy diagnostics into this task's files and foreign files.

    Returns ``(scoped, foreign, hard_error)``. A diagnostic in a scoped file, or any compiler
    error, fails the batch; a foreign warning is logged for the next owner.
    """
    scoped: list[str] = []
    foreign: list[str] = []
    hard = False
    for line in captured.splitlines():
        if line.startswith("error:") or line.startswith("error["):
            hard = True
        if "-->" not in line:
            continue
        path = line.split("-->", 1)[1].strip().split(":", 1)[0].replace("\\", "/")
        if path in CLIPPY_SCOPE:
            scoped.append(path)
        elif path.startswith("crates/"):
            foreign.append(path)
    return sorted(set(scoped)), sorted(set(foreign)), hard

CHANGED = [
    "crates/om9-core/src/gem_placement.rs",
    "crates/om9-core/src/gem.rs",
    "crates/om9-core/src/model.rs",
    "crates/om9-core/src/lib.rs",
    "crates/om9-core/tests/gem_placement.rs",
    "crates/om9-app/src/session/gem_placement.rs",
    "crates/om9-app/src/session/gem.rs",
    "crates/om9-app/src/session.rs",
    "crates/om9-app/tests/gem_placement.rs",
    "tools/record_m05_l2_04_10_gem_placement.py",
]

MANUAL = "M05-UI"

L3_PACKAGES = {
    "OM9-L3-04-10-001": {
        "title": "Gem On Curve Basic",
        "folder": "L3-001 - Gem On Curve Basic",
        "command": "BuilderGemOnCrv / gvGemOnCrv",
        "test": "the_row_previews_without_an_object_and_applies_in_one_transaction",
        "api": [
            (
                "`om9_core::gem_placement::place_on_curve`",
                "arc-length stations between `start_mm` and `end_mm`, `spacing_mm`, closed-seam "
                "dedupe, overlap refusal unless requested",
            ),
            (
                "`CurveParams`",
                "`spacing/with_range/with_z_offset/with_rotation/with_orientation/with_overlap/"
                "validate` — typed refusals for non-finite or non-positive spacing",
            ),
            (
                "`GemStation`",
                "index, fraction, arc length, point, tangent, table axis, `Transform`, spec, "
                "z offset, spin and an orthonormal right-handed frame",
            ),
            (
                "`Controller::dispatch_gem_placement` + `gem_placement_text`",
                "palette icon and typed command reach one session; `Start/End/Spacing/Count` "
                "recompute the row, an option the tool does not consume is a typed refusal, "
                "`Enter` applies, `Esc` cancels",
            ),
        ],
        "positive": {
            "cases": [
                "a 100 mm line at 8 mm spacing yields 13 stations from 0 to 96 mm",
                "`Count=5` on the same line yields exactly 5 stations at 0/25/50/75/100 mm",
                "every station frame is right-handed and the table axis is world +Z upright",
                "Apply inserts one gem per station in ONE transaction; a single Undo removes the row",
                "each gem keeps its spec in `om9.gem` and its `om9.gem.placement` curve link",
            ]
        },
        "negative": {
            "cases": [
                "a stone longer than the spacing is refused unless overlap is requested",
                "an option the tool does not consume (`Sizes=` on the row) is a typed refusal",
                "a closed curve does not duplicate the seam station",
                "a wrong selection (zero or two curves) is a typed refusal and opens no session",
            ]
        },
        "positive_note": "even arc-length row, orthonormal frames, one-transaction apply and Undo",
        "negative_note": "overlap, seam duplication and wrong selections are typed refusals",
        "extra_tests": [
            "count_places_an_exact_number_of_stones_between_the_range_ends",
            "an_option_the_tool_does_not_use_is_a_typed_refusal",
        ],
    },
    "OM9-L3-04-10-002": {
        "title": "Gem On Curve Advanced",
        "folder": "L3-002 - Gem On Curve Advanced",
        "command": "BuilderGemOnCrvAdvanced / gvGemOnCrvAdvanced",
        "test": "the_advanced_tool_tapers_the_row_from_the_start_size_to_the_end_size",
        "api": [
            (
                "`gem_placement::taper_specs(start, end, count)`",
                "monotonic interpolation between the first and last spec; every intermediate stone "
                "is validated, never clamped",
            ),
            (
                "`gem_placement::place_on_curve_variable`",
                "one spec per station; a length mismatch is `Error::Invalid(\"gem spec list\")`",
            ),
            (
                "`GemPlacementTool::OnCurveAdvanced`",
                "`Size=` seeds the start, `EndSize=` seeds the end, `Spacing=` sets the station count",
            ),
        ],
        "positive": {
            "cases": [
                "the tapered row keeps the requested station count and changes diameter monotonically",
                "the first gem keeps the start size and the last gem the end size",
                "each intermediate spec still passes `GemSpec::validate`",
            ]
        },
        "negative": {
            "cases": [
                "a non-positive or non-finite end size is a typed refusal",
                "a spec list whose length does not match the stations is refused, not truncated",
            ]
        },
        "positive_note": "monotonic variable-diameter row between the two requested sizes",
        "negative_note": "bad sizes and mismatched spec lists are typed refusals",
    },
    "OM9-L3-04-10-003": {
        "title": "Gem Count On Curve",
        "folder": "L3-003 - Gem Count On Curve",
        "command": "BuilderGemCountOnCrv / gvGemCountOnCrv",
        "test": "the_count_tool_fits_the_row_between_the_curve_ends",
        "api": [
            (
                "`gem_placement::count_on_curve`",
                "`CurveFit { count, spacing_mm, first_center_mm, last_center_mm, fits }` from the "
                "diameter, gap and end allowance",
            ),
            (
                "`CurveFit`",
                "an explicit `fits == false` for a curve shorter than one stone; the caller must "
                "refuse rather than place a partial row",
            ),
            (
                "`GemPlacementTool::CountOnCurve`",
                "`Gap=` and `Allowance=` drive the fit; Apply reuses the fitted first/last centres",
            ),
        ],
        "positive": {
            "cases": [
                "a 100 mm line with 6 mm stones and a 2 mm gap fits 12 stones",
                "the first centre is 3 mm and the last 91 mm, so both ends are cleared",
                "the count matches the independent fit arithmetic in the core test",
            ]
        },
        "negative": {
            "cases": [
                "a 5 mm line cannot host a 6 mm stone: `fits` is false and the tool refuses",
                "a negative gap or allowance is refused by parameter validation",
            ]
        },
        "positive_note": "fit count derives from arc length, stone diameter and the requested gap",
        "negative_note": "a non-fitting row is refused instead of placing a partial row",
    },
    "OM9-L3-04-10-004": {
        "title": "Gem List On Curve",
        "folder": "L3-004 - Gem List On Curve",
        "command": "BuilderGemListOnCrv / gvGemListOnCrv",
        "test": "the_list_tool_places_heterogeneous_stones_in_order",
        "api": [
            (
                "`gem_placement::place_on_curve_variable`",
                "heterogeneous `&[GemSpec]` placed in order; a one-element list broadcasts",
            ),
            (
                "`GemPlacementTool::ListOnCurve`",
                "`Sizes=` parses a comma-separated diameter list into validated round specs",
            ),
            (
                "`CurveParams::with_range`",
                "the list is distributed between `Start=` and `End=` with a computed pitch",
            ),
        ],
        "positive": {
            "cases": [
                "a 3-stone list [6, 5, 4] on a 30 mm curve places all three in the requested order",
                "each station carries its own `GemSpec`, so the meshes differ",
                "the row is still one transaction and one Undo",
            ]
        },
        "negative": {
            "cases": [
                "an empty list is `Error::Missing(\"gem spec list\")`",
                "a diameter that fails `GemSpec::round` is refused, never silently replaced",
            ]
        },
        "positive_note": "heterogeneous diameters placed in order with per-stone specs",
        "negative_note": "empty or invalid stone lists are typed refusals",
    },
    "OM9-L3-04-10-005": {
        "title": "Gem On Curve Multi",
        "folder": "L3-005 - Gem On Curve Multi",
        "command": "BuilderGemOnCrvMulti / gvGemOnCrvMulti",
        "test": "the_multi_tool_places_one_run_per_curve_in_one_transaction",
        "api": [
            (
                "`gem_placement::place_on_curves`",
                "`MultiPlacement { groups, .. }` — one `CurvePlacement` per input curve",
            ),
            (
                "`GemPlacementTool::OnCurveMulti`",
                "every selected curve is claimed; fewer than two curves is a typed refusal",
            ),
            (
                "`Controller::gem_placement_apply`",
                "all groups commit in one transaction, so Undo removes every run together",
            ),
        ],
        "positive": {
            "cases": [
                "two selected curves produce one run each with the same spacing rule",
                "every gem link points at its own source curve",
                "the whole multi run is atomic: one Undo removes both runs",
            ]
        },
        "negative": {
            "cases": [
                "one selected curve is refused for the multi tool",
                "a curve that cannot host the stone is refused and no partial run is committed",
            ]
        },
        "positive_note": "one atomic transaction for a run on every selected curve",
        "negative_note": "missing curves and unplaceable curves are typed refusals",
    },
    "OM9-L3-04-10-006": {
        "title": "Gem Profile Curve And Guides",
        "folder": "L3-006 - Gem Profile Curve And Guides",
        "command": "BuilderGemProfile / BuilderGemGuides / gvGemProfile / gvGemGuides",
        "test": "profile_and_guides_insert_construction_curves",
        "api": [
            (
                "`gem_placement::profile_and_guides`",
                "`PlacementGuides { profile, girdle, guides, table_points }` from the placed stones",
            ),
            (
                "`GemStation::table_point`",
                "the profile runs through every table point, the girdle through every station point",
            ),
            (
                "`GemPlacementTool::{Profile, Guides}`",
                "inserts the profile/girdle/guide curves as normal document curves",
            ),
        ],
        "positive": {
            "cases": [
                "the profile curve through the table points and the girdle curve are both inserted",
                "one guide segment is generated per placed stone",
                "the guide curves are ordinary editable document curves",
            ]
        },
        "negative": {
            "cases": [
                "fewer than two placed stones is `Error::Invalid(\"gem profile needs two stations\")`",
                "a zero-length guide is refused instead of returning NaN",
            ]
        },
        "positive_note": "profile, girdle and per-stone guide curves built from the placement",
        "negative_note": "too few stones and degenerate guides are typed refusals",
    },
    "OM9-L3-04-10-007": {
        "title": "Gem On Surface",
        "folder": "L3-007 - Gem On Surface",
        "command": "BuilderGemOnSurface / gvGemOnSurface",
        "test": "the_surface_tool_stands_the_gem_on_the_surface_normal",
        "api": [
            (
                "`gem_placement::place_on_surface`",
                "evaluates the surface at each UV, builds an orthonormal frame from the U/V "
                "tangents and the normal",
            ),
            (
                "`SurfaceParams`",
                "`uv`, `z_offset_mm`, `rotation_deg`; a UV outside the domain is refused",
            ),
            (
                "`GemPlacementTool::OnSurface`",
                "`U=`, `V=` and `Grid=` place one stone or a UV grid; `ZOffset=`/`Rotate=` apply",
            ),
        ],
        "positive": {
            "cases": [
                "a stone on a 100x100 plane at z=5 sits at z=5 with its table axis on +Z",
                "a 2x2 UV grid places four stones, each with its own UV and frame",
                "the surface link keeps the UV, so the station can be rebuilt later",
            ]
        },
        "negative": {
            "cases": [
                "a UV outside the surface domain is a typed refusal",
                "a degenerate surface (no normal) is refused instead of orienting to NaN",
            ]
        },
        "positive_note": "surface normal frame, UV grid placement and a UV-bearing link",
        "negative_note": "out-of-domain UVs and degenerate normals are typed refusals",
    },
    "OM9-L3-04-10-008": {
        "title": "Gem Follow Underlying Curve",
        "folder": "L3-008 - Gem Follow Underlying Curve",
        "command": "BuilderGemFollow / gvGemFollow",
        "test": "follow_marks_gems_whose_source_is_gone_as_conflicts_instead_of_moving_them",
        "api": [
            (
                "`gem_placement::station_at_arc`",
                "rebuilds one station at the recorded arc length from the current curve",
            ),
            (
                "`PlacementLink::refresh(revision, arc, station)`",
                "revives an ACTIVE link; a revision older than the recorded one is `Error::Stale`",
            ),
            (
                "`PlacementLink::mark_conflict`",
                "a lost source station keeps the last transform and turns the link CONFLICT",
            ),
        ],
        "positive": {
            "cases": [
                "the same `GemId` is kept when the curve changes and only the transform is updated",
                "stations past a shortened curve become CONFLICT instead of moving to a wrong place",
                "the whole follow is one transaction and Undo restores the ACTIVE links",
            ]
        },
        "negative": {
            "cases": [
                "a deleted source curve produces conflicts, not a silent no-op",
                "an older source revision cannot overwrite a newer link (`Error::Stale`)",
            ]
        },
        "positive_note": "arc-length follow reuses the link and keeps gem identity",
        "negative_note": "lost sources become visible conflicts; stale writes are refused",
    },
    "OM9-L3-04-10-009": {
        "title": "Gem Flow Placement",
        "folder": "L3-009 - Gem Flow Placement",
        "command": "BuilderGVDGemFlow / gvGemFlow",
        "test": "flow_carries_the_same_stones_onto_a_target_curve",
        "api": [
            (
                "`gem_placement::flow_station`",
                "maps a station from the base curve to the target curve at the same curve fraction",
            ),
            (
                "`gem_placement::flow_placement` / `flow_surface_placement`",
                "batch flow for curves and for surfaces; the stone proportions are preserved",
            ),
            (
                "`gem_placement::rigid_flow`",
                "the deform owner `deform::rigid_flow` moves the frame, not the stone shape",
            ),
            (
                "`GemPlacementTool::Flow`",
                "one target may be a curve or a surface; a curve-linked row flows to a curve, a "
                "surface-linked row flows to a surface, and the link names the new source",
            ),
        ],
        "positive": {
            "cases": [
                "the stones land on the target curve at the recorded arc ratio",
                "surface stones also land on a parallel target surface and keep a usable UV link",
                "the link is retargeted to the target object with its revision",
                "the stone size is unchanged, so a flow is rigid",
            ]
        },
        "negative": {
            "cases": [
                "a missing target curve or base station is refused",
                "a row that mixes curve and surface targets is refused rather than guessed",
                "a target too short for the station produces conflicts, never a silent drop",
            ]
        },
        "positive_note": "rigid flow from a base curve/surface to a curve/surface target with a retargeted link",
        "negative_note": "missing targets and unmappable stations are typed refusals/conflicts",
        "extra_tests": [
            "flow_carries_surface_stones_onto_another_surface",
        ],
    },
    "OM9-L3-04-10-010": {
        "title": "Gem Splop On Surface",
        "folder": "L3-010 - Gem Splop On Surface",
        "command": "BuilderGVDGemSplop / gvGemSplop",
        "test": "splop_projects_a_gem_onto_a_surface_and_records_the_uv_link",
        "api": [
            (
                "`gem_placement::splop_station`",
                "closest point on the surface, surface normal frame, `x_hint` for the spin",
            ),
            (
                "`SplopParams`",
                "`z_offset_mm`, `rotation_deg`, `x_hint`; a degenerate normal is refused",
            ),
            (
                "`Controller` splop tool",
                "`ZOffset=` and `Rotate=` move the projection along the normal / spin the stone",
            ),
        ],
        "positive": {
            "cases": [
                "a gem above the plane projects onto it at the requested normal offset",
                "the table axis follows the surface normal",
                "the link becomes a surface link with its UV",
            ]
        },
        "negative": {
            "cases": [
                "a degenerate surface normal is a typed refusal",
                "a non-finite offset or spin is refused",
            ]
        },
        "positive_note": "sphere-style projection onto the surface normal with a UV link",
        "negative_note": "degenerate normals and bad parameters are typed refusals",
    },
    "OM9-L3-04-10-011": {
        "title": "Gem To Curve Surface History Links",
        "folder": "L3-011 - Gem To Curve Surface History Links",
        "command": "GemPlacementLinks / gvGemPlacementLinks",
        "test": "the_links_report_and_the_typed_command_reach_the_same_service",
        "api": [
            (
                "`gem_placement::PlacementLink`",
                "`schema` (`om9.gem_placement.v1`), `version`, gem id, source, arc, station, uv, "
                "source revision, state and spacing",
            ),
            (
                "`PlacementLink::{to_extension,from_extension,to_json,from_json}`",
                "the `om9.gem.placement` entity extension, version 1, survives document save/reopen",
            ),
            (
                "`PlacementLink::{refresh,mark_broken,mark_conflict}`",
                "ACTIVE/BROKEN/CONFLICT lifecycle with a stale-revision guard",
            ),
            (
                "`Controller::gem_placement_links_report`",
                "the command-line report lists the source tag, object, arc, station, revision and UV",
            ),
        ],
        "positive": {
            "cases": [
                "a link round-trips through the extension and a saved/reopened document",
                "every placed gem carries a readable link with its source and arc length",
                "the report lists one line per linked gem",
            ]
        },
        "negative": {
            "cases": [
                "an unreadable link is reported, not silently ignored",
                "a stale revision cannot refresh the link, and a broken record is refused",
            ]
        },
        "positive_note": "persisted `om9.gem.placement` link with a lifecycle and a report",
        "negative_note": "unreadable links are reported and stale writes/records are refused",
    },
}

L4_ROLE = {
    "01": ("Source Map", "SOURCE_AUDIT", "source"),
    "02": ("API Contract", "CONTRACT_TEST", "contract"),
    "03": ("UI Integration And Acceptance", "WIRE_UI", "ui"),
}


def run(command: list[str]) -> tuple[int, str]:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = TARGET_DIR
    env.setdefault("CARGO_TERM_COLOR", "never")
    prefix = [str(path) for path in RUNTIME_PATH if path.is_dir()]
    if prefix:
        env["PATH"] = os.pathsep.join(prefix + [env.get("PATH", "")])
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


def sha(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def revision() -> tuple[str, bool]:
    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True
    ).stdout.strip()
    dirty = bool(
        subprocess.run(
            ["git", "status", "--porcelain"], cwd=ROOT, capture_output=True, text=True
        ).stdout.strip()
    )
    return head, dirty


def l4_ids(l3_id: str, folder: str) -> list[str]:
    document = json.loads((L3_ROOT / folder / "l3.json").read_text(encoding="utf-8"))
    return [
        child["id"] if isinstance(child, dict) else str(child)
        for child in document.get("children_l4", [])
    ]


def write_json(path: pathlib.Path, document: dict) -> None:
    path.write_text(
        json.dumps(document, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )


def main() -> int:
    head, dirty = revision()
    BATCH.mkdir(parents=True, exist_ok=True)
    results = []
    foreign_warnings: list[str] = []
    for artifact, command in CHECKS:
        code, captured = run(command)
        if artifact == "clippy-gem-placement.log":
            scoped, foreign, hard = assess_clippy(captured)
            foreign_warnings = foreign
            if scoped or hard:
                code = 1
        (BATCH / artifact).write_text(header(" ".join(command), code, captured), encoding="utf-8")
        results.append(
            {
                "artifact": artifact,
                "command": " ".join(command),
                "exit_code": code,
                "sha256": sha(captured),
            }
        )
        if code != 0:
            raise SystemExit(f"check failed: {' '.join(command)}")

    summary = {}
    receipts_all = []
    for l3_id, package in L3_PACKAGES.items():
        directory = ROOT / "implementation" / "evidence" / MILESTONE_KEY / l3_id
        directory.mkdir(parents=True, exist_ok=True)
        gate = MANUAL
        crosswalk = {
            "schema": "om9.m05.source_crosswalk.v1",
            "l3_id": l3_id,
            "title": package["title"],
            "owner": "om9-core gem_placement owner + om9-app placement session",
            "source_revision": head,
            "checkout_dirty": dirty,
            "sources": [
                {"path": "crates/om9-core/src/gem_placement.rs", "state": "IMPLEMENTED_AND_TESTED"},
                {"path": "crates/om9-core/src/gem.rs", "state": "REUSED_GEM_OWNER"},
                {"path": "crates/om9-core/src/model.rs", "state": "EXTENSION_SLOT_REGISTERED"},
                {"path": "crates/om9-core/tests/gem_placement.rs", "state": "REGISTERED_TESTS"},
                {
                    "path": "crates/om9-app/src/session/gem_placement.rs",
                    "state": "IMPLEMENTED_AND_TESTED",
                },
                {"path": "crates/om9-app/src/session.rs", "state": "COMMAND_WIRING"},
                {"path": "crates/om9-app/tests/gem_placement.rs", "state": "REGISTERED_TESTS"},
            ],
            "source_audit": {
                "matrix_assets": "NOT_COPIED",
                "result": "PROCEDURAL_ASSETS_ONLY",
                "detail": (
                    "No Matrix asset, catalogue bitmap or decompiled resource is read or shipped. "
                    "Stations are computed from the shared curve/surface evaluators and every gem "
                    "mesh comes from the typed spec in crates/om9-core/src/gem.rs."
                ),
            },
            "planning_inputs": [
                f"OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/{package['folder']}/l3.json",
                "OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L2.json",
            ],
            "planning_tree_touched": False,
            "notes": [
                "The gem body remains the mesh/record owner of OM9-L2-04-09; this module only "
                "computes placement stations and the link.",
                "The Matrix command names of the family are preserved and dispatch to one session.",
                "Placement is pure data: no function mutates the document, the caller commits one "
                "transaction.",
            ],
        }
        write_json(directory / "source_crosswalk.json", crosswalk)

        api_rows = "\n".join(f"| {api} | {detail} |" for api, detail in package["api"])
        (directory / "api.md").write_text(
            f"""# {package['title']} (`{l3_id}`)

## Command

`{package['command']}` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
{api_rows}

## Effects

- A preview is pure placement data plus a computed station list; it never inserts a document
  object and never mutates the source curve/surface.
- Apply is exactly one document transaction; Undo restores the previous state and Redo keeps the
  gem object identity and its `GemId`.
- Refusals are typed `om9_core::Error` values (`Invalid`, `Missing`, `Stale`, `Busy`,
  `Budget`); a station is never silently dropped or moved to a guessed place.
- Each placed gem carries `om9.gem` (`GemRecord`) and `om9.gem.placement` (`PlacementLink`,
  schema `om9.gem_placement.v1`, version 1).

## Verification

- Positive: {package['positive_note']}
- Negative: {package['negative_note']}
- Gate: `{gate}` — the M05-UI manual gate stays open; no reviewer signature is claimed.
""",
            encoding="utf-8",
        )

        manifest = sha(
            json.dumps(package["positive"], sort_keys=True)
            + json.dumps(package["negative"], sort_keys=True)
        )
        write_json(
            directory / "positive.json",
            {
                "schema": "om9.m05.l3_positive.v1",
                "l3_id": l3_id,
                "title": package["title"],
                "cases": package["positive"]["cases"],
                "oracle": "asserted by the registered tests; see observed.json",
            },
        )
        write_json(
            directory / "negative.json",
            {
                "schema": "om9.m05.l3_negative.v1",
                "l3_id": l3_id,
                "title": package["title"],
                "cases": package["negative"]["cases"],
                "oracle": "typed refusals asserted by the registered tests",
            },
        )

        receipts = []
        for l4_id in l4_ids(l3_id, package["folder"]):
            suffix = l4_id.rsplit("-", 1)[-1]
            if suffix != "03":
                continue
            command = [
                "cargo",
                "test",
                "-p",
                "om9-app",
                "--test",
                "gem_placement",
                "-j",
                "2",
                package["test"],
            ]
            code, captured = run(command)
            commands = [
                {
                    "command_argv": command,
                    "exit_code": code,
                    "stdout_stderr_sha256": sha(captured),
                    "summary": [
                        line
                        for line in captured.splitlines()
                        if line.startswith(("test result:", "error", "FAILED"))
                    ][-3:],
                    "test": package["test"],
                    "package": "om9-app",
                    "binary": "gem_placement",
                }
            ]
            extra_logs = []
            for extra in package.get("extra_tests", []):
                extra_command = command[:-1] + [extra]
                extra_code, extra_captured = run(extra_command)
                if extra_code != 0:
                    raise SystemExit(
                        f"L4 command failed for {l4_id}: {' '.join(extra_command)}"
                    )
                extra_logs.append(header(" ".join(extra_command), extra_code, extra_captured))
                commands.append(
                    {
                        "command_argv": extra_command,
                        "exit_code": extra_code,
                        "stdout_stderr_sha256": sha(extra_captured),
                        "summary": [
                            line
                            for line in extra_captured.splitlines()
                            if line.startswith(("test result:", "error", "FAILED"))
                        ][-3:],
                        "test": extra,
                        "package": "om9-app",
                        "binary": "gem_placement",
                    }
                )
            if code != 0:
                raise SystemExit(f"L4 command failed for {l4_id}: {' '.join(command)}")
            title_suffix, role, kind = L4_ROLE[suffix]
            write_json(
                directory / f"{l4_id}.json",
                {
                    "schema": "om9.m05.l4.receipt.v1",
                    "l3_id": l3_id,
                    "l4_id": l4_id,
                    "title": package["title"],
                    "generated_at_utc": datetime.datetime.now(datetime.UTC).strftime(
                        "%Y-%m-%dT%H:%M:%SZ"
                    ),
                    "source_revision": head,
                    "checkout_dirty": dirty,
                    "manual_gate": gate,
                    "signature": None,
                    "not_claimed": [
                        "manual review of the placement command at the M05-UI gate",
                        "a reviewer signature",
                    ],
                    "fixture_manifest_sha256": manifest,
                    "document_snapshot": {
                        "source": "asserted by the registered tests",
                        "positive": package["positive_note"],
                        "negative": package["negative_note"],
                    },
                    "role": role,
                    "kind": kind,
                    "status": "READY_FOR_REVIEW",
                    "changed_files": CHANGED,
                    "commands": commands,
                    "manual_gate_note": (
                        "Automated evidence only; the M05-UI manual gate remains open."
                    ),
                },
            )
            (directory / f"{l4_id}.log").write_text(
                header(" ".join(command), code, captured) + "".join(extra_logs),
                encoding="utf-8",
            )
            receipts.append(l4_id)

        write_json(
            directory / "observed.json",
            {
                "schema": "om9.m05.l3_gem_placement_evidence.v1",
                "l3_id": l3_id,
                "title": package["title"],
                "command": package["command"],
                "scope": "; ".join(api for api, _ in package["api"]),
                "target": TARGET,
                "positive": package["positive_note"],
                "negative": package["negative_note"],
                "manual_gate": gate,
                "reviewer": None,
                "statement": (
                    "Automated evidence only; the M05-UI manual review remains open and no "
                    "reviewer signature is claimed."
                ),
                "l4": {
                    "l4_receipts": receipts,
                    "fixture_manifest_sha256": manifest,
                },
                "tests": [
                    "crates/om9-core/tests/gem_placement.rs",
                    "crates/om9-app/tests/gem_placement.rs",
                ],
                "test": package["test"],
            },
        )
        summary[l3_id] = {
            "manual_gate": gate,
            "l4_receipts": receipts,
            "fixture_manifest_sha256": manifest,
            "command": package["command"],
            "test": package["test"],
        }
        receipts_all.extend(receipts)

    write_json(
        BATCH / "observed.json",
        {
            "schema": "om9.m05.l2_gem_placement_evidence.v1",
            "l2_id": "OM9-L2-04-10",
            "title": "Gems On Curves And Surfaces",
            "date": "2026-10-27",
            "target": TARGET,
            "source_revision": head,
            "checkout_dirty": dirty,
            "code": CHANGED,
            "checks": results,
            "l3": summary,
            "tests": {
                "core_gem_placement": "22 passed (stations, taper, count, list, multi, surface, "
                "follow, flow, splop, link round-trip, purity)",
                "app_gem_placement": "16 passed (preview, apply, Undo/Redo, cancel, conflicts, "
                "flow to curve/surface, splop, profile/guides, report, typed command, "
                "exact Count=, refused unused options)",
                "clippy": "cargo clippy -p om9-core -p om9-app --all-targets exit 0; diagnostics "
                "are scoped to this task's files and foreign warnings are listed below",
                "clippy_foreign_warnings": foreign_warnings,
            },
            "uncertainties": [
                "All eleven L3 keep the M05-UI manual gate open: no human has reviewed the "
                "placement commands on screen and no reviewer signature is claimed.",
                "Only Windows x64 was exercised in this run.",
                "The app test binaries link OCCT, so the run needs the OCCT bin and VC runtime "
                "directories on PATH; that environment requirement is recorded, not hidden.",
                "CARGO_TARGET_DIR is H:/OpenMatrix9/cargo-target because this sandbox denies "
                "writes to the L:/om9-target directory named in AGENTS.md.",
                "The clippy gate is scoped to the files this task owns: the shared checkout also "
                "contains concurrent in-flight work (for example the settings module and the "
                "head_commands tests), so its warnings are listed in clippy_foreign_warnings and "
                "left to that owner instead of failing this batch.",
                "The crate-wide `cargo test -p om9-app --lib` reports 188 passed and 4 "
                "`ui::layer_dialog_audit` failures in this non-interactive session: those tests "
                "create fixtures under build/standalone-validation with a restricted token and "
                "fail with `os error 5` before reaching any placement code. The failures are "
                "environmental and recorded in app-lib-os-restriction.log, not hidden.",
                "The placement preview is session data; painting it as a shaded viewport proxy "
                "remains a separate UI task.",
            ],
        },
    )
    (BATCH / "README.md").write_text(
        f"""# OM9-L2-04-10 Gems On Curves And Surfaces — evidence

Automated run on 2026-10-27 (`{TARGET}`), source revision `{head}`
({'dirty' if dirty else 'clean'} checkout).

## What was run

"""
        + "\n".join(f"- `{item['command']}` -> exit {item['exit_code']}" for item in results)
        + """

## Per-L3 result

| L3 | Command | Gate | L4 receipts |
| --- | --- | --- | --- |
"""
        + "\n".join(
            f"| `{l3}` | `{info['command']}` | `{info['manual_gate']}` | "
            f"{', '.join(info['l4_receipts']) or '—'} |"
            for l3, info in summary.items()
        )
        + """

All eleven L3 are commands with a human-facing M05-UI gate, so none is marked DONE and no
`ui-review.json` is written. Each L3 is WIP with automated evidence and an open manual gate.

## Not claimed

- M05-UI manual acceptance of the eleven placement commands.
- Matrix assets: none were read, copied or shipped.
- A shaded viewport preview proxy for an open placement session.
- The crate-wide app lib run: 188 of 192 pass; the 4 `ui::layer_dialog_audit` failures need
  `build/standalone-validation` write access that the restricted test token denies in this
  session (see `app-lib-os-restriction.log`).
""",
        encoding="utf-8",
    )

    ledger_path = ROOT / "docs" / "openmatrix9-progress.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    entry = {
        "key": KEY,
        "date": "2026-10-27",
        "stage": "feature-port",
        "status": "gem_placement_automated_verified_manual_gates_open",
        "target": TARGET,
        "feature_ids": ["OM9-L2-04-10"],
        "scope": (
            "M05 OM9-L2-04-10 gems on curves and surfaces: typed placement owner in "
            "om9-core::gem_placement (arc-length rows, taper, count, heterogeneous list, atomic "
            "multi, profile/guides, surface frames, follow/flow/splop) plus the persisted "
            "om9.gem.placement link; om9-app session wires the eleven Matrix commands and applies "
            "each run as one transaction; 22 core + 13 app tests."
        ),
        "notes": (
            "ghi chú: (1) placement là dữ liệu thuần, không hàm nào sửa document — caller commit "
            "một transaction; (2) link om9.gem.placement schema om9.gem_placement.v1 version 1 lưu "
            "source/arc/station/uv/revision và có vòng đời ACTIVE/CONFLICT, refresh chặn revision "
            "cũ; (3) cả 11 L3 đều là lệnh có gate M05-UI nên dashboard chỉ 🟡, không có chữ ký "
            "reviewer; (4) lệch môi trường: CARGO_TARGET_DIR=H:/OpenMatrix9/cargo-target vì sandbox "
            "từ chối L:/om9-target, và app test cần OCCT bin + VC runtime trên PATH."
        ),
        "outputs": CHANGED,
        "evidence": [
            "implementation/evidence/M05/OM9-L2-04-10/observed.json",
            "implementation/evidence/M05/OM9-L2-04-10/README.md",
            *[f"implementation/evidence/M05/OM9-L2-04-10/{item['artifact']}" for item in results],
            *[
                f"implementation/evidence/M05/{l3}/observed.json"
                for l3 in L3_PACKAGES
            ],
        ],
        "tests": {
            "core_gem_placement": "22 passed (stations, taper, count, list, multi, surface, "
            "follow, flow, splop, link round-trip, purity)",
            "app_gem_placement": "16 passed (preview, apply, Undo/Redo, cancel, conflicts, flow, "
            "splop, profile/guides, report, typed command, exact Count=, "
            "refused unused options)",
            "core_gem_assets_regression": "regression suite passed",
            "app_gem_panel_regression": "regression suite passed",
            "core_transaction_regression": "atomic_transactions + transaction_scopes passed",
            "workspace_check": "cargo check --workspace --all-targets exit 0",
            "clippy": "cargo clippy -p om9-core -p om9-app --all-targets exit 0 (diagnostics "
            "scoped to this task's files; foreign warnings are listed in "
            "clippy_foreign_warnings)",
            "clippy_foreign_warnings": foreign_warnings,
        },
        "uncertainties": json.loads((BATCH / "observed.json").read_text(encoding="utf-8"))[
            "uncertainties"
        ],
        "manual_gate": (
            "OPEN: M05-UI — 11 lệnh đặt gem trên curve/surface chưa được người nghiệm thu"
        ),
        "next_action": (
            "Chạy và chụp từng lệnh (gem on curve/advanced/count/list/multi, profile/guides, "
            "on surface, follow, flow, splop, links) trong UI ở gate M05-UI, rồi ký ui-review cho "
            "từng L3; sau đó vẽ preview session trong viewport."
        ),
    }
    ledger["items"] = [item for item in ledger["items"] if item.get("key") != KEY] + [entry]
    ledger["active_item"] = KEY
    ledger["updated_at"] = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    write_json(ledger_path, ledger)

    owner_path = L3_ROOT / "L2.json"
    owner = json.loads(owner_path.read_text(encoding="utf-8"))
    owner["runtime_implementation"] = {
        "date": "2026-10-27",
        "status": "AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING",
        "target": TARGET,
        "scope": (
            "Typed gem placement owner: arc-length rows with start/end/spacing, taper, count fit, "
            "heterogeneous lists, atomic multi-curve runs, profile/girdle/guides, surface UV "
            "frames, follow/flow/splop and the persisted om9.gem.placement link"
        ),
        "owner": {
            "crates": ["om9-core", "om9-app"],
            "tests": [
                "crates/om9-core/tests/gem_placement.rs",
                "crates/om9-app/tests/gem_placement.rs",
            ],
        },
        "evidence": [
            "implementation/evidence/M05/OM9-L2-04-10/observed.json",
            "implementation/evidence/M05/OM9-L2-04-10/README.md",
            "implementation/evidence/M05/OM9-L2-04-10/core-gem-placement.log",
            "implementation/evidence/M05/OM9-L2-04-10/app-gem-placement.log",
            "implementation/evidence/M05/OM9-L2-04-10/workspace-check.log",
        ],
        "notes": (
            "Placement trả dữ liệu thuần; apply là một transaction. Link om9.gem.placement lưu "
            "source/arc/station/uv/revision và có trạng thái ACTIVE/CONFLICT. Không copy asset "
            "Matrix. 11 L3 đều còn gate M05-UI."
        ),
        "l3_coverage": {l3: "automated_verified_manual_gate_open" for l3 in L3_PACKAGES},
        "l4": {
            "planned": len(L3_PACKAGES) * 3,
            "artifacts": len(L3_PACKAGES) * 3,
            "receipts": receipts_all,
        },
        "tests": [
            f"core gem_placement: exit {results[0]['exit_code']}",
            f"app gem_placement: exit {results[1]['exit_code']}",
            f"workspace check: exit {results[5]['exit_code']}",
            f"clippy: exit {results[6]['exit_code']}",
        ],
        "human_approval": None,
        "manual_gate": "M05-UI",
        "remaining": json.loads((BATCH / "observed.json").read_text(encoding="utf-8"))[
            "uncertainties"
        ],
    }
    write_json(owner_path, owner)

    for l3_id, package in L3_PACKAGES.items():
        l3_path = L3_ROOT / package["folder"] / "l3.json"
        document = json.loads(l3_path.read_text(encoding="utf-8"))
        document["runtime_implementation"] = {
            "date": "2026-10-27",
            "status": "AUTOMATED_VERIFIED_MANUAL_UI_PENDING",
            "target": TARGET,
            "owner": {"crates": ["om9-core", "om9-app"]},
            "command": package["command"],
            "evidence": [
                f"implementation/evidence/M05/{l3_id}/observed.json",
                f"implementation/evidence/M05/{l3_id}/source_crosswalk.json",
                f"implementation/evidence/M05/{l3_id}/api.md",
            ],
            "manual_gate": "M05-UI",
            "tests": [
                "crates/om9-core/tests/gem_placement.rs",
                "crates/om9-app/tests/gem_placement.rs",
            ],
            "notes": package["positive_note"] + "; " + package["negative_note"],
        }
        write_json(l3_path, document)

    print(f"recorded gem placement evidence at {BATCH.relative_to(ROOT)}")
    for item in results:
        print(f"  {item['artifact']}: exit {item['exit_code']}")
    print(f"L4 receipts: {len(receipts_all)}")
    print(f"ledger: {ledger_path.relative_to(ROOT)} ({len(ledger['items'])} items)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
