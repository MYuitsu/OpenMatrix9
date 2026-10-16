"""Create a fresh close checkpoint; retain prior source-bound receipts."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / "tools/validate_owned_file_win32.py").read_text(encoding="utf-8")
changes = {
    "owned-file-win32": "owned-file-close",
    "validated_owned_file_win32_adapter_and_replay_slice": "validated_owned_file_window_close_slice",
    '== 483': '== 494',
    '("om9-app", "file_lifecycle", 17)': '("om9-app", "file_lifecycle", 20)',
    '("om9-app", "file_widgets", 16)': '("om9-app", "file_widgets", 21)',
    '"ui::files::tests", 1': '"ui::files::tests", 3',
    '("om9-native-owner", "file_native_adapter", "file_dialog::tests", 3)]': '("om9-native-owner", "file_native_adapter", "file_dialog::tests", 3),\n                    ("om9-native-owner", "root_close_guard", "dialog_probe::tests::root_close_refuses", 1)]',
    'ui_modes = [("--files", "owned-file-close-native", 8)': 'ui_modes = [("--file-close", "owned-file-close-wm-native", 11), ("--files", "owned-file-close-native", 8)',
    '"native_checks": [8, 10, 13, 6]': '"native_checks": [11, 8, 10, 13, 6]',
    'Freeze File UI/native cancellation slice; full File/codecs/RCORE gates stay open.': 'Freeze owned window-close slice; full File/codecs/RCORE gates stay open.',
}
for old, new in changes.items():
    assert old in source, old
    source = source.replace(old, new)
source = source.replace('"focused_checks_are_workspace_subset": True,', '"focused_checks_are_workspace_subset": True, "close_intent_connected": True, "native_wm_close_requests": 2,')
source = source.replace('"Common native OM9 menu/CMD/F6/toolbar and owned prompts connected; native path selection/overwrite success is not measured."', '"Owned dirty close is connected in UI and minimized logic; native WM_CLOSE/Cancel and Save then chooser Cancel measured. Native Discard/Save-success shutdown and successful OS selection/overwrite remain unmeasured."')
(ROOT / "tools/validate_owned_file_close.py").write_text(source, encoding="utf-8")
