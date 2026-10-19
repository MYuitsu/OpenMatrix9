"""Synchronize measured Layers widget scope without promoting full acceptance."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "OpenMatrix9_Codex_Spec_v2"
RECEIPT = "docs/validation/2026-10-14-owned-layers-ui.json"
REPORT = "docs/validation/2026-10-14-owned-layers-ui.md"


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def write(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def append_once(path, marker, section):
    text = path.read_text(encoding="utf-8")
    if marker not in text:
        path.write_text(text.rstrip() + "\n\n" + section + "\n", encoding="utf-8")


def main():
    receipt = json.loads((ROOT / RECEIPT).read_text(encoding="utf-8"))
    assert receipt["status"] == "validated_owned_layers_controller_widgets_scene_slice"
    assert not receipt["goal_complete"] and not receipt["full_layers_acceptance"]
    for path, sha in receipt["source_sha256"].items():
        assert digest(ROOT / path) == sha, path
    for row in receipt["native_ui"]:
        assert digest(ROOT / row["path"]) == row["sha256"]
        assert digest(ROOT / row["binary"]["path"]) == row["binary"]["sha256"]
        assert digest(ROOT / row["screenshot"]["path"]) == row["screenshot"]["sha256"]
    baseline = json.loads((PACKAGE / "SOURCE_READ_AUDIT.json").read_text(encoding="utf-8"))
    for row in baseline["files"]:
        path = ROOT / "OpenMatrix9_Codex_Spec_v1" / row["path"]
        assert path.stat().st_size == row["bytes"] and digest(path) == row["sha256"], row["path"]
    passed, ignored, focused = receipt["workspace_passed"], receipt["workspace_ignored"], receipt["focused_tests"]
    nested_peaks = [row["nested_peak"] for row in receipt["scene_allocation_peaks"]]
    cloud_peaks = [row["cloud_peak"] for row in receipt["scene_allocation_peaks"]]
    report = f"""# Owned Layers panel/CMD/scene checkpoint — 2026-10-14

[Measured source-bound receipt](2026-10-14-owned-layers-ui.json) covers a part of
Task3 in [the complete Layers plan](../superpowers/plans/2026-10-14-owned-layers.md).
Full OM9-LAYER-001, M9IMG-04 and RCORE M01–M12 remain open.

Safe Rust owns current layer, assignment, lazy groups, selection, Hidden and
permissions. Basic rows keep Metal/Gem at left and User01–04/Heads/Finger/Cutting/
Creation at right. Advanced is collapsed by default, expands in the screenshot
order, and keeps stable preset IDs independently of labels (slot21 is User25).
Lights is slot33. Name/swatch left-click chooses current; arrow left-click
assigns selection without changing current; swatch right-click replaces eligible
visible/selectable/drawable membership. Locks, visibility and group bars preserve
explicit object visibility/selectability/Hidden. Hide sets only Hidden, reverse
Hide uses visible drawable roots outside the selected set, and Show restores
only Hidden. Controls use authored vector shapes so native missing-font glyphs
cannot replace triangles/locks/visibility/carets with tofu boxes.

`Layer=1` through `Layer=33`, canonical names, Lights and None/0 route through
the same owned controller. Unquoted numeric/None/0 tokens retain their reserved
meaning; quoted names first resolve a unique existing case-insensitive label.
Thus an already-valid Advanced custom label such as `\"1\"` or `\"None\"` is
addressable without weakening stable slot syntax or banning old metadata.
The parser is bounded to256 bytes and rejects control characters/unbalanced
quotes. Pending foundation Point/Line sessions retain start point, CPlane,
BothSides, camera and input draft across a successful owned layer action;
current is read again at actual output acceptance. Off accepts invisible output,
lock refuses output without losing the draft. Assign creates a missing group
and applies membership atomically in either request order with one Undo.

Controller selection is admitted before replacing its owned tree. Membership
uses two borrowed passes grouped by root, not one whole-scene scan per root;
all-hidden Cloud buffers and hidden definition leaves do not create selectable
phantoms. Selection-source collection checks max_outputs and held world/cache/
selection storage before allocation. Scene metadata refresh proves unchanged
identity/context/geometry/placement, reuses immutable display buffers and
refreshes inherited masks/colors/stamps. Old and prospective cache metadata,
world and Undo states remain held in admission before publication.

