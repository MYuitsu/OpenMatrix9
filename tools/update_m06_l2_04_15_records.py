"""Reflect the recorded `OM9-L2-04-15` implementation in the M06 milestone records.

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
    / "L2-04-15 - 02-febe-prong-channel-metal-and-bead"
)
MILESTONE = ROOT / "OM9_Milestones" / "M06 - Settings and Prongs"
LEDGER = ROOT / "docs" / "openmatrix9-progress.json"
DATE = "2026-10-27"

FEATURES = {
    "OM9-L3-04-15-001": ("Prong Builder Gem Line", "BuilderProngAdder", "Two Shared Prongs default; Mode/Diameter/Height/Nudge/Ends/Spacing"),
    "OM9-L3-04-15-002": ("Prong Editor Selected", "BuilderProngEditor", "Diameter/Height/Angle/Nudge/Fillet on selected committed prongs"),
    "OM9-L3-04-15-003": ("Prong On Surface", "BuilderProngOnSurface", "Diameter/Height/Drop/UV; surface normal drives the prong axis"),
    "OM9-L3-04-15-004": ("Metal Piece From Gems", "BuilderMetalPiece", "Mode/Wall/Height/Depth/Offset/Border; Curve mode is a typed refusal"),
    "OM9-L3-04-15-005": ("Channel Border Curve", "BuilderChannelBorderCurve", "Offset/EndOffset/VOffset/Corner/Extension Line|Arc|Smooth"),
    "OM9-L3-04-15-006": ("Bead On Curve Milgrain", "BuilderBeadOnCrv", "Start/End/Spacing/Count/Size/Taper; F4, Down and Object directions"),
    "OM9-L3-04-15-007": ("Bead On Surface", "BuilderBeadOnSurface", "Scale/Offset/Angle; one bead at the picked uv"),
    "OM9-L3-04-15-008": ("Pull Object To Rail Setting", "BuilderPullToRail", "Mode=Rigid|Deform; a selected circle is the rail"),
    "OM9-L3-04-15-009": ("Setting Row End Modes And Prong Spacing", None, "Ends= and Spacing= on the prong commands; no separate icon"),
    "OM9-L3-04-15-010": ("Settings Associative Edit And Mass", "SettingsMass", "text command Material=; reuses the metal density owner"),
}

LIMITATIONS = {
    "OM9-L3-04-15-001": [
        "4 Prongs Close builds the sheet's closer pair: both North-side prongs sit inside the 4 Prongs 0 spacing and stay two distinct prongs.",
        "A tight row can put the facing prongs of two stones on one point; the builder merges that pair into one shared prong and reports merged_prongs rather than refusing the style.",
        "A selection without gem objects is refused: the builder consumes the shared gem row instead of guessing.",
    ],
    "OM9-L3-04-15-002": [
        "The editor rebuilds from the om9.settings.prong payload; a body without it is refused rather than re-derived.",
    ],
    "OM9-L3-04-15-003": ["Drop is negative-up per the sheet wording; the UI still has to confirm the sign on the preview."],
    "OM9-L3-04-15-004": [
        "Curve mode needs a curve: it follows the picked curve, or the committed channel border with Border=On; with neither it is a typed Missing('metal piece curve'), never a row-offset body.",
        "Border=On consumes the last committed channel border; the app never re-derives a border from the row.",
    ],
    "OM9-L3-04-15-005": ["Corner smoothing is the documented 0..1 angle; the exact Matrix corner radius law is not verified."],
    "OM9-L3-04-15-006": [
        "F4 direction falls back to world +Z at the world centre.",
        "Count and Spacing are alternatives: setting one clears the other, and the last bead sits on the end fraction.",
    ],
    "OM9-L3-04-15-007": ["The base bead diameter is 1.0 mm (OM9 engineering default); the sheet only gives the scale."],
    "OM9-L3-04-15-008": ["Only a selected circle rail is accepted; an arc or line rail returns a typed refusal."],
    "OM9-L3-04-15-009": ["These are command options, not a toolbar icon; the record says so instead of inventing an icon id."],
    "OM9-L3-04-15-010": [
        "SettingsMass is a text command without a toolbar slot; mass covers settings features committed in this session.",
        "Cutter features are committed as bodies but never weighed; the report counts them under excluded_cutters.",
    ],
}

# The planned contract left every input UNRESOLVED. Each entry below is the parameter that
# actually implements it, so the record states a real type, unit and default instead of a gap.
RESOLVED_INPUTS = {
    "OM9-L3-04-15-001": (
        "settings::gems::GemRow::from_document (gem owner records) + ProngParams {mode, large/small diameter and height, nudge, spacing, end_mode}",
        "row: GemRow; params: ProngParams (mm, degrees)",
    ),
    "OM9-L3-04-15-002": (
        "ProngSite from the om9.settings.prong payload + ProngEdit {diameter, height, drop, fillet, taper, rotate_deg, angle_deg, nudge}",
        "selected: Vec<(ObjectId, ProngPayload)>; edit: ProngEdit (mm, degrees, 0..1)",
    ),
    "OM9-L3-04-15-003": (
        "geometry::Surface + uv + SurfaceProngParams {diameter, height, drop, fillet, taper, rotate_deg, flip}",
        "surface: Surface (UV domain); uv: [f64;2]; params (mm, degrees)",
    ),
    "OM9-L3-04-15-004": (
        "GemRow + MetalPieceParams {mode, wall, height, depth, floor, offsets, mirror, use_border} + optional ChannelBorder",
        "row: GemRow; params (mm, bool); border: ChannelBorder",
    ),
    "OM9-L3-04-15-005": (
        "GemRow + ChannelBorderParams {side_offset, end_offset, v_offset, corner_angle, extension, tweak_corners, snap_to_corners, show_prongs}",
        "row: GemRow; params (mm, degrees, bool)",
    ),
    "OM9-L3-04-15-006": (
        "geometry::Curve + BeadOnCurveParams {start/end fraction, spacing or count, sizes, taper, roll, z offset, mirror, direction}",
        "curve: Curve; params (fraction 0..1, mm, degrees)",
    ),
    "OM9-L3-04-15-007": (
        "geometry::Surface + uv + BeadOnSurfaceParams {base_diameter, scale, vertical_offset, object_angle, flip}",
        "surface: Surface; uv: [f64;2]; params (mm, degrees)",
    ),
    "OM9-L3-04-15-008": (
        "geometry::Mesh or ObjectId + Rail {center, axis, radius, range} + RailPullMode {Rigid, Deform}",
        "targets: Vec<ObjectId>; rail: Rail (mm, degrees); mode",
    ),
    "OM9-L3-04-15-009": (
        "ProngParams.end_mode + ProngParams.spacing_mm consumed by place_prongs",
        "end_mode: EndMode; spacing_mm: Option<f64> (mm)",
    ),
    "OM9-L3-04-15-010": (
        "ProngFeature/settings_volumes + FeatureMassPart + MetalMaterialRegistry (density g/cm3) + settings_mass",
        "parts: Vec<FeatureMassPart> (mm3, material id); excluded cutters tracked separately",
    ),
}


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8-sig"))


def save(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def main() -> int:
    module = load(MODULE / "L2.json")
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
                    "cargo test -p om9-core --test settings_family",
                ],
                "app": ["cargo test -p om9-app --test settings_commands"],
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
        contract = record.get("parameter_contract")
        if isinstance(contract, dict):
            contract["status"] = "DOMAIN_IMPLEMENTED_AUTOMATED_UI_REVIEW_OPEN"
            contract["implementation_l4"] = l4_ids
            raw, type_default = RESOLVED_INPUTS[task_id]
            # The planner could not type these inputs; the implementation now can.
            contract["domain_inputs"] = [
                {"raw": raw, "type": type_default, "status": "RESOLVED_BY_IMPLEMENTATION"}
            ]
            contract["missing"] = []
            interaction = contract.get("interaction")
            if isinstance(interaction, dict):
                interaction["option_scope"] = "DOMAIN_COMMAND"
                interaction["option_order"] = _options(task_id)
                interaction["option_count"] = len(interaction["option_order"])
        save(directory / "l3.json", record)
        module_features.append({"l3": task_id, "feature": feature, "evidence": evidence})

    module["planning_state"] = "IMPLEMENTED_AUTOMATED_EVIDENCE_UI_REVIEW_OPEN"
    module["l4_status"] = "AUTOMATED_EVIDENCE_RECORDED_UI_REVIEW_PENDING"
    module["implementation"] = {
        "schema": "om9.l2_implementation.v1",
        "recorded_at": DATE,
        "status": "IMPLEMENTED_AUTOMATED_EVIDENCE_RECORDED_UI_REVIEW_OPEN",
        "scope": "10 L3 command adapters over one shared settings domain: prongs, prong editor, surface prong, metal piece, channel border, bead on curve, bead on surface, pull to rail, row end modes and the settings mass read-out.",
        "shared_input": "settings::gems::GemRow built from the gem owner's om9.gem records",
        "shared_output": "prong/bead/metal bodies with the om9.settings.prong payload; mass through metal_weight",
        "features": module_features,
        "automated_evidence": {
            "cargo check -p om9-core -j 2": 0,
            "cargo check -p om9-app -j 2": 0,
            "cargo test -p om9-core --lib settings:: -j 2": "90 passed",
            "cargo test -p om9-core --test settings_family -j 2": "14 passed",
            "cargo test -p om9-app --test settings_commands -j 2": "24 passed",
            "cargo test -p om9-core -j 2": "781 passed across 70 suites",
            "summary": "implementation/evidence/M06/OM9-L3-04-15-001/summary.json",
        },
        "manual_gate": {
            "required": True,
            "gate": "M06-UI",
            "signed": False,
            "note": "no human UI review is claimed; the dashboard keeps the 10 L3 at WIP until a reviewer signs",
        },
        "limitations": sorted({item for values in LIMITATIONS.values() for item in values}),
    }
    save(MODULE / "L2.json", module)

    l1 = load(MILESTONE / "L1.json")
    l1["planning_state"] = "M06_L2_04_15_IMPLEMENTED_AUTOMATED_UI_REVIEW_OPEN"
    l1["planning_only"] = False
    runtime = l1.setdefault("runtime_implementation", {})
    runtime["settings_parts_2026_10_27"] = {
        "date": DATE,
        "status": "implemented_automated_evidence_recorded_ui_review_open",
        "module": "OM9-L2-04-15",
        "l3_recorded": 10,
        "manual_gate_promoted": False,
        "manual_gate": "M06-UI",
        "evidence": [
            "implementation/evidence/M06/OM9-L3-04-15-001/summary.json",
            "implementation/evidence/M06/OM9-L3-04-15-001/logs/",
            "implementation/design/M06/",
            "implementation/fixtures/M06/",
        ],
        "next_action": "Open the OpenMatrix9 window and run each Builder icon: preview, change one option, Enter to commit and Cancel, then sign the M06-UI review for each L3.",
        "limitations": sorted({item for values in LIMITATIONS.values() for item in values}),
    }
    l1["execution_readiness"] = {
        **l1.get("execution_readiness", {}),
        "target": "L2-04-15 has automated Rust evidence; L2-04-14 is still planning-only",
        "automated_evidence_recorded": True,
        "manual_gate_open": "M06-UI",
    }
    save(MILESTONE / "L1.json", l1)

    ledger = load(LEDGER)
    key = "m06-settings-parts-2026-10-27"
    ledger["items"] = [item for item in ledger["items"] if item.get("key") != key]
    ledger["items"].insert(
        0,
        {
            "key": key,
            "date": DATE,
            "stage": "feature-port",
            "status": "implemented_automated_evidence_recorded_manual_review_not_claimed",
            "target": "Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=H:/OpenMatrix9/cargo-target; -j 2",
            "feature_ids": [item["l3"] for item in module_features],
            "scope": "M06 L2-04-15 (02-febe-prong-channel-metal-and-bead): eight Builder command adapters plus the row end options and the settings mass text command over one shared gem row; prong bodies carry om9.settings.prong so the editor rebuilds in place. All 22 documented prong styles build: 4 Prongs Close keeps the closer pair, and a tight row merges the facing prongs of two stones into one shared prong instead of refusing. Metal Piece follows a committed channel border (Border=On) or a picked curve (Mode=Curve), Bead On Curve accepts a count or a spacing, and cutters never enter the mass. 90 settings unit tests, 14 integration tests and 24 app tests pass (781 core tests overall). The dashboard keeps the 10 L3 at WIP because no human UI review is signed.",
            "outputs": [
                "crates/om9-core/src/settings/mod.rs",
                "crates/om9-core/src/settings/gems.rs",
                "crates/om9-core/src/settings/prong.rs",
                "crates/om9-core/src/settings/metal_piece.rs",
                "crates/om9-core/src/settings/channel_border.rs",
                "crates/om9-core/src/settings/bead.rs",
                "crates/om9-core/src/settings/rail.rs",
                "crates/om9-core/src/settings/associative.rs",
                "crates/om9-app/src/session/settings_commands.rs",
                "crates/om9-app/src/session.rs",
                "crates/om9-core/tests/settings_family.rs",
                "crates/om9-app/tests/settings_commands.rs",
                "tools/record_m06_l2_04_15_evidence.py",
                "implementation/evidence/M06/OM9-L3-04-15-001/summary.json",
            ],
            "evidence": [
                item_path
                for feature in module_features
                for item_path in feature["evidence"]
            ]
            + [
                "implementation/evidence/M06/OM9-L3-04-15-001/logs/core-settings.log",
                "implementation/evidence/M06/OM9-L3-04-15-001/logs/core-family.log",
                "implementation/evidence/M06/OM9-L3-04-15-001/logs/app-settings.log",
                "implementation/evidence/M06/OM9-L3-04-15-001/logs/core-all.log",
                "MILESTONE_DASHBOARD.md",
            ],
            "limitations": sorted({item for values in LIMITATIONS.values() for item in values}),
            "notes": "M06 L2-04-15: 10 L3 implemented in Rust with automated receipts (90 unit / 14 integration / 24 app tests; 781 core tests). Dashboard shows 10 WIP, 0 DONE: the M06-UI preview/commit/cancel review is still human work. The last two refusals are gone: 4 Prongs Close builds the closer pair, 6 Prongs 0 builds on a tight row and merges only a facing pair that lands on one point, and Metal Piece Curve mode follows a picked curve. Bead On Curve takes Count= or Spacing=, and SettingsMass excludes cutters.",
            "next_action": "UI review pass over the eight Builder icons plus SettingsMass (preview, change one option, Enter, Cancel), then sign each L3 review.",
        },
    )
    ledger["updated_at"] = datetime.now().astimezone().isoformat(timespec="seconds")
    ledger["active_item"] = key
    try:
        save(LEDGER, ledger)
    except PermissionError:
        # The sandbox keeps docs/ read-only for shell writes; the ledger entry is applied with
        # the editor instead and the rest of the record update still stands.
        print(f"ledger not writable here; apply the {key} item with the editor")
    return 0


def _options(task_id: str) -> list[str]:
    mapping = {
        "OM9-L3-04-15-001": ["Mode=", "Diameter=", "Height=", "Nudge=", "Ends=", "Spacing="],
        "OM9-L3-04-15-002": ["Diameter=", "Height=", "Angle=", "Nudge=", "Fillet="],
        "OM9-L3-04-15-003": ["Diameter=", "Height=", "Drop=", "UV="],
        "OM9-L3-04-15-004": ["Mode=", "Wall=", "Height=", "Depth=", "Offset=", "Border="],
        "OM9-L3-04-15-005": ["Offset=", "EndOffset=", "VOffset=", "Corner=", "Extension="],
        "OM9-L3-04-15-006": ["Start=", "End=", "Spacing=", "Count=", "Size=", "Taper="],
        "OM9-L3-04-15-007": ["Scale=", "Offset=", "Angle="],
        "OM9-L3-04-15-008": ["Mode="],
        "OM9-L3-04-15-009": ["Ends=", "Spacing="],
        "OM9-L3-04-15-010": ["Material="],
    }
    return mapping[task_id] + ["Enter", "Cancel"]


if __name__ == "__main__":
    raise SystemExit(main())
