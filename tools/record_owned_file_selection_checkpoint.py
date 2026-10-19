"""Record native chooser success and actual CLI startup, keeping full gates open."""
import datetime as dt
import json
import yaml
from record_owned_file_ui_checkpoint import ROOT, PACKAGE, digest, write

RECEIPT = "docs/validation/2026-10-14-owned-file-selection.json"
REPORT = "docs/validation/2026-10-14-owned-file-selection.md"
MARKER = "## Native File selection and startup — 2026-10-14"
IDS = ["OM9-FILE-002", "OM9-FILE-003", "OM9-FILE-005", "OM9-FILE-006"]


def append(path, text):
    old = path.read_text(encoding="utf-8")
    if MARKER not in old:
        path.write_text(old.rstrip() + "\n\n" + text + "\n", encoding="utf-8")


def main():
    receipt = json.loads((ROOT / RECEIPT).read_text(encoding="utf-8"))
    assert receipt["status"] == "validated_owned_file_native_selection_startup_slice"
    assert receipt["workspace_passed"] == 498 and not receipt["goal_complete"]
    for name, sha in receipt["source_sha256"].items():
        assert digest(ROOT / name) == sha, name
    for row in receipt["native_ui"]:
        assert digest(ROOT / row["path"]) == row["sha256"]
        assert digest(ROOT / row["binary"]["path"]) == row["binary"]["sha256"]
        for key in ["selected_archive", "original_archive", "startup_fixture"]:
            if row.get(key): assert digest(ROOT / row[key]["path"]) == row[key]["sha256"]
    baseline = json.loads((PACKAGE / "SOURCE_READ_AUDIT.json").read_text(encoding="utf-8"))
    for row in baseline["files"]:
        path = ROOT / "OpenMatrix9_Codex_Spec_v1" / row["path"]
        assert path.stat().st_size == row["bytes"] and digest(path) == row["sha256"]
    passed, focused = receipt["workspace_passed"], receipt["focused_tests"]
    meta = {
        "scope": "native_file_selection_alternate_path_overwrite_clean_startup",
        "receipt": "../" + RECEIPT, "report": "../" + REPORT,
        "workspace_tests": passed, "workspace_ignored_sdk": receipt["workspace_ignored"],
        "focused_subset": focused, "new_adapter_probe_tests": 2, "new_audit_guard_tests": 2,
        "native_success_checks": 5, "real_windows_selected_paths": 3,
        "native_clean_startup_checks": 3, "startup_uses_actual_ui_path_cli": True,
        "save_as_alternate_path_and_owned_replace_verified": True,
        "original_and_seed_preservation_sha256_verified": True,
        "native_missing_replay_startup_verified": False,
        "native_regression_checks": [11, 8, 10, 13, 6],
        "cold_job_owned_om9_processes": receipt["cold_om9_processes"],
        "additional_bounded_inspect_children": 2,
        "full_feature_acceptance": False, "new_sdk_geometry_jobs": 0,
    }
    report = f"""# Native File selection and startup — 2026-10-14

[Current source-bound receipt](2026-10-14-owned-file-selection.json) extends
[owned window close](2026-10-14-owned-file-close.md). The prior turn made
verified progress; all684 recorded source hashes matched at this turn's start.
The complete foundation/six-screenshot goal remains active and unpromoted.

The selected authored adapter retains normal production Open/Save behavior.
An explicit own-app audit variant prefills an authored fixture filename using
`IFileDialog::SetFileName`. It still calls native Show and GetResult; the hint
is never returned as a fabricated selection. A joined probe finds only a modal
on the verified root's PID/UI thread, checks direct owner, class #32770/exact
title and the enabled same-thread IDOK child, then posts fixed WM_COMMAND1.
The audit adapter rejects any returned path different from its predeclared
owned fixture before controller I/O, so an unexpected directory/name cannot
write outside the test target. Normal production selection remains unrestricted.
No desktop input or foreign-app control is used. Native handle validity and
NUL-terminated COM allocation remain explicit unsafe contracts, not a memory
safety certificate. windows-sys0.61.2 selects the Input/KeyboardAndMouse
feature only for IsWindowEnabled query; no desktop input API is called.
See Microsoft [SetFileName](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifiledialog-setfilename)
and [GetResult](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifiledialog-getresult).

The actual native replay authors a Point, invokes Save through CMD and obtains
`selected.om9` through the real Windows dialog. Its archive equals the owned
document. The audit makes an exact owned copy as existing `alternate.om9`, then
authors another Point and invokes SaveAs. Native dialog acceptance returns the
alternate path but leaves both files and the active path unchanged until the
owned OM9 Replace button is clicked. SHA256 captures require both copies equal
and non-null at seeding, both unchanged before Replace, original unchanged
afterward/finally, and replaced alternate bytes different from the seed.
Replace adopts alternate path after archive acceptance without adding geometry
Undo. New then clears runtime/Undo/path through the actual toolbar; Open obtains
alternate from the real Windows chooser and restores two objects/current path.
Save/SaveAs/Open all consume captured operation tokens normally.

Open deliberately renews owner_session, epoch and session_generation. The
comparison requires exactly that renewal and all other fields equal for this
authored recipe-free fixture; it does not infer general graph/replay fidelity.
The initial failed audit compared the entire Document including renewed runtime
fields. Its three accepted OS selections/failure are retained in
[attempts](2026-10-14-owned-file-selection-attempts.json), never counted as a
pass. A unit check verifies that normalization still rejects name/context data
changes; another ensures absent owned overwrite never injects acceptance.

A separate native process runs actual `--ui path`, using the normal guarded
Controller::from_file constructor. An explicit audit-directory environment
variable only enables observation/screenshot/teardown; no startup document or
input is substituted. Active path, filename/title, full supported persistent
report, empty diagnostics/clean state and actual four-view rendering are
checked against a bounded cold CLI inspection of the same unchanged archive.
This measures the clean-startup branch. Native missing-module warnings,
unavailable resources and broad DPI/IME cases remain required continuations.

Measured **{passed} workspace tests / {receipt['workspace_ignored']} ignored SDK
tests**, **{focused} focused checks are a workspace subset**, 86 suites. Four
new unit tests join the existing File20 controller/File21 widgets and broader
regressions. Format, warning-strict all-target Clippy, workspace and release
build pass sequentially with Cargo `-j2`. Seven Job-owned cold children exit
with zero active processes; two additional bounded `--inspect` children return
success and are reaped by the UI harness, without claiming Job accounting for
them. Native success5checks/3 actual selected paths and startup3checks are
separate from overlapping File-close11/File8 and Layers10+13/viewport6 replay
checks. Immutable per-replay PNGs and original/replaced authored archives bind
to source/binary/fixture SHA256. No new geometry SDK jobs; v1 unchanged.
The first passing checkpoint, before the audit's final target-path admission
guard, is retained in
[historical snapshot](2026-10-14-owned-file-selection-before-target-guard/receipt.json)
with verbatim report/receipt and separate native JSON/PNG/archive copies.
The new checkpoint was rebuilt and verified after that guard was added.

Independent read-only review found no Important/Critical product/native-guard
defects. Reviewer found a P2 proof gap where only object count demonstrated
file preservation; complete file fingerprints now close that gap. Reviewer
ran no build/test/GUI. Prior reports and failed attempts retain their scope.

Required next work: versioned complete view/CPlane/camera/material/annotation
persistence and migrations, .3dm/file codec capability/loss integration;
native missing-replay startup and successful-save/Discard-button shutdown;
asynchronous/cancellable I/O, hung-modal and global UI/native/worker budgets,
complete owner disposal. Full130core/M01–M12/File/Layers/M9IMG/607-feature/
148-capability gates remain open. Total future measurement batch count remains
unknown; all unclosed packages are retained.
"""
    (ROOT / REPORT).write_text(report, encoding="utf-8")
    path = PACKAGE / "MIGRATION_LEDGER.json"
    data = json.loads(path.read_text(encoding="utf-8")); data["foundation_implementation"]["owned_file_selection"] = meta; write(path, data)
    specs = []
    for name in ["FEATURES.json", "FEATURE_LOOKUP.json"]:
        path = PACKAGE / name
        data = json.loads(path.read_text(encoding="utf-8"))
        for fid in IDS:
            feature = next(row for row in data if row["id"] == fid) if isinstance(data, list) else data[fid]
            assert feature["implementation_status"] == "not_implemented_on_standalone_om9"
            feature["standalone_file_selection_evidence"] = meta
            if name == "FEATURES.json": specs.append(feature["spec_path"])
        write(path, data)
        if name == "FEATURES.json": (PACKAGE / "FEATURES.yaml").write_text(yaml.safe_dump(data, allow_unicode=True, sort_keys=False, width=120), encoding="utf-8")
    note = MARKER + f"\n\n[Scoped native File evidence](../../../{REPORT}): {passed} workspace/{focused} focused subset;3 real Windows Show/GetResult selections, alternate SaveAs/owned Replace adopts path only after acceptance with original/seed SHA256 preservation. Native actual --ui path clean startup3checks and cold report match. Success5/close11/File8/Layers10+13/viewport6 are separate overlapping replays;7 Job-owned cold children plus2 bounded CLI inspections. Missing-module native startup, native success/Discard shutdown, complete persistence/.3dm/async/budgets/disposal and all full gates remain open."
    for path in [*(PACKAGE / name for name in specs), PACKAGE / "specs/00-rhino-core/00-qymcad-foundation-migration.md", PACKAGE / "specs/00-rhino-core/MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md", PACKAGE / "specs/00-rhino-core/UI_DETAILS.md"]: append(path, note)
    for path in [PACKAGE / "IMPLEMENTATION_STATUS.md", PACKAGE / "README.md"]: append(path, note.replace("../../../" + REPORT, "../" + REPORT))
    append(ROOT / "README.en.md", note.replace("../../../" + REPORT, REPORT))
    append(ROOT / "docs/standalone-foundation.md", note.replace("../../../" + REPORT, "../" + REPORT))
    append(ROOT / "README.md", MARKER + f"\n\nĐã kiểm chứng [chọn/lưu/mở tệp qua Windows thật]({REPORT}): {passed} test workspace,3 đường dẫn được Show/GetResult trả về, SaveAs sang tệp khác chỉ nhận đường dẫn sau Replace và giữ nguyên SHA256 tệp gốc. Tiến trình mới chạy --ui path giữ đúng dữ liệu/đường dẫn/tiêu đề và bốn viewport. Goal đầy đủ tiếp tục: persistence hoàn chỉnh/.3dm, missing-module startup, teardown/budgets/async và các gate đầy đủ còn mở.")
    path = ROOT / "docs/openmatrix9-progress.json"
    data = json.loads(path.read_text(encoding="utf-8")); data["updated_at"] = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")
    item = next(row for row in data["items"] if row.get("key") == "rcore00-standalone-foundation")
    for output in [RECEIPT, REPORT]:
        if output not in item["outputs"]: item["outputs"].append(output)
    item["evidence"] = [row for row in item["evidence"] if not row.startswith("Native File selection/startup;")]
    item["evidence"].append(f"Native File selection/startup; {passed} workspace/{focused} focused subset;3 real selected paths/5 checks, alternate owned Replace with file hashes, actual --ui path3 clean-startup checks;7 Job cold children+2 bounded CLI inspections; full gates open.")
    item["next_action"] = "Implement complete versioned workspace view/CPlane/camera/material/annotation persistence and migrations, .3dm/loss capability integration. Cover native missing-replay startup and successful Save/Discard shutdown, full owner disposal/global budgets/async-cancellable I/O. Continue full130core/RCORE/Matrix9 acceptance without promoting partial File evidence."
    write(path, data)
    path = ROOT / "docs/validation/opennurbs-test-roadmap.json"
    data = json.loads(path.read_text(encoding="utf-8")); data["owned_file_selection_checkpoint"] = meta; write(path, data)
    append(ROOT / "docs/validation/opennurbs-test-roadmap.md", note.replace("../../../" + REPORT, "2026-10-14-owned-file-selection.md") + " No new geometry SDK jobs; remaining total batch count unknown.")
    append(ROOT / "docs/superpowers/plans/2026-10-14-owned-file-lifecycle.md", note.replace("../../../" + REPORT, "../../validation/2026-10-14-owned-file-selection.md"))
    manifest = [{"path": path.relative_to(PACKAGE).as_posix(), "bytes": path.stat().st_size, "sha256": digest(path)} for path in sorted(PACKAGE.rglob("*")) if path.is_file() and path.name != "MANIFEST.json" and not {"target", "__pycache__"}.intersection(path.parts)]
    write(PACKAGE / "MANIFEST.json", manifest)
    print(json.dumps({"workspace": passed, "focused_subset": focused, "v1_unchanged": len(baseline["files"]), "v2_manifest": len(manifest), "new_sdk_geometry_jobs": 0, "goal_complete": False}))


if __name__ == "__main__": main()
