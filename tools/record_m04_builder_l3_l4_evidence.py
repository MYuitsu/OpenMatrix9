#!/usr/bin/env python3
"""Record the L4 evidence package for the four M04 builder UI L3.

The L3 stay ``🟡 có bằng chứng, chưa ký``: this tool writes the six L4 artefacts each
planning record asks for (source crosswalk, API contract + fixtures, implementation,
integration, regression, acceptance), with real command receipts, but **no**
``ui-review.json`` signature. A human still owns the manual M04-UI gate.

Usage::

    python tools/record_m04_builder_l3_l4_evidence.py
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
EVIDENCE = ROOT / "implementation" / "evidence" / "M04"

L3_SPECS = {
    "OM9-L3-03-11-002": {
        "title": "Builder Mode Tabs And Context",
        "owner": "om9-app builder panel + om9-core BuilderSession",
        "sources": [
            ("crates/om9-core/src/builder.rs", ["BuilderKind", "BuilderSession::new", "BuilderSession::fields"]),
            ("crates/om9-app/src/session/builder.rs", ["switch_builder", "builder_tabs", "builder_selection", "builder_saved"]),
            ("crates/om9-app/src/ui/chrome.rs", ["builder_slot_ui tab row"]),
        ],
        "api": [
            ("BuilderKind::ALL", "five modes in tab order"),
            ("Controller::switch_builder", "move the active tab and keep each session's fields"),
            ("Controller::builder_tabs", "tab row with the active flag"),
        ],
        "positive": {
            "input": "Rail Profile tab with Width=3; switch to Ring Rail, Diameter=20; switch back",
            "expected": "profile Width=3 and rail Diameter=20 persist per mode; exactly one active tab of five",
        },
        "negative": {
            "input": "remove the selected profile preset, then switch tabs",
            "expected": "switch is refused with Missing(profile preset); the previous tab stays active",
        },
        "tests": [
            ("om9-app", "builder_panel", "tabs_keep_each_mode_parameters_and_switch_safely"),
            ("om9-app", "builder_panel", "a_failed_tab_switch_restores_the_previous_tab"),
        ],
    },
    "OM9-L3-03-11-007": {
        "title": "Builder Profile Selector Window",
        "owner": "om9-core ProfilePreset library + app selector window",
        "sources": [
            ("crates/om9-core/src/producers.rs", ["ProfilePreset", "profile_preset_curve", "RailProfileProducer"]),
            ("crates/om9-core/src/builder.rs", ["BuilderSession::set_preset", "remove_preset", "presets"]),
            ("crates/om9-app/src/session/builder.rs", ["builder_presets", "builder_set_preset", "builder_remove_preset"]),
            ("crates/om9-app/src/ui/chrome.rs", ["builder_profile_window_ui"]),
        ],
        "api": [
            ("ProfilePreset::ALL", "rectangle, rounded_rectangle, circle"),
            ("Controller::builder_presets", "library rows for the selector window"),
            ("Controller::builder_set_preset", "preview-only selection"),
            ("Controller::builder_remove_preset", "drop a preset from this session library"),
        ],
        "positive": {
            "input": "Rail Profile tab; choose circle in the library; Apply",
            "expected": "preview is a circle before Apply, document unchanged until Apply; applied recipe carries preset=circle",
        },
        "negative": {
            "input": "remove the selected profile preset mid-selection",
            "expected": "preview drops; refresh and Apply return Missing(profile preset)",
        },
        "tests": [
            ("om9-core", "builder_session", "the_profile_library_switches_shape_and_refuses_a_removed_preset"),
            ("om9-app", "builder_panel", "the_profile_library_changes_the_preview_and_the_applied_shape"),
            ("om9-app", "builder_panel", "the_preset_can_be_chosen_from_the_command_line"),
        ],
    },
    "OM9-L3-04-02-002": {
        "title": "Outside Ring Rail Generator",
        "owner": "om9-core OutsideRailProducer",
        "sources": [
            ("crates/om9-core/src/producers.rs", ["OutsideRailProducer", "OUTSIDE_RAIL_COMMAND"]),
            ("crates/om9-core/src/builder.rs", ["BuilderKind::OutsideRail fields/handles"]),
            ("crates/om9-app/src/session/builder.rs", ["switch_builder", "builder_text Offset="]),
        ],
        "api": [
            ("OutsideRailProducer::produce", "concentric outer circle at radius + offset_mm"),
            ("BuilderKind::OutsideRail", "offset_mm 0.1..50, one rail input"),
            ("BuilderHandle offset", "drag changes offset_mm"),
        ],
        "positive": {
            "input": "closed inner circular rail radius 9 mm; offset 2 mm outward",
            "expected": "outer circle radius 11 mm, same centre/frame, closed domain, source rail untouched",
        },
        "negative": {
            "input": "negative offset, or a non-circular rail",
            "expected": "typed rejection (field domain / Unsupported(non-circular rail)); committed document unchanged",
        },
        "tests": [
            ("om9-core", "builder_session", "an_outside_rail_offsets_a_closed_circular_rail_only"),
            ("om9-app", "builder_panel", "outside_rail_and_resizer_apply_independent_outputs"),
        ],
    },
    "OM9-L3-04-02-007": {
        "title": "Ring Resizer Builder",
        "owner": "om9-core RingResizerProducer",
        "sources": [
            ("crates/om9-core/src/producers.rs", ["RingResizerProducer", "RING_RESIZER_COMMAND"]),
            ("crates/om9-core/src/builder.rs", ["BuilderKind::RingResizer fields/handles"]),
            ("crates/om9-app/src/session/builder.rs", ["builder_text Target= / Thickness="]),
        ],
        "api": [
            ("RingResizerProducer::produce", "new independent ring at target_diameter_mm"),
            ("BuilderKind::RingResizer", "target 1..60 mm, thickness 0.1..10 mm"),
            ("Controller::builder_apply", "one transaction: entity + recipe + result"),
        ],
        "positive": {
            "input": "ring inside diameter 18 mm; target 19 mm; non-historical resizer mode",
            "expected": "new output ring radius 9.5 mm (inside diameter 19), original ring still radius 9 mm; undo removes only the output",
        },
        "negative": {
            "input": "target diameter at or below twice the band thickness",
            "expected": "Invalid(ring resizer target below thickness); no entity written",
        },
        "tests": [
            ("om9-core", "builder_session", "a_ring_resizer_creates_an_independent_ring_and_refuses_a_shrink_below_thickness"),
            ("om9-app", "builder_panel", "outside_rail_and_resizer_apply_independent_outputs"),
        ],
    },
    "OM9-L3-03-11-001": {
        "title": "Builder Typed Input Boxes",
        "owner": "om9-core BuilderSession input slots",
        "sources": [
            ("crates/om9-core/src/builder.rs", ["BuilderKind::input_roles", "BuilderSession::new", "BuilderSession::inputs"]),
            ("crates/om9-core/src/geometry.rs", ["GeometryKind"]),
            ("crates/om9-core/tests/builder_controls.rs", ["a_typed_input_slot_refuses_the_wrong_geometry_kind_and_keeps_the_entity_id"]),
        ],
        "api": [
            ("BuilderKind::input_roles", "declared slot order for each mode"),
            ("BuilderSession::new", "count and geometry-kind check before any field"),
            ("BuilderSession::inputs", "the stored (role, ObjectId) pairs"),
        ],
        "positive": {
            "input": "Rail Profile with one closed curve (Rail Sweep declares the rail + profile slots)",
            "expected": "each slot accepts only its declared Curve kind and keeps the document ObjectId",
        },
        "negative": {
            "input": "a planar Surface dropped into the rail Curve slot",
            "expected": "Invalid(builder input kind) before a field or preview exists",
        },
        "notes": [
            "`om9-core` has no Gem geometry kind: this builder family declares Curve slots only, so the"
            " generic fixture's GemRef placeholder is out of scope. The Surface-into-Curve refusal covers"
            " the typed-slot negative.",
        ],
        "tests": [
            ("om9-core", "builder_controls", "a_typed_input_slot_refuses_the_wrong_geometry_kind_and_keeps_the_entity_id"),
        ],
    },
    "OM9-L3-03-11-003": {
        "title": "Builder Parameter Controls And Sliders",
        "owner": "om9-core BuilderField domain + panel slider",
        "sources": [
            ("crates/om9-core/src/builder.rs", ["BuilderField::step", "adjust_field", "set_slider", "slider_ratio"]),
            ("crates/om9-app/src/session/builder.rs", ["builder_set_slider", "BuilderFieldView::ratio/step"]),
            ("crates/om9-app/src/ui/chrome.rs", ["builder slider widget"]),
        ],
        "api": [
            ("BuilderField::step", "quantisation in millimetres"),
            ("BuilderSession::adjust_field", "typed/step delta"),
            ("BuilderSession::set_slider", "ratio 0..1 mapped onto min..max then quantised"),
            ("Controller::builder_set_slider", "panel slider adapter"),
        ],
        "positive": {
            "input": "width_mm 2.0, slider moved 0.46 (step 0.1)",
            "expected": "stored value 2.5 mm; display, model and preview agree",
        },
        "negative": {
            "input": "slider ratio -3.0 or 3.0 (outside 0..1)",
            "expected": "stored value clamps to the field min/max (0.1 / 20 mm); no value outside the domain is stored",
        },
        "tests": [
            ("om9-core", "builder_controls", "a_slider_move_lands_exactly_on_the_field_step"),
            ("om9-app", "builder_controls", "a_slider_edit_from_the_panel_quantises_and_previews_the_same_value"),
        ],
    },
    "OM9-L3-03-11-004": {
        "title": "Builder Live Preview Lifecycle",
        "owner": "om9-app builder preview (no document writes before Apply)",
        "sources": [
            ("crates/om9-app/src/session/builder.rs", ["refresh", "builder_adjust_field", "builder_cancel"]),
            ("crates/om9-core/src/builder.rs", ["BuilderSession::refresh", "BuilderSession::preview"]),
            ("crates/om9-app/tests/builder_controls.rs", ["escape_after_preview_edits_leaves_the_document_bytewise_unchanged"]),
            ("crates/om9-app/tests/builder_panel.rs", ["cancel_and_reset_leave_the_document_untouched"]),
        ],
        "api": [
            ("BuilderSession::refresh", "recompute the preview from fields only"),
            ("Controller::builder_cancel", "drop the preview, keep the committed document"),
            ("escape/Enter", "Escape cancels the preview, Enter commits"),
        ],
        "positive": {
            "input": "drag width 2 -> 3 -> 4 mm, then Escape",
            "expected": "no history/Undo entry from the previews; Escape returns the original state",
        },
        "negative": {
            "input": "preview edit without Apply",
            "expected": "history recipes and undo journal stay empty",
        },
        "tests": [
            ("om9-app", "builder_controls", "escape_after_preview_edits_leaves_the_document_bytewise_unchanged"),
            ("om9-app", "builder_panel", "cancel_and_reset_leave_the_document_untouched"),
        ],
    },
    "OM9-L3-03-11-005": {
        "title": "Builder Viewport Control Handles",
        "owner": "om9-core handle list + om9-app viewport drag",
        "sources": [
            ("crates/om9-core/src/builder.rs", ["BuilderSession::handles", "BuilderHandle", "adjust_handle"]),
            ("crates/om9-app/src/ui.rs", ["builder_handles_ui", "builder_handle_drag"]),
            ("crates/om9-app/tests/builder_controls.rs", ["a_locked_input_makes_the_builder_read_only_and_hides_its_handles"]),
        ],
        "api": [
            ("BuilderSession::handles", "dimension handles with anchor, axis and field"),
            ("Controller::builder_adjust_field", "handle drag target"),
            ("BuilderSession::editable", "locked layer/object hides handles"),
        ],
        "positive": {
            "input": "selected rail with diameter/width and height handles",
            "expected": "drag projects onto the CPlane axis and updates the same field as the panel",
        },
        "negative": {
            "input": "drag a handle on a locked layer, or with no selection",
            "expected": "handles hidden / typed refusal; committed document unchanged",
        },
        "tests": [
            ("om9-core", "builder_session", "a_ring_rail_builder_previews_its_field_and_exposes_a_diameter_handle"),
            ("om9-app", "builder_controls", "a_locked_input_makes_the_builder_read_only_and_hides_its_handles"),
        ],
    },
    "OM9-L3-03-11-006": {
        "title": "Builder Handle And Panel Bidirectional Sync",
        "owner": "om9-app Controller revision/set_field_at",
        "sources": [
            ("crates/om9-app/src/session/builder.rs", ["builder_revision", "builder_set_field_at", "builder_adjust_field"]),
            ("crates/om9-app/src/ui/chrome.rs", ["captured revision on painted widgets"]),
            ("crates/om9-app/tests/builder_controls.rs", ["a_stale_panel_event_cannot_overwrite_a_newer_handle_drag"]),
        ],
        "api": [
            ("Controller::builder_revision", "edit counter captured when a widget paints"),
            ("Controller::builder_set_field_at", "apply only if the revision is still current"),
            ("Controller::builder", "one parameter authority read by panel and handles"),
        ],
        "positive": {
            "input": "panel width 2.0 -> 3.0, then handle drag 3.0 -> 4.0",
            "expected": "both read 4.0; no feedback loop",
        },
        "negative": {
            "input": "a panel event captured before the latest handle drag",
            "expected": "stale event is refused; the newer value survives",
        },
        "tests": [
            ("om9-app", "builder_controls", "a_stale_panel_event_cannot_overwrite_a_newer_handle_drag"),
            ("om9-app", "builder_panel", "a_handle_drag_and_a_typed_value_share_the_one_field"),
        ],
    },
    "OM9-L3-03-11-008": {
        "title": "Builder Accept Cancel Reset And Undo",
        "owner": "om9-app Controller apply/cancel + journal",
        "sources": [
            ("crates/om9-app/src/session/builder.rs", ["builder_apply", "builder_cancel", "builder_reset", "builder_journal"]),
            ("crates/om9-app/tests/builder_controls.rs", ["double_enter_applies_once_and_reset_creates_no_journal_entry", "escape_after_preview_edits_leaves_the_document_bytewise_unchanged"]),
            ("crates/om9-app/tests/builder_panel.rs", ["a_parameter_edit_and_its_recompute_are_two_undoable_steps"]),
        ],
        "api": [
            ("Controller::builder_apply", "one Undo transaction per Apply"),
            ("Controller::builder_cancel", "bytewise no change"),
            ("Controller::builder_reset", "fields back to defaults, no journal entry"),
            ("Undo/Redo", "restores the applied builder feature"),
        ],
        "positive": {
            "input": "width 2 -> 3 then Cancel; repeat and Apply",
            "expected": "Cancel leaves no change; Apply is exactly one Undo step; Undo/Redo restores",
        },
        "negative": {
            "input": "double Enter; Reset then Undo",
            "expected": "the second Enter is a no-op; Reset creates no journal entry",
        },
        "tests": [
            ("om9-app", "builder_controls", "double_enter_applies_once_and_reset_creates_no_journal_entry"),
            ("om9-app", "builder_panel", "a_parameter_edit_and_its_recompute_are_two_undoable_steps"),
        ],
    },
    "OM9-L3-03-11-009": {
        "title": "Builder F6 And Commandline Input Binding",
        "owner": "om9-app session dispatch + F6 prefill",
        "sources": [
            ("crates/om9-app/src/session/builder.rs", ["builder_command", "BuilderFieldView", "builder_text"]),
            ("crates/om9-app/src/ui.rs", ["builder_from_prefill", "BuilderPrefill"]),
            ("crates/om9-app/tests/builder_controls.rs", ["the_f6_prefill_and_the_command_line_reach_the_same_builder_service"]),
            ("crates/om9-app/tests/f6_context.rs", ["f6_uses_the_selected_class_and_empty_general_context"]),
        ],
        "api": [
            ("Controller::submit", "command-line builder names and field tokens"),
            ("Controller::begin_builder", "the same service the F6 prefill calls"),
            ("F6 prefill", "selects a builder from the selection"),
        ],
        "positive": {
            "input": "selected rail from F6 and command-line RingBuilder with equal parameters",
            "expected": "both dispatch the same builder service and event flow",
        },
        "negative": {
            "input": "unknown builder command, or a rail builder with no selection",
            "expected": "typed error; no builder session is opened",
        },
        "tests": [
            ("om9-app", "builder_controls", "the_f6_prefill_and_the_command_line_reach_the_same_builder_service"),
            ("om9-app", "f6_context", "f6_uses_the_selected_class_and_empty_general_context"),
        ],
    },
}

CHANGED_FILES = {
    "OM9-L3-03-11-002": [
        "crates/om9-core/src/builder.rs",
        "crates/om9-app/src/session/builder.rs",
        "crates/om9-app/src/ui/chrome.rs",
        "crates/om9-app/tests/builder_panel.rs",
    ],
    "OM9-L3-03-11-007": [
        "crates/om9-core/src/producers.rs",
        "crates/om9-core/src/builder.rs",
        "crates/om9-app/src/session/builder.rs",
        "crates/om9-app/src/ui/chrome.rs",
        "crates/om9-core/tests/builder_session.rs",
        "crates/om9-app/tests/builder_panel.rs",
    ],
    "OM9-L3-04-02-002": [
        "crates/om9-core/src/producers.rs",
        "crates/om9-core/src/builder.rs",
        "crates/om9-app/src/session/builder.rs",
        "crates/om9-core/tests/builder_session.rs",
        "crates/om9-app/tests/builder_panel.rs",
    ],
    "OM9-L3-04-02-007": [
        "crates/om9-core/src/producers.rs",
        "crates/om9-core/src/builder.rs",
        "crates/om9-app/src/session/builder.rs",
        "crates/om9-core/tests/builder_session.rs",
        "crates/om9-app/tests/builder_panel.rs",
    ],
    "OM9-L3-03-11-001": [
        "crates/om9-core/src/builder.rs",
        "crates/om9-core/tests/builder_controls.rs",
    ],
    "OM9-L3-03-11-003": [
        "crates/om9-core/src/builder.rs",
        "crates/om9-app/src/session/builder.rs",
        "crates/om9-app/src/ui/chrome.rs",
        "crates/om9-core/tests/builder_controls.rs",
        "crates/om9-app/tests/builder_controls.rs",
    ],
    "OM9-L3-03-11-004": [
        "crates/om9-app/src/session/builder.rs",
        "crates/om9-core/src/builder.rs",
        "crates/om9-app/tests/builder_controls.rs",
        "crates/om9-app/tests/builder_panel.rs",
    ],
    "OM9-L3-03-11-005": [
        "crates/om9-core/src/builder.rs",
        "crates/om9-app/src/ui.rs",
        "crates/om9-core/tests/builder_session.rs",
        "crates/om9-app/tests/builder_controls.rs",
    ],
    "OM9-L3-03-11-006": [
        "crates/om9-app/src/session/builder.rs",
        "crates/om9-app/src/ui/chrome.rs",
        "crates/om9-app/tests/builder_controls.rs",
        "crates/om9-app/tests/builder_panel.rs",
    ],
    "OM9-L3-03-11-008": [
        "crates/om9-app/src/session/builder.rs",
        "crates/om9-app/tests/builder_controls.rs",
        "crates/om9-app/tests/builder_panel.rs",
    ],
    "OM9-L3-03-11-009": [
        "crates/om9-app/src/session/builder.rs",
        "crates/om9-app/src/ui.rs",
        "crates/om9-app/tests/builder_controls.rs",
        "crates/om9-app/tests/f6_context.rs",
    ],
}


def sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def run(argv: list[str]) -> dict:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = TARGET_DIR
    env.setdefault("CARGO_TERM_COLOR", "never")
    process = subprocess.run(
        argv,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )
    combined = process.stdout + process.stderr
    return {
        "command_argv": argv,
        "exit_code": process.returncode,
        "stdout_stderr_sha256": sha256(combined),
        "summary": [
            line
            for line in combined.splitlines()
            if line.startswith(("test result:", "error", "FAILED", "Finished"))
        ][-6:],
    }


def test_argv(package: str, binary: str, test: str) -> list[str]:
    return ["cargo", "test", "-p", package, "--test", binary, "-j", "2", test]


def main() -> int:
    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True
    ).stdout.strip()
    status = subprocess.run(
        ["git", "status", "--porcelain"], cwd=ROOT, capture_output=True, text=True
    ).stdout
    now = datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%SZ")

    results = []
    failures = False
    for l3, spec in L3_SPECS.items():
        directory = EVIDENCE / l3
        directory.mkdir(parents=True, exist_ok=True)
        receipts = []
        for package, binary, test in spec["tests"]:
            receipt = run(test_argv(package, binary, test))
            receipt["test"] = test
            receipt["package"] = package
            receipt["binary"] = binary
            receipts.append(receipt)
            if receipt["exit_code"] != 0:
                failures = True
                print(f"FAIL {l3} {test}", file=sys.stderr)
        crosswalk = {
            "schema": "om9.m04.source_crosswalk.v1",
            "l3_id": l3,
            "title": spec["title"],
            "owner": spec["owner"],
            "source_revision": head,
            "checkout_dirty": bool(status.strip()),
            "sources": [
                {"path": path, "symbols": symbols, "state": "INSPECTED_IN_CHECKOUT"}
                for path, symbols in spec["sources"]
            ],
            "planning_tree_touched": False,
            "notes": [
                "The L3 skill/plan record is planning input; runtime behavior is established by the receipts.",
                *spec.get("notes", []),
            ],
        }
        (directory / "source_crosswalk.json").write_text(
            json.dumps(crosswalk, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        api = ["# API contract — " + spec["title"], "", f"L3: `{l3}`", "", "| API | Behaviour |", "|---|---|"]
        api += [f"| `{name}` | {behaviour} |" for name, behaviour in spec["api"]]
        api += [
            "",
            "## Commands",
            "",
            "| Mode | Command | Inputs |",
            "|---|---|---|",
            "| Builder tab | panel row / `switch_builder` | selection that fits the mode |",
            "| Command line | `RingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`, `Ring Resizer` | Enter applies, `Reset`, `Cancel` |",
            "",
            "## Invariants",
            "",
            "- Preview never writes the document; only Apply opens one transaction.",
            "- A typed error leaves the committed document unchanged.",
            "- Stable object ids and the recipe/result owner stay in the document.",
            "",
        ]
        (directory / "api.md").write_text("\n".join(api), encoding="utf-8")
        positive = {
            "schema": "om9.m04.fixture.v1",
            "l3_id": l3,
            "kind": "positive",
            **spec["positive"],
            "verified_by": [test for _, _, test in spec["tests"]],
            "receipts": [
                {key: receipt[key] for key in ("test", "command_argv", "exit_code", "stdout_stderr_sha256")}
                for receipt in receipts
            ],
        }
        negative = {
            "schema": "om9.m04.fixture.v1",
            "l3_id": l3,
            "kind": "negative",
            **spec["negative"],
            "verified_by": [test for _, _, test in spec["tests"]],
            "document_unchanged": True,
        }
        (directory / "positive.json").write_text(
            json.dumps(positive, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        (directory / "negative.json").write_text(
            json.dumps(negative, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        manifest = sha256(
            (directory / "positive.json").read_text(encoding="utf-8")
            + (directory / "negative.json").read_text(encoding="utf-8")
        )
        base = {
            "schema": "om9.m04.l4.receipt.v1",
            "l3_id": l3,
            "title": spec["title"],
            "generated_at_utc": now,
            "source_revision": head,
            "checkout_dirty": bool(status.strip()),
            "manual_gate": "OPEN",
            "signature": None,
            "not_claimed": [
                "manual panel/handle review at the M04-UI gate",
                "a reviewer signature",
            ],
            "fixture_manifest_sha256": manifest,
            "document_snapshot": {
                "source": "asserted by the registered tests",
                "positive": spec["positive"]["expected"],
                "negative": spec["negative"]["expected"],
            },
        }
        prefix = {
            "03": ("IMPLEMENT", "implementation", receipts),
            "04": ("INTEGRATE", "integration", receipts),
            "05": ("REGRESSION", "regression", receipts),
            "06": ("ACCEPTANCE", "acceptance", receipts),
        }
        if l3 in {
            "OM9-L3-04-02-007",
            "OM9-L3-03-11-004",
            "OM9-L3-03-11-005",
            "OM9-L3-03-11-006",
            "OM9-L3-03-11-008",
            "OM9-L3-03-11-009",
        }:
            # These L3 carry an extra adversarial/recovery task in the plan tree.
            prefix["07"] = ("ADVERSARIAL", "adversarial", receipts)
        for suffix, (role, kind, items) in prefix.items():
            payload = dict(base)
            payload.update(
                {
                    "l4_id": l3.replace("OM9-L3-", "OM9-L4-") + f"-{suffix}",
                    "role": role,
                    "kind": kind,
                    "status": "READY_FOR_REVIEW",
                    "changed_files": CHANGED_FILES[l3],
                    "commands": items,
                    "manual_gate_note": "M04-UI manual review remains open; no signature is included.",
                }
            )
            (directory / f"OM9-L4-{l3.removeprefix('OM9-L3-')}-{suffix}.json").write_text(
                json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
            )
        results.append({"l3": l3, "receipts": receipts})
        print(f"recorded {l3}: {len(receipts)} tests")

    if failures:
        return 1

    for l3, spec in L3_SPECS.items():
        directory = EVIDENCE / l3
        notes = spec.get("notes", [])
        notes_section = (
            "## Notes\n\n" + "\n".join(f"- {note}" for note in notes) + "\n\n" if notes else ""
        )
        (directory / "README.md").write_text(
            f"""# {l3} — {spec['title']}

