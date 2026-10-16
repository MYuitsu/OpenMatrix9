"""Record the measured UI slice; preserve every full acceptance gate."""
from pathlib import Path
import datetime as dt
import json

ROOT = Path(__file__).resolve().parents[1]


def main():
    workspace = json.loads((ROOT / "docs/validation/2026-10-13-standalone-workspace.json").read_text(encoding="utf-8"))
    menu = json.loads((ROOT / "docs/validation/2026-10-14-viewport-title-menu/ui-receipt.json").read_text(encoding="utf-8"))
    assert workspace["passed"] == 360 and workspace["ignored"] == 23
    assert menu["status"] == "validated_viewport_title_menu_slice" and len(menu["checks"]) == 6
    assert menu["objects"] == 0 and menu["undo"] == 0 and not menu["goal_complete"]
    report = """# Matrix9 S6 viewport-title menu — 2026-10-14

The supplied S6 image is a vocabulary/layout reference. This is a measured
implementation slice of M9IMG-09–11; none of the twelve complete screenshot
acceptance cases or M01–M12 gates is promoted.

Rust owns an ordered typed registry for every visible S6 label, separator and
submenu marker. Maximize/restore and Wireframe are enabled. Shaded, Rendered,
plugin presets and the unimplemented submenu operations are visibly disabled
with a reason. No unseen submenu contents are inferred.

Actions capture coordinator/document, runtime view and layout owner identities,
view state, a shared monotonic layout generation and a Controller-owned action
generation. Accepted actions cannot be replayed after cloned workspace rollback
or a new layout clock with copied view states. Camera/CPlane/mode remain exact
across maximize/restore. Wireframe changes only the captured view; a no-op keeps
its view revision and model Undo/Redo unchanged.

The menu focuses its first enabled item and contains Tab/Shift-Tab/Up/Down.
Command input waits while the menu owns the frame. Escape, stale-view closure
and an outside pointer gesture through release preserve the numeric draft,
pending command, selection and model. A subsequent independent canvas click
still accepts the pending Point. Long-menu scrolling exposes the last entry in
a 1280×500 headless window; the scrollbar is bounded to the available work area.

Validation: 17 focused menu tests (included in the workspace total), 360 workspace
tests/23 separately executed native Rust integration tests/69 suites, formatting
and warning-strict all-target Clippy. Independent review reproduced and verified
the rollback, fresh-clock, keyboard, dismissal-click and stale-Escape regressions.

The release native winit/wgpu window replay passes six checks: popup opening,
keyboard Maximize, keyboard Restore with exact view states, Escape, outside-click
dismissal and reopening. The literal pending Point draft remains `1,2,`; objects
and model Undo remain zero. The 1600×1000 framebuffer was inspected. The owned
process has 84 loaded modules, no child processes and no legacy host dependency.

Receipts: [native UI](2026-10-14-viewport-title-menu/ui-receipt.json),
[workspace](2026-10-13-standalone-workspace.json),
[plan](../superpowers/plans/2026-10-14-viewport-title-menu.md).

Unverified: Shaded and other render backends, all disabled viewport operations,
high DPI/multiple monitors/OS IME/task-modal interaction, GPU geometry caching,
large-scene performance and complete Matrix9/RCORE-00 acceptance. Arbitrary
external public-field mutations restored byte-for-byte are not an observed
mutation lifecycle; the captured fingerprint and supported reducers are scoped
to the checks above. Native CAD/exchange command acceptance remains separate.
"""
    report_path = ROOT / "docs/validation/2026-10-14-viewport-title-menu.md"
    report_path.write_text(report, encoding="utf-8")
    readme = ROOT / "README.md"
    text = readme.read_text(encoding="utf-8").replace("342 workspace Rust tests plus 23 separately executed native integration tests", "360 workspace Rust tests plus 23 separately executed native integration tests")
    note = "The S6 viewport title menu now has typed labels, scoped Maximize/Wireframe and protected keyboard/dismissal interaction; [native menu evidence](docs/validation/2026-10-14-viewport-title-menu.md). "
    if note not in text:
        text = text.replace("Full Matrix9, native family/remapping,", note + "Full Matrix9, native family/remapping,", 1)
    readme.write_text(text, encoding="utf-8")
    status = ROOT / "OpenMatrix9_Codex_Spec_v2/IMPLEMENTATION_STATUS.md"
    text = status.read_text(encoding="utf-8").replace("12 ca M9IMG vẫn chưa chạy", "12 ca M9IMG vẫn chưa được nghiệm thu toàn bộ")
    start = text.index("RCORE-00 foundation (2026-10-13):")
    end = text.find("\n\n", start)
    if end == -1: end = len(text)
    text = text[:start] + (
        "RCORE-00 foundation (2026-10-14): `partially_implemented` trong workspace `standalone/`; "
        "360 kiểm tra Rust workspace và 23 kiểm tra native integration chạy riêng đã qua (383 distinct tests). "
        "[Bằng chứng và giới hạn](../docs/validation/2026-10-13-standalone-foundation-progress.md). "
        "Menu title viewport S6 có registry đúng nhãn/thứ tự, Maximize/Restore/Wireframe, chặn capture cũ và giữ draft/lệnh khi đóng menu; "
        "17 test tập trung là subset workspace, sáu kiểm tra cửa sổ native đã qua: "
        "[menu evidence](../docs/validation/2026-10-14-viewport-title-menu.md). "
        "Shaded/render/submenus/DPI và các ca M9IMG đầy đủ còn mở. "
        "[Retained-current cold publication](../docs/validation/2026-10-14-standalone-retained-current.json) "
        "chỉ nghiệm thu codec subset đã kiểm; full native family/remapping/clipboard/archive/performance còn mở. "
        "607 feature, 148 capability và M01–M12 không nâng trạng thái từ các slice này."
    ) + text[end:]
    status.write_text(text, encoding="utf-8")
    ledger = ROOT / "docs/validation/2026-10-13-standalone-foundation-progress.md"
    text = ledger.read_text(encoding="utf-8")
    marker = "## Matrix9 S6 viewport title menu — 2026-10-14"
    section = marker + "\n\n360 workspace Rust tests plus 23 separately executed native integration tests (383 distinct); 17 menu tests are included in the workspace total. Six native menu replay checks pass; 1600×1000 framebuffer inspected, 84 loaded modules and no children. Maximize/restore and Wireframe are scoped; unavailable modes/submenus remain disabled. Clone/fresh-layout aliases, keyboard ownership, outside-click and stale-Escape regressions were reproduced then verified. [Details and open gates](2026-10-14-viewport-title-menu.md). Full M01–M12/607 features/148 capabilities remain unpromoted.\n"
    if marker in text:
        start = text.index(marker)
        end = text.find("\n## ",start+len(marker))
        if end == -1: end = len(text)
        text = text[:start]+section+text[end:]
    else: text = text.rstrip()+"\n\n"+section
    ledger.write_text(text, encoding="utf-8")
    roadmap = ROOT / "docs/validation/opennurbs-test-roadmap.json"
    data = json.loads(roadmap.read_text(encoding="utf-8"))
    data["updated_at"] = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")
    data["viewport_title_menu_checkpoint"] = {"status":"passed_scoped", "focused_tests":17, "focused_tests_are_workspace_subset":True, "native_ui_checks":6, "evidence":"docs/validation/2026-10-14-viewport-title-menu.md", "full_acceptance":False}
    checkpoint = data["standalone_foundation_checkpoint"]
    checkpoint.update(rust_workspace_passed=workspace["passed"],workspace_rust_tests=workspace["passed"],
                      rust_suites=workspace["suites"],workspace_suites=workspace["suites"],
                      distinct_rust_tests_exercised=workspace["passed"]+workspace["ignored"],
                      evidence="docs/validation/2026-10-13-standalone-workspace.json",
                      measured_at=workspace["validated_at"])
    roadmap.write_text(json.dumps(data,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
    print(json.dumps({"workspace_tests":workspace["passed"],"distinct_tests":workspace["passed"]+workspace["ignored"],"focused_subset":17,"native_menu_checks":6,"goal_complete":False}))


if __name__ == "__main__":
    main()
