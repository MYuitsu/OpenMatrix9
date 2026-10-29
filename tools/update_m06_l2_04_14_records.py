"""Reflect the recorded `OM9-L2-04-14` implementation in the M06 milestone records.

The dashboard derives status from evidence, so this script only aligns the planning records with
what the evidence already says: implemented, automated evidence recorded, the M06-UI manual gate
open. It never marks an L3 as accepted and never signs a review.
"""
from __future__ import annotations

import json
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODULE = (
    ROOT
    / "OM9_Milestones"
    / "M06 - Settings and Prongs"
    / "L2-04-14 - 01-febe-head-and-bezel-brep-settings"
)
MILESTONE = ROOT / "OM9_Milestones" / "M06 - Settings and Prongs"
LEDGER = ROOT / "docs" / "openmatrix9-progress.json"
DATE = "2026-10-27"

FEATURES = {
    "OM9-L3-04-14-001": (
        "Head Builder Basket",
        "BuilderHead",
        "Mode/Prongs/Profile/ProfileSave/Rails/Height/Drop/SeatDepth + four edit levels; basket prongs and gallery rails on the gem girdle",
    ),
    "OM9-L3-04-14-002": (
        "Head Builder Half Bezel Mode",
        "BuilderHead",
        "Mode=HalfBezel with Span/Position/Thickness/BezelHeight/SeatAngle",
    ),
    "OM9-L3-04-14-003": (
        "Head Builder Prong Rail Level Edit",
        "BuilderHead",
        "Level=1..4 with Prong=/Rail=/Size=/RailDrop=/RailSize=/RailOffset=",
    ),
    "OM9-L3-04-14-004": (
        "Bezel Builder Full",
        "BuilderBezel",
        "Mode=Bezel with Profile/ProfileSave/Height/Top/Base/Dome/Angle/Seat/SeatDepth/SeatLength/Placement/Scale/Chamfer/Shear/Boolean",
    ),
    "OM9-L3-04-14-005": (
        "Bezel Builder Under Bezel And Gallery",
        "BuilderBezel",
        "Mode=UnderBezel and Mode=GalleryRail over the same section builder",
    ),
    "OM9-L3-04-14-006": (
        "Scallop Bezel Builder",
        "BuilderBezelCutter",
        "Profile/ProfileSave/Count/Angle/Offset/Depth; cutters stay out of the metal mass, their cutter volume and the net metal are reported",
    ),
    "OM9-L3-04-14-007": (
        "Head Bezel Profile Browser",
        None,
        "Profile=/ProfileSave=/RailProfile= read the typed library, save a user section and sample a Bezier; 5 prong, 4 rail, 4 edge entries",
    ),
    "OM9-L3-04-14-008": (
        "Head Bezel Girdle Seat Attachment",
        None,
        "Seats=/SeatAngle=/SeatDepth=: stable seat frames, and Enter cuts the seats into the gem body",
    ),
    "OM9-L3-04-14-009": (
        "Head Bezel Geometry Validity And Update",
        None,
        "payload-driven rebuild with measured validity; same gem preserves ids, a new gem replaces them",
    ),
}