Scene startup and Undo/Redo refresh now preflight expanded occurrences, cache
buffers, paths, diagnostics and scratch before allocation. Every Instance
occurrence is charged before descent, including empty shared definitions:
100 roots over seven binary levels have25,500 visits even with zero drawable
leaves and are refused at max_nodes2000 while the502-node catalog validates.
Scene allocations are reserved before filling so moving Vec reallocation does
not add an unadmitted old/new coexistence peak. These fixture bounds are not
global GPU/cross-job memory or full performance certification.
The isolated test-only allocator forces alloc/copy/free on realloc and measures
the whole test executable heap: nested 4,096-leaf peaks {min(nested_peaks):,}–
{max(nested_peaks):,} bytes under 2,500,000; Cloud 65,537-point peaks
{min(cloud_peaks):,}–{max(cloud_peaks):,} bytes under 5,000,000. Independent review
reproduced the moving-realloc RED 2,872,138 before this fix and GREEN 2,186,682
after it, plus initial 1.9-MB refusal, 3-MB publication refusal, empty-DAG budget
and hidden Cloud selection. Its 18 checks are separate review probes and are
not added to the portable workspace total.

The native replay uses the real egui controls and actual CMD through this
application's raw-input hook; it never injects desktop input into other apps.
Thirteen Layers checks verify pending Line/current/output, swatch selection,
lazy Assign, Hide/Show, group Off/On, Lock/Unlock, Advanced/collapse and actual
CMD Undo/Redo restoring the same preset UUIDs. Removing a duplicate running
command row from Main Menu restores the approved CMD-only prompt location and
keeps layer hit regions stable when a command accepts. A widget regression
checks that stability. Six native viewport-menu checks were rerun for popup,
numeric draft, Maximize/Restore, Escape and outside dismissal.
The S6 popup uses a light background with dark enabled labels, fixed content
width and work-area height; it scrolls in short windows. Native capture waits
for the opening animation to settle. When the tall menu overlaps its anchor,
the replay renders one closed frame before the next gesture so egui's prior
Area hit regions have expired; the production action/ownership gates are
unchanged. The receipt requires the popup still open when capturing. Actual
native PNG inspection confirms the complete S6 list through Viewport Properties.
These checks do not establish immediate reopening with no closed paint frame.

Measured: **{passed} workspace tests / {ignored} ignored SDK tests**;
**{focused} focused checks are a workspace subset**. Format, warning-strict
all-target Clippy and release build passed. Two own-app 1600×1000 native windows
produced source/binary/screenshot-bound receipts, observed window handles and
loaded modules, with no descendants or forbidden legacy host dependencies.
Four bounded cold OM9 children from the workspace suite exited cleanly and
verify owned metadata/schema migration, not interactive save/reopen acceptance.
Older native SDK/schema-policy receipts retain their historical source checkpoints;
there is no fresh combined SDK count in this UI batch.

