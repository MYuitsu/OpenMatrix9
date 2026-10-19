"""Create the current verifier without changing historical service evidence."""
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
s=(ROOT/'tools/validate_owned_file_service.py').read_text(encoding='utf-8')
replacements={
    'Freeze Task1 File lifecycle service; File UI/codecs/full RCORE gates stay open.':'Freeze File UI/native cancellation slice; full File/codecs/RCORE gates stay open.',
    '2026-10-14-owned-file-service.json':'2026-10-14-owned-file-ui.json',
    'owned-file-service':'owned-file-ui',
    '== 458':'== 475',
    '("om9-app", "file_lifecycle", 17),':'("om9-app", "file_lifecycle", 17), ("om9-app", "file_widgets", 16),',
    'ui_modes = [("--layer-dialogs",':'ui_modes = [("--files", "owned-file-ui-native", 8), ("--layer-dialogs",',
    'validated_owned_file_lifecycle_service_slice':'validated_owned_file_ui_native_cancel_slice',
    '"file_ui_connected": False':'"file_ui_connected": True',
    'Task1 File lifecycle service is wired to controller APIs only; File menu/CMD/picker UI is not enabled or accepted.':'Common native OM9 menu/CMD/F6/toolbar and owned prompts connected; native path selection/overwrite success is not measured.',
    'Cold fixtures validate controller Save/New/Open/SaveAs and archive state; interactive native picker/save/open, complete view/material persistence and .3dm remain pending.':'Actual widgets and three real Windows picker cancellations are distinct from scripted path delivery. Real OS selection/save/reopen, duplicate overwrite prompt UX, native backend error diagnostics, complete view/material persistence and .3dm remain pending.',
    '"native_checks": [10, 13, 6]':'"native_checks": [8, 10, 13, 6]',
    'ROOT / "native/CMakeLists.txt", ROOT / "rust/src/layers.rs"':'ROOT / "native/CMakeLists.txt", ROOT / "rust/src/layers.rs", ROOT / "Resources/licenses/rfd-0.17.2.txt"',
}
for old,new in replacements.items():
    assert old in s,old
    s=s.replace(old,new)
(ROOT/'tools/validate_owned_file_ui.py').write_text(s,encoding='utf-8')