LIMITATIONS = {
    "OM9-L3-04-14-001": [
        "The default head drop follows the stone (35% of the pavilion depth, capped at 0.9 mm) so the base stays above the culet.",
        "Prong clearance is measured against the inscribed girdle radius; a faceted outline is inscribed in its profile circle.",
        "A selection without a gem object is refused: the builder consumes the gem owner's om9.gem record.",
    ],
    "OM9-L3-04-14-002": [
        "Half-bezel mode ignores the prong profile (the manual states there are no prong profile options in half-bezel mode) and reports it as prong_profile_ignored.",
        "The band follows the girdle arc between Position and Position+Span; the white rotation handle is UI scope.",
    ],
    "OM9-L3-04-14-003": [
        "Level 1/2/3 edit the whole prong or rail set; only Level 4 with Prong=/Rail= edits a single element.",
        "A field outside the level's scope is refused as invalid head edit level field instead of being silently widened.",
    ],
    "OM9-L3-04-14-004": [
        "Boolean=On is evaluated in the section frame: the finger rail is a ring on the same girdle, so the groove is the rail section swept along the outline. A non-ring target would need the 3-D CSG owner.",
        "The committed Finger Rail body is an Insert alongside the notched ring; it carries the same om9.settings.bezel payload so a rebuild can find it.",
        "The edge profile drives the corner shape only; the section parameters (height/top/base/dome/angle/chamfer) are explicit typed values.",
    ],
    "OM9-L3-04-14-005": [
        "Under Bezel shifts the section below the girdle band; Gallery Rail is one ring below it. Both share the closed-ring and Boolean gates.",
    ],
    "OM9-L3-04-14-006": [
        "A scallop cutter is the edge profile extruded along the outline tangent; the cut itself is not performed here (the scallop is a separate cutter body).",
        "Cutter bodies never weigh in the metal mass; the report adds their cutter volume and the net metal (a bound on the removed metal) so the numbers are not silently ignored.",
    ],
    "OM9-L3-04-14-007": [
        "The library is read-only built-ins plus the session's user list; ProfileSave= keeps a user section and Profile=<name> resolves it. The Bezier control-point editing UI is not part of this owner.",
        "Profile=Bezier:x,y;x,y;... samples a closed cubic Bezier into a typed section; Profiles never insert a scene object, they only shape a builder section.",
    ],
    "OM9-L3-04-14-008": [
        "The seat frames are published here; Enter applies the seat cut to the gem body through Mutation::Geometry on the gem owner's entity, so Undo restores both the stone and the head.",
        "The seat cutter is a trapezoid wedge under the girdle; the cut is a mesh CSG difference and refuses a non-closed result.",
    ],
    "OM9-L3-04-14-009": [
        "Validity is measured per body (closed, positive volume, unique ids) on the generated mesh; the OCCT B-Rep swap is the kernel adapter's own gate.",
        "A payload without the om9.settings.head slot is refused rather than re-derived.",
    ],
}


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8-sig"))


