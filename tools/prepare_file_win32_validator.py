"""Create fresh evidence tooling without altering historical File UI receipts."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
s=(ROOT/'tools/validate_owned_file_ui.py').read_text(encoding='utf-8')
replacements={
    '2026-10-14-owned-file-ui.json':'2026-10-14-owned-file-win32.json',
    'owned-file-ui':'owned-file-win32',
    '== 475':'== 483',
    'validated_owned_file_ui_native_cancel_slice':'validated_owned_file_win32_adapter_and_replay_slice',
    'ROOT / "Resources/licenses/rfd-0.17.2.txt"':'ROOT / "Resources/licenses/windows-0.62.2.txt"',
    'duplicate overwrite prompt UX, native backend error diagnostics':'successful OS selection/owned overwrite behavior',
}
for old,new in replacements.items():
    assert old in s,old
    s=s.replace(old,new)
old='    commands += [("workspace",'
new='''    unit_focused = [("om9-app", "layer_replay_guards", "ui::layer_dialog_audit::tests", 4),
                    ("om9-app", "file_native_error_ui", "ui::files::tests", 1),
                    ("om9-native-owner", "file_native_adapter", "file_dialog::tests", 3)]
    commands += [(name, ["cargo", "test", "--locked", "-j", "2", "-p", package, "--lib", pattern, "--", "--nocapture"])
                 for package,name,pattern,_ in unit_focused]
    commands += [("workspace",'''
assert old in s;s=s.replace(old,new)
s=s.replace('    assert by_name["workspace"]["passed"]', '    assert all(by_name[name]["passed"] == count for _,name,_,count in unit_focused)\n    assert by_name["workspace"]["passed"]')
s=s.replace('sum(count for _, _, count in focused)', 'sum(count for _, _, count in focused) + sum(count for _,_,_,count in unit_focused)')
(ROOT/'tools/validate_owned_file_win32.py').write_text(s,encoding='utf-8')
