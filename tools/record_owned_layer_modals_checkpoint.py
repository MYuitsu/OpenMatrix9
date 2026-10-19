"""Record scoped modal evidence, preserving v1 and all full-acceptance gates."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "OpenMatrix9_Codex_Spec_v2"
RECEIPT = "docs/validation/2026-10-14-owned-layer-modals.json"
REPORT = "docs/validation/2026-10-14-owned-layer-modals.md"
MARKER = "## Owned Layers modal checkpoint — 2026-10-14"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def append_once(path, section):
    text = path.read_text(encoding="utf-8")
    if MARKER not in text:
        path.write_text(text.rstrip() + "\n\n" + section + "\n", encoding="utf-8")


def main():
    receipt = json.loads((ROOT / RECEIPT).read_text(encoding="utf-8"))
    assert receipt["status"] == "validated_owned_layer_modals_slice"
    assert not receipt["goal_complete"] and not receipt["full_layers_acceptance"]
    for path, sha in receipt["source_sha256"].items():
        assert digest(ROOT / path) == sha, path
    for row in receipt["native_ui"]:
        assert digest(ROOT / row["path"]) == row["sha256"]
        assert digest(ROOT / row["binary"]["path"]) == row["binary"]["sha256"]
        for image in [row["screenshot"], *row["dialog_screenshots"]]:
            assert digest(ROOT / image["path"]) == image["sha256"]
    baseline = json.loads((PACKAGE / "SOURCE_READ_AUDIT.json").read_text(encoding="utf-8"))
    for row in baseline["files"]:
        path = ROOT / "OpenMatrix9_Codex_Spec_v1" / row["path"]
        assert path.stat().st_size == row["bytes"] and digest(path) == row["sha256"]
    passed, ignored, focused = receipt["workspace_passed"], receipt["workspace_ignored"], receipt["focused_tests"]
    publication = [row["publication_peak"] for row in receipt["modal_allocation_peaks"]]
    snapshot = [row["snapshot_peak"] for row in receipt["modal_allocation_peaks"]]
    report = f"""# Owned Layers modal checkpoint — 2026-10-14

[Source-bound receipt](2026-10-14-owned-layer-modals.json) verifies the
Task3 dialog continuation in [the Layers plan](../superpowers/plans/2026-10-14-owned-layers.md).
This supersedes the earlier unavailable-dialog limitation for the implemented
owned-root slice. Full OM9-LAYER-001, M9IMG-04 and RCORE M01–M12 remain open.

Safe Rust owns a Rename/Color/Show draft and opaque dialog token. The token
binds coordinator authority/context, document identity/session/epoch/generation/
revision/context revision and a monotonic serial. It is checked on every edit
and affirmation. Foreign, consumed, replaced-session and Undo-roundtrip handles
cannot mutate or close a newer owner; cancel releases only the exact active
handle. Opening, editing and cancellation do not materialize lazy layers or
write model/Undo. Pending foundation Line start/preview and numeric CMD draft
survive the dialogs. Actual native replay also verifies all four cameras remain
unchanged. Broader document switching through a native open-file UI is pending.

Shift+right-click Advanced name opens Rename, and Advanced swatch opens Color;
only slots17..32 allow them. Rename trims and reuses the existing128 UTF-8 byte,
nonempty/control-free/unique/reserved-name policy. UI draft storage is bounded
to512 bytes; invalid input stays editable. Color uses bounded three-character
integer RGB fields0..255, with Up/Down stepping one unit. Invalid RGB cannot
silently affirm; existing alpha is retained. Affirmation creates a missing group
and changes its target atomically in one Undo, retaining preset slot/UUID/current
and membership. By-layer scene color updates reuse immutable geometry buffers.
Accepted no-op Rename/Color/empty Show closes without allocating action buffers,
changing Undo or erasing Redo, including a tight-budget fixture.

Right Show owns an unchecked list of captured OM9-Hidden roots. Choosing entries
and OK clears only those roots' Hidden flag; Cancel writes nothing. Explicit
visibility/selectability, layer Off/Lock, unrelated Hidden roots, canvas selection
and current remain independent. Each row routes through captured SourceStamp and
object identity even when labels duplicate. Definition-member Hidden masks stay
independent; complete structural/imported/builder families are not certified.
The initially unchecked choice policy is an OM9 ruling; screenshots alone do not
establish the original dialog's default selection.