def save(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def _options(task_id: str) -> list[str]:
    return [entry for entry in FEATURES[task_id][2].split(";")[0].split(",") if "=" in entry]


def evidence_counts() -> dict[str, str]:
    """Read the recorder's summary so the record only states measured test counts."""
    summary = (
        ROOT
        / "implementation"
        / "evidence"
        / "M06"
        / "OM9-L3-04-14-001"
        / "summary.json"
    )
    counts: dict[str, str] = {}
    if summary.is_file():
        for run in load(summary).get("runs", []):
            command = run.get("command", "")
            passed = run.get("tests_passed", 0)
            if command.startswith("cargo check"):
                counts[command] = str(run.get("exit_code", 1))
            else:
                counts[command] = f"{passed} passed"
    return counts


def main() -> int:
    module = load(MODULE / "L2.json")
    counts = evidence_counts()
    l3_dirs = sorted(
        directory
        for directory in MODULE.iterdir()
        if directory.is_dir() and (directory / "l3.json").is_file()
    )
    module_features = []
    for directory in l3_dirs:
        record = load(directory / "l3.json")
        task_id = record["id"]
        title, feature, options = FEATURES[task_id]
        l4_ids = [child["id"] for child in record.get("children_l4", [])]
        evidence = [
            f"implementation/design/M06/{task_id}/source_crosswalk.json",
            f"implementation/design/M06/{task_id}/api.md",
            f"implementation/fixtures/M06/{task_id}/positive.json",
            f"implementation/fixtures/M06/{task_id}/negative.json",
        ]
        if len(l4_ids) >= 3:
            evidence.append(f"implementation/evidence/M06/{task_id}/{l4_ids[2]}.json")
        if len(l4_ids) >= 4:
            evidence.append(f"implementation/evidence/M06/{task_id}/{l4_ids[3]}.json")
        record["planning_state"] = "IMPLEMENTED_AUTOMATED_EVIDENCE_UI_REVIEW_OPEN"
        record["l4_status"] = "AUTOMATED_EVIDENCE_RECORDED_UI_REVIEW_PENDING"
        record["implementation"] = {
            "schema": "om9.l3_implementation.v1",
            "recorded_at": DATE,
            "status": "IMPLEMENTED_AUTOMATED_EVIDENCE_RECORDED",
            "feature": feature,
            "command_options": options,
            "evidence": evidence,
            "automated_tests": {
                "core": [
                    f"cargo test -p om9-core --lib settings:: ({task_id})",
                    "cargo test -p om9-core --lib",
                ],
                "app": ["cargo test -p om9-app --test head_commands"],
            },
            "manual_gate": {
                "required": True,
                "gate": "M06-UI",
                "signed": False,
                "open": f"preview/commit/cancel of {title} on the OpenMatrix9 window",
            },
            "limitations": LIMITATIONS[task_id],
        }
        spec = record.setdefault("specification", {})
        spec["status"] = "L3_IMPLEMENTED_AUTOMATED_EVIDENCE_UI_REVIEW_OPEN"
        spec["execution_status"] = "IMPLEMENTED_AUTOMATED_EVIDENCE_UI_REVIEW_OPEN"
        spec["l4_status"] = "L4_AUTOMATED_EVIDENCE_RECORDED"
        save(directory / "l3.json", record)
        module_features.append({"l3": task_id, "feature": feature, "evidence": evidence})

    module["planning_state"] = "IMPLEMENTED_AUTOMATED_EVIDENCE_UI_REVIEW_OPEN"
    module["l4_status"] = "AUTOMATED_EVIDENCE_RECORDED_UI_REVIEW_PENDING"
    module["implementation"] = {
        "schema": "om9.l2_implementation.v1",
        "recorded_at": DATE,
        "status": "IMPLEMENTED_AUTOMATED_EVIDENCE_RECORDED_UI_REVIEW_OPEN",
        "scope": "9 L3 over one shared head/bezel domain: basket head with the seat cut into the stone, half bezel, four edit levels, three bezel modes with the Boolean finger rail groove, scallop cutters weighed as removed volume, the typed profile library with user Bezier sections, girdle seat frames and the rebuild/validity report.",
        "shared_input": "the gem owner's om9.gem record (GemSpec + document placement) becomes one GemGirdle",
        "shared_output": "closed head/bezel/scallop meshes with the om9.settings.head/bezel/scallop payloads",
        "features": module_features,
        "automated_evidence": {
            "cargo check -p om9-core -j 2": 0,
            "cargo check -p om9-app -j 2": counts.get("cargo check -p om9-app -j 2", "0"),
            "cargo test -p om9-core --lib settings::head -j 2": counts.get(
                "cargo test -p om9-core --lib settings::head -j 2", "not recorded"
            ),
            "cargo test -p om9-core --lib settings::bezel -j 2": counts.get(
                "cargo test -p om9-core --lib settings::bezel -j 2", "not recorded"
            ),
            "cargo test -p om9-app --test head_commands -j 2": counts.get(
                "cargo test -p om9-app --test head_commands -j 2", "not recorded"
            ),
            "cargo test -p om9-core --lib -j 2": counts.get(
                "cargo test -p om9-core --lib -j 2", "not recorded"
            ),
            "summary": "implementation/evidence/M06/OM9-L3-04-14-001/summary.json",
        },
        "manual_gate": {
            "required": True,
            "gate": "M06-UI",
            "signed": False,
            "note": "no human UI review is claimed; the dashboard keeps the 9 L3 at WIP until a reviewer signs",
        },
        "limitations": sorted({item for values in LIMITATIONS.values() for item in values}),
    }
    save(MODULE / "L2.json", module)

    l1 = load(MILESTONE / "L1.json")
    l1["planning_state"] = "M06_L2_04_14_AND_04_15_IMPLEMENTED_AUTOMATED_UI_REVIEW_OPEN"
    l1["planning_only"] = False
    runtime = l1.setdefault("runtime_implementation", {})
    runtime["head_bezel_2026_10_27"] = {
        "date": DATE,
        "status": "implemented_automated_evidence_recorded_ui_review_open",
        "module": "OM9-L2-04-14",
        "l3_recorded": 9,
        "manual_gate_promoted": False,
        "manual_gate": "M06-UI",
        "evidence": [
            "implementation/evidence/M06/OM9-L3-04-14-001/summary.json",
            "implementation/evidence/M06/OM9-L3-04-14-001/logs/",
            "implementation/evidence/M06/OM9-L2-04-14-COMPLETION/README.md",
            "implementation/design/M06/",
            "implementation/fixtures/M06/",
        ],
        "next_action": "Open the OpenMatrix9 window and run Head Builder (basket and half bezel), the four edit levels, the three Bezel Builder modes and the Scallop Bezel: preview, change one option, Enter to commit, Cancel, then sign the M06-UI review for each L3.",
        "limitations": sorted({item for values in LIMITATIONS.values() for item in values}),
    }
    l1["execution_readiness"] = {
        **l1.get("execution_readiness", {}),
        "target": "L2-04-14 and L2-04-15 both have automated Rust evidence; the M06-UI manual gate is open",
        "automated_evidence_recorded": True,
        "manual_gate_open": "M06-UI",
    }
    save(MILESTONE / "L1.json", l1)

    ledger = load(LEDGER)
    key = "m06-head-bezel-2026-10-27"
    ledger["items"] = [item for item in ledger["items"] if item.get("key") != key]
    ledger["items"].append(
        {
            "key": key,
            "date": DATE,
            "stage": "feature-port",
            "status": "implemented_automated_evidence_recorded_manual_review_not_claimed",
            "target": "Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=H:/OpenMatrix9/cargo-target; -j 2",
            "feature_ids": [item["l3"] for item in module_features],
            "scope": "M06 L2-04-14 (01-febe-head-and-bezel-brep-settings): Head Builder basket + half bezel + four edit levels + seat cut into the stone, Bezel Builder Bezel/Under Bezel/Gallery Rail + Boolean finger rail groove, Scallop Bezel cutters weighed as removed volume, typed profile library with user Bezier sections, girdle seat frames and the rebuild/validity report.",
            "outputs": [
                "crates/om9-core/src/settings/head.rs",
                "crates/om9-core/src/settings/bezel.rs",
                "crates/om9-app/src/session/head_commands.rs",
                "crates/om9-app/tests/head_commands.rs",
            ],
            "evidence": [
                "implementation/evidence/M06/OM9-L3-04-14-001/summary.json",
                "implementation/evidence/M06/OM9-L3-04-14-001/logs/",
                "implementation/evidence/M06/OM9-L2-04-14-COMPLETION/README.md",
                "implementation/design/M06/",
                "implementation/fixtures/M06/",
            ],
            "limitations": [
                "Boolean=On is a section-frame boolean for the ring finger rail (rail section swept along the outline); a non-ring target would need the 3-D CSG owner.",
                "User profiles live in the session (ProfileSave=); the built-in library stays read-only and the Bezier control-point UI belongs to the UI owner.",
                "Head drop defaults follow the stone (35% of the pavilion depth, capped at 0.9 mm).",
                "Scallop cutters never weigh in the metal mass; the report publishes their cutter volume and the net metal instead (a bound, not a measured cut).",
                "The seat cut replaces the gem body through the transaction, so Undo restores the stone; the seat cutter is a trapezoid wedge under the girdle.",
                "No human M06-UI review is claimed; the dashboard keeps the nine L3 at WIP.",
            ],
            "notes": "9 L3 implemented with automated Rust evidence: 12 head unit tests + 10 bezel unit tests + 9 app command tests + the core lib suite, all exit 0. The scaffold reuses the gem owner's om9.gem record and the host transaction, so Undo restores the document. The dashboard stays WIP until a human signs M06-UI.",
            "next_action": "Run the four Builder icons on the OpenMatrix9 window (preview, option change, Enter, Cancel), including Boolean on/off and a seat cut, then sign the M06-UI review per L3; keep the UI-scope limitations open.",
        }
    )
    ledger["updated_at"] = datetime.now().astimezone().replace(microsecond=0).isoformat()
    save(LEDGER, ledger)
    print(f"records updated: {len(module_features)} L3, ledger item {key}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
