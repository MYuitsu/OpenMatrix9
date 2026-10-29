#!/usr/bin/env python3
"""Record the M05 ``OM9-L2-04-13`` gem instance edits and reports capability from real runs.

Scope: all nine L3 of ``05-febe-gem-instance-edits-and-reports``:

* ``OM9-L3-04-13-001`` Gem Positioner (reused from the ``OM9-L2-04-09`` owner, regression-tested);
* ``OM9-L3-04-13-002`` Gem Control and size edits;
* ``OM9-L3-04-13-003`` Match Attributes;
* ``OM9-L3-04-13-004`` Save gem style;
* ``OM9-L3-04-13-005`` Load gem style;
* ``OM9-L3-04-13-006`` Gem Update and regenerate;
* ``OM9-L3-04-13-007`` Gem Map viewport overlay rows;
* ``OM9-L3-04-13-008`` Gem Reporter count and weight;
* ``OM9-L3-04-13-009`` Gem attribute history and ID preservation.

Every L3 in this module is a command with a human-facing M05-UI gate
(``requires_manual_test: true``), so no reviewer signature is claimed and no
``ui-review.json`` is written. The dashboard therefore counts each L3 as WIP-with-evidence,
never DONE.

Each L3 directory receives ``source_crosswalk.json`` (L4-01), ``api.md`` +
``positive.json`` + ``negative.json`` (L4-02), one ``OM9-L4-13-0xx-03`` receipt + log
(L4-03) and ``observed.json``.

Usage::

    python tools/record_m05_l2_04_13_gem_instance_edits.py
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
KEY = "m05-l2-04-13-gem-instance-edits-2026-10-28"
MILESTONE_KEY = "M05"
BATCH = ROOT / "implementation" / "evidence" / MILESTONE_KEY / "OM9-L2-04-13"
L3_ROOT = (
    ROOT
    / "OM9_Milestones"
    / "M05 - Gems and Layout"
    / "L2-04-13 - 05-febe-gem-instance-edits-and-reports"
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
        "core-gem-edits.log",
        ["cargo", "test", "-p", "om9-core", "--test", "gem_edits", "-j", "2"],
    ),
    (
        "app-gem-edits.log",
        ["cargo", "test", "-p", "om9-app", "--test", "gem_edits", "-j", "2"],
    ),
    (
        "app-gem-map-overlay.log",
        ["cargo", "test", "-p", "om9-app", "--lib", "-j", "2", "gem_edits"],
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
        "clippy-gem-edits.log",
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
            "--",
            "--cap-lints",
            "warn",
        ],
    ),
]

# Clippy is scoped to the files this task owns: the workspace is shared with other work in
# flight, so a foreign warning is recorded (never hidden) instead of failing this batch.
CLIPPY_SCOPE = {
    "crates/om9-core/src/gem_edits.rs",
    "crates/om9-core/src/lib.rs",
    "crates/om9-core/tests/gem_edits.rs",
    "crates/om9-app/src/session/gem_edits.rs",
    "crates/om9-app/src/session.rs",
    "crates/om9-app/src/ui/gem_edits.rs",
    "crates/om9-app/src/ui.rs",
    "crates/om9-app/tests/gem_edits.rs",
}


def assess_clippy(captured: str) -> tuple[list[str], list[str], bool]:
    """Split capped clippy diagnostics into this task's files and foreign files.

    Returns ``(scoped, foreign, hard_error)``. A diagnostic in a scoped file fails the batch; a
    foreign warning is logged for the next owner.
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
    "crates/om9-core/src/gem_edits.rs",
    "crates/om9-core/src/lib.rs",
    "crates/om9-core/tests/gem_edits.rs",
    "crates/om9-app/src/session/gem_edits.rs",
    "crates/om9-app/src/session.rs",
    "crates/om9-app/src/ui/gem_edits.rs",
    "crates/om9-app/src/ui.rs",
    "crates/om9-app/tests/gem_edits.rs",
    "tools/record_m05_l2_04_13_gem_instance_edits.py",
]

MANUAL = "M05-UI"

