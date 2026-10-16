"""Synchronize Task1 service evidence; File UI/full feature gates remain open."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "OpenMatrix9_Codex_Spec_v2"
RECEIPT = "docs/validation/2026-10-14-owned-file-service.json"
REPORT = "docs/validation/2026-10-14-owned-file-service.md"
MARKER = "## Owned File lifecycle service — 2026-10-14"
IDS = ["OM9-FILE-002", "OM9-FILE-003", "OM9-FILE-005", "OM9-FILE-006"]


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def append_once(path, text):
    old = path.read_text(encoding="utf-8")
    if MARKER not in old:
        path.write_text(old.rstrip() + "\n\n" + text + "\n", encoding="utf-8")


def main():
    receipt = json.loads((ROOT / RECEIPT).read_text(encoding="utf-8"))
    assert receipt["status"] == "validated_owned_file_lifecycle_service_slice"
    assert not receipt["goal_complete"] and not receipt["full_file_acceptance"] and not receipt["file_ui_connected"]
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
    passed, focused, ignored = receipt["workspace_passed"], receipt["focused_tests"], receipt["workspace_ignored"]
    report = f"""# Owned File lifecycle service — 2026-10-14

[Source-bound receipt](2026-10-14-owned-file-service.json) proves Task1 of
[the complete File lifecycle plan](../superpowers/plans/2026-10-14-owned-file-lifecycle.md).
The previous goal turn was progress: the modal source checkpoint was still
hash-current when this turn began. This checkpoint extends implementation
toward the original RCORE-00/screenshot objective; it does not redefine it.

**File menu/CMD/toolbar/picker UI is still disabled and unaccepted.** This is an
owned controller/I/O service prerequisite. All New/Open/Save/Save As feature
statuses and complete RCORE/M9IMG acceptance gates stay open. The29 native UI
checks below are regressions of existing Layers/viewport routes, not File UI tests.

Safe Rust owns FileKind/phase, active path, saved fingerprint, restoration
diagnostics and opaque response tokens. New/Open on dirty state enters explicit
Save/Discard/Cancel; failed/canceled preparation preserves the old document,
scene, pending Line, current/selection and executable Redo. Candidate archive
restoration, fresh runtime authority and scene are prepared before replacing
the controller. New/Open acceptance clears the prior command/selection/Undo
owner. Same-file reopen preserves durable IDs but renews owner session and
coordinator authority, so old layer/viewport/file callbacks cannot revive.

Dirty comparison streams current supported document serialization into SHA256
without a whole JSON output buffer, caches it by coordinator epoch and compares
the saved content fingerprint. Revision alone is insufficient: Undo followed by
a different branch can reuse the revision. Initial controller input is the clean
baseline; edited content is dirty, Save becomes clean only after atomic output,
Undo/Redo back to saved content becomes clean again, and a different equal-
revision branch remains dirty. Missing replay modules remain diagnostics and
produce a dirty unresolved restore rather than a false clean/pass.

First Save requests a native `.om9` path; known-path Save writes its active path.
Save As requires selection and explicit overwrite confirmation. Paths are owned,
absolute and bounded to32,768 encoded bytes. Unsupported suffixes, missing parent,
corrupt/missing input, readonly destinations and budget failures preserve model,
Undo, active file identity and a good destination. A failed known-path Save closes
that attempt instead of becoming a Save As picker. The existing archive verifies
private bytes/staging and atomically publishes through the selected OS owner.
Successful Save has no geometry Undo effect. Save/New/Open/SaveAs cover the complete
currently supported Document, including Hidden/off/locked Layers, explicit object
preferences, current and durable identities; missing view/material schema is still
a required continuation, not silently considered saved.

Every asynchronous response boundary has a fresh token serial, with coordinator
authority/context and document identity/session/epoch/generation/revisions. An
independent review demonstrated that reusing one token across SavePath→OpenPath
accepted a duplicate Save picker response as Open. Three actual service tests
reproduced and fixed this class: consumed Save replies cannot select an Open file,
old overwrite approval cannot authorize a reselected destination, and a rejected
missing path reply cannot become valid after another actor creates the file.
Phase transitions and rejected path/overwrite replies consume the old token;
foreign/stale/consumed cancel cannot close a newer owner. Nonce overflow is checked
before I/O. Accepted intermediate Save returns SavedPending/SavedAndReplaced and
records an owned acceptance receipt before any follow-up replacement preparation.
The reviewer's initial max-name7 post-save failure hypothesis was disproved by
archive admission and was removed; it is not counted as RED/GREEN evidence.

