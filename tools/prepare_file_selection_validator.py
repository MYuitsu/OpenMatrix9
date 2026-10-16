"""Create a fresh source-bound native selection/startup checkpoint."""
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / "tools/validate_owned_file_close.py").read_text(encoding="utf-8")
changes = {
    "owned-file-close": "owned-file-selection",
    "validated_owned_file_window_close_slice": "validated_owned_file_native_selection_startup_slice",
    "== 494": "== 498",
    '"file_dialog::tests", 3': '"file_dialog::tests", 4',
    '("om9-native-owner", "root_close_guard", "dialog_probe::tests::root_close_refuses", 1)]': '("om9-native-owner", "root_close_guard", "dialog_probe::tests::root_close_refuses", 1),\n                    ("om9-native-owner", "native_accept_guard", "dialog_probe::tests::accept_probe", 1),\n                    ("om9-app", "native_selection_guards", "ui::file_success_audit::tests", 2)]',
    'ui_modes = [("--file-close",': 'ui_modes = [("--file-success", "owned-file-selection-success", 5), ("--startup-file", "owned-file-selection-startup", 3), ("--file-close",',
    '"native_checks": [11, 8, 10, 13, 6]': '"native_checks": [5, 3, 11, 8, 10, 13, 6]',
    '"focused_checks_are_workspace_subset": True,': '"focused_checks_are_workspace_subset": True, "native_successful_selection_verified": True, "native_clean_startup_verified": True,',
}
for old, new in changes.items():
    assert old in source, old
    source = source.replace(old, new)
old = 'command = ["rtk", "proxy", "python", "tools/validate_standalone_ui.py", mode, "--output", directory]'
new = '''command = ["rtk", "proxy", "python", "tools/validate_standalone_ui.py", mode]
        if mode == "--startup-file":
            assert startup_fixture is not None
            command.append(str(ROOT / startup_fixture["path"]))
        command += ["--output", directory]'''
assert old in source
source = source.replace(old, new).replace('    native_ui = []', '    native_ui = []\n    startup_fixture = None\n    selected_cold = None')
old = '        native_ui.append({'
new = '''        if mode == "--file-success":
            startup_fixture = ui["native_selected_archive"]
            selected_cold = ui["native_selected_cold_inspect"]
            assert digest(ROOT / startup_fixture["path"]) == startup_fixture["sha256"]
            assert digest(ROOT / ui["native_original_archive"]["path"]) == ui["original_saved_archive_sha256"]
        if mode == "--startup-file":
            assert ui["native_startup_fixture"] == startup_fixture
            assert ui["native_startup_cold_inspect"] == selected_cold
        native_ui.append({'''
assert old in source
source = source.replace(old, new)
source = source.replace('"children": ui["children"]})', '"children": ui["children"], "selected_archive": ui.get("native_selected_archive"), "original_archive": ui.get("native_original_archive"), "startup_fixture": ui.get("native_startup_fixture")})')
source = source.replace('"Actual widgets and three real Windows picker cancellations are distinct from scripted path delivery. Real OS selection/save/reopen, successful OS selection/owned overwrite behavior, complete view/material persistence and .3dm remain pending."', '"Actual three Windows selected paths, SaveAs alternate-path adoption/owned overwrite, cold inspection and native --ui clean startup measured; no scripted path is treated as an OS selection. Missing-module native startup, complete view/material persistence and .3dm remain pending."')
source = source.replace('Native Discard/Save-success shutdown and successful OS selection/overwrite remain unmeasured.', 'Native Discard/Save-success shutdown remains unmeasured; native OS successful selection/owned overwrite has separate measured replay.')
(ROOT / "tools/validate_owned_file_selection.py").write_text(source, encoding="utf-8")
