"""Record measured File UI scope, without promoting complete File/RCORE gates."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import tomllib
import yaml

ROOT=Path(__file__).resolve().parents[1]
PACKAGE=ROOT/'OpenMatrix9_Codex_Spec_v2'
RECEIPT='docs/validation/2026-10-14-owned-file-ui.json'
REPORT='docs/validation/2026-10-14-owned-file-ui.md'
MARKER='## Owned File UI and native cancellation — 2026-10-14'
IDS=['OM9-FILE-002','OM9-FILE-003','OM9-FILE-005','OM9-FILE-006']

def digest(path):
    with path.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
def write(path,data):path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def append(path,text):
    old=path.read_text(encoding='utf-8')
    if MARKER not in old:path.write_text(old.rstrip()+'\n\n'+text+'\n',encoding='utf-8')

def main():
    receipt=json.loads((ROOT/RECEIPT).read_text(encoding='utf-8'))
    assert receipt['status']=='validated_owned_file_ui_native_cancel_slice'
    assert receipt['file_ui_connected'] and not receipt['goal_complete'] and not receipt['full_file_acceptance']
    for path,sha in receipt['source_sha256'].items():assert digest(ROOT/path)==sha,path
    for row in receipt['native_ui']:
        assert digest(ROOT/row['path'])==row['sha256']
        assert digest(ROOT/row['binary']['path'])==row['binary']['sha256']
        for image in [row['screenshot'],*row['dialog_screenshots']]:assert digest(ROOT/image['path'])==image['sha256']
    baseline=json.loads((PACKAGE/'SOURCE_READ_AUDIT.json').read_text(encoding='utf-8'))
    for row in baseline['files']:
        path=ROOT/'OpenMatrix9_Codex_Spec_v1'/row['path']
        assert path.stat().st_size==row['bytes'] and digest(path)==row['sha256']
    passed,focused,ignored=receipt['workspace_passed'],receipt['focused_tests'],receipt['workspace_ignored']
    lock=tomllib.loads((ROOT/'Cargo.lock').read_text())
    rfd=next(row for row in lock['package'] if row['name']=='rfd')
    assert rfd['version']=='0.17.2'
    meta={'scope':'native_om9_common_file_ui_and_real_windows_cancel', 'receipt':'../'+RECEIPT,'report':'../'+REPORT,
          'workspace_tests':passed,'workspace_ignored_sdk':ignored,'focused_subset':focused,'file_widget_tests':16,
          'file_controller_tests':17,'native_file_checks':8,'real_windows_picker_cancellations':3,
          'native_regression_checks':[10,13,6],'cold_om9_processes':receipt['cold_om9_processes'],
          'file_ui_connected':True,'real_os_path_selection_verified':False,'full_feature_acceptance':False,
          'new_sdk_geometry_jobs':0}
    attempts='docs/validation/2026-10-14-owned-file-ui-native-attempts.json'
    failure=json.loads((ROOT/attempts).read_text(encoding='utf-8'))
    assert failure['status']=='failed_native_replay_then_isolated_retry_passed_root_cause_unresolved'
    assert len(failure['isolated_retry_checks'])==10
    meta['native_regression_attempts']={'failure_trace':'../'+attempts,'failed_replays':1,'isolated_retry_checks':10,'root_cause':'unresolved','current_verifier_passed':True}
    report=f'''# Owned File UI and native cancellation — 2026-10-14

[Source-bound receipt](2026-10-14-owned-file-ui.json) extends the historical
[controller service checkpoint](2026-10-14-owned-file-service.md) toward the full
[File lifecycle plan](../superpowers/plans/2026-10-14-owned-file-lifecycle.md).
The original RCORE-00 plus screenshot goal remains active. No complete File,
Layers, M9IMG, M01–M12,607-feature or148-capability gate is promoted.

Native `.om9` New/Open/Save/Save As now share controller effects across the File
menu, CMD, F6 and Standard New/Open/Save icons. Ctrl+N/O/S and Ctrl+Shift+S use
the same operations. Unsupported `.3dm` Import/Export stays visible and disabled.
Typed File literals are consumed as commands; toolbar/menu/shortcuts preserve
an unrelated numeric Line draft. Save/Discard/Cancel and owned overwrite prompts
bind buttons to the current authority/response serial. Cancel/error keeps the
old document/scene/draft and executable Redo; accepted replacement clears old
command/transcript editor state, selection/Undo/popups and refreshes title/path.
`--ui path` now constructs the same guarded from-file controller and exposes
missing-module restoration diagnostics instead of losing active path/diagnostics.
That startup constructor is tested; actual native `--ui path` launch remains
unmeasured in this checkpoint.

**{passed} workspace tests / {ignored} ignored SDK tests**, {focused} focused
checks are a workspace subset.16 actual egui File widget checks include menu,
toolbar, keyboard shortcuts, Save/Discard/Cancel/Replace, scripted owned path
delivery, failed suffix retry without automatic picker looping, stale replies,
old pointer releases, preserved draft/Redo, missing-module startup and30 prompt
cycles with stable area/scroll owners and retained CMD editor state. Scripted
path delivery exercises real archive I/O and controller ownership; it does not
claim a successful selection inside the Windows chooser.

Three input defects were witnessed RED→GREEN: IME Enter could activate Save;
holding the CMD opening Enter could repeat into Save/New on a known active path;
CMD composition lost its guard when focus moved into a File prompt. File IME
now follows OS events across focus changes, Commit/Enter waits for a later key,
and repeated Enter/Space cannot affirm. An independent read-only reviewer
identified the last two cases and reviewed the fixes; no remaining demonstrated
Important/Critical finding was reported. The reviewer did not run tests; passing
test logs belong to the main verifier.

The release native-window replay verifies8 File checks across toolbar, File
menu, F6 and CMD. It actually opens3 Windows common dialogs (Save, Open, Save As)
through the synchronous adapter. An explicit audit probe enumerates only the
verified OM9 root UI thread, requires same PID/thread, direct owner, `#32770`
class and exact application-supplied title, then posts `WM_CLOSE`. Each dialog
is observed, returns Cancel, is destroyed within5 seconds and its probe is
joined. No desktop input or other application is controlled. Loaded modules,
root window and zero child processes are observed; source/binary/PNG hashes
are bound in the receipt. This is native cancellation evidence, not OS path
selection, OS save/open success or broad OS/DPI/IME acceptance.

Images use per-replay checkpoint directories. Earlier generic PNG cache paths
were mutable; earlier receipts retain historical hashes, while this receipt is
the current source/binary/image authority.

The same release binary also passes10 Layers modal,13 earlier Layers and6
viewport-menu regressions. Seven existing bounded cold CLI children exit with
zero active processes. Format, warning-strict all-target Clippy, full workspace
and release checks run sequentially with Cargo `-j2`. No fresh SDK geometry job
runs; historical openNURBS/OCCT evidence retains its own source checkpoints.

One earlier Layers modal replay missed its initial Advanced click and then sent
editing events into CMD because the audit advanced without confirming modal
readiness. The [failed trace](2026-10-14-owned-file-ui-native-attempts.json) is
retained. An isolated same-binary retry and the final full verifier pass10 checks;
no production fix was inferred. Independent review confirmed the cascade but
could not establish the first missed-click cause. Native timestamps, raw OS
focus/events, hit-test responses and stage readiness still need diagnosis;
full Layers/native input and performance acceptance remain open.

Windows native exception: pinned MIT `rfd 0.17.2`, Cargo checksum
`{rfd['checksum']}`, default features disabled, no Linux dialog backend selected.
Its closure uses existing log/raw-window-handle/windows-sys dependencies;
raw-window-handle0.6.2 is also pinned directly for the audit parent handle.
The unchanged MIT notice is [preserved](../../Resources/licenses/rfd-0.17.2.txt),
SHA256 `{digest(ROOT/'Resources/licenses/rfd-0.17.2.txt')}`.
Safe OM9 Rust owns document/transaction authority and owned paths; rfd invokes
unsafe Win32 COM. Narrow audit Win32 handle code lives in om9-native-owner;
application/core/archive model logic remains safe Rust. No third-party/native
memory-safety certificate is implied. See [rfd API](https://docs.rs/rfd/0.17.2/rfd/struct.FileDialog.html).

Known adapter limitations remain explicit: rfd collapses COM/build/Show/result
HRESULT errors into `None`, so the current wrapper cannot distinguish those
failures from Cancel (catch_unwind only reports panic). Windows Save defaults
to overwrite confirmation; OM9 retains its own fresh owned confirmation because
a chooser result must not blindly approve a destination created after selection.
An existing destination can therefore prompt twice. This preserves confirmation
but remains a UX/API continuation. See [Win32 dialog options](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/ne-shobjidl_core-_fileopendialogoptions).

Controller aggregate admission covers retained model/Undo/scene/workspace/file
owners. MatrixApp Transcript/recall, TextEdit/egui/GPU caches and native dialog
memory are not covered by that controller peak claim. Full input-owner/worker
admission, cancellable/background I/O, actionable native backend failure,
single overwrite UX, actual OS successful select/save/open and native loaded-file
startup remain required. So do versioned view/CPlane/camera/material/annotation
persistence, `.3dm` interoperability/losses, dirty window close and every complete
original File/RCORE/Matrix9 requirement. Future full batch count remains unknown.
'''
    (ROOT/REPORT).write_text(report,encoding='utf-8')
    ledger=PACKAGE/'MIGRATION_LEDGER.json';data=json.loads(ledger.read_text(encoding='utf-8'));data['foundation_implementation']['owned_file_ui']=meta;write(ledger,data)
    specs=[]
    for name in ['FEATURES.json','FEATURE_LOOKUP.json']:
        path=PACKAGE/name;data=json.loads(path.read_text(encoding='utf-8'))
        for fid in IDS:
            feature=next(row for row in data if row['id']==fid) if isinstance(data,list) else data[fid]
            assert feature['implementation_status']=='not_implemented_on_standalone_om9'
            feature['standalone_file_ui_evidence']=meta
            if name=='FEATURES.json':specs.append(feature['spec_path'])
        write(path,data)
        if name=='FEATURES.json':(PACKAGE/'FEATURES.yaml').write_text(yaml.safe_dump(data,allow_unicode=True,sort_keys=False,width=120),encoding='utf-8')
    note=MARKER+f'\n\n[Current scoped evidence](../../../{REPORT}): {passed} workspace tests/{focused} focused subset;16 File widgets;8 native File checks and3 real Windows picker cancellations, distinct from scripted path delivery. Native `.om9` menu/CMD/F6/Standard routes and owned prompts are connected.10+13+6 Layers/viewport regressions and7 cold children are separate. OS successful selection/save/open, native backend diagnostics/single overwrite UX, UI/native peak ownership, native startup, close/async/full persistence/.3dm remain required. All full feature/RCORE/M9IMG gates and catalog statuses remain open.'
    for path in [*(PACKAGE/name for name in specs),PACKAGE/'specs/00-rhino-core/00-qymcad-foundation-migration.md',PACKAGE/'specs/00-rhino-core/MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md',PACKAGE/'specs/00-rhino-core/UI_DETAILS.md']:append(path,note)
    for path in [PACKAGE/'IMPLEMENTATION_STATUS.md',PACKAGE/'README.md']:append(path,note.replace('../../../'+REPORT,'../'+REPORT))
    append(ROOT/'README.en.md',note.replace('../../../'+REPORT,REPORT))
    append(ROOT/'README.md',MARKER+f'\n\nĐã nối [giao diện File native OM9]({REPORT}): {passed} test workspace,16 ca widget và8 kiểm tra File trong cửa sổ native;3 dialog Windows mở/hủy/đóng sạch. Đã sửa Enter lặp và IME chuyển focus có thể tự xác nhận Save/New. Chọn/lưu/mở thành công bằng OS, xử lý lỗi native, persistence đầy đủ, đóng cửa sổ chưa lưu và `.3dm` còn tiếp tục; toàn goal chưa hoàn tất.')
    append(ROOT/'docs/standalone-foundation.md',note.replace('../../../'+REPORT,'../'+REPORT))
    progress=ROOT/'docs/openmatrix9-progress.json';data=json.loads(progress.read_text(encoding='utf-8'));data['updated_at']=dt.datetime.now(dt.timezone.utc).isoformat(timespec='seconds')
    item=next(row for row in data['items'] if row.get('key')=='rcore00-standalone-foundation')
    for path in [RECEIPT,REPORT,'crates/om9-app/src/ui/files.rs','crates/om9-app/src/native_picker.rs','crates/om9-native-owner/src/dialog_probe.rs']:
        if path not in item['outputs']:item['outputs'].append(path)
    prefix='Owned File UI/native cancellation;';item['evidence']=[row for row in item['evidence'] if not row.startswith(prefix)]
    item['evidence'].append(prefix+f' {passed} workspace/{focused} focused subset;16 widgets;8 File native checks/3 real OS cancels;10+13+6 regressions/7 cold children. Source/binary bound;review fixes RED→GREEN;0 new SDK geometry jobs;full gates open.')
    item['next_action']='Diagnose the retained nondeterministic Layers native replay with raw timing/focus/hit-test evidence and readiness guards. Continue complete File lifecycle: native HRESULT diagnostics and single overwrite UX, actual OS successful selection/save/open and native --ui path startup, full UI/native owner admission and dirty close/async behavior; then versioned view/material/annotation persistence, supported .3dm interoperability and full130core/RCORE/Matrix9 acceptance.'
    write(progress,data)
    roadmap=ROOT/'docs/validation/opennurbs-test-roadmap.json';data=json.loads(roadmap.read_text(encoding='utf-8'));data['owned_file_ui_checkpoint']={**meta,'new_sdk_jobs':0};write(roadmap,data)
    append(ROOT/'docs/validation/opennurbs-test-roadmap.md',note.replace('../../../'+REPORT,'2026-10-14-owned-file-ui.md')+' No new SDK geometry jobs; remaining total batches unknown and all unclosed packages retained.')
    plan=ROOT/'docs/superpowers/plans/2026-10-14-owned-file-lifecycle.md'
    content=plan.read_text(encoding='utf-8')
    for text in ['Route menu/CMD/F6/Standard New/Open/Save icons','Render owned Save/Discard/Cancel and overwrite prompts','Use a real OS file chooser and owned callback','Independent review; fix Important/Critical findings']:
        content=content.replace('- [ ] '+text,'- [x] '+text)
    plan.write_text(content,encoding='utf-8');append(plan,note.replace('../../../'+REPORT,'../../validation/2026-10-14-owned-file-ui.md'))
    manifest=[]
    for path in sorted(PACKAGE.rglob('*')):
        if path.is_file() and path.name!='MANIFEST.json' and not {'target','__pycache__'}.intersection(path.parts):
            manifest.append({'path':path.relative_to(PACKAGE).as_posix(),'bytes':path.stat().st_size,'sha256':digest(path)})
    write(PACKAGE/'MANIFEST.json',manifest)
    print(json.dumps({'workspace':passed,'focused_subset':focused,'file_widgets':16,'native_file_checks':8,'real_os_cancels':3,'v1_unchanged':len(baseline['files']),'v2_manifest':len(manifest),'goal_complete':False}))

if __name__=='__main__':main()