Pending: Advanced rename/color and selective-Show delayed dialogs (currently
visible-unavailable), their Cancel/stale/owner lifecycle; current-output binding
for other command families; complete keyboard/DPI/IME behavior, interactive
cold-save workflow, structural/native family combinations, full large-scene
latency/global memory and all complete Layers/M9IMG/RCORE acceptance gates.
Synchronous scene capture is traversal-bounded but has no cancellation token;
general async cancellation remains pending.
No full feature/capability/catalog implementation status is promoted.
"""
    (ROOT / REPORT).write_text(report, encoding="utf-8")
    meta = {"scope": "owned_layers_controller_widgets_scene_only", "receipt": "../" + RECEIPT,
            "report": "../" + REPORT, "workspace_tests": passed, "workspace_ignored_sdk": ignored,
            "focused_subset": focused, "cold_om9_processes": 4, "native_layers_checks": 13,
            "native_viewport_regression_checks": 6, "full_feature_acceptance": False}
    ledger = PACKAGE / "MIGRATION_LEDGER.json"
    data = json.loads(ledger.read_text(encoding="utf-8"))
    data["foundation_implementation"]["owned_layers_ui"] = meta
    write(ledger, data)
    for filename in ["FEATURES.json", "FEATURE_LOOKUP.json"]:
        path = PACKAGE / filename
        data = json.loads(path.read_text(encoding="utf-8"))
        feature = next(f for f in data if f["id"] == "OM9-LAYER-001") if isinstance(data, list) else data["OM9-LAYER-001"]
        assert feature["implementation_status"] == "not_implemented_on_standalone_om9"
        feature["standalone_layers_ui_evidence"] = meta
        write(path, data)
        if filename == "FEATURES.json":
            (PACKAGE / "FEATURES.yaml").write_text(yaml.safe_dump(data, allow_unicode=True, sort_keys=False, width=120), encoding="utf-8")
    marker = "## Standalone Layers widget evidence — 2026-10-14"
    append_once(PACKAGE / "specs/01-core/om9-layer-001-layer-management.md", marker,
                marker + f"\n\nCurrent/CMD/lazy Assign, basic/Advanced/Lights controls, selection and Hidden/permission actions now have [measured widget/scene evidence](../../../{REPORT}). {passed} portable workspace tests/{focused} focused subsets;13 native Layers checks and6 native viewport regressions. Advanced rename/color/selective-Show dialogs and complete feature acceptance remain open. For an existing custom label equal to a reserved CMD token, quote the label; unquoted numeric/None/0 keeps reserved slot/current semantics. Older layer primitives/schema receipts retain historical source checkpoints.")
    screenshot = PACKAGE / "specs/00-rhino-core/MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md"
    append_once(screenshot, "## Bằng chứng panel Layers — 2026-10-14",
                "## Bằng chứng panel Layers — 2026-10-14\n\nM9IMG-04 có thêm [bằng chứng panel/CMD/selection/scene](../../../" + REPORT + f"): {passed} workspace tests,13 thao tác Layers qua cửa sổ native và6 hồi quy menu viewport. Advanced rename/color/selective Show còn unavailable; toàn bộ case M9IMG-04 vẫn mở. Bằng chứng schema/policy và menu trước đó ở checkpoint lịch sử; không coi tổng số kiểm tra là số feature đã nghiệm thu.")
    note = f"Owned Layers widget checkpoint: {passed} workspace tests/{focused} focused subsets;13 actual native Layers checks and6 viewport-menu regressions, four bounded cold OM9 children. [Measured scope](../{REPORT}). Advanced dialogs, other command families and full Layers/M9IMG/RCORE gates remain open; earlier SDK/UI receipts retain historical checkpoints."
    for path in [PACKAGE / "IMPLEMENTATION_STATUS.md", PACKAGE / "README.md"]:
        append_once(path, "Owned Layers widget checkpoint:", note)
    for path in [ROOT / "README.md", ROOT / "README.en.md"]:
        append_once(path, "Owned Layers widget checkpoint:", note.replace("../" + REPORT, REPORT))
    for name in ["specs/00-rhino-core/00-qymcad-foundation-migration.md", "specs/00-rhino-core/UI_DETAILS.md"]:
        append_once(PACKAGE / name, "Owned Layers widget checkpoint:", note.replace("../" + REPORT, "../../../" + REPORT))
    progress = ROOT / "docs/openmatrix9-progress.json"
    data = json.loads(progress.read_text(encoding="utf-8"))
    data["updated_at"] = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")
    item = next(row for row in data["items"] if row.get("key") == "rcore00-standalone-foundation")
    for path in [RECEIPT, REPORT, "crates/om9-app/src/session/layers.rs", "crates/om9-app/src/ui/layers.rs"]:
        if path not in item["outputs"]:
            item["outputs"].append(path)
    prefix = "Owned Layers controller/widgets;"
    item["evidence"] = [row for row in item["evidence"] if not row.startswith(prefix)]
    item["evidence"].append(prefix + f" {passed} workspace/{focused} focused subsets;13 native Layers and6 viewport-menu checks; four bounded cold OM9 children. Atomic lazy current/Assign, owned selection/Hidden/permissions, Point/Line output acceptance and pre-admitted scene cache. Full Layers and RCORE gates remain open; older SDK checkpoints are historical.")
    item["next_action"] = "Implement approved Advanced rename/color and selective-Show delayed dialogs with Cancel/stale guards; verify actual controls and cold state, then continue every full RCORE-00/screenshot requirement."
    write(progress, data)
    roadmap = ROOT / "docs/validation/opennurbs-test-roadmap.json"
    data = json.loads(roadmap.read_text(encoding="utf-8"))
    data["owned_layers_ui_checkpoint"] = {**meta, "new_sdk_jobs": 0}
    write(roadmap, data)
    append_once(ROOT / "docs/validation/opennurbs-test-roadmap.md", "Owned Layers widget checkpoint:",
                note.replace("../" + REPORT, "2026-10-14-owned-layers-ui.md") + " This is a UI/portable batch, with zero new SDK jobs; existing native package gaps and the unknown total future batch count remain unchanged.")
    append_once(ROOT / "docs/superpowers/plans/2026-10-14-owned-layers.md", "Task3 scoped checkpoint:",
                f"Task3 scoped checkpoint: {passed} workspace/{focused} focused subsets;13 native Layers checks plus6 viewport-menu regressions and four cold children. Basic/Advanced/Lights/current/CMD/lazy Assign/selection/Hidden/permission widgets are connected. Delayed rename/color/selective-Show dialogs remain open; [evidence](../../validation/2026-10-14-owned-layers-ui.json). Full Layers/RCORE acceptance is unchanged.")
    manifest = []
    for path in sorted(PACKAGE.rglob("*")):
        if path.is_file() and path.name != "MANIFEST.json" and not {"target", "__pycache__"}.intersection(path.parts):
            content = path.read_bytes()
            manifest.append({"path": path.relative_to(PACKAGE).as_posix(), "bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()})
    write(PACKAGE / "MANIFEST.json", manifest)
    print(json.dumps({"workspace_tests": passed, "focused_subset": focused,
                      "native_ui_checks": [13, 6], "v1_unchanged": len(baseline["files"]),
                      "v2_manifest": len(manifest), "goal_complete": False}))


if __name__ == "__main__":
    main()
