"""Record measured schema/policy scope; leave complete Layers/RCORE gates open."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "OpenMatrix9_Codex_Spec_v2"
RECEIPT = "docs/validation/2026-10-14-owned-layer-state.json"
REPORT = "docs/validation/2026-10-14-owned-layer-state.md"


def write(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def append_once(path, marker, section):
    text = path.read_text(encoding="utf-8")
    if marker not in text:
        path.write_text(text.rstrip() + "\n\n" + section + "\n", encoding="utf-8")


def main():
    receipt = json.loads((ROOT / RECEIPT).read_text(encoding="utf-8"))
    assert receipt["status"] == "validated_owned_layer_state_schema4_policy_projection_slice"
    assert receipt["archive_schema"] == 4 and not receipt["goal_complete"] and not receipt["full_layers_acceptance"]
    for path, sha in receipt["source_sha256"].items():
        assert digest(ROOT / path) == sha, path
    assert digest(ROOT / receipt["binary"]["path"]) == receipt["binary"]["sha256"]
    native = receipt["native_export_receipt"]
    assert digest(ROOT / native["path"]) == native["sha256"]
    audit = json.loads((PACKAGE / "SOURCE_READ_AUDIT.json").read_text(encoding="utf-8"))
    for row in audit["files"]:
        path = ROOT / "OpenMatrix9_Codex_Spec_v1" / row["path"]
        assert path.stat().st_size == row["bytes"] and digest(path) == row["sha256"], row["path"]
    passed = receipt["workspace_passed"]
    ignored = receipt["workspace_ignored"]
    focused = receipt["focused_tests"]
    peaks = [row["peak_committed_bytes"] for row in receipt["cold_fixtures"]]
    report = f"""# Owned layer state/schema policy — 2026-10-14

Measured [source-bound receipt](2026-10-14-owned-layer-state.json) extends Task1
with a scoped part of Task2 in [the full Layers plan](../superpowers/plans/2026-10-14-owned-layers.md).
Full OM9-LAYER-001, M9IMG-04 and RCORE M01–M12 remain open.

Safe Rust now owns durable preset slots 1–33 independently of UUID/display label,
using the existing local OM9 names/colors with explicit LGPL provenance. Basic
and Lights presets retain their fixed palette/name; only Advanced17–32 admit
custom labels/colors. Slots are unique, immutable across Update and range checked.
Names enforce trim/control/128-byte/reserved-name rules; streamed Unicode lower
case comparisons reject collisions with other layers, including two proposed
renames in one batch before any private clone. Generic imported duplicate labels
remain valid unless they collide with an Advanced custom label. No old layer
is assigned a slot merely from its name or table order.

`Attributes.hidden` is independent of explicit visible/selectable, layer Off and
lock. Transactional Hidden changes preserve those preferences and use the same
session/source generation, attribute revision, Cancel/Undo/Redo and private
publication barriers. Scene tests cover root and definition-member masks for
display, picking, edit and reference; Show restores only OM9 Hidden.

`.om9` schema4 saves the slot and independent Hidden. Opening schema1/2/3 requires
the corresponding explicit migration. Legacy preflight rejects new fields even
false/null in root/member attributes and layers, while preserving Cloud.hidden
and unrelated geometry/recipe fields. Cancel, budget and restoration counter
overflow leave the original file unchanged. Actual `om9 --migrate-v3` followed
by a fresh owned process reopens schema4 without inferred preset identity.

Native schema1 geometry wire remains closed: borrowed serialization omits preset
and Hidden fields, projects object visibility as explicit visible && !Hidden,
and never edits the OM9 document. Actual SDK write/fresh inspection/publication
of a hidden root Point verifies that projection. Export receipts expose counts
for dropped independent Hidden distinctions/preset slots. Diagnostics admit
the live JSON and all response strings/vector storage before allocation.
Retained native projection uses the same visibility rule; its wider cold suite
is historical and is not newly accepted by this batch.