Old retained world/Undo/scene/selection/dialog/file owners coexist with parse/
restore/candidate scene. `TransactionCoordinator::new_with_held` reserves external
owners during startup validation and identity capture while preserving the future
coordinator's full configured limits. File metadata/diagnostics/path storage joins
auxiliary usage; existing Workspace BTree/clock, recent command buffers and repeat
string are also charged, including prospective Workspace startup before allocation.
A10.5-MB test admits either old1,000 long-label roots or a65,537-point candidate
alone but refuses their Open coexistence without changing the old state. The
existing forced-moving allocation probes were rerun within their declared bounds.
These checks are not global GUI/GPU/worker/RAM or file-latency certification.

Measured: **{passed} workspace tests / {ignored} ignored SDK tests**;
**{focused} focused checks are a workspace subset**. Seventeen File controller
tests, one held-startup test and one File cold fixture are included. The cold
fixture uses actual controller Save/New/Open/SaveAs, then two bounded independent
OM9 CLI children compare geometry/attributes/Layers and confirm the original file
bytes survive Save As. Together with the earlier cold fixtures, seven children
exit cleanly. Format, warning-strict all-target Clippy and release build passed.

Three actual release native window regressions pass10 Layers dialogs,13 earlier
Layers and6 viewport-menu checks, with observed windows/modules/no children and
source/binary/screenshot hashes including compiled SVG inputs. No new SDK/native
geometry jobs ran. Historical SDK receipts retain their prior source checkpoints.
The first unrestricted workspace compile exhausted Windows commit/pagefile and
failed; a later overlapping targeted build also locked a test executable. These
were failed infrastructure attempts, not product passes. The final verifier runs
Cargo sequentially with `-j2`, preserves all tests and records the passing logs.

