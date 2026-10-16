"""Synchronize current selected Win32 evidence; preserve complete goal gates."""
import datetime as dt
import hashlib,json,tomllib
from pathlib import Path
import yaml
from record_owned_file_ui_checkpoint import ROOT,PACKAGE,digest,write

RECEIPT='docs/validation/2026-10-14-owned-file-win32.json'
REPORT='docs/validation/2026-10-14-owned-file-win32.md'
MARKER='## Selected Win32 File adapter and replay guards — 2026-10-14'
IDS=['OM9-FILE-002','OM9-FILE-003','OM9-FILE-005','OM9-FILE-006']
def append(path,text):
    old=path.read_text(encoding='utf-8')
    if MARKER not in old:path.write_text(old.rstrip()+'\n\n'+text+'\n',encoding='utf-8')

def main():
    r=json.loads((ROOT/RECEIPT).read_text(encoding='utf-8'))
    assert r['status']=='validated_owned_file_win32_adapter_and_replay_slice' and not r['goal_complete']
    for path,sha in r['source_sha256'].items():assert digest(ROOT/path)==sha,path
    for row in r['native_ui']:
        assert digest(ROOT/row['path'])==row['sha256'] and digest(ROOT/row['binary']['path'])==row['binary']['sha256']
        for image in [row['screenshot'],*row['dialog_screenshots']]:assert digest(ROOT/image['path'])==image['sha256']
    baseline=json.loads((PACKAGE/'SOURCE_READ_AUDIT.json').read_text(encoding='utf-8'))
    for row in baseline['files']:
        path=ROOT/'OpenMatrix9_Codex_Spec_v1'/row['path'];assert path.stat().st_size==row['bytes'] and digest(path)==row['sha256']
    lock=tomllib.loads((ROOT/'standalone/Cargo.lock').read_text());assert not any(row['name']=='rfd' for row in lock['package'])
    binding=next(row for row in lock['package'] if row['name']=='windows');assert binding['version']=='0.62.2'
    passed,focused=r['workspace_passed'],r['focused_tests']
    meta={'scope':'selected_win32_file_adapter_and_owned_replay_guards','receipt':'../'+RECEIPT,'report':'../'+REPORT,
          'workspace_tests':passed,'workspace_ignored_sdk':r['workspace_ignored'],'focused_subset':focused,
          'new_audit_unit_tests':4,'new_adapter_unit_tests':3,'new_file_ui_error_test':1,
          'file_widget_tests':16,'file_controller_tests':17,'native_file_checks':8,'real_windows_cancels':3,
          'native_regression_checks':[10,13,6],'cold_om9_processes':r['cold_om9_processes'],
          'typed_native_hresult_errors':True,'save_options_unit_verified':True,
          'full_native_overwrite_interaction_verified':False,'real_os_path_selection_verified':False,
          'historical_missed_click_cause':'unresolved; current diagnostics and guarded replay added',
          'full_feature_acceptance':False,'new_sdk_geometry_jobs':0}
    report=f'''# Selected Win32 File adapter and replay guards — 2026-10-14

[Current source-bound receipt](2026-10-14-owned-file-win32.json) extends the
[File UI checkpoint](2026-10-14-owned-file-ui.md) and the complete
[File lifecycle plan](../superpowers/plans/2026-10-14-owned-file-lifecycle.md).
Previous turn was verified progress; all683 recorded source hashes matched at
this turn's start. The full RCORE-00/six-screenshot objective remains active;
all full File/Layers/M9IMG/M01–M12/607-feature/148-capability gates remain open.

The authored safe caller now uses selected Win32 common-dialog APIs through
the existing pinned windows0.62.2 binding. rfd is removed from Cargo dependency
closure; its prior receipt/provenance/notice are retained as history. Windows
COM unsafe code lives in om9-native-owner/file_dialog.rs. The adapter validates
same process/current UI-thread HWND before COM, accepts bounded borrowed paths,
returns only an owned optional PathBuf or typed stage/HRESULT error, and never
receives document/transaction/native-shape authority. Setters reject NUL or
oversized UTF-16; returned paths preserve Windows Unicode and free native task
memory through RAII. Native NUL-terminated allocation validity is an explicit
FFI contract, not a native memory-safety certificate.

A thread-bound !Send/!Sync guard balances successful COM initialization,
including S_FALSE. Failed initialization constructs no guard. Path allocation,
shell item and dialog interfaces drop before CoUninitialize. An actual fresh
MTA thread test verifies STA conflict remains an error and leaves the caller's
MTA initialization intact. Only Show's HRESULT_FROM_WIN32(ERROR_CANCELLED)
becomes Cancel; other initialization/setup/Show/result errors are visible for
explicit UI retry. A real invalid-owner adapter error test preserves pending
File token, Line/numeric draft, model/Undo and disables automatic picker loops.
See [Show](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-imodalwindow-show)
and [COM balancing](https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-couninitialize).

Save options remove FOS_OVERWRITEPROMPT and retain filesystem/path restrictions;
OM9's existing fresh owned Replace confirmation remains the output authority.
This resolves the selected API's duplicate-prompt policy without treating a
returned chooser path as overwrite approval. Unit options and real dialog
cancellation are measured; successful OS selection of an existing destination
and the complete single-confirmation save interaction still require evidence.
See [Win32 options](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/ne-shobjidl_core-_fileopendialogoptions).

The retained historical Layers replay failure is not reclassified as a product
fix. Its original first missed-click cause cannot be established from the old
trace. New4 audit tests reproduce a long Advanced gesture exceeding ordinary
egui click duration, prevent absent-modal Ctrl+A/text from reaching CMD, prove
ordinary accepted clicks can be instrumented without recursive context locking,
and bound startup focus waiting without fabricating raw.focused. Replay now
stops on missing Advanced acceptance, expected modal/editor focus or native
focus before editing events. Trace records native time/focus/event kinds,
Advanced rect/enabled/sizing/hover/down/click and top layer. Text event payloads
are not dumped. Native startup briefly lacked real focus in a measured run;
the audit requests focus for its own viewport once and waits at most3 seconds.
It does not alter production click thresholds or inject desktop input/control
another app. A new diagnostic clicked() nested-input-lock bug was caught and
fixed; the ordinary-click test covers the actual successful path.

Measured: **{passed} workspace tests / {r['workspace_ignored']} ignored SDK
tests**, **{focused} focused checks are a workspace subset**.8 new unit checks
join the existing16 File widgets,17 File service tests and broader regressions.
Format, all-target warning-strict Clippy, full workspace and release build pass
sequentially with Cargo `-j2`. Seven bounded cold OM9 children exit with no active
processes. Release native replay passes8 File checks/3 real Windows cancels,
10 Layers modal,13 other Layers and6 viewport-menu checks, with source/binary/
per-replay PNG hashes, observed root windows/modules and zero direct children.
The own-modal cancellation probe remains a measured polling/WM_CLOSE boundary;
fully hung native calls/modal deadlines remain part of complete cancellation
acceptance. No new native geometry SDK job runs.

Independent read-only review found no introduced Important/Critical defect in
the selected adapter, RAII, error preservation or audit guards. Reviewer ran no
tests/builds/GUI; all measured results come from the main verifier. Microsoft
windows0.62.2 is already in the locked dependency set; selected COM/Shell features
are enabled, default features off. Its chosen MIT notice is
[preserved](../../Resources/licenses/windows-0.62.2.txt), SHA256
`{digest(ROOT/'Resources/licenses/windows-0.62.2.txt')}`;
Cargo checksum `{binding['checksum']}`. No backend code from rfd is copied.

Next required work: actual native successful path selection/save/open/overwrite
and --ui-path startup diagnostics; complete UI/native/worker budget ownership;
dirty close, asynchronous/cancellable I/O and hung-modal behavior; versioned
view/CPlane/camera/material/annotation persistence and supported .3dm/losses;
full130core/RCORE/Matrix9 acceptance. Current controllers still save only the
supported schema. No new feature/capability status is promoted; total future
measurement batches remain unknown.
'''
    (ROOT/REPORT).write_text(report,encoding='utf-8')
    ledger=PACKAGE/'MIGRATION_LEDGER.json';d=json.loads(ledger.read_text(encoding='utf-8'));d['foundation_implementation']['owned_file_win32']=meta;write(ledger,d)
    specs=[]
    for name in ['FEATURES.json','FEATURE_LOOKUP.json']:
        path=PACKAGE/name;d=json.loads(path.read_text(encoding='utf-8'))
        for fid in IDS:
            f=next(row for row in d if row['id']==fid) if isinstance(d,list) else d[fid]
            assert f['implementation_status']=='not_implemented_on_standalone_om9';f['standalone_file_win32_evidence']=meta
            if name=='FEATURES.json':specs.append(f['spec_path'])
        write(path,d)
        if name=='FEATURES.json':(PACKAGE/'FEATURES.yaml').write_text(yaml.safe_dump(d,allow_unicode=True,sort_keys=False,width=120),encoding='utf-8')
    note=MARKER+f'\n\n[Current scoped evidence](../../../{REPORT}): {passed} workspace/{focused} focused subsets; selected authored Win32 adapter exposes HRESULT errors and disables OS overwrite prompt for the existing owned confirmation.4 audit+3 native+1 UI-error checks;8 native File checks/3 real Windows cancels;10+13+6 regressions/7 cold children. Readiness guards prevent missing-modal events from editing CMD; original missed-click cause remains unknown. OS selection/save/open/overwrite/native startup, full budgets/close/async/persistence/.3dm and all full feature/RCORE/Matrix9 gates remain required.'
    for path in [*(PACKAGE/name for name in specs),PACKAGE/'specs/00-rhino-core/00-qymcad-foundation-migration.md',PACKAGE/'specs/00-rhino-core/MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md',PACKAGE/'specs/00-rhino-core/UI_DETAILS.md']:append(path,note)
    for path in [PACKAGE/'IMPLEMENTATION_STATUS.md',PACKAGE/'README.md']:append(path,note.replace('../../../'+REPORT,'../'+REPORT))
    append(ROOT/'README.en.md',note.replace('../../../'+REPORT,REPORT));append(ROOT/'standalone/README.md',note.replace('../../../'+REPORT,'../'+REPORT))
    append(ROOT/'README.md',MARKER+f'\n\nĐã thêm [adapter chọn tệp Win32]({REPORT}): {passed} test workspace; HRESULT lỗi được báo rõ, Cancel tách riêng và overwrite do token OM9 xác nhận. Replay Layers chờ focus thật và dừng khi dialog chưa mở;3 dialog Windows hủy/đóng sạch. Chọn/lưu/mở thành công bằng OS, startup từ tệp, budgets, close/async, persistence đầy đủ và `.3dm` vẫn tiếp tục; toàn goal chưa hoàn tất.')
    path=ROOT/'docs/openmatrix9-progress.json';d=json.loads(path.read_text(encoding='utf-8'));d['updated_at']=dt.datetime.now(dt.timezone.utc).isoformat(timespec='seconds');item=next(row for row in d['items'] if row.get('key')=='rcore00-standalone-foundation')
    for output in [RECEIPT,REPORT,'standalone/crates/om9-native-owner/src/file_dialog.rs']:
        if output not in item['outputs']:item['outputs'].append(output)
    prefix='Selected Win32 File adapter/replay;';item['evidence']=[row for row in item['evidence'] if not row.startswith(prefix)];item['evidence'].append(prefix+f' {passed} workspace/{focused} focused subset;8 new units;8 native File/3 real cancels;10+13+6 regressions/7 cold children. Typed HRESULT errors, owned overwrite policy and replay readiness/focus measured;0 fresh geometry SDK jobs;full gates open.')
    item['next_action']='Complete actual native successful File selection/save/open/owned overwrite and --ui path startup using owned fixture/dialog evidence; add dirty close and full UI/native/worker budget/cancellation ownership. Continue versioned view/material/annotation persistence and supported .3dm interoperability, full130core/RCORE/Matrix9 requirements. Preserve historical missed-click uncertainty; inspect new raw timing/focus/hit-test evidence if replay fails again.';write(path,d)
    path=ROOT/'docs/validation/opennurbs-test-roadmap.json';d=json.loads(path.read_text(encoding='utf-8'));d['owned_file_win32_checkpoint']={**meta,'new_sdk_jobs':0};write(path,d)
    append(ROOT/'docs/validation/opennurbs-test-roadmap.md',note.replace('../../../'+REPORT,'2026-10-14-owned-file-win32.md')+' No fresh geometry SDK jobs; total future batch count unknown; all unclosed packages retained.')
    append(ROOT/'docs/superpowers/plans/2026-10-14-owned-file-lifecycle.md',note.replace('../../../'+REPORT,'../../validation/2026-10-14-owned-file-win32.md'))
    manifest=[{'path':p.relative_to(PACKAGE).as_posix(),'bytes':p.stat().st_size,'sha256':digest(p)} for p in sorted(PACKAGE.rglob('*')) if p.is_file() and p.name!='MANIFEST.json' and not {'target','__pycache__'}.intersection(p.parts)]
    write(PACKAGE/'MANIFEST.json',manifest)
    print(json.dumps({'workspace':passed,'focused_subset':focused,'v1_unchanged':len(baseline['files']),'v2_manifest':len(manifest),'new_sdk_geometry_jobs':0,'goal_complete':False}))

if __name__=='__main__':main()
