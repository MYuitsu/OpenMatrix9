"""Record the selected OS adapter; preserve existing geometry provenance."""
import hashlib
import json
from pathlib import Path
import tomllib

ROOT=Path(__file__).resolve().parents[1]
path=ROOT/'standalone/provenance.json'
data=json.loads(path.read_text(encoding='utf-8'))
lock=tomllib.loads((ROOT/'standalone/Cargo.lock').read_text())
rfd=next(row for row in lock['package'] if row['name']=='rfd')
entry={
    'name':'selected_windows_owned_file_dialog',
    'mode':'selected_native_os_adapter_dependency',
    'source_path':'standalone/crates/om9-app/src/native_picker.rs',
    'upstream_url':'https://github.com/PolyMeilex/rfd',
    'upstream_version':'0.17.2','cargo_checksum':rfd['checksum'],
    'license':'MIT','license_notice':'Resources/licenses/rfd-0.17.2.txt',
    'license_sha256':hashlib.sha256((ROOT/'Resources/licenses/rfd-0.17.2.txt').read_bytes()).hexdigest(),
    'dependency_closure':['rfd0.17.2 Windows synchronous common dialog; default features off','raw-window-handle0.6.2','windows-sys0.61.2 Win32 COM/Shell','log0.4; existing locked closure'],
    'abi':'borrowed eframe Frame parent handle; owned optional PathBuf result; no document or transaction pointer',
    'owner':'safe Rust OM9 File controller owns tokens, dirty state, delayed intent and atomic publication; third-party rfd owns native COM dialog lifetime',
    'thread':'synchronous UI thread; no rfd asynchronous API used. Opt-in own-window WM_CLOSE probe is separately bounded and joined.',
    'errors':'rfd maps native HRESULT failure to None just like Cancel; panic is reported. Native error/Cancel distinction remains a required continuation.',
    'scope':'native .om9 single-file Open/Save chooser. Real Windows cancel/destroy lifecycle measured; successful OS selection/save/open and single overwrite UX remain unmeasured.',
    'unsafe_boundary':'rfd invokes Win32 COM unsafe; safe caller does not certify native memory safety. Authored audit-only handle queries/message posting isolated in om9-native-owner/src/dialog_probe.rs; no desktop input or foreign-window control.',
    'fixtures':['om9-app::file_widgets','om9-app::file_lifecycle','native File UI raw-event replay and3 owned real Windows picker cancellations'],
    'receipt':'docs/validation/2026-10-14-owned-file-ui.json','full_file_acceptance':False,
}
data['native_adapters']=[row for row in data['native_adapters'] if row['name']!=entry['name']]+[entry]
path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
