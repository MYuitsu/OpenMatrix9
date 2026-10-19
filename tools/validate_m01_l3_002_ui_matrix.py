"""Real-window size matrix launcher for OM9-L3-01-01-002 (closeout item 2).

SIMULATED/AUTOMATED evidence. It does NOT replace the human M01-UI sign-off.

Rust-first exception (AGENTS.md): this is the only non-Rust file of the item. It
exists because the matrix needs a real GPU window, which cannot live inside
`cargo test`. It holds no business logic: it launches the real `om9.exe
--ui-audit` with audit-only environment variables, waits, reads the
`ui-receipt.json` and `workspace.png` the application itself wrote, checks the
receipt invariants and writes `matrix-summary.json`. Precedent:
`tools/validate_standalone_ui.py`.

Supported range (user decision 2026-10-16): the window may not be smaller than
1280x720 logical points; on a screen smaller than that in logical points (for
example 1366x768 at 125%/150% DPI) the minimum is capped to the usable area.

Layout modes (user decision 2026-10-16, replaces the removed
`OM9_UI_AUDIT_RIGHT_DOCK`, which docked LAYERS on the right and is not Matrix9
behaviour):

* `default` (`OM9_UI_AUDIT_LAYOUT_MODE=default`): nothing is docked right. Every
  Matrix-parity capture uses this mode. The left column must show ICON HISTORY,
  MAIN MENU, DISPLAY, SNAPS, INFO & SETTINGS, LAYERS, PROJECTS, BUILDERS, with
  LAYERS directly below INFO & SETTINGS and above PROJECTS, and no right dock.
* `user-docked-right` (`OM9_UI_AUDIT_LAYOUT_MODE=user-docked-right`): the
  neutral fixture SNAPS is docked right, as if the user had dragged it there.
  Labelled "user-initiated OM9-extension layout, NOT a Matrix9 parity capture".
  LAYERS is never docked right in any mode.

`--self-test` feeds doctored receipts to `check_receipt` and exits non-zero if
a wrong receipt is accepted. It never starts a window.

Honest limits: every window is opened on the primary display only. The
`dpi150-*` and `lap*` cases use the audit-only `OM9_UI_AUDIT_ZOOM` (egui zoom)
and `OM9_UI_AUDIT_MONITOR` (a simulated monitor size) while the OS scale stays
whatever it is. They are NOT a real per-monitor DPI test, no second monitor is
involved and the work area is only estimated (monitor minus a fixed reserve).
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import shutil
import struct
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LABEL = "SIMULATED/AUTOMATED - does not replace the human M01-UI sign-off"
MODE_LABELS = {
    "default": "Matrix-parity capture",
    "user-docked-right": "user-initiated OM9-extension layout, NOT a Matrix9 parity capture",
}
MODES = tuple(MODE_LABELS)
# First width (points) at which the right dock appears when the user has docked
# a panel right (fixture: Snaps), at height 800, as printed by `cargo test -p
# om9-app --test workbench_regions_sizes -- --nocapture` (line `OM9_SWEEP
# first_dock_width=`). It is the chosen right-dock threshold
# (`NARROW_WINDOW_WIDTH`).
DEFAULT_FIRST_DOCK_WIDTH = 1200
RECEIPT_TIMEOUT_SECONDS = 60
# The documented left column (MATRIX9_UI_CONTRACT / user screenshot 2026-10-16).
DOCUMENTED_LEFT = [
    "icon-history", "main-menu", "display", "snaps",
    "info-settings", "layers", "projects", "builder-slot",
]
RIGHT_DOCK_FIXTURE = "snaps"
LEFT_RAIL_DEFAULT = 300.0
ICON_BASE_EDGE, ICON_MIN_EDGE, ICON_MAX_EDGE = 22.0, 18.0, 34.0
OUTER_TOLERANCE = 1.0
REQUIRED_CHECKS = [
    "audit CMD focus ready",
    "CMD line accepted once",
    "actual CMD Undo",
    "actual CMD Redo",
    "Esc canceled draft without publication",
]
UNSUPPORTED_CASE = "unsupported-1000x600-startup-refused"
# The supported minimum is 1280x720 logical points. The 1199/1200 boundary and
# the sizes below the supported range cannot be real windows (startup refuses
# them), so they are covered by the frame tests only.
FRAME_TEST_ONLY = [
    {"case": "dock-boundary-1199-and-1200", "covered_by": "workbench_regions_sizes::dock_threshold_sweep_1150_to_1250, left_stack_layers::max_rail_keeps_the_canvas_minimum and dock_layout::right_dock_threshold_boundary_is_1200_logical_points"},
    {"case": "normal-1920x1080, 1600x1000, 1280x720 at 150% zoom, ultrawide-5120x1440 (larger than the 1440x900 test display; the host clamps a bigger window)", "covered_by": "workbench_regions_sizes::supported_sizes_keep_the_contract, simulated_dpi_scales_keep_the_contract_in_point_space, left_rail_max_justification_table and left_stack_layers::layers_sits_between_info_and_projects_at_supported_frames"},
    {"case": "capped-high-dpi-911x480 (1366x768 at 150%)", "covered_by": "workbench_regions_sizes::capped_high_dpi_case_911x480 and left_stack_layers"},
    {"case": "unsupported size, error path (799x600, 800x399)", "covered_by": "workbench_regions_sizes::unsupported_size_error_path"},
    {"case": "Layers position after Reset and after save/reload, rail 380 and the drop zone drag", "covered_by": "left_stack_layers::layers_stays_in_the_left_column_after_reset_menus_from_a_perturbed_layout / ..._after_a_save_and_reload_round_trip / icon_edge_follows_the_rail_width and chrome::dragging_a_menu_to_the_right_edge_docks_it_and_shows_the_hidden_dock"},
]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def effective_min(monitor_points: list[float] | None) -> list[float]:
    """Mirror of `startup::effective_min_size` (reserve 16 x 80 pt, floor 800x400)."""
    nominal, floor, reserve = [1280.0, 720.0], [800.0, 400.0], [16.0, 80.0]
    if monitor_points is None:
        return nominal
    return [min(nominal[i], max(floor[i], monitor_points[i] - reserve[i])) for i in range(2)]


def rail_icon_edge(outer_width: float) -> float:
    """Mirror of `dock_layout::rail_icon_edge` (clamped, rounded half away from zero)."""
    raw = ICON_BASE_EDGE * outer_width / LEFT_RAIL_DEFAULT
    return float(int(min(max(raw, ICON_MIN_EDGE), ICON_MAX_EDGE) + 0.5))


def cases(first: int) -> list[dict]:
    """Real-window cases.

    `window` is physical pixels at the native OS scale, `zoom` the audit zoom
    (points = pixels / zoom), `monitor` an optional simulated monitor in
    physical pixels. `expect_min` is the minimum window the app must send.
    `mode` is the audit layout mode, `rail` the audit-only left-rail width.
    """
    def case(name, width, height, zoom=None, monitor=None, simulated=False, mode="default", rail=None):
        scale = zoom or 1.0
        points_width = width / scale
        monitor_points = None if monitor is None else [monitor[0] / scale, monitor[1] / scale]
        return {
            "case": name,
            "mode": mode,
            "window": [width, height],
            "zoom": zoom,
            "monitor": monitor,
            "rail": rail,
            # Default mode never has a right dock. A user-docked panel shows the
            # dock only from the 1200 point threshold.
            "expect_right_dock": mode == "user-docked-right" and points_width >= first,
            "expect_outer": float(rail) if rail is not None else LEFT_RAIL_DEFAULT,
            "expect_min": effective_min(monitor_points),
            "expect_min_source": "audit-simulated" if monitor is not None else "viewport",
            "simulated_dpi": simulated,
        }

    return [
        # Real windows must fit the test display (the Windows host clamps a
        # larger window to the monitor; the clamp check below fails such a case).
        case("min-1280x720", 1280, 720),
        case("laptop-1366x728", 1366, 728),
        case("wide-1440x720", 1440, 720),
        case("display-1440x900", 1440, 900),
        case("tall-1280x900", 1280, 900),
        # 1366x768 laptop: simulated monitor (and simulated DPI via the audit zoom).
        case("lap100-1366x690", 1366, 690, monitor=[1366, 768], simulated=True),
        case("lap125-1366x680", 1366, 680, zoom=1.25, monitor=[1366, 768], simulated=True),
        case("lap150-1366x700", 1366, 700, zoom=1.5, monitor=[1366, 768], simulated=True),
        # Widening/narrowing the left rail: the SVG icons follow its width.
        case("rail240-1280x720", 1280, 720, rail=240),
        case("rail460-1280x720", 1280, 720, rail=460),
        # The user docks the neutral fixture (Snaps) right: the 1200 point
        # threshold decides whether the dock shows or yields.
        case("udr-1280x720", 1280, 720, mode="user-docked-right"),
        case("udr-lap125-1366x680", 1366, 680, zoom=1.25, monitor=[1366, 768], simulated=True, mode="user-docked-right"),
        case("udr-lap150-1366x700", 1366, 700, zoom=1.5, monitor=[1366, 768], simulated=True, mode="user-docked-right"),
    ]


def rects_overlap(a: list[float], b: list[float]) -> bool:
    """Interiors of two [x, y, w, h] rects intersect (0.01 px slack)."""
    return (
        a[0] + 0.01 < b[0] + b[2]
        and b[0] + 0.01 < a[0] + a[2]
        and a[1] + 0.01 < b[1] + b[3]
        and b[1] + 0.01 < a[1] + a[3]
    )


def bar_xywh(bar: dict) -> list[float]:
    return [bar["x"], bar["y"], bar["w"], bar["h"]]


def inside(rect: list[float], outer: list[float], slack: float = 0.5) -> bool:
    return (
        rect[0] >= outer[0] - slack
        and rect[1] >= outer[1] - slack
        and rect[0] + rect[2] <= outer[0] + outer[2] + slack
        and rect[1] + rect[3] <= outer[1] + outer[3] + slack
    )


def check_layers(dock: dict, regions: dict) -> list[str]:
    """LAYERS is in the left column, directly between INFO & SETTINGS and PROJECTS,
    never right or floating, and its title bar is visible inside the sidebar."""
    problems: list[str] = []
    left = dock.get("left_sections") or []
    if "layers" not in left:
        problems.append("layers missing from left_sections")
    if "layers" in (dock.get("right_sections") or []):
        problems.append("layers in right_sections")
    if "layers" in (dock.get("floating_sections") or []):
        problems.append("layers in floating_sections")
    bars = dock.get("section_title_bars_px") or {}
    info, layers, projects = (bars.get(key) for key in ("info-settings", "layers", "projects"))
    if not (info and layers and projects):
        problems.append("layers title bar: info-settings, layers or projects title bar is missing from the receipt")
        return problems
    if layers.get("column") != "left":
        problems.append(f"layers title bar column is {layers.get('column')!r}, expected 'left'")
    if not (info["y"] + info["h"] <= layers["y"] + 0.01 and layers["y"] + layers["h"] <= projects["y"] + 0.01):
        problems.append(
            "layers title bar is not between info-settings and projects "
            f"(info y={info['y']}, layers y={layers['y']}, projects y={projects['y']})"
        )
    sidebar = regions.get("sidebar_px")
    if sidebar is None:
        problems.append("regions.sidebar_px is null, layers could not be placed inside the sidebar")
    elif not inside(bar_xywh(layers), sidebar):
        problems.append(f"layers title bar {bar_xywh(layers)} is outside the sidebar {sidebar}")
    return problems


def check_dock(case: dict, receipt: dict, regions: dict) -> list[str]:
    """Layout-mode, left-column and rail-width invariants of `receipt['dock']`."""
    problems: list[str] = []
    mode = case["mode"]
    dock = receipt.get("dock")
    if not isinstance(dock, dict):
        return ["receipt.dock is missing"]
    audit = receipt.get("audit") or {}
    if audit.get("layout_mode") != mode:
        problems.append(f"audit.layout_mode is {audit.get('layout_mode')!r}, expected {mode!r}")
    if audit.get("mode_label") != MODE_LABELS[mode]:
        problems.append(f"audit.mode_label is {audit.get('mode_label')!r}, expected {MODE_LABELS[mode]!r}")
    if audit.get("rail_width_override") != case["rail"]:
        problems.append(f"audit.rail_width_override is {audit.get('rail_width_override')!r}, expected {case['rail']!r}")
    expected_left = (
        list(DOCUMENTED_LEFT)
        if mode == "default"
        else [key for key in DOCUMENTED_LEFT if key != RIGHT_DOCK_FIXTURE]
    )
    if dock.get("left_sections") != expected_left:
        problems.append(f"left_sections is {dock.get('left_sections')!r}, expected {expected_left!r}")
    problems += check_layers(dock, regions)
    right = dock.get("right_sections")
    if mode == "default":
        if dock.get("right_dock_visible") is not False:
            problems.append(f"right_dock_visible is {dock.get('right_dock_visible')!r} in default mode, expected false")
        if regions.get("right_dock_px") is not None:
            problems.append(f"regions.right_dock_px is {regions.get('right_dock_px')!r} in default mode, expected null")
        if right != []:
            problems.append(f"right_sections is {right!r} in default mode, expected empty")
    else:
        if right != [RIGHT_DOCK_FIXTURE]:
            problems.append(f"right_sections is {right!r} in user-docked-right mode, expected {[RIGHT_DOCK_FIXTURE]!r}")
        if dock.get("right_dock_visible") != case["expect_right_dock"]:
            problems.append(
                f"right_dock_visible is {dock.get('right_dock_visible')!r}, expected {case['expect_right_dock']!r}"
            )
    if dock.get("sidebar_scroll_offset") != 0:
        problems.append(f"sidebar_scroll_offset is {dock.get('sidebar_scroll_offset')!r}, expected 0")
    outer = dock.get("left_rail_outer_px")
    if not isinstance(outer, (int, float)) or abs(outer - case["expect_outer"]) > OUTER_TOLERANCE:
        problems.append(f"left_rail_outer_px is {outer!r}, expected {case['expect_outer']} +-{OUTER_TOLERANCE}")
    expected_edge = rail_icon_edge(case["expect_outer"])
    if dock.get("left_icon_edge_px") != expected_edge:
        problems.append(f"left_icon_edge_px is {dock.get('left_icon_edge_px')!r}, expected {expected_edge}")
    if dock.get("left_icon_rects_in_sidebar") is not True:
        problems.append(f"left_icon_rects_in_sidebar is {dock.get('left_icon_rects_in_sidebar')!r}, expected true")
    return problems


def check_receipt(case: dict, receipt: dict, png: Path) -> tuple[list[str], dict]:
    """Return (problems, observations) for one receipt."""
    problems: list[str] = []
    regions = receipt.get("regions", {})
    dock = receipt.get("dock") if isinstance(receipt.get("dock"), dict) else {}
    observed = {
        "layout_mode": (receipt.get("audit") or {}).get("layout_mode"),
        "regions_error": regions.get("error"),
        "window_points": regions.get("window"),
        "screen_points": regions.get("screen_points"),
        "pixels_per_point": regions.get("pixels_per_point"),
        "zoom_factor": regions.get("zoom_factor"),
        "right_dock_shown": regions.get("right_dock_px") is not None,
        "left_sections": dock.get("left_sections"),
        "right_sections": dock.get("right_sections"),
        "layers_title_bar_px": (dock.get("section_title_bars_px") or {}).get("layers"),
        "sidebar_px": regions.get("sidebar_px"),
        "sidebar_scroll_offset": dock.get("sidebar_scroll_offset"),
        "left_rail_outer_px": dock.get("left_rail_outer_px"),
        "left_rail_inner_px": dock.get("left_rail_inner_px"),
        "left_icon_edge_px": dock.get("left_icon_edge_px"),
        "left_icon_count": dock.get("left_icon_count"),
        "left_stack_content_px": dock.get("left_stack_content_px"),
        "status_text_overlap": regions.get("status_text_overlap"),
        "status_message_px": regions.get("status_message_px"),
        "status_summary_px": regions.get("status_summary_px"),
        "min_window": regions.get("min_window"),
        "passes": receipt.get("passes"),
        "audit_checks": len(receipt.get("checks", [])),
    }
    if regions.get("error") is not None:
        problems.append(f"regions.error is not null: {regions.get('error')}")
    views = receipt.get("view_rects", {})
    if len(views) != 4:
        problems.append(f"expected 4 views, got {len(views)}")
    names = sorted(views)
    for index, first in enumerate(names):
        for second in names[index + 1:]:
            if rects_overlap(views[first], views[second]):
                problems.append(f"views {first} and {second} overlap")
    command = regions.get("command_input_px")
    if command is None:
        problems.append("command input rect missing")
    else:
        for name, rect in views.items():
            if rects_overlap(command, rect):
                problems.append(f"command input overlaps view {name}")
    # `status_text_overlap` is false when a rect is missing, so the rects must
    # be present for the false to mean anything.
    for key in ("status_message_px", "status_summary_px"):
        if regions.get(key) is None:
            problems.append(f"{key} is null, the status text could not be measured")
    if regions.get("status_text_overlap") is not False:
        problems.append(f"status_text_overlap is {regions.get('status_text_overlap')!r}, expected false")
    minimum = regions.get("min_window") or {}
    sent = minimum.get("sent")
    if sent is None or any(abs(sent[i] - case["expect_min"][i]) > 0.1 for i in range(2)):
        problems.append(f"min_window.sent is {sent!r}, expected {case['expect_min']!r}")
    if minimum.get("source") != case["expect_min_source"]:
        problems.append(f"min_window.source is {minimum.get('source')!r}, expected {case['expect_min_source']!r}")
    if observed["right_dock_shown"] != case["expect_right_dock"]:
        problems.append(
            f"right dock shown={observed['right_dock_shown']}, expected {case['expect_right_dock']}"
        )
    problems += check_dock(case, receipt, regions)
    # The scripted CMD session: all five checks, none FAILED, and no error text.
    checks = receipt.get("checks") or []
    for name in REQUIRED_CHECKS:
        if name not in checks:
            problems.append(f"audit check {name!r} is missing")
    for name in checks:
        if str(name).startswith("FAILED"):
            problems.append(f"audit check failed: {name!r}")
    if "unsupported" in str(receipt.get("cmd", "")):
        problems.append("cmd contains 'unsupported': the scripted CMD input was rejected")
    expected_ppp = case["zoom"] or 1.0
    ppp = regions.get("pixels_per_point")
    observed["pixels_per_point_flag"] = "as requested"
    if ppp is None or abs(ppp - expected_ppp) > 0.01:
        # Not a failure of the layout: the OS scale is not 100%. Flag it.
        observed["pixels_per_point_flag"] = (
            f"DIFFERENT from requested {expected_ppp}: reported {ppp}; the OS display scale is not 100%"
        )
    # The Windows host clamps a window larger than the monitor. A clamped window
    # would silently measure a different size than the case name, so fail it.
    scale = case["zoom"] or 1.0
    want = [case["window"][0] / scale, case["window"][1] / scale]
    got = regions.get("window")
    if got is None or any(abs(got[i] - want[i]) > 1.0 for i in range(2)):
        problems.append(f"window is {got!r} points, requested {want!r}: the display clamped the window")
    if receipt.get("screenshot_saved") is not True:
        problems.append("screenshot not saved")
    return problems, observed


def run_case(binary: Path, case: dict, output: Path, scratch: Path) -> dict:
    folder = output / case["mode"] / case["case"]
    folder.mkdir(parents=True, exist_ok=True)
    environment = {key: os.environ[key] for key in ("SystemRoot", "TEMP", "TMP") if key in os.environ}
    environment.update(TEMP=str(scratch), TMP=str(scratch))
    width, height = case["window"]
    # OM9_UI_AUDIT_RIGHT_DOCK was removed (the app refuses to start with it).
    audit_env = {
        "OM9_UI_AUDIT_WINDOW": f"{width}x{height}",
        "OM9_UI_AUDIT_LAYOUT_MODE": case["mode"],
    }
    if case["zoom"] is not None:
        audit_env["OM9_UI_AUDIT_ZOOM"] = str(case["zoom"])
    if case["monitor"] is not None:
        audit_env["OM9_UI_AUDIT_MONITOR"] = f"{case['monitor'][0]}x{case['monitor'][1]}"
    if case["rail"] is not None:
        audit_env["OM9_UI_AUDIT_RAIL_WIDTH"] = str(case["rail"])
    result = {
        "case": case["case"],
        "mode": case["mode"],
        "mode_label": MODE_LABELS[case["mode"]],
        "label": LABEL if case["simulated_dpi"] else "AUTOMATED real window (primary display, OS scale as reported)",
        "env": dict(audit_env),
        "requested_window": case["window"],
        "expect_right_dock": case["expect_right_dock"],
        "expect_left_rail_outer": case["expect_outer"],
        "expect_min_window": case["expect_min"],
        "simulated_dpi": case["simulated_dpi"],
    }
    with tempfile.TemporaryDirectory(prefix="om9-matrix-", dir=scratch) as temporary:
        profile = Path(temporary)
        assert profile.resolve().is_relative_to(scratch.resolve())
        environment.update(audit_env)
        # Own layout store: the run never reads or writes the user's layout.
        environment["OM9_UI_LAYOUT"] = str(profile / "layout.json")
        audit = profile / "audit"
        started = time.monotonic()
        try:
            completed = subprocess.run(
                [str(binary), "--ui-audit", str(audit)],
                cwd=profile,
                env=environment,
                capture_output=True,
                timeout=RECEIPT_TIMEOUT_SECONDS,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            )
            result["exit_code"] = completed.returncode
            result["stderr"] = completed.stderr.decode(errors="replace")[-400:]
        except subprocess.TimeoutExpired:
            result["exit_code"] = None
            result["problems"] = [f"timeout after {RECEIPT_TIMEOUT_SECONDS}s"]
            result["passed"] = False
            return result
        result["elapsed_seconds"] = round(time.monotonic() - started, 2)
        # The audit writes its files shortly around exit; wait for them.
        receipt_path = audit / "ui-receipt.json"
        png_path = audit / "workspace.png"
        deadline = time.monotonic() + 10
        while not (receipt_path.is_file() and png_path.is_file()) and time.monotonic() < deadline:
            time.sleep(0.2)
        problems: list[str] = []
        if result["exit_code"] != 0:
            problems.append(f"exit code {result['exit_code']}")
        if not (receipt_path.is_file() and png_path.is_file()):
            problems.append("receipt or screenshot missing")
            result.update(problems=problems, passed=False)
            return result
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        found, observed = check_receipt(case, receipt, png_path)
        problems += found
        shutil.copyfile(receipt_path, folder / "ui-receipt.json")
        shutil.copyfile(png_path, folder / "workspace.png")
        header = (folder / "workspace.png").read_bytes()[:24]
        if header[:8] != b"\x89PNG\r\n\x1a\n":
            problems.append("screenshot is not a PNG")
            observed["screenshot_px"] = None
        else:
            observed["screenshot_px"] = list(struct.unpack(">II", header[16:24]))
        result.update(
            observed=observed,
            receipt=(folder / "ui-receipt.json").relative_to(ROOT).as_posix(),
            receipt_sha256=sha256(folder / "ui-receipt.json"),
            screenshot=(folder / "workspace.png").relative_to(ROOT).as_posix(),
            screenshot_sha256=sha256(folder / "workspace.png"),
            problems=problems,
            passed=not problems,
        )
    return result


def run_unsupported_case(binary: Path, output: Path, scratch: Path) -> dict:
    """unsupported size, error path: a 1000x600 window is below 1280x720 and the
    start must be refused (non-zero exit, no receipt, no screenshot)."""
    name = UNSUPPORTED_CASE
    folder = output / "default" / name
    folder.mkdir(parents=True, exist_ok=True)
    environment = {key: os.environ[key] for key in ("SystemRoot", "TEMP", "TMP") if key in os.environ}
    environment.update(TEMP=str(scratch), TMP=str(scratch))
    audit_env = {
        "OM9_UI_AUDIT_WINDOW": "1000x600",
        "OM9_UI_AUDIT_LAYOUT_MODE": "default",
        # Without this a native error dialog would block the run.
        "OM9_STARTUP_DIALOG": "0",
    }
    result = {
        "case": name,
        "mode": "default",
        "mode_label": MODE_LABELS["default"],
        "label": "unsupported size, error path (real om9.exe, window below 1280x720)",
        "env": dict(audit_env),
        "requested_window": [1000, 600],
        "expect": "startup refused: non-zero exit and no ui-receipt.json / workspace.png",
    }
    with tempfile.TemporaryDirectory(prefix="om9-matrix-", dir=scratch) as temporary:
        profile = Path(temporary)
        assert profile.resolve().is_relative_to(scratch.resolve())
        environment.update(audit_env)
        environment["OM9_UI_LAYOUT"] = str(profile / "layout.json")
        audit = profile / "audit"
        try:
            completed = subprocess.run(
                [str(binary), "--ui-audit", str(audit)],
                cwd=profile,
                env=environment,
                capture_output=True,
                timeout=RECEIPT_TIMEOUT_SECONDS,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            )
        except subprocess.TimeoutExpired:
            result.update(exit_code=None, problems=[f"timeout after {RECEIPT_TIMEOUT_SECONDS}s"], passed=False)
            return result
        problems: list[str] = []
        stderr = completed.stderr.decode(errors="replace")
        result["exit_code"] = completed.returncode
        result["stderr"] = stderr[-400:]
        if completed.returncode == 0:
            problems.append("a window below 1280x720 was accepted (exit 0)")
        if (audit / "ui-receipt.json").is_file() or (audit / "workspace.png").is_file():
            problems.append("a receipt or screenshot was written for a refused start")
        if "below the minimum window size" not in stderr:
            problems.append("the refusal text 'below the minimum window size' is not on stderr")
        (folder / "stderr.txt").write_text(stderr, encoding="utf-8")
        result.update(problems=problems, passed=not problems)
    return result


# --- self-test -------------------------------------------------------------

def good_receipt(case: dict) -> dict:
    """A synthetic, known-good receipt for `case` (never produced by a window)."""
    scale = case["zoom"] or 1.0
    width, height = case["window"][0] / scale, case["window"][1] / scale
    mode = case["mode"]
    left = list(DOCUMENTED_LEFT) if mode == "default" else [k for k in DOCUMENTED_LEFT if k != RIGHT_DOCK_FIXTURE]
    outer = case["expect_outer"]
    bars = {}
    y = 23.0
    for key in left:
        bars[key] = {"x": 2.0, "y": y, "w": outer - 5.0, "h": 14.0, "column": "left"}
        y += 55.0 if key not in ("projects",) else 148.0
    sidebar = [2.0, 23.0, outer - 5.0, height - 25.0]
    dock_px = [width - 300.0, 23.0, 298.0, height - 25.0] if case["expect_right_dock"] else None
    return {
        "passes": 20,
        "checks": list(REQUIRED_CHECKS),
        "cmd": "Line\n0,0,0\n",
        "screenshot_saved": True,
        "view_rects": {
            "Perspective": [sidebar[0] + outer, 60.0, 400.0, 300.0],
            "Top": [sidebar[0] + outer + 410.0, 60.0, 400.0, 300.0],
            "Front": [sidebar[0] + outer, 370.0, 400.0, 300.0],
            "Right": [sidebar[0] + outer + 410.0, 370.0, 400.0, 300.0],
        },
        "audit": {"layout_mode": mode, "mode_label": MODE_LABELS[mode], "rail_width_override": case["rail"]},
        "dock": {
            "left_sections": left,
            "right_sections": [] if mode == "default" else [RIGHT_DOCK_FIXTURE],
            "floating_sections": [],
            "right_dock_visible": case["expect_right_dock"],
            "left_rail_outer_px": outer,
            "left_rail_inner_px": outer - 5.0,
            "left_icon_edge_px": rail_icon_edge(outer),
            "left_icon_count": 51,
            "left_icon_rects_in_sidebar": True,
            "sidebar_scroll_offset": 0.0,
            "section_title_bars_px": bars,
        },
        "regions": {
            "window": [width, height],
            "pixels_per_point": scale,
            "min_window": {"sent": case["expect_min"], "source": case["expect_min_source"]},
            "error": None,
            "sidebar_px": sidebar,
            "right_dock_px": dock_px,
            "command_input_px": [sidebar[0] + outer, 23.0, 600.0, 30.0],
            "status_message_px": [0.0, height - 20.0, 300.0, 18.0],
            "status_summary_px": [400.0, height - 20.0, 300.0, 18.0],
            "status_text_overlap": False,
        },
    }


def self_test() -> int:
    """Doctored receipts must be rejected with the expected problem text."""
    all_cases = {c["case"]: c for c in cases(DEFAULT_FIRST_DOCK_WIDTH)}
    failures: list[str] = []

    def problems_of(case_name: str, doctor=None) -> list[str]:
        case = all_cases[case_name]
        receipt = good_receipt(case)
        if doctor is not None:
            doctor(receipt)
        return check_receipt(case, receipt, Path("unused.png"))[0]

    for name in ("min-1280x720", "rail240-1280x720", "rail460-1280x720", "udr-1280x720", "udr-lap150-1366x700"):
        found = problems_of(name)
        if found:
            failures.append(f"known-good receipt for {name} was rejected: {found}")

    def move_layers_right(r):
        r["dock"]["left_sections"].remove("layers")
        r["dock"]["right_sections"].append("layers")

    def remove_layers_left(r):
        r["dock"]["left_sections"].remove("layers")

    def layers_before_info(r):
        left = r["dock"]["left_sections"]
        left.remove("layers")
        left.insert(left.index("info-settings"), "layers")
        bars = r["dock"]["section_title_bars_px"]
        bars["layers"]["y"], bars["info-settings"]["y"] = bars["info-settings"]["y"], bars["layers"]["y"]

    def layers_bar_below_sidebar(r):
        r["dock"]["section_title_bars_px"]["layers"]["y"] = r["regions"]["sidebar_px"][1] + r["regions"]["sidebar_px"][3] + 40.0

    def visible_dock(r):
        r["dock"]["right_dock_visible"] = True
        r["regions"]["right_dock_px"] = [980.0, 23.0, 298.0, 690.0]

    def layers_floating(r):
        r["dock"]["floating_sections"] = ["layers"]

    def scrolled(r):
        r["dock"]["sidebar_scroll_offset"] = 40.0

    def bad_icon(r):
        r["dock"]["left_icon_edge_px"] = 22.0 if r["dock"]["left_rail_outer_px"] != 300.0 else 30.0

    def bad_label(r):
        r["audit"]["mode_label"] = "Matrix-parity capture" if r["audit"]["layout_mode"] != "default" else "user-initiated"

    def bad_mode(r):
        r["audit"]["layout_mode"] = "user-docked-right" if r["audit"]["layout_mode"] == "default" else "default"

    def bad_outer(r):
        r["dock"]["left_rail_outer_px"] += 10.0

    def cmd_error(r):
        r["cmd"] = "Error: unsupported unknown/unavailable command"

    def missing_focus_check(r):
        r["checks"].remove("audit CMD focus ready")

    def icons_outside(r):
        r["dock"]["left_icon_rects_in_sidebar"] = False

    def fixture_not_snaps(r):
        r["dock"]["right_sections"] = ["layers"]
        r["dock"]["left_sections"] = [k for k in DOCUMENTED_LEFT if k != "layers"]

    doctored = [
        ("min-1280x720", "Layers moved to right_sections", move_layers_right, "layers in right_sections"),
        ("min-1280x720", "Layers removed from left", remove_layers_left, "layers missing from left_sections"),
        ("min-1280x720", "Layers placed before Info & Settings", layers_before_info, "layers title bar is not between info-settings and projects"),
        ("min-1280x720", "Layers placed before Info & Settings (order)", layers_before_info, "left_sections is"),
        ("min-1280x720", "Layers title bar below the sidebar", layers_bar_below_sidebar, "outside the sidebar"),
        ("min-1280x720", "Layers floating", layers_floating, "layers in floating_sections"),
        ("min-1280x720", "default mode with a visible right dock", visible_dock, "in default mode"),
        ("min-1280x720", "sidebar scrolled", scrolled, "sidebar_scroll_offset"),
        ("min-1280x720", "wrong icon edge", bad_icon, "left_icon_edge_px"),
        ("rail460-1280x720", "wrong icon edge at rail 460", bad_icon, "left_icon_edge_px"),
        ("min-1280x720", "wrong mode label", bad_label, "audit.mode_label"),
        ("min-1280x720", "wrong layout mode", bad_mode, "audit.layout_mode"),
        ("rail240-1280x720", "wrong outer width", bad_outer, "left_rail_outer_px"),
        ("min-1280x720", "icons outside the sidebar", icons_outside, "left_icon_rects_in_sidebar"),
        ("min-1280x720", "spurious CMD error text", cmd_error, "cmd contains 'unsupported'"),
        ("min-1280x720", "missing CMD focus check", missing_focus_check, "audit CMD focus ready"),
        ("udr-1280x720", "Layers as the user-docked fixture", fixture_not_snaps, "layers in right_sections"),
        ("udr-1280x720", "udr Layers as the user-docked fixture (fixture check)", fixture_not_snaps, "right_sections is"),
    ]
    for name, label, doctor, expected in doctored:
        found = problems_of(name, doctor)
        if not any(expected in problem for problem in found):
            failures.append(f"doctored receipt ACCEPTED or wrong problem: {name} / {label}: expected {expected!r}, got {found}")
        else:
            print(f"rejected as expected: {name} / {label} -> {expected}")
    if failures:
        for failure in failures:
            print(f"SELF-TEST FAIL {failure}")
        return 1
    print(f"self-test passed: 5 known-good receipts accepted, {len(doctored)} doctored receipts rejected")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/om9.exe")
    parser.add_argument(
        "--output",
        type=Path,
        default=ROOT / "build/standalone-validation/ui-artifacts"
        / f"{dt.date.today().isoformat()}-m01-l3-002-left-dock/matrix",
    )
    parser.add_argument("--first-dock-width", type=int, default=DEFAULT_FIRST_DOCK_WIDTH)
    parser.add_argument("--only", help="run only the case with this exact name")
    parser.add_argument("--mode", choices=(*MODES, "all"), default="all", help="audit layout mode(s) to run")
    parser.add_argument("--self-test", action="store_true", help="check the receipt checks against doctored receipts and exit")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    binary = args.binary.resolve(strict=True)
    output = args.output.resolve()
    scratch = ROOT / "build/standalone-validation"
    scratch.mkdir(parents=True, exist_ok=True)
    if os.name != "nt":
        raise RuntimeError("Windows real-window observation unavailable")
    selected = [
        c
        for c in cases(args.first_dock_width)
        if args.only in (None, c["case"]) and args.mode in ("all", c["mode"])
    ]
    run_unsupported = args.only in (None, UNSUPPORTED_CASE) and args.mode in ("all", "default")
    if not selected and not run_unsupported:
        raise ValueError(f"no case named {args.only!r} in mode {args.mode!r}")
    results = [run_case(binary, case, output, scratch) for case in selected]
    if run_unsupported:
        results.append(run_unsupported_case(binary, output, scratch))
    summary = {
        "schema": "om9.m01.l3-002.ui-matrix.v2",
        "label": LABEL,
        "binary": {"path": binary.relative_to(ROOT).as_posix(), "sha256": sha256(binary)},
        "runner": {"path": Path(__file__).resolve().relative_to(ROOT).as_posix(), "sha256": sha256(Path(__file__).resolve())},
        "rust_first_exception": "Python launcher only: needs a real GPU window, cannot run inside cargo test, holds no business logic.",
        "modes": MODE_LABELS,
        "mode_filter": args.mode,
        "first_dock_width_points": args.first_dock_width,
        "honest_limits": [
            "Primary display only (it reported 1440x900 at 100% in this run); no second monitor was involved. Windows larger than that are clamped by the host and are covered by frame tests (1600x1000, 1920x1080) and the computed layout table, not by real windows.",
            "dpi150-* and lap125/lap150 cases use OM9_UI_AUDIT_ZOOM (egui zoom), not real OS per-monitor DPI.",
            "lap100/lap125/lap150 cases use OM9_UI_AUDIT_MONITOR=1366x768, a SIMULATED monitor size; the real monitor of the test machine is larger.",
            "The usable area is estimated as monitor size minus a fixed reserve of 16 x 80 points (frame, title bar, taskbar); the real work area needs a native call and is not read.",
            "rail240/rail460 set the left-rail width through the audit-only OM9_UI_AUDIT_RAIL_WIDTH; dragging the rail edge with a mouse is not exercised by this launcher.",
            "user-docked-right cases dock the neutral fixture (Snaps) through the audit hook; dragging a panel to the right edge with a mouse is covered only by the chrome.rs frame test.",
            "A maximized start on a real 1366x768 device at 125%/150% DPI is not covered here.",
            "No human looked at the window while it ran; this is not the M01-UI sign-off.",
        ],
        "frame_test_only": FRAME_TEST_ONLY,
        "cases": results,
        "all_passed": all(r.get("passed") for r in results),
        "timestamp": dt.datetime.now().astimezone().isoformat(timespec="seconds"),
    }
    output.mkdir(parents=True, exist_ok=True)
    (output / "matrix-summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for r in results:
        print(f"{'PASS' if r.get('passed') else 'FAIL'} [{r.get('mode')}] {r['case']} exit={r.get('exit_code')} {r.get('problems') or ''}")
    print(f"matrix-summary: {output / 'matrix-summary.json'}")
    return 0 if summary["all_passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
