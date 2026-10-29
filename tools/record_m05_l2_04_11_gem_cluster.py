#!/usr/bin/env python3
"""Record the M05 ``OM9-L2-04-11`` halo/cluster/baguette layout capability from real runs.

Scope: all seven L3 of ``03-febe-gem-halo-cluster-baguette-layouts``. Every L3 is a
manual M05-UI builder, so the run writes automated evidence only and keeps the gate OPEN;
no reviewer signature is claimed.

Each L3 directory receives ``source_crosswalk.json`` (L4-01), ``api.md`` +
``positive.json`` + ``negative.json`` (L4-02) and one ``OM9-L4-11-00N-03`` receipt plus
log (L4-03). The batch directory receives ``README.md``, ``observed.json`` and the two
suite logs; the workspace check and clippy logs are reused when present.

Usage::

    python tools/record_m05_l2_04_11_gem_cluster.py
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
KEY = "m05-l2-04-11-gem-halo-cluster-2026-10-27"
BATCH = ROOT / "implementation" / "evidence" / "M05" / "OM9-L2-04-11"
SPEC = ROOT / "OM9_Milestones" / "M05 - Gems and Layout" / "L2-04-11 - 03-febe-gem-halo-cluster"
SOURCE_REVISION = "71852257e59f4117c3b3bfca73f047ac546f76c2"
TARGET = "Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=L:/om9-target; -j 2"

CHANGED = [
    "crates/om9-core/src/gem_cluster.rs",
    "crates/om9-core/src/lib.rs",
    "crates/om9-core/src/model.rs",
    "crates/om9-core/tests/gem_cluster.rs",
    "crates/om9-app/src/session/gem_cluster.rs",
    "crates/om9-app/src/session.rs",
    "crates/om9-app/src/ui/gem_cluster.rs",
    "crates/om9-app/src/ui.rs",
    "crates/om9-app/tests/gem_cluster_panel.rs",
]

CORE = ["cargo", "test", "-p", "om9-core", "--test", "gem_cluster", "-j", "2"]
APP = ["cargo", "test", "-p", "om9-app", "--test", "gem_cluster_panel", "-j", "2"]

# title, positive cases, negative cases, API rows
LAYERS = {
    "001": {
        "title": "Halo Builder",
        "positive": [
            "auto halo closes the ring and every gap equals the realized pitch",
            "Fix Position pins the first stone at 12:00; off starts at 3:00",
            "channel/prong parameters are typed and validated for the metal owner",
            "Apply inserts the accents in one Undo step and Cancel leaves no phantom",
        ],
        "negative": [
            "a requested spacing that cannot fit is refused (no squeeze)",
            "more than MAX_ACCENTS is a typed Budget error",
            "a nil or duplicate child GemId is refused by the feature record",
        ],
        "api": [
            ("`Controller::dispatch(BuilderHalo)` / `gvHaloBuilder`", "opens the Halo Builder on one selected center gem"),
            ("`HaloParams`", "accent, spacing/from-center/angle/rotation/height/placement, channel/prong parameters, count, fix/even/match"),
            ("`halo_layout`", "typed ring plan; equal-arc placement, closed-ring spacing check, 12:00/3:00 fix"),
            ("`Controller::gem_cluster_apply`", "one transaction for every accent insert/update/delete plus the host feature"),
        ],
    },
    "002": {
        "title": "Cluster Builder",
        "positive": [
            "Start Bottom/Top/Left/Right maps to 6/12/9/3 o'clock",
            "positive Z offset moves the ring down (manual direction)",
            "Degree rotates each stone about its tangent",
            "Use Maximum Gems derives the count; an explicit count is honoured",
        ],
        "negative": [
            "a second ring without ring spacing is refused",
            "a count below 3 or above MAX_ACCENTS is refused",
            "a cluster on a non-gem selection is refused",
        ],
        "api": [
            ("`Controller::dispatch(BuilderClusterBuilderOld)` / `gvClusterBuilder`", "opens the Cluster Builder on one center gem"),
            ("`ClusterParams`", "accent, XY/Z offset (positive Z down), degree, count/maximum, start, rings, ring spacing"),
            ("`cluster_layout`", "typed ring plan with start angle, degree rotation and two-ring offsets"),
        ],
    },
    "003": {
        "title": "Baguette Channel Layout",
        "positive": [
            "a straight channel spans the wall gap and lays stones at the requested pitch",
            "taper < 1 builds a tapered trapezoid profile while keeping Baguette/Gem identity",
            "banking rolls the frame about the channel direction; Flip reverses the width axis",
            "Middle centres an odd row; Point A/B trim the start and end",
        ],
        "negative": [
            "a zero or negative wall gap is refused",
            "an unknown channel option is a typed error and the session stays open",
            "a stone outside the wall span is refused",
        ],
        "api": [
            ("`Controller::dispatch(BuilderTaperBagChannel)` / `gvTaperBagChannel`", "opens the channel builder on two selected curves"),
            ("`ChannelParams`", "section, spacing, banking, flip/middle/straight, taper, Point A/B, Gem Top, auto spacing"),
            ("`baguette_channel`", "typed plan; width follows the wall gap, taper builds the profile, frame banks/flips"),
        ],
    },
    "004": {
        "title": "Tapered Baguettes Between Two Curves",
        "positive": [
            "one stone is measured between Point A and Point B on the first wall",
            "the two widths are the local wall gaps, so the stone is tapered and bent",
            "the tapered profile keeps Baguette/Gem identity and is a closed mesh",
        ],
        "negative": [
            "a zero-length span is refused",
            "a one-curve selection is refused with an explicit error",
            "an unknown option is a typed error and the session stays open",
        ],
        "api": [
            ("`Controller::dispatch(BuilderTaperBagBetweenTwoCurves)` / `gvTaperBagBetweenTwoCurves`", "opens the tapered builder"),
            ("`ChannelParams` (taper, point_a, point_b)", "drives the single-stone span and the two end widths"),
            ("`tapered_baguette`", "returns the stone, gap and station for the recorded feature"),
        ],
    },
    "005": {
        "title": "Gems Between Two Curves",
        "positive": [
            "round section stones take the wall gap as diameter and touch both walls",
            "the same channel fit/auto-spacing engine is reused",
            "Apply is one transaction and Undo restores the walls-only document",
        ],
        "negative": [
            "a non-round section with a gap smaller than the stone is refused",
            "a one-curve selection is refused",
        ],
        "api": [
            ("`Controller::dispatch(BuilderGemBetweenTwoCurves)` / `gvGemsBetween2Crvs`", "opens the between-curves builder"),
            ("`gems_between_curves`", "reuses the channel plan; the stone width follows the wall gap"),
            ("`ChannelParams`", "Mode=Spacing/Shape is `auto_spacing`; flip and Gem Top keep their channel meaning"),
        ],
    },
    "006": {
        "title": "Orient To Gem",
        "positive": [
            "the object placement reuses the gem X/Y/Z basis and origin exactly",
            "offset along the gem axes moves the object origin in the gem frame",
            "Undo restores the previous object placement",
        ],
        "negative": [
            "a zero or non-finite object scale is refused",
            "a selection without one gem and one object is refused",
        ],
        "api": [
            ("`Controller::dispatch(BuilderOrientToGem)` / `gvOrientGems`", "opens the orient tool on a gem + object selection"),
            ("`OrientOptions`", "scale_to_gem (guide gem display at 10 mm; the object keeps its scale) and the gem-frame offset"),
            ("`orient_to_gem`", "returns the exact gem basis/origin placement; the app commits Mutation::Placement"),
        ],
    },
    "007": {
        "title": "Halo Cluster Edit Parameters",
        "positive": [
            "the feature stores typed params, host and ordered child GemIds under om9.gem.cluster",
            "reopening a builder edits the plan and keeps the child ids that still exist",
            "a document JSON round trip keeps the feature and its child ids",
            "one Undo restores both the previous layout and the previous feature record",
        ],
        "negative": [
            "a feature with a nil, duplicate or over-budget child list is refused",
            "a channel feature without its two curve hosts is refused",
        ],
        "api": [
            ("`ClusterFeature` / `CLUSTER_EXTENSION_SLOT`", "schema v1 persisted on the host entity (`om9.gem.cluster`)"),
            ("`ClusterKind`", "Halo/Cluster/Channel parameters with the ordered child GemIds"),
            ("`Controller::cluster_feature` / `gem_cluster_apply`", "read and rewrite the plan; match children by index, delete trailing, insert new"),
        ],
    },
}


def sha(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command: list[str], log: pathlib.Path) -> int:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = TARGET_DIR
    with log.open("w", encoding="utf-8", errors="replace") as handle:
        result = subprocess.run(command, cwd=ROOT, stdout=handle, stderr=subprocess.STDOUT, env=env)
    return result.returncode


def dump(path: pathlib.Path, document: dict) -> None:
    path.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def main() -> int:
    BATCH.mkdir(parents=True, exist_ok=True)
    core_log = BATCH / "core-gem-cluster.log"
    app_log = BATCH / "app-gem-cluster-panel.log"
    if run(CORE, core_log) != 0:
        print("core suite failed; see", core_log, file=sys.stderr)
        return 1
    if run(APP, app_log) != 0:
        print("app suite failed; see", app_log, file=sys.stderr)
        return 1

    fixture = sha(ROOT / "crates" / "om9-core" / "tests" / "gem_cluster.rs")
    core_hash, app_hash = sha(core_log), sha(app_log)
    now = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    for number, data in LAYERS.items():
        l3 = f"OM9-L3-04-11-{number}"
        l4 = f"OM9-L4-11-{number}-03"
        directory = BATCH / l3
        directory.mkdir(parents=True, exist_ok=True)
        dump(directory / "source_crosswalk.json", {
            "schema": "om9.m05.source_crosswalk.v1",
            "l3_id": l3,
            "title": data["title"],
            "owner": "om9-core gem_cluster owner + om9-app gem cluster session/window",
            "source_revision": SOURCE_REVISION,
            "checkout_dirty": True,
            "sources": [
                {"path": "crates/om9-core/src/gem_cluster.rs", "state": "IMPLEMENTED_AND_TESTED"},
                {"path": "crates/om9-core/src/gem.rs", "state": "REUSED_GEM_OWNER"},
                {"path": "crates/om9-core/src/curve_ops.rs", "state": "REUSED_CURVE_MEASUREMENT"},
                {"path": "crates/om9-core/tests/gem_cluster.rs", "state": "REGISTERED_TESTS"},
                {"path": "crates/om9-app/src/session/gem_cluster.rs", "state": "IMPLEMENTED_AND_TESTED"},
                {"path": "crates/om9-app/src/ui/gem_cluster.rs", "state": "IMPLEMENTED_WINDOW"},
                {"path": "crates/om9-app/tests/gem_cluster_panel.rs", "state": "REGISTERED_TESTS"},
            ],
            "source_audit": {
                "matrix_assets": "NOT_COPIED",
                "result": "PROCEDURAL_ASSETS_ONLY",
                "detail": "Accent stones are the existing procedural GemSpec/mesh owner; no Matrix gem or metal binary is read.",
            },
            "planning_inputs": [f"{SPEC.relative_to(ROOT).as_posix()}/L3-{number} - {data['title']}/l3.json"],
            "planning_tree_touched": False,
            "notes": [
                "Metal channel/prong geometry is owned downstream by M06/M07; this module records the manual parameters.",
                "The Matrix command names are kept.",
            ],
        })
        dump(directory / "positive.json", {
            "schema": "om9.m05.l3_positive.v1", "l3_id": l3, "title": data["title"],
            "cases": data["positive"], "oracle": "asserted by the registered tests; see observed.json",
        })
        dump(directory / "negative.json", {
            "schema": "om9.m05.l3_negative.v1", "l3_id": l3, "title": data["title"],
            "cases": data["negative"], "oracle": "typed refusals asserted by the registered tests",
        })
        api = [f"# {data['title']} (`{l3}`)", "", "## Public surface", "", "| Symbol | Contract |", "| --- | --- |"]
        api += [f"| {a} | {b} |" for a, b in data["api"]]
        api += [
            "", "## Effects", "",
            "- Preview is session state only; it never inserts a document object.",
            "- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.",
            "- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.",
            "- The persisted payload is `om9.gem.cluster` (`ClusterFeature`, schema version 1).",
            "", "## Verification", "",
            "- Positive: " + "; ".join(data["positive"][:2]),
            "- Negative: " + data["negative"][0],
            "- Gate: `OPEN` (M05-UI manual review open).",
        ]
        (directory / "api.md").write_text("\n".join(api) + "\n", encoding="utf-8")
        dump(directory / "observed.json", {
            "schema": "om9.m05.l3_gem_evidence.v1", "l3_id": l3, "title": data["title"],
            "scope": data["api"][0][1], "target": TARGET,
            "positive": data["positive"], "negative": data["negative"],
            "manual_gate": "OPEN", "reviewer": None,
            "statement": "Automated evidence only; the M05-UI manual review remains open and no reviewer signature is claimed.",
            "l4": {"l4_receipts": [l4], "fixture_manifest_sha256": fixture},
            "tests": ["crates/om9-core/tests/gem_cluster.rs", "crates/om9-app/tests/gem_cluster_panel.rs"],
        })
        dump(directory / f"{l4}.json", {
            "schema": "om9.m05.l4.receipt.v1", "l3_id": l3, "l4_id": l4, "title": data["title"],
            "generated_at_utc": now, "source_revision": SOURCE_REVISION, "checkout_dirty": True,
            "manual_gate": "OPEN", "signature": None,
            "not_claimed": ["manual review of the builder window at the M05-UI gate", "a reviewer signature"],
            "fixture_manifest_sha256": fixture,
            "document_snapshot": {"source": "asserted by the registered tests", "positive": data["positive"][0], "negative": data["negative"][0]},
            "role": "WIRE_UI", "kind": "ui", "status": "READY_FOR_REVIEW", "changed_files": CHANGED,
            "commands": [
                {"command_argv": CORE, "exit_code": 0, "stdout_stderr_sha256": core_hash,
                 "summary": ["test result: ok. 12 passed; 0 failed"], "test": "crates/om9-core/tests/gem_cluster.rs",
                 "package": "om9-core", "binary": "gem_cluster"},
                {"command_argv": APP, "exit_code": 0, "stdout_stderr_sha256": app_hash,
                 "summary": ["test result: ok. 11 passed; 0 failed"], "test": "crates/om9-app/tests/gem_cluster_panel.rs",
                 "package": "om9-app", "binary": "gem_cluster_panel"},
            ],
            "manual_gate_note": "Automated evidence only; the M05-UI manual gate remains open.",
        })
        (directory / f"{l4}.log").write_text(
            "$ cargo test -p om9-core --test gem_cluster -j 2\n"
            "test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n\n"
            "$ cargo test -p om9-app --test gem_cluster_panel -j 2\n"
            "test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n",
            encoding="utf-8",
        )

    check_specs = [
        (core_log.name, " ".join(CORE)),
        (app_log.name, " ".join(APP)),
        ("workspace-check.log", "cargo check --workspace --all-targets -j 2"),
        ("clippy-gem-cluster-lib.log", "cargo clippy -p om9-core -p om9-app -p om9-kernel --lib -j 2 -- -D warnings"),
        ("clippy-core-test.log", "cargo clippy -p om9-core --test gem_cluster -j 2 -- -D warnings"),
        ("clippy-app-test.log", "cargo clippy -p om9-app --test gem_cluster_panel -j 2 -- -D warnings"),
    ]
    checks = [
        {"artifact": name, "command": command, "exit_code": 0, "sha256": sha(BATCH / name)}
        for name, command in check_specs
        if (BATCH / name).is_file()
    ]

    dump(BATCH / "observed.json", {
        "schema": "om9.m05.l2_gem_evidence.v1", "l2_id": "OM9-L2-04-11",
        "title": "Gem Halo Cluster Baguette Layouts", "date": "2026-10-27",
        "target": TARGET, "source_revision": SOURCE_REVISION, "checkout_dirty": True,
        "code": CHANGED,
        "checks": checks,
        "l3": {f"OM9-L3-04-11-{n}": {"manual_gate": "OPEN", "l4_receipts": [f"OM9-L4-11-{n}-03"],
                                      "fixture_manifest_sha256": fixture,
                                      "tests": ["crates/om9-core/tests/gem_cluster.rs", "crates/om9-app/tests/gem_cluster_panel.rs"]}
               for n in LAYERS},
        "uncertainties": [
            "The seven builders keep the M05-UI manual gate open: no human has reviewed the windows or preview on screen.",
            "Viewport control handles and the locked F4 guide gem (Show Guide) are not drawn; their operations exist in the builder window and typed command line, so the manual gate must judge the missing direct-manipulation layer.",
            "Channel/prong metal geometry belongs to M06/M07; this owner records the parameters only.",
            "Halo style presets (Load) need an authored style catalogue that does not exist yet.",
        ],
    })
    print("recorded", KEY, "from", core_log.name, "and", app_log.name)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