Snapshot vectors/string capacity, current dialog and bounded UI input/error
reserve are held in controller scene admission before allocations and publication.
max_outputs rejects oversized hidden lists before copying labels. The isolated
test-only allocator forces moving realloc and measures the whole test heap:
held-modal publication refuses without changing epoch/draft/model at
{min(publication):,}–{max(publication):,} bytes under2,500,000;1,000 hidden roots
with8,192-byte labels refuse snapshot allocation at{min(snapshot):,}–{max(snapshot):,}
bytes under14,000,000. These fixture bounds do not certify global egui/GPU caches.

Actual widget regressions reproduced and fixed three reviewed defects:
7,700-byte/multiline labels formerly produced1,664/13,013px rows while virtualization
assumed18px; clipped single-line rows now stay17px with a clickable label/checkbox
and preserve original text/stamp. Repeated modal serials formerly retained an
Area/ScrollState per open; one reusable area/scroll owner now remains at2/6 total
states after1/10/30 closed cycles, resetting scroll for each new draft. Action
button/choice IDs still include authority+serial: releasing an old OK after cancel/
reopen cannot affirm the new token. Token-specific editor state is removed on close,
while CMD editor state survives. Independent review reproduced each RED/GREEN.

Measured: **{passed} portable workspace tests / {ignored} ignored SDK tests**;
**{focused} focused checks are a workspace subset**. Warning-strict all-target
Clippy, format and release build passed. Eleven controller and19 actual-widget
tests cover stale/foreign/cancel/no-op/invalid/IME-event/budget paths. Four cold
fixture tests spawn five bounded own OM9 children; each exits cleanly. The new
fixture commits actual dialogs, checks Undo/Redo, saves/reopens `.om9` in a fresh
CLI child and rejects an old modal handle after same-file reopening. This is not
an accepted interactive native Save/Open workflow.

Release native window replays pass10 dialog,13 earlier Layers and6 viewport-menu
checks. Receipts bind source hashes, executable, screenshot, observed native
window/modules and no descendants/forbidden legacy hosts. Compiled SVG inputs
are included in the new source freeze. Three screenshots capture Rename, Color
and Show while open; the final closed workspace screenshot is separate. The own-
app raw-input hook exercises real egui widgets/CMD; it never injects desktop
input into another application. Debug PNG encoding measured879–967ms, exceeding
egui's0.8-second click limit when it landed between replay press/release. Replay
now waits for each screenshot save acknowledgment before its next gesture;
production click handling is unchanged. This audit path is not performance proof.

