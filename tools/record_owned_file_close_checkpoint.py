"""Synchronize verified close evidence without promoting full acceptance."""
import datetime as dt
import json
from pathlib import Path
import yaml
from record_owned_file_ui_checkpoint import ROOT, PACKAGE, digest, write

RECEIPT = "docs/validation/2026-10-14-owned-file-close.json"
REPORT = "docs/validation/2026-10-14-owned-file-close.md"
MARKER = "## Owned window close — 2026-10-14"
IDS = ["OM9-FILE-002", "OM9-FILE-003", "OM9-FILE-005", "OM9-FILE-006"]


def append(path, text):
    old = path.read_text(encoding="utf-8")
    if MARKER not in old:
        path.write_text(old.rstrip() + "\n\n" + text + "\n", encoding="utf-8")


def main():
    receipt = json.loads((ROOT / RECEIPT).read_text(encoding="utf-8"))
    assert receipt["status"] == "validated_owned_file_window_close_slice"
    assert not receipt["goal_complete"] and receipt["workspace_passed"] == 494
    for name, sha in receipt["source_sha256"].items():
        assert digest(ROOT / name) == sha, name
    for row in receipt["native_ui"]:
        assert digest(ROOT / row["path"]) == row["sha256"]
        assert digest(ROOT / row["binary"]["path"]) == row["binary"]["sha256"]
    baseline = json.loads((PACKAGE / "SOURCE_READ_AUDIT.json").read_text(encoding="utf-8"))
    for row in baseline["files"]:
        path = ROOT / "OpenMatrix9_Codex_Spec_v1" / row["path"]
        assert path.stat().st_size == row["bytes"] and digest(path) == row["sha256"]
    passed, focused = receipt["workspace_passed"], receipt["focused_tests"]
    meta = {
        "scope": "owned_window_close_and_native_cancel",
        "receipt": "../" + RECEIPT, "report": "../" + REPORT,
        "workspace_tests": passed, "workspace_ignored_sdk": receipt["workspace_ignored"],
        "focused_subset": focused, "file_controller_tests": 20, "file_widget_tests": 21,
        "new_controller_close_tests": 3, "new_widget_close_tests": 5,
        "new_minimized_logic_tests": 2, "new_root_guard_test": 1,
        "native_close_replay_checks": 11, "native_close_os_requests": 2,
        "native_close_replay_real_picker_cancels": 4,
        "native_file_regression_checks": 8, "native_regression_checks": [10, 13, 6],
        "cold_om9_processes": receipt["cold_om9_processes"],
        "native_successful_save_shutdown_verified": False,
        "native_discard_widget_shutdown_verified": False,
        "full_feature_acceptance": False, "new_sdk_geometry_jobs": 0,
    }
    report = f"""# Owned window close — 2026-10-14

[Source-bound receipt](2026-10-14-owned-file-close.json) extends
[selected Win32 File evidence](2026-10-14-owned-file-win32.md).
The full foundation/six-screenshot goal remains active and unpromoted.

Safe Rust now represents window shutdown as internal `FileIntent::WindowClose`,
without assigning a new catalog feature ID. Both normal egui UI and eframe's
minimized/occluded logic pass veto a dirty native close request. The same owned
Save/Discard/Cancel prompt controls continuation. A clean document may close
immediately. Existing File or Layers modal ownership vetoes closing; it does not
implicitly cancel that other operation or queue a new shutdown decision.

Discard approves shutdown without replacing the document, clearing its draft,
changing geometry/selection/views or publishing Undo. Save approves only after
the supported `.om9` archive is accepted; first Save uses the native chooser,
known-path Save retains the path. Failed output and canceled chooser keep the
window and prior state. `CloseReady` retains the captured operation token until
native shutdown and is revalidated before every Close command/close event.
Mutation makes old consent stale. UI input is suspended after valid approval.
Close still relies on native application teardown; complete late-job/observer/
resource disposal and global bounded shutdown remain required foundation work.

Measured **{passed} workspace tests / {receipt['workspace_ignored']} ignored SDK
tests**, **{focused} focused checks are a workspace subset**, 86 suites.
New tests cover clean/dirty Discard, first/known-path Save, Cancel, save failure,
stale consent, repeated window requests, other File ownership, active-path
failure and minimized logic. The active-path failure test replaces its own
archive fixture with an empty same-name directory and verifies retained path,
prompt/model/Undo/Line draft/dirty state and unchanged directory. It does not
claim that fixture replacement was performed by the product.

The new native replay passes 11 checks, including two actual root `WM_CLOSE`
events, Cancel preserving the window and Save followed by a real Windows picker
Cancel. Its four chooser cancellations are separate from the existing File
regression's three; the eight original File checks overlap the new replay.
The fixed message is posted only to the supplied same-process/current-thread
root HWND through the selected native-owner adapter. There is no desktop input,
window search or foreign application control. Real native Discard-button and
successful-save shutdown remain unmeasured; controller/widget evidence is not
relabelled as native OS evidence. Audit teardown explicitly uses captured close
intent plus Discard for the throwaway audit document.

The initial native attempt is retained in
[attempts](2026-10-14-owned-file-close-native-attempts.json). Its final assertion
ran before the UI pass serviced SavePath; the resulting receipt subsequently
contained four completed picker cancellations. The harness now waits for owned
prompt/chooser readiness before advancing. No product threshold is relaxed and
no failed attempt is counted as passing.

A first aggregate attempt then stopped the Layers modal regression at step5
because actual native focus was lost. That separate failed receipt is retained
in the same attempts record; its cause is unproven. The guard prevented further
synthetic input. Fresh passing evidence retains its own source/binary hashes.

Format, warning-strict all-target Clippy, workspace, release build and five
native replays pass sequentially with Cargo `-j2`. Layers regressions pass
10 modal plus13 other checks, viewport-menu6 checks; seven bounded cold OM9
children exit with zero active processes. Receipt hashes bind source, executable
and immutable per-replay screenshots. Independent read-only review found no
Important/Critical defect in the product close flow; reviewers ran no build,
test or GUI. The reviewer found a test-only unwind cleanup hole: the active-path
directory fixture could cause Files::drop to panic again after an assertion.
An exact-path, nonrecursive RAII guard now cleans that directory before the
fixture owner, including unwind, and success checks it is gone. No new SDK
geometry jobs. V1 fingerprints remain unchanged.

Required next work: actual native successful chooser selection/save/open and
owned overwrite, actual `--ui path` startup diagnostics, complete view/CPlane/
camera/material/annotation persistence with migrations and `.3dm`, asynchronous
I/O/hung-modal/full UI-native-worker budget ownership, native success/Discard
shutdown and complete owner disposal. Full130core/M01–M12/File/Layers/M9IMG/
607-feature/148-capability gates remain open. Total future measurement batch
count remains unknown; all unclosed packages are retained.
"""
    (ROOT / REPORT).write_text(report, encoding="utf-8")
    ledger = PACKAGE / "MIGRATION_LEDGER.json"
    data = json.loads(ledger.read_text(encoding="utf-8"))
    data["foundation_implementation"]["owned_file_close"] = meta
    write(ledger, data)
    specs = []
    for name in ["FEATURES.json", "FEATURE_LOOKUP.json"]:
        path = PACKAGE / name
        data = json.loads(path.read_text(encoding="utf-8"))
        for fid in IDS:
            feature = next(row for row in data if row["id"] == fid) if isinstance(data, list) else data[fid]
            assert feature["implementation_status"] == "not_implemented_on_standalone_om9"
            feature["standalone_file_close_evidence"] = meta
            if name == "FEATURES.json": specs.append(feature["spec_path"])
        write(path, data)
        if name == "FEATURES.json":
            (PACKAGE / "FEATURES.yaml").write_text(yaml.safe_dump(data, allow_unicode=True, sort_keys=False, width=120), encoding="utf-8")
    note = MARKER + f"\n\n[Scoped close evidence](../../../{REPORT}): {passed} workspace/{focused} focused subset. Owned Save/Discard/Cancel controls window closing in normal UI and minimized logic; stale consent/save failure/chooser Cancel cannot approve shutdown. Native replay11 checks includes2 root WM_CLOSE requests and4 real picker cancels; original File8 and Layers10+13/viewport6 are separate overlapping regressions;7 cold children exit. Successful OS selection/save/open/overwrite/startup, native Save-success/Discard-button shutdown, complete teardown/budgets/async/persistence/.3dm and full gates remain required."
    for path in [*(PACKAGE / name for name in specs), PACKAGE / "specs/00-rhino-core/00-qymcad-foundation-migration.md", PACKAGE / "specs/00-rhino-core/MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md", PACKAGE / "specs/00-rhino-core/UI_DETAILS.md"]:
        append(path, note)
    for path in [PACKAGE / "IMPLEMENTATION_STATUS.md", PACKAGE / "README.md"]:
        append(path, note.replace("../../../" + REPORT, "../" + REPORT))
    append(ROOT / "README.en.md", note.replace("../../../" + REPORT, REPORT))
    append(ROOT / "standalone/README.md", note.replace("../../../" + REPORT, "../" + REPORT))
    append(ROOT / "README.md", MARKER + f"\n\nĐã nối [luồng đóng cửa sổ khi chưa lưu]({REPORT}) qua Save/Discard/Cancel, kể cả nhánh thu nhỏ cửa sổ. {passed} test workspace;2 yêu cầu WM_CLOSE thật và4 dialog Windows hủy sạch trong replay mới. Lưu lỗi hoặc hủy giữ tài liệu/bản nháp/cửa sổ. Goal đầy đủ vẫn đang thực hiện; chọn/lưu/mở thành công qua OS, startup, teardown/budgets/persistence/.3dm và các gate đầy đủ còn mở.")
    path = ROOT / "docs/openmatrix9-progress.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    data["updated_at"] = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")
    item = next(row for row in data["items"] if row.get("key") == "rcore00-standalone-foundation")
    for output in [RECEIPT, REPORT]:
        if output not in item["outputs"]: item["outputs"].append(output)
    item["evidence"].append(f"Owned window close; {passed} workspace/{focused} focused subset;11 new tests; native11 checks/2 WM_CLOSE/4 cancels plus original8 and10+13+6 regressions;7 cold children. All full gates open.")
    item["next_action"] = "Complete actual OS successful chooser selection/save/open/owned overwrite and native --ui path startup diagnostics; native successful-save/Discard-button shutdown and full owner disposal. Continue full budgets/async/cancellable I/O, versioned view/material/annotation persistence, .3dm and full130core/RCORE/Matrix9 acceptance."
    write(path, data)
    path = ROOT / "docs/validation/opennurbs-test-roadmap.json"
    data = json.loads(path.read_text(encoding="utf-8")); data["owned_file_close_checkpoint"] = meta; write(path, data)
    append(ROOT / "docs/validation/opennurbs-test-roadmap.md", note.replace("../../../" + REPORT, "2026-10-14-owned-file-close.md") + " No new geometry SDK jobs; future batch count unknown.")
    append(ROOT / "docs/superpowers/plans/2026-10-14-owned-file-lifecycle.md", note.replace("../../../" + REPORT, "../../validation/2026-10-14-owned-file-close.md"))
    manifest = [{"path": path.relative_to(PACKAGE).as_posix(), "bytes": path.stat().st_size, "sha256": digest(path)} for path in sorted(PACKAGE.rglob("*")) if path.is_file() and path.name != "MANIFEST.json" and not {"target", "__pycache__"}.intersection(path.parts)]
    write(PACKAGE / "MANIFEST.json", manifest)
    print(json.dumps({"workspace": passed, "focused_subset": focused, "v1_unchanged": len(baseline["files"]), "v2_manifest": len(manifest), "new_sdk_geometry_jobs": 0, "goal_complete": False}))


if __name__ == "__main__":
    main()