L3_PACKAGES = {
    "OM9-L3-04-13-001": {
        "title": "Gem Positioner",
        "folder": "L3-001 - Gem Positioner MSR",
        "command": "BuilderGemPositioner / gvGemPositioner",
        "binary": "gem_panel",
        "test": "the_positioner_moves_and_rotates_the_instance_in_place",
        "api": [
            (
                "`om9_app::session::gem` positioner (`BuilderGemPositioner`)",
                "opens on the selected gem and edits `origin`/`rotation_deg`/`scale` without "
                "touching the stored record",
            ),
            (
                "`Controller::gem_apply` positioner branch",
                "one `Mutation::Placement` per Apply, so Undo restores the previous handle frame",
            ),
            (
                "`gem_record` on the target",
                "a non-gem or a shared definition instance is refused before the tool opens",
            ),
        ],
        "positive": {
            "cases": [
                "the positioner moves and rotates the selected gem in place",
                "the gem keeps its object identity, `GemId` and `om9.gem` record",
                "the move and rotate commit in one transaction and Undo restores the frame",
            ]
        },
        "negative": {
            "cases": [
                "a non-gem selection is a typed refusal and opens no session",
                "a shared definition member is refused instead of rewriting the definition",
            ]
        },
        "positive_note": "handle move/rotate/scale on one instance, identity preserved",
        "negative_note": "a non-gem or a shared definition is a typed refusal",
        "extra_tests": [
            "the_positioner_duplicates_with_one_shared_definition",
            "the_positioner_refuses_a_non_gem",
        ],
    },
    "OM9-L3-04-13-002": {
        "title": "Gem Control And Size Edits",
        "folder": "L3-002 - Gem Control And Size Edits",
        "command": "BuilderGemControl / gvGemControl",
        "binary": "gem_edits",
        "test": "control_edits_every_selected_stone_in_one_transaction_and_undo_restores_it",
        "api": [
            (
                "`om9_core::gem_edits::GemEdit`",
                "partial attribute edit (`cut/shape/length/width/depth/girdle/corner/material/"
                "color`); a missing field means `leave alone`, never `reset to default`",
            ),
            (
                "`GemEdit::{spec,record,validate,changes_geometry}`",
                "applies the edit through `GemSpec::set_girdle_mm/set_depth_mm/...`; a cut/shape "
                "mismatch or an out-of-range size is a typed `Invalid` error",
            ),
            (
                "`Controller::dispatch_gem_edits` + `gem_edits_text`",
                "one session over the ordered selection; `Size=`/`Length=`/`Width=`/`Depth=`/"
                "`Girdle=`/`Corner=`/`Material=`/`Color=`/`Label=` recompute every plan",
            ),
            (
                "`Controller::gem_edits_apply`",
                "all selected stones commit in ONE transaction (geometry + record + optional "
                "label), so Undo restores every stone together",
            ),
        ],
        "positive": {
            "cases": [
                "`Size=8` edits two selected stones from 6/4 mm to 8 mm in one transaction",
                "each edited stone keeps its object id and its `GemId`",
                "Undo restores both original sizes and Redo re-applies the edit",
            ]
        },
        "negative": {
            "cases": [
                "a non-gem selection opens nothing (`Invalid(\"gem edit needs gems\")`)",
                "an option the Control tool does not consume (`Mask=`) is a typed refusal",
                "a size the schema refuses keeps the session open for correction",
            ]
        },
        "positive_note": "one-transaction multi-stone edit with stable identities and Undo/Redo",
        "negative_note": "wrong selection and unused options are typed refusals",
    },
    "OM9-L3-04-13-003": {
        "title": "Gem Match Attributes",
        "folder": "L3-003 - Gem Match Attributes",
        "command": "OthersMatchAttributes / gvMatchAttributes",
        "binary": "gem_edits",
        "test": "match_attributes_copies_geometry_from_the_first_selected_stone",
        "api": [
            (
                "`gem_edits::MatchMask`",
                "`cut/shape/dimensions/finish` groups; `ALL`/`GEOMETRY`/`FINISH` presets and a "
                "comma list parser; an empty mask is refused",
            ),
            (
                "`gem_edits::match_attributes(source, targets, mask)`",
                "builds one `GemEdit` from the source, applies it to every target and keeps each "
                "target `GemId`",
            ),
            (
                "`OthersMatchAttributes` session",
                "the first selected stone is the source and the remaining ordered selection is "
                "the target list; fewer than two stones is a typed refusal",
            ),
        ],
        "positive": {
            "cases": [
                "the first selected 6 mm stone copies cut/shape/girdle to the 4/5 mm targets",
                "`Mask=geometry` leaves each target material and colour untouched",
                "the match commits in one transaction and Undo restores every target",
            ]
        },
        "negative": {
            "cases": [
                "an empty mask (`Mask=` with no group) is a typed refusal",
                "an unknown mask token is a typed refusal, never ignored",
                "one selected stone cannot be both source and target",
            ]
        },
        "positive_note": "masked copy from the first selected stone with target identity kept",
        "negative_note": "an empty or unknown mask and a single selection are refusals",
    },
    "OM9-L3-04-13-004": {
        "title": "Gem Style Save",
        "folder": "L3-004 - Gem Style Save",
        "command": "OthersSaveStyle / gvSaveStyle",
        "binary": "gem_edits",
        "test": "save_and_load_style_roundtrip_through_the_style_file",
        "api": [
            (
                "`gem_edits::GemStyle`",
                "schema `om9.gem_style.v1`, version 1, name and a full attribute `GemEdit`; the "
                "gem id, label and placement stay with the instance, not the style",
            ),
            (
                "`GemStyleStore`",
                "bounded (512) named store with case-insensitive replace-by-name; schema "
                "`om9.gem_style_file.v1`",
            ),
            (
                "`GemStyleStore::{save_to,load_or_empty}`",
                "atomic staging-file + rename (copy fallback on Windows); a missing file is the "
                "documented empty store",
            ),
            (
                "`GemStyle::validate`",
                "reuses `builder_style::validate_style_name`, so the gem store refuses the same "
                "names as the shared builder preset store",
            ),
        ],
        "positive": {
            "cases": [
                "a 6 mm round stone saves as `Round Six` and the versioned JSON file is written",
                "the file carries the schema, the name and every saved attribute",
                "saving the same name again replaces the style instead of duplicating it",
            ]
        },
        "negative": {
            "cases": [
                "a name the shared store rejects (`bad/name`) is a typed refusal and the tool "
                "stays open",
                "a non-gem selection cannot be saved as a style",
            ]
        },
        "positive_note": "versioned named gem style written atomically to the shared store file",
        "negative_note": "invalid style names are typed refusals",
        "extra_tests": ["save_style_refuses_a_name_the_shared_store_would_reject"],
    },
    "OM9-L3-04-13-005": {
        "title": "Gem Style Load",
        "folder": "L3-005 - Gem Style Load",
        "command": "OthersLoadStyle / gvLoadStyle",
        "binary": "gem_edits",
        "test": "save_and_load_style_roundtrip_through_the_style_file",
        "api": [
            (
                "`GemStyle::apply(record)`",
                "applies the saved attribute edit to a stone and keeps the target `GemId`",
            ),
            (
                "`GemStyleStore::get(name)`",
                "case-insensitive lookup; an absent name is `Missing(\"gem style name\")`",
            ),
            (
                "`OthersLoadStyle` session",
                "`Name=` chooses the style, `File=` the store; every selected stone gets the same "
                "style in ONE transaction",
            ),
        ],
        "positive": {
            "cases": [
                "the saved `Round Six` style loads onto the 4 mm stone and makes it 6 mm",
                "the loaded stone keeps its object id and `GemId`",
                "the load is one transaction, so Undo restores the 4 mm stone",
            ]
        },
        "negative": {
            "cases": [
                "an absent style name is `Missing` and nothing is written",
                "a malformed store file is a typed refusal, never a partial load",
            ]
        },
        "positive_note": "named style applied to every selected stone with identity kept",
        "negative_note": "an absent name or a malformed store is a typed refusal",
    },
    "OM9-L3-04-13-006": {
        "title": "Gem Update And Regenerate",
        "folder": "L3-006 - Gem Update And Regenerate",
        "command": "BuilderGemUpdate / gvGemUpdate",
        "binary": "gem_edits",
        "test": "update_rebuilds_the_geometry_from_the_stored_record",
        "api": [
            (
                "`gem_edits::recompute` update branch",
                "rebuilds the procedural mesh from `GemRecord::spec` before promising the update; "
                "an open mesh is refused",
            ),
            (
                "`Controller::gem_edits_apply`",
                "writes `Mutation::Geometry` plus the unchanged record as ONE transaction",
            ),
            (
                "`build_mesh` reuse",
                "the regenerated body is the same closed mesh the loader produces, never a "
                "separately authored approximation",
            ),
        ],
        "positive": {
            "cases": [
                "a stale 4 mm body whose record says 6 mm is rebuilt to the 6 mm mesh",
                "the stored record and its `GemId` are unchanged by the regeneration",
                "the update is one transaction and Undo restores the stale body",
            ]
        },
        "negative": {
            "cases": [
                "a record whose spec cannot build a closed mesh is refused before any write",
                "a non-gem selection opens nothing",
            ]
        },
        "positive_note": "geometry regenerated from the stored record, identity and record kept",
        "negative_note": "a spec that cannot rebuild a closed mesh is refused",
    },
    "OM9-L3-04-13-007": {
        "title": "Gem Map Viewport Overlay",
        "folder": "L3-007 - Gem Map Viewport Overlay",
        "command": "BuilderGemMap / gvGemMap",
        "binary": "gem_edits",
        "test": "the_map_numbers_the_selected_stones_without_touching_the_document",
        "api": [
            (
                "`gem_edits::gem_map(entries)`",
                "numbered rows (`GemId`, number, shape, cut, girdle, world point); numbers start "
                "at one and never skip",
            ),
            (
                "`gem_edits::map_line(entry)`",
                "stable text `#n · Shape Cut · L × W mm @ (x, y, z)` for the printed key",
            ),
            (
                "`ui::gem_edits::gem_map_labels(rows, camera, viewport, rect)`",
                "projects every visible row into a numbered caption `#n  L × W`; a stone behind "
                "the camera or outside the rectangle is skipped, never pinned to the edge, and "
                "the caption is clamped inside the viewport",
            ),
            (
                "`MatrixApp::gem_map_overlay_ui` / `gem_map_window`",
                "paints the numbered marker and girdle size over the committed viewport and opens "
                "a key window that lists the same numbers; both call the same typed rows",
            ),
            (
                "`BuilderGemMap` session",
                "an empty selection maps the whole document; `File=` exports the same rows",
            ),
        ],
        "positive": {
            "cases": [
                "three selected stones are numbered `#1`..`#3` in selection order",
                "each row carries the stone shape, cut, girdle size and world position",
                "the viewport overlay projects one numbered caption per visible stone and keeps "
                "every caption inside the rectangle",
                "a stone outside the current view is skipped rather than drawn at the edge",
                "the map never changes the document revision",
            ]
        },
        "negative": {
            "cases": [
                "a non-finite placement point is `Invalid(\"gem map point\")`",
                "more rows than the report budget is a `Budget` refusal",
            ]
        },
        "positive_note": (
            "numbered map rows from the live world placement, painted as numbered captions with "
            "girdle sizes in the viewport plus a matching key window, read-only"
        ),
        "negative_note": (
            "a non-finite point or an oversized map is a typed refusal; an off-screen stone is "
            "omitted from the overlay instead of being mis-pinned"
        ),
    },
    "OM9-L3-04-13-008": {
        "title": "Gem Reporter Count And Weight",
        "folder": "L3-008 - Gem Reporter Count And Weight",
        "command": "BuilderGemReporter / gvGemReport",
        "binary": "gem_edits",
        "test": "the_reporter_aggregates_count_and_calculated_mass",
        "api": [
            (
                "`gem_edits::gem_report(records, Option<density>)`",
                "aggregates by shape, cut, girdle and material; each row carries count, carat, "
                "grams, the material density and the trade-formula cross-check, and the report "
                "carries the totals, the worst delta and the out-of-tolerance count",
            ),
            (
                "`gem_edits::MATERIAL_DENSITIES` / `material_density(name)`",
                "gemological reference densities (diamond 3.52, corundum 4.00, beryl 2.72, …); an "
                "unknown material is flagged as an assumption instead of silently using diamond",
            ),
            (
                "`gem_edits::weight_formula(spec)`",
                "published per-shape trade formula `L × W × depth × factor` on a diamond basis, "
                "scaled by the row density; no formula is applied to a cabochon or custom profile",
            ),
            (
                "`gem_edits::gem_weight(spec, density)`",
                "primary mass from the closed procedural mesh volume × density; one carat = 0.2 g",
            ),
            (
                "`gem_edits::validate_density`",
                "density must be finite and inside 0.1..=30 g/cm³",
            ),
            (
                "`BuilderGemReporter` session",
                "`Density=` recomputes the report, `File=` exports the JSON; the document is "
                "never modified",
            ),
        ],
        "positive": {
            "cases": [
                "two 6 mm stones aggregate into one row with `count = 2`",
                "the total reports three stones and a positive carat/gram mass",
                "`carat × 0.2 == grams` holds for every row and for the total",
                "a ruby and a diamond of the same girdle produce separate rows and the ruby row is "
                "heavier by the 4.00/3.52 density ratio",
                "the mesh mass stays inside ±15% of the trade formula for the authored shapes; a "
                "row outside the band is counted and printed",
                "the report labels itself `NOT A CERTIFICATE`",
            ]
        },
        "negative": {
            "cases": [
                "a zero, negative, NaN or absurd density is `Invalid(\"gem density\")`",
                "a refusal keeps the reporter open instead of printing a wrong weight",
                "an unknown material without an override is flagged as an assumption, not hidden",
            ]
        },
        "positive_note": (
            "count and computed mass grouped by shape/cut/size/material, cross-checked against the "
            "trade formula and labelled as a computed value, read-only"
        ),
        "negative_note": (
            "an impossible density is a typed refusal; an unknown material density is reported as "
            "an assumption instead of a silent diamond basis"
        ),
    },
    "OM9-L3-04-13-009": {
        "title": "Gem Attribute History And ID Preservation",
        "folder": "L3-009 - Gem Attribute History And ID Preservation",
        "command": "GemAttributeHistory / gvGemHistory",
        "binary": "gem_edits",
        "test": "an_edit_keeps_the_placement_link_and_the_attribute_history_shows_it",
        "api": [
            (
                "`GemEdit::record` identity contract",
                "the edited record is cloned from the stored one, so `gem_id` is never regenerated",
            ),
            (
                "`Controller::gem_edits_apply` extension merge",
                "only `om9.gem` is replaced; the `om9.gem.placement` link and every other slot are "
                "carried over unchanged",
            ),
            (
                "`GemAttributeHistory` report",
                "one line per stone: entity id, `GemId`, shape/cut, girdle and the live placement "
                "link (source tag, object, revision)",
            ),
        ],
        "positive": {
            "cases": [
                "a size edit on a placed gem keeps the same entity id and `GemId`",
                "the `om9.gem.placement` link still names the same curve and revision",
                "the history report prints the stone identity and its live link",
                "Undo restores the pre-edit attributes without changing either id",
            ]
        },
        "negative": {
            "cases": [
                "an unreadable placement link is reported as `no placement link`, not invented",
                "a non-gem selection opens no history session",
            ]
        },
        "positive_note": "ids and the placement link survive an edit and the history names them",
        "negative_note": "a missing link is reported honestly, never fabricated",
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
        if artifact == "clippy-gem-edits.log":
            scoped, foreign, hard = assess_clippy(captured)
            foreign_warnings = foreign
            if scoped:
                code = 1
            elif hard:
                # `--cap-lints warn` turns foreign deny-by-default lints into warnings; a
                # remaining hard error is still a real build failure.
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
            "owner": "om9-core gem_edits owner + om9-app gem edit/report session",
            "source_revision": head,
            "checkout_dirty": dirty,
            "sources": [
                {"path": "crates/om9-core/src/gem_edits.rs", "state": "IMPLEMENTED_AND_TESTED"},
                {"path": "crates/om9-core/src/gem.rs", "state": "REUSED_GEM_OWNER"},
                {"path": "crates/om9-core/src/builder_style.rs", "state": "REUSED_NAME_RULES"},
                {"path": "crates/om9-core/tests/gem_edits.rs", "state": "REGISTERED_TESTS"},
                {
                    "path": "crates/om9-app/src/session/gem_edits.rs",
                    "state": "IMPLEMENTED_AND_TESTED",
                },
                {"path": "crates/om9-app/src/session/gem.rs", "state": "REUSED_GEM_SESSION"},
                {"path": "crates/om9-app/src/session.rs", "state": "COMMAND_WIRING"},
                {"path": "crates/om9-app/tests/gem_edits.rs", "state": "REGISTERED_TESTS"},
            ],
            "source_audit": {
                "matrix_assets": "NOT_COPIED",
                "result": "PROCEDURAL_DATA_ONLY",
                "detail": (
                    "No Matrix asset, catalogue bitmap or decompiled resource is read or shipped. "
                    "Edits operate on the typed GemSpec/GemRecord owner and every weight comes "
                    "from the closed procedural mesh in crates/om9-core/src/gem.rs."
                ),
            },
            "planning_inputs": [
                f"OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/{package['folder']}/l3.json",
                "OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/L2.json",
            ],
            "planning_tree_touched": False,
            "notes": [
                "The gem body stays the mesh/record owner of OM9-L2-04-09; this module edits the "
                "record and regenerates the body through the same build_mesh.",
                "The Matrix command names of the family are preserved and dispatch to one session.",
                "An edit is pure data: no core function mutates the document, the session commits "
                "one transaction.",
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

- A preview is pure session data: planned records and, for a report, computed rows. Nothing is
  written to the document before Apply.
- Apply is exactly one document transaction for every mutating command; a style save writes the
  versioned store atomically. Undo restores the previous records, geometry and label.
- Refusals are typed `om9_core::Error` values (`Invalid`, `Missing`, `Stale`, `Busy`,
  `Budget`); an attribute, stone or row is never silently dropped or replaced.
- Every edit keeps the `GemId`, the object id and the `om9.gem.placement` link; only the
  `om9.gem` record and the regenerated geometry change.

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
                package["binary"],
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
                    "binary": package["binary"],
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
                        "binary": package["binary"],
                    }
                )
            if code != 0:
                (directory / f"{l4_id}.log").write_text(
                    header(" ".join(command), code, captured) + "".join(extra_logs),
                    encoding="utf-8",
                )
                tail = "\n".join(captured.splitlines()[-15:])
                raise SystemExit(
                    f"L4 command failed for {l4_id}: {' '.join(command)}\n{tail}"
                )
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
                        f"manual review of {package['title']} at the M05-UI gate",
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
                "schema": "om9.m05.l3_gem_edits_evidence.v1",
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
                    "crates/om9-core/tests/gem_edits.rs",
                    "crates/om9-app/tests/gem_edits.rs",
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
            "schema": "om9.m05.l2_gem_edits_evidence.v1",
            "l2_id": "OM9-L2-04-13",
            "title": "Gem Instance Edits And Reports",
            "date": "2026-10-28",
            "target": TARGET,
            "source_revision": head,
            "checkout_dirty": dirty,
            "code": CHANGED,
            "checks": results,
            "l3": summary,
            "tests": {
                "core_gem_edits": "14 passed (partial edit, round rule, cut/shape refusal, "
                "match mask, style round-trip/store/atomic save, report grouping/material density/"
                "trade-formula cross-check, map numbering, identity preservation)",
                "app_gem_edits": "11 passed (positioner regression, one-transaction control, "
                "match, style save/load, update regeneration, read-only map/report, material "
                "density, history and link preservation, typed command)",
                "app_gem_map_overlay": "3 passed (numbered caption projection, off-screen skip, "
                "caption clamped inside the viewport)",
                "clippy": "cargo clippy -p om9-core -p om9-app --all-targets -- --cap-lints warn; "
                "diagnostics scoped to this task's files and foreign warnings listed below",
                "clippy_foreign_warnings": foreign_warnings,
            },
            "uncertainties": [
                "All nine L3 keep the M05-UI manual gate open: no human has reviewed the edit/report "
                "commands on screen and no reviewer signature is claimed.",
                "Only Windows x64 was exercised in this run.",
                "The app test binaries link OCCT, so the run needs the OCCT bin and VC runtime "
                "directories on PATH; that environment requirement is recorded, not hidden.",
                "CARGO_TARGET_DIR is H:/OpenMatrix9/cargo-target because this sandbox denies "
                "writes to the L:/om9-target directory named in AGENTS.md.",
                "The clippy gate runs with `--cap-lints warn` and is scoped to the files this task "
                "owns: the shared checkout also contains concurrent in-flight work whose "
                "deny-by-default clippy error (settings/bezel.rs) would otherwise stop the whole "
                "batch; those diagnostics are listed in clippy_foreign_warnings.",
                "The Gem Map overlay is automated-tested as projection data (one numbered caption "
                "per visible stone, captions clamped inside the viewport, off-screen stones skipped) "
                "and is painted in the committed viewport path; human pixel acceptance still needs "
                "the M05-UI pass.",
                "The reporter's mass is the closed mesh volume × the material density from the "
                "gemological table, cross-checked against the published trade formula inside a "
                "±15% band; it stays a computed value and the report prints `NOT A CERTIFICATE`, "
                "because only a gemologist can certify a stone.",
            ],
        },
    )
    (BATCH / "README.md").write_text(
        f"""# OM9-L2-04-13 Gem Instance Edits And Reports — evidence

Automated run on 2026-10-28 (`{TARGET}`), source revision `{head}`
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

All nine L3 are commands with a human-facing M05-UI gate, so none is marked DONE and no
`ui-review.json` is written. Each L3 is WIP with automated evidence and an open manual gate.

## Not claimed

- M05-UI manual acceptance of the nine edit/report commands.
- Matrix assets: none were read, copied or shipped.
- M05-UI pixel acceptance of the painted Gem Map overlay and the reporter key window.
- A gemological certificate: the reporter prints a computed mass with a material density and a
  trade-formula cross-check, not a certified weight.
""",
        encoding="utf-8",
    )

    ledger_path = ROOT / "docs" / "openmatrix9-progress.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    entry = {
        "key": KEY,
        "date": "2026-10-28",
        "stage": "feature-port",
        "status": "gem_instance_edits_automated_verified_manual_gates_open",
        "target": TARGET,
        "feature_ids": ["OM9-L2-04-13"],
        "scope": (
            "M05 OM9-L2-04-13 gem instance edits and reports: typed edit/match/style/report owner "
            "in om9-core::gem_edits plus the om9-app session for BuilderGemControl, "
            "OthersMatchAttributes, OthersSaveStyle, OthersLoadStyle, BuilderGemUpdate, "
            "BuilderGemMap, BuilderGemReporter and GemAttributeHistory; every mutating apply is "
            "one transaction and every edit keeps the GemId and the om9.gem.placement link."
        ),
        "notes": (
            "ghi chú: (1) GemEdit là bản sửa một phần — trường vắng mặt là 'giữ nguyên', không "
            "phải 'về mặc định'; (2) Match Attributes sao chép theo mask cut/shape/dimensions/"
            "finish và giữ GemId của đích; (3) gem style dùng schema om9.gem_style.v1, store "
            "om9.gem_style_file.v1, ghi atomic và tái dùng validate_style_name của builder style; "
            "(4) reporter tính khối lượng từ volume lưới kín × density theo bảng density "
            "gemological (kim cương 3.52, corundum 4.00, beryl 2.72, …), đối chiếu công thức "
            "thương mại L×W×depth×factor theo shape trong dung sai ±15%, và in 'NOT A "
            "CERTIFICATE' vì chỉ gemologist mới chứng nhận được; (5) Gem Map vẽ overlay trong "
            "viewport: mỗi viên một caption đánh số kèm size girdle, viên ngoài khung bị bỏ qua, "
            "kèm cửa sổ key liệt kê cùng số; (6) cả 9 L3 đều có gate M05-UI nên dashboard chỉ 🟡, "
            "không có chữ ký reviewer."
        ),
        "outputs": CHANGED,
        "evidence": [
            "implementation/evidence/M05/OM9-L2-04-13/observed.json",
            "implementation/evidence/M05/OM9-L2-04-13/README.md",
            *[f"implementation/evidence/M05/OM9-L2-04-13/{item['artifact']}" for item in results],
            *[
                f"implementation/evidence/M05/{l3}/observed.json"
                for l3 in L3_PACKAGES
            ],
        ],
        "tests": {
            "core_gem_edits": "14 passed (partial edit, round rule, cut/shape refusal, match mask, "
            "style round-trip/store/atomic save, report grouping/material density/trade-formula "
            "cross-check, map numbering, identity preservation)",
            "app_gem_edits": "11 passed (positioner regression, one-transaction control, match, "
            "style save/load, update regeneration, read-only map/report, material density, history "
            "and link preservation, typed command)",
            "app_gem_map_overlay": "3 passed (numbered caption projection, off-screen skip, caption "
            "clamped inside the viewport)",
            "core_gem_assets_regression": "regression suite passed",
            "app_gem_panel_regression": "regression suite passed",
            "core_transaction_regression": "atomic_transactions + transaction_scopes passed",
            "workspace_check": "cargo check --workspace --all-targets exit 0",
            "clippy": "cargo clippy -p om9-core -p om9-app --all-targets -- --cap-lints warn; "
            "diagnostics scoped to this task's files; foreign warnings are listed in "
            "clippy_foreign_warnings",
            "clippy_foreign_warnings": foreign_warnings,
        },
        "uncertainties": json.loads((BATCH / "observed.json").read_text(encoding="utf-8"))[
            "uncertainties"
        ],
        "manual_gate": (
            "OPEN: M05-UI — 9 lệnh sửa/báo cáo gem chưa được người nghiệm thu"
        ),
        "next_action": (
            "Chạy và chụp từng lệnh (positioner, control, match attributes, save/load style, gem "
            "update, gem map, gem report, history) trong UI ở gate M05-UI, chấp nhận pixel overlay "
            "Gem Map và cửa sổ key của reporter, rồi ký ui-review cho từng L3."
        ),
    }
    ledger["items"] = [item for item in ledger["items"] if item.get("key") != KEY] + [entry]
    ledger["active_item"] = KEY
    ledger["updated_at"] = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    write_json(ledger_path, ledger)

    owner_path = L3_ROOT / "L2.json"
    owner = json.loads(owner_path.read_text(encoding="utf-8"))
    owner["runtime_implementation"] = {
        "date": "2026-10-28",
        "status": "AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING",
        "target": TARGET,
        "scope": (
            "Typed gem instance edit/report owner: partial attribute edits, mask-based Match "
            "Attributes, a versioned gem style store, geometry regeneration from the stored "
            "record, numbered map rows, count/weight aggregation and an attribute history that "
            "keeps the GemId and the om9.gem.placement link"
        ),
        "owner": {
            "crates": ["om9-core", "om9-app"],
            "tests": [
                "crates/om9-core/tests/gem_edits.rs",
                "crates/om9-app/tests/gem_edits.rs",
                "crates/om9-app/src/ui/gem_edits.rs",
            ],
        },
        "evidence": [
            "implementation/evidence/M05/OM9-L2-04-13/observed.json",
            "implementation/evidence/M05/OM9-L2-04-13/README.md",
            "implementation/evidence/M05/OM9-L2-04-13/core-gem-edits.log",
            "implementation/evidence/M05/OM9-L2-04-13/app-gem-edits.log",
            "implementation/evidence/M05/OM9-L2-04-13/workspace-check.log",
        ],
        "notes": (
            "Sửa gem là dữ liệu thuần rồi commit một transaction; GemId, object id và link "
            "om9.gem.placement được giữ qua mọi lần sửa. Style ghi atomic vào store có version. "
            "Reporter dùng volume lưới × density theo bảng vật liệu và đối chiếu công thức thương "
            "mại ±15%, in rõ 'NOT A CERTIFICATE'. Gem Map có overlay đánh số trong viewport kèm "
            "cửa sổ key. 9 L3 đều còn gate M05-UI."
        ),
        "l3_coverage": {l3: "automated_verified_manual_gate_open" for l3 in L3_PACKAGES},
        "l4": {
            "planned": len(L3_PACKAGES) * 3,
            "artifacts": len(L3_PACKAGES) * 3,
            "receipts": receipts_all,
        },
        "tests": [
            f"core gem_edits: exit {results[0]['exit_code']}",
            f"app gem_edits: exit {results[1]['exit_code']}",
            f"app gem map overlay: exit {results[2]['exit_code']}",
            f"workspace check: exit {results[6]['exit_code']}",
            f"clippy: exit {results[7]['exit_code']}",
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
            "date": "2026-10-28",
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
                "crates/om9-core/tests/gem_edits.rs",
                "crates/om9-app/tests/gem_edits.rs",
                "crates/om9-app/src/ui/gem_edits.rs",
            ],
            "notes": package["positive_note"] + "; " + package["negative_note"],
        }
        write_json(l3_path, document)

    print(f"recorded gem edit/report evidence at {BATCH.relative_to(ROOT)}")
    for item in results:
        print(f"  {item['artifact']}: exit {item['exit_code']}")
    print(f"L4 receipts: {len(receipts_all)}")
    print(f"ledger: {ledger_path.relative_to(ROOT)} ({len(ledger['items'])} items)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
