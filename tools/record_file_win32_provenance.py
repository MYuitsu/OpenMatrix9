"""Record authored selected Win32 adapter, retaining old rfd provenance as history."""
import hashlib,json,tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
path=ROOT/'standalone/provenance.json'
data=json.loads(path.read_text(encoding='utf-8'))
lock=tomllib.loads((ROOT/'standalone/Cargo.lock').read_text())
assert not any(row['name']=='rfd' for row in lock['package'])
windows=next(row for row in lock['package'] if row['name']=='windows')
for row in data['native_adapters']:
    if row['name']=='selected_windows_owned_file_dialog':
        row['mode']='historical_adapter_dependency_superseded';row['runtime_dependency']=False;row['replaced_by']='authored_windows_common_file_dialog'
entry={
    'name':'authored_windows_common_file_dialog','mode':'authored_selected_native_os_adapter',
    'source_path':'standalone/crates/om9-native-owner/src/file_dialog.rs',
    'upstream_url':'https://github.com/microsoft/windows-rs','bindings_version':'0.62.2','cargo_checksum':windows['checksum'],
    'license_notice':'Resources/licenses/windows-0.62.2.txt','license':'MIT (selected dual-license option)',
    'license_sha256':hashlib.sha256((ROOT/'Resources/licenses/windows-0.62.2.txt').read_bytes()).hexdigest(),
    'dependency_closure':['existing windows0.62.2 pinned generated bindings; default features off; Win32_Foundation/System_Com/UI_Shell/UI_Shell_Common','existing windows-core/result/strings/interface/future closure','windows-sys0.61.2 own-window PID/thread check and audit-only modal probe','raw-window-handle0.6.2 for borrowed eframe HWND'],
    'abi':'verified same-PID/current-thread HWND input; borrowed bounded Windows paths, owned optional PathBuf output and typed stage/HRESULT error. No document/transaction/native shape pointer crosses.',
    'owner':'safe OM9 controller owns token/intent/dirty state/publication. Thread-bound RAII balances successful COM initialization, interfaces and COM task-string allocations; no third-party backend code copied.',
    'thread':'synchronous current UI STA; MTA conflict is an error, not silently canceled. All COM owners drop before CoUninitialize, including S_FALSE balancing.',
    'errors':'only Show HRESULT_FROM_WIN32(ERROR_CANCELLED) is Cancel. Other setup/Show/result HRESULTs surface in bounded UI error for explicit retry. Parent/path admission fails before document mutation.',
    'scope':'native .om9 single-file Open/Save; Save disables OS overwrite prompt for the fresh owned OM9 confirmation. Actual OS successful selection/save/open still requires evidence.',
    'unsafe_boundary':'generated Win32 COM calls and task-allocated PWSTR conversion are isolated in om9-native-owner. Bounded UTF-16 conversion assumes documented native NUL-terminated allocation; no native memory-safety certificate.',
    'fixtures':['file_dialog::tests (3 including actual MTA/STA error)','ui::files::tests (1 native owner error preserves draft/operation)','ui::layer_dialog_audit::tests (4 guarded replay/focus/timing cases)','native File8checks/3 WindowsCancel and Layers modal10checks'],
    'receipt':'docs/validation/2026-10-14-owned-file-win32.json','full_file_acceptance':False,
}
data['native_adapters']=[row for row in data['native_adapters'] if row['name']!=entry['name']]+[entry]
path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