Measured: **{passed} workspace tests**, **{ignored} ignored SDK tests**;
**{focused} focused checks are included in the workspace total**. Format and
warning-strict all-target Clippy passed. Four actual owned OM9 children exited
with no active descendants, under a 10-second/64-KiB stdout cap; measured private
commit peaks {min(peaks):,}–{max(peaks):,} bytes for these small authored fixtures.
The separate [native export receipt](2026-10-14-owned-layer-state-exchange.json)
runs **{native['tests']} SDK integration tests / {native['jobs']} jobs**, outside
the workspace portable count; every job was reaped. Independent review ran
30 focused checks and reproduced the two corrected admission defects.

Pending: pure CMD/group action routing, current output rebinding at actual
command acceptance, complete Layers widgets and gestures, selection and delayed
rename/color/selective-Show dialogs, Advanced/Lights UI, native hidden definition
members/locked-hidden combinations, whole family/Rhino/performance/global-memory
acceptance. Older native/UI and broad foundation receipts retain their prior
source checkpoints; no combined current-source SDK total is claimed here.
No full feature/capability status or acceptance checklist is promoted.
"""
    (ROOT / REPORT).write_text(report, encoding="utf-8")
    meta = {"scope": "owned_layer_state_schema4_policy_projection_only",
            "receipt": "../" + RECEIPT, "report": "../" + REPORT,
            "archive_schema": 4, "workspace_tests": passed, "workspace_ignored_sdk": ignored,
            "focused_subset": focused, "cold_om9_processes": 4,
            "separate_native_tests": native["tests"], "separate_native_jobs": native["jobs"],
            "full_feature_acceptance": False}
    ledger = PACKAGE / "MIGRATION_LEDGER.json"
    data = json.loads(ledger.read_text(encoding="utf-8"))
    data["foundation_implementation"]["owned_layer_state_schema4"] = meta
    write(ledger, data)
    for filename in ["FEATURES.json", "FEATURE_LOOKUP.json"]:
        path = PACKAGE / filename
        data = json.loads(path.read_text(encoding="utf-8"))
        feature = next(f for f in data if f["id"] == "OM9-LAYER-001") if isinstance(data, list) else data["OM9-LAYER-001"]
        assert feature["implementation_status"] == "not_implemented_on_standalone_om9"
        feature["standalone_layer_state_evidence"] = meta
        write(path, data)
        if filename == "FEATURES.json":
            (PACKAGE / "FEATURES.yaml").write_text(yaml.safe_dump(data, allow_unicode=True, sort_keys=False, width=120), encoding="utf-8")
    feature_path = PACKAGE / "specs/01-core/om9-layer-001-layer-management.md"
    marker = "## Standalone layer state evidence — 2026-10-14"
    section = marker + f"\n\nSchema4/preset identity/independent Hidden and explicit legacy migration have [measured evidence](../../../{REPORT}). {passed} workspace tests; {focused} focused subsets; four bounded cold OM9 children and {native['tests']} separate native export tests/{native['jobs']} jobs. Batch rename/basic palette admission defects are fixed. Actual panel/CMD/group/current-output/selection/modal and complete Layer acceptance remain open. The primitive receipt above is historical for the changed source; no full feature status is promoted."
    append_once(feature_path, marker, section)
    screenshot = PACKAGE / "specs/00-rhino-core/MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md"
    text = screenshot.read_text(encoding="utf-8").replace("## Nghiệm thu bổ sung — tất cả chưa chạy", "## Nghiệm thu bổ sung — chưa có ca nào được nghiệm thu đầy đủ")
    screenshot.write_text(text, encoding="utf-8")
    append_once(screenshot, "## Bằng chứng triển khai từng phần", "## Bằng chứng triển khai từng phần\n\nMenu title viewport M9IMG-09–11 có [bằng chứng scoped](../../../docs/validation/2026-10-14-viewport-title-menu.md) tại checkpoint nguồn riêng. Layers M9IMG-04 có [schema/policy/cold metadata](../../../" + REPORT + ") tại checkpoint mới; chưa nối đủ panel/CMD/modal/current-output. Hai phạm vi này không đóng bất kỳ ca M9IMG nào hoặc toàn bộ giao diện; không dùng ảnh làm runtime pass.")
    note = f"Owned layer state schema4 checkpoint: {passed} workspace tests/{focused} focused subsets; four bounded OM9 cold children; {native['tests']} separate SDK export tests/{native['jobs']} jobs. [Measured scope](../{REPORT}). Actual Layers panel/CMD/current-output/modal/selection and full RCORE acceptance remain open. Older receipts below retain historical source checkpoints."
    append_once(PACKAGE / "IMPLEMENTATION_STATUS.md", "Owned layer state schema4 checkpoint:", note)
    append_once(ROOT / "README.md", "Owned layer state schema4 checkpoint:", note.replace("../" + REPORT, REPORT))
    progress = ROOT / "docs/openmatrix9-progress.json"
    data = json.loads(progress.read_text(encoding="utf-8"))
    data["updated_at"] = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")
    item = next(row for row in data["items"] if row.get("key") == "rcore00-standalone-foundation")
    for path in [RECEIPT, REPORT, native["path"], "standalone/crates/om9-core/src/layers/preset.rs"]:
        if path not in item["outputs"]:
            item["outputs"].append(path)
    prefix = "Owned layer state schema4;"
    item["evidence"] = [row for row in item["evidence"] if not row.startswith(prefix)]
    item["evidence"].append(prefix + f" {passed} workspace/{focused} focused subset tests; explicit schema1/2/3 migration, durable preset/Hidden, atomic policy, root/member scene and closed native visibility/loss wire. Four bounded cold OM9 children; separate {native['tests']} SDK tests/{native['jobs']} jobs. Whole Layers/UI/current-output/selection/modal and all full RCORE gates remain open; older combined receipts are historical.")
    item["next_action"] = "Connect approved Layers panel/CMD/group/current-output/selection/modal lifecycle to owned state; continue the full RCORE-00 and screenshot acceptance without closing the goal on a slice."
    write(progress, data)
    roadmap = ROOT / "docs/validation/opennurbs-test-roadmap.json"
    data = json.loads(roadmap.read_text(encoding="utf-8"))
    data["owned_layer_state_schema4_checkpoint"] = meta
    write(roadmap, data)
    append_once(ROOT / "docs/validation/opennurbs-test-roadmap.md", "Owned layer state schema4 checkpoint:", note.replace("../" + REPORT, "2026-10-14-owned-layer-state.md"))
    append_once(ROOT / "docs/superpowers/plans/2026-10-14-owned-layers.md", "Task2 scoped checkpoint:", f"Task2 scoped checkpoint: {passed} workspace/{focused} focused subset tests; durable schema4/preset/Hidden, strict explicit migration, four owned cold children, closed native projection/loss receipt and independent review. [Evidence](../../validation/2026-10-14-owned-layer-state.json). Task2 remains open for group/CMD/current-output and broader native combinations; Task3/4 and the complete Layers/RCORE goal remain open.")
    manifest = []
    for path in sorted(PACKAGE.rglob("*")):
        if path.is_file() and path.name != "MANIFEST.json" and not {"target", "__pycache__"}.intersection(path.parts):
            content = path.read_bytes()
            manifest.append({"path": path.relative_to(PACKAGE).as_posix(), "bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()})
    write(PACKAGE / "MANIFEST.json", manifest)
    print(json.dumps({"workspace_tests": passed, "focused_subset": focused, "cold_processes": 4,
                      "native_jobs": native["jobs"], "v1_unchanged": len(audit["files"]),
                      "v2_manifest": len(manifest), "goal_complete": False}))


if __name__ == "__main__":
    main()