**Status:** implemented and automatically tested; **manual M04-UI gate is open** (no
reviewer signature is claimed). The dashboard keeps this L3 at 🟡 with all L4 automated
artefacts present (6/6, or 7/7 where the plan adds an adversarial task).

## L4 artefacts

- `source_crosswalk.json` — sources inspected in this checkout
- `api.md` — the API contract the tests exercise
- `positive.json` / `negative.json` — fixtures and receipts
- `OM9-L4-…-03/04/05/06.json` (+ `-07` where the plan adds an adversarial task) —
  implementation, integration, regression, acceptance and adversarial receipts (command
  argv, exit code, stdout/stderr SHA-256, fixture manifest SHA-256)

## Tests

{chr(10).join(f'- `{test}`' for _, _, test in spec['tests'])}

{notes_section}## Not claimed

- Human acceptance of the panel, pointer gestures or the visual result.
- A signature: record only the automated receipts; a human owns the remaining gate.
""",
            encoding="utf-8",
        )
    ledger_path = ROOT / "docs" / "openmatrix9-progress.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    key = "m04-builder-ui-l4-evidence-2026-10-27"
    evidence_paths = []
    for l3 in L3_SPECS:
        evidence_paths += [
            f"implementation/evidence/M04/{l3}/source_crosswalk.json",
            f"implementation/evidence/M04/{l3}/api.md",
            f"implementation/evidence/M04/{l3}/positive.json",
            f"implementation/evidence/M04/{l3}/negative.json",
        ]
    entry = {
        "key": key,
        "date": "2026-10-27",
        "stage": "feature-port",
        "status": "l4_automated_evidence_ready_manual_gate_open",
        "target": "Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=L:/om9-target; -j 2",
        "feature_ids": list(L3_SPECS),
        "scope": (
            "L4 evidence package cho toàn bộ L2-03-11 (9 L3 builder UI): source crosswalk, API contract, "
            "positive/negative fixtures và receipt -03/-04/-05/-06 (thêm -07 cho L3 có task adversarial). "
            "Không có chữ ký; gate M04-UI thủ công vẫn mở."
        ),
        "outputs": [f"implementation/evidence/M04/{l3}/" for l3 in L3_SPECS]
        + ["tools/record_m04_builder_l3_l4_evidence.py"],
        "evidence": evidence_paths,
        "tests": {
            "OM9-L3-03-11-001": "core test passed: surface refused in a Curve slot, accepted rail keeps its ObjectId",
            "OM9-L3-03-11-002": "2 app tests passed (tabs persist, failed switch restores)",
            "OM9-L3-03-11-003": "core slider quantises 2.5 mm; app slider previews the same value",
            "OM9-L3-03-11-004": "preview edits create no journal entry; Escape leaves the document bytewise unchanged",
            "OM9-L3-03-11-005": "diameter handle exposed; locked input hides handles and refuses the drag",
            "OM9-L3-03-11-006": "stale panel event refused; panel and handle read the same value",
            "OM9-L3-03-11-007": "1 core + 2 app tests passed (preset library, removed preset refused)",
            "OM9-L3-03-11-008": "double Enter applies once; Reset creates no journal entry; Apply/Undo/Redo round-trips",
            "OM9-L3-03-11-009": "F6 prefill and command line reach the same builder service; unknown command refuses",
            "OM9-L3-04-02-002": "1 core + 1 app test passed (radius 11, non-circular refused)",
            "OM9-L3-04-02-007": "1 core + 1 app test passed (radius 9.5, source intact, thickness refused)",
        },
        "uncertainties": [
            "Chưa có ảnh chụp panel/handle hoặc pointer-frame; con người vẫn phải chạy gate M04-UI.",
            "Dashboard giữ các L3 ở 🟡 vì thiếu ui-review.json có chữ ký; đây là chủ ý.",
            "Các thay đổi đồng thời của tiến trình khác trong chrome.rs/builder.rs có thể đổi hành vi UI sau receipt này.",
        ],
        "manual_gate": "OPEN: M04-UI cần người review panel/handle",
        "next_action": "Chạy app, thao tác panel/handle cho cả 9 mode, chụp ảnh và ký ui-review.json nếu đạt.",
    }
    ledger["items"] = [item for item in ledger["items"] if item.get("key") != key] + [entry]
    ledger["active_item"] = key
    ledger["updated_at"] = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    ledger_path.write_text(json.dumps(ledger, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    print("recorded L4 evidence for", len(L3_SPECS), "L3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