Pending tasks in the complete plan: actual common menu/CMD/F6/toolbar entry and
owned dirty/overwrite prompts; selected native OS chooser and late-callback/close
lifecycle; cancellable/background I/O and worker admission; complete view/CPlane/
camera/material/annotation persistence and explicit migration; supported `.3dm`
codec/UI interoperability and losses; complete resource/native/structural families,
OS input/DPI/global caches/latency and every full File/Layers/RCORE gate. A save
destination's external concurrent replacement policy also needs its full output
publication evidence. No feature/catalog implementation status is promoted.
"""
    (ROOT / REPORT).write_text(report, encoding="utf-8")
    meta = {"scope": "owned_file_lifecycle_controller_io_only", "receipt": "../" + RECEIPT,
            "report": "../" + REPORT, "workspace_tests": passed, "workspace_ignored_sdk": ignored,
            "focused_subset": focused, "cold_om9_processes": receipt["cold_om9_processes"],
            "new_file_cold_processes": 2, "file_controller_tests": 17, "file_ui_connected": False,
            "native_regression_checks": [10, 13, 6], "full_feature_acceptance": False}
    ledger_path = PACKAGE / "MIGRATION_LEDGER.json"
    data = json.loads(ledger_path.read_text(encoding="utf-8"))
    data["foundation_implementation"]["owned_file_service"] = meta
    write(ledger_path, data)
    specs = []
    for name in ["FEATURES.json", "FEATURE_LOOKUP.json"]:
        path = PACKAGE / name
        data = json.loads(path.read_text(encoding="utf-8"))
        for fid in IDS:
            feature = next(row for row in data if row["id"] == fid) if isinstance(data, list) else data[fid]
            assert feature["implementation_status"] == "not_implemented_on_standalone_om9"
            feature["standalone_file_service_evidence"] = meta
            if name == "FEATURES.json":
                specs.append(feature["spec_path"])
        write(path, data)
        if name == "FEATURES.json":
            (PACKAGE / "FEATURES.yaml").write_text(yaml.safe_dump(data, allow_unicode=True, sort_keys=False, width=120), encoding="utf-8")
    note = MARKER + f"\n\n[Controller/I/O evidence](../../../{REPORT}): {passed} workspace tests/{focused} focused subset;17 File service checks and two new bounded cold OM9 children. "
    note += "New/Open/Save/SaveAs ownership, dirty fingerprints, per-response tokens and atomic archive I/O are implemented as Task1 prerequisites. File menu/CMD/native chooser remain disabled/unaccepted; .3dm and full view/material persistence remain required. Existing native Layers/viewport regressions are separate from File UI evidence. Full feature/M01–M12 gates and catalog statuses remain open."
    for path in [*(PACKAGE / name for name in specs),
                 PACKAGE / "specs/00-rhino-core/00-qymcad-foundation-migration.md",
                 PACKAGE / "specs/00-rhino-core/MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md",
                 PACKAGE / "specs/00-rhino-core/UI_DETAILS.md"]:
        append_once(path, note)
    for path in [PACKAGE / "IMPLEMENTATION_STATUS.md", PACKAGE / "README.md"]:
        append_once(path, note.replace("../../../" + REPORT, "../" + REPORT))
    append_once(ROOT / "README.en.md", note.replace("../../../" + REPORT, REPORT))
    append_once(ROOT / "README.md", MARKER + f"\n\nĐã thêm [service vòng đời File]({REPORT}): {passed} test workspace;17 ca controller và hai tiến trình OM9 mở lạnh mới. "
                "New/Open/Save/Save As có dirty, Save/Discard/Cancel, token từng phản hồi và atomic I/O. Menu/CMD/hộp chọn tệp File chưa bật; .3dm, persistence view/material và nghiệm thu toàn bộ File/RCORE còn tiếp tục. Ba replay native hiện có là hồi quy Layers/viewport, không phải nghiệm thu File UI.")
    progress = ROOT / "docs/openmatrix9-progress.json"
    data = json.loads(progress.read_text(encoding="utf-8"))
    data["updated_at"] = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")
    item = next(row for row in data["items"] if row.get("key") == "rcore00-standalone-foundation")
    for path in [RECEIPT, REPORT, "docs/superpowers/plans/2026-10-14-owned-file-lifecycle.md",
                 "standalone/crates/om9-app/src/file_lifecycle.rs", "standalone/crates/om9-app/src/session/files.rs"]:
        if path not in item["outputs"]:
            item["outputs"].append(path)
    prefix = "Owned File lifecycle service;"
    item["evidence"] = [row for row in item["evidence"] if not row.startswith(prefix)]
    item["evidence"].append(prefix + f" {passed} workspace/{focused} focused subsets;17 controller checks;two new File cold children/seven total.10+13+6 native regressions cover existing Layers/viewport only;File UI disabled;zero new SDK jobs/full gates remain open.")
    item["next_action"] = "Execute Task2 of docs/superpowers/plans/2026-10-14-owned-file-lifecycle.md: common menu/CMD/F6/Standard File entries, owned dirty/overwrite prompts and selected native OS chooser with token/lifetime guards; then full view/material/.3dm/async/resource/close and remaining RCORE requirements."
    write(progress, data)
    roadmap = ROOT / "docs/validation/opennurbs-test-roadmap.json"
    data = json.loads(roadmap.read_text(encoding="utf-8"))
    data["owned_file_service_checkpoint"] = {**meta, "new_sdk_jobs": 0}
    write(roadmap, data)
    append_once(ROOT / "docs/validation/opennurbs-test-roadmap.md", note.replace("../../../" + REPORT, "2026-10-14-owned-file-service.md") + " No new SDK jobs; unclosed packages and unknown total future batch count are unchanged.")
    plan = ROOT / "docs/superpowers/plans/2026-10-14-owned-file-lifecycle.md"
    content = plan.read_text(encoding="utf-8")
    first, later = content.split("## Task2:", 1)
    plan.write_text(first.replace("- [ ]", "- [x]") + "## Task2:" + later, encoding="utf-8")
    append_once(plan, note.replace("../../../" + REPORT, "../../validation/2026-10-14-owned-file-service.md"))
    manifest = []
    for path in sorted(PACKAGE.rglob("*")):
        if path.is_file() and path.name != "MANIFEST.json" and not {"target", "__pycache__"}.intersection(path.parts):
            content = path.read_bytes()
            manifest.append({"path": path.relative_to(PACKAGE).as_posix(), "bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()})
    write(PACKAGE / "MANIFEST.json", manifest)
    print(json.dumps({"workspace": passed, "focused_subset": focused, "file_ui_connected": False,
                      "cold_processes": receipt["cold_om9_processes"], "v1_unchanged": len(baseline["files"]),
                      "v2_manifest": len(manifest), "goal_complete": False}))


if __name__ == "__main__":
    main()