Pending: output-layer binding for command families beyond foundation Point/Line;
full structural/native/member behavior, MatrixArt/material, interactive save/open,
OS IME/DPI/multi-monitor, full keyboard navigation, async scene cancellation,
global cross-job/render-cache budgets and large-scene latency. No new SDK jobs
were run; earlier SDK receipts retain their historical checkpoints. No complete
feature, capability or migration gate is promoted by this checkpoint.
"""
    (ROOT / REPORT).write_text(report, encoding="utf-8")
    meta = {"scope": "owned_layers_delayed_modals_only", "receipt": "../" + RECEIPT,
            "report": "../" + REPORT, "workspace_tests": passed, "workspace_ignored_sdk": ignored,
            "focused_subset": focused, "cold_om9_processes": receipt["cold_om9_processes"],
            "native_dialog_checks": 10, "native_layers_regression_checks": 13,
            "native_viewport_regression_checks": 6, "full_feature_acceptance": False}
    ledger = PACKAGE / "MIGRATION_LEDGER.json"
    data = json.loads(ledger.read_text(encoding="utf-8"))
    data["foundation_implementation"]["owned_layers_modals"] = meta
    write(ledger, data)
    for filename in ["FEATURES.json", "FEATURE_LOOKUP.json"]:
        path = PACKAGE / filename
        data = json.loads(path.read_text(encoding="utf-8"))
        feature = next(row for row in data if row["id"] == "OM9-LAYER-001") if isinstance(data, list) else data["OM9-LAYER-001"]
        assert feature["implementation_status"] == "not_implemented_on_standalone_om9"
        feature["standalone_layers_modals_evidence"] = meta
        write(path, data)
        if filename == "FEATURES.json":
            (PACKAGE / "FEATURES.yaml").write_text(yaml.safe_dump(data, allow_unicode=True, sort_keys=False, width=120), encoding="utf-8")
    text = (MARKER + f"\n\nOwned Advanced Rename/Color and selective Show now have [measured modal evidence](../../../{REPORT}): "
            f"{passed} workspace tests/{focused} focused subsets;10 native dialog checks plus13 Layers and6 viewport regressions;five bounded cold OM9 children. "
            "The earlier unavailable-dialog limitation is historical for this owned-root slice. Token/session/stale guards, Cancel/no-op/Undo/Redo, "
            "bounded RGB and snapshot storage, fixed single-line rows and reusable UI owners are verified. "
            "Full Layer output families, interactive Save/Open, OS IME/DPI, async capture/global budgets and complete Layers/M9IMG/RCORE gates remain open.")
    for name in ["specs/01-core/om9-layer-001-layer-management.md", "specs/00-rhino-core/MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md",
                 "specs/00-rhino-core/00-qymcad-foundation-migration.md", "specs/00-rhino-core/UI_DETAILS.md"]:
        append_once(PACKAGE / name, text)
    note = text.replace("../../../" + REPORT, "../" + REPORT)
    for path in [PACKAGE / "IMPLEMENTATION_STATUS.md", PACKAGE / "README.md"]:
        append_once(path, note)
    for path in [ROOT / "README.md", ROOT / "README.en.md"]:
        append_once(path, text.replace("../../../" + REPORT, REPORT))
    progress = ROOT / "docs/openmatrix9-progress.json"
    data = json.loads(progress.read_text(encoding="utf-8"))
    data["updated_at"] = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")
    item = next(row for row in data["items"] if row.get("key") == "rcore00-standalone-foundation")
    for path in [RECEIPT, REPORT, "crates/om9-app/src/layer_dialog.rs",
                 "crates/om9-app/src/session/layer_dialogs.rs", "crates/om9-app/src/ui/layer_dialogs.rs"]:
        if path not in item["outputs"]:
            item["outputs"].append(path)
    prefix = "Owned Layers modal checkpoint;"
    item["evidence"] = [row for row in item["evidence"] if not row.startswith(prefix)]
    item["evidence"].append(prefix + f" {passed} workspace/{focused} focused subsets;10 native dialogs+13 Layers+6 viewport regressions;five cold OM9 children. Complete gates remain open; no new SDK jobs.")
    item["next_action"] = "Continue every RCORE-00/screenshot requirement: owned interactive save/open workflow, full output families and structural/native coverage, OS input/DPI, global budget/cancellation/performance gates; preserve partial acceptance."
    write(progress, data)
    roadmap = ROOT / "docs/validation/opennurbs-test-roadmap.json"
    data = json.loads(roadmap.read_text(encoding="utf-8"))
    data["owned_layers_modals_checkpoint"] = {**meta, "new_sdk_jobs": 0}
    write(roadmap, data)
    append_once(ROOT / "docs/validation/opennurbs-test-roadmap.md", text.replace("../../../" + REPORT, "2026-10-14-owned-layer-modals.md"))
    plan = ROOT / "docs/superpowers/plans/2026-10-14-owned-layers.md"
    content = plan.read_text(encoding="utf-8")
    prefix, continuation = content.split("## Task3 dialog continuation", 1)
    # Only this fully verified continuation is checked; broad feature tasks stay open.
    content = prefix + "## Task3 dialog continuation" + continuation.replace("- [ ]", "- [x]")
    plan.write_text(content, encoding="utf-8")
    append_once(plan, text.replace("../../../" + REPORT, "../../validation/2026-10-14-owned-layer-modals.md"))
    manifest = []
    for path in sorted(PACKAGE.rglob("*")):
        if path.is_file() and path.name != "MANIFEST.json" and not {"target", "__pycache__"}.intersection(path.parts):
            content = path.read_bytes()
            manifest.append({"path": path.relative_to(PACKAGE).as_posix(), "bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()})
    write(PACKAGE / "MANIFEST.json", manifest)
    print(json.dumps({"workspace": passed, "focused_subset": focused, "native_checks": [10, 13, 6],
                      "cold_om9_processes": receipt["cold_om9_processes"], "v1_unchanged": len(baseline["files"]),
                      "v2_manifest": len(manifest), "goal_complete": False}))


if __name__ == "__main__":
    main()
