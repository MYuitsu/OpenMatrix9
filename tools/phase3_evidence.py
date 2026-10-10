"""Bind actual owned application evidence to exact Phase3 source/runtime."""
from pathlib import Path
import json,hashlib,argparse
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def verify(path):
    path=Path(path);data=json.loads(path.read_text());root=Path(data['source_root']);runtime=path.parent
    files=[]
    for folder in ['Gui','rust/src','cmake','Resources']:files.extend(p for p in (root/folder).rglob('*') if p.is_file())
    files.extend(list(root.glob('*.py'))+[root/'CMakeLists.txt',root/'rust/Cargo.toml',root/'rust/Cargo.lock'])
    actual={p.relative_to(root).as_posix():sha(p) for p in files}
    if actual!=data['source_sha256']:raise RuntimeError('Phase3 source inventory/hash differs from built manifest')
    for name,digest in data['runtime_sha256'].items():
        if sha(runtime/name)!=digest:raise RuntimeError('Runtime mismatch: '+name)
    return data
def bind(manifest,report,macro,exe,pid,exit_code):
    verify(manifest);report=Path(report);r=json.loads(report.read_text());runtime=Path(manifest).parent
    if not r.get('ok') or exit_code!=0 or pid<=0:raise RuntimeError('Cannot bind unowned or failed host evidence')
    if Path(exe).resolve()!= (runtime/'bin/FreeCAD.exe').resolve():raise RuntimeError('Wrong host executable')
    if r.get('module') and Path(r['module']).resolve()!=(runtime/'bin/OpenMatrix9Gui.pyd').resolve():raise RuntimeError('Wrong loaded module')
    if r.get('module'):r['module_sha256']=sha(r['module'])
    r['phase3_binding']=dict(manifest=str(Path(manifest).resolve()),manifest_sha256=sha(manifest),source_and_runtime_verified=True,owned_pid=pid,process_exit_code=exit_code,executable=str(Path(exe).resolve()),executable_sha256=sha(exe),macro=str(Path(macro).resolve()),macro_sha256=sha(macro))
    report.write_text(json.dumps(r,indent=2))
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['verify','bind']);p.add_argument('manifest');p.add_argument('--report');p.add_argument('--macro');p.add_argument('--exe');p.add_argument('--pid',type=int);p.add_argument('--exit-code',type=int);a=p.parse_args()
    if a.mode=='verify':verify(a.manifest)
    else:bind(a.manifest,a.report,a.macro,a.exe,a.pid,a.exit_code)
