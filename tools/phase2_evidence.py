"""Bind owned application reports to the unchanged Phase2 source/runtime."""
from pathlib import Path
import argparse,json,hashlib,os,platform
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def verify(path):
    path=Path(path);data=json.loads(path.read_text());root=Path(data['source_root']);runtime=path.parent
    for name,value in data['source_sha256'].items():
        if sha(root/name)!=value:raise RuntimeError('Source mismatch: '+name)
    for name,value in data['runtime_sha256'].items():
        if sha(runtime/name)!=value:raise RuntimeError('Runtime mismatch: '+name)
    return data
def bind(manifest,report,macro,exe,pid,exit_code):
    data=verify(manifest);report=Path(report);r=json.loads(report.read_text());runtime=Path(manifest).parent
    if not r.get('ok') or exit_code!=0:raise RuntimeError('Cannot bind failed application report')
    if r.get('macro_sha256') and r['macro_sha256']!=sha(macro):raise RuntimeError('Executed macro changed before evidence binding')
    if not pid:pid=r.get('pid')
    if r.get('module'):
        if Path(r['module']).resolve()!= (runtime/'bin/OpenMatrix9Gui.pyd').resolve():raise RuntimeError('Unexpected loaded native module')
        if r.get('module_sha256') and r['module_sha256']!=sha(r['module']):raise RuntimeError('Loaded module hash mismatch')
        r['module_sha256']=sha(r['module'])
    fixture_hashes={p.relative_to(report.parent).as_posix():sha(p) for p in report.parent.rglob('*') if p.is_file() and p.suffix.lower() in ('.3dm','.fcstd')}
    r['phase2_binding']={'manifest':str(Path(manifest).resolve()),'manifest_sha256':sha(manifest),'source_and_runtime_verified':True,'executable':str(Path(exe).resolve()),'executable_sha256':sha(exe),'owned_pid':pid,'process_exit_code':exit_code,'macro':str(Path(macro).resolve()),'macro_sha256':sha(macro),'fixture_sha256':fixture_hashes,'hardware':{'logical_cpu':os.cpu_count(),'platform':platform.platform()}}
    report.write_text(json.dumps(r,indent=2),encoding='utf-8')
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['verify','bind']);p.add_argument('manifest');p.add_argument('--report');p.add_argument('--macro');p.add_argument('--exe');p.add_argument('--pid',type=int);p.add_argument('--exit-code',type=int);a=p.parse_args()
    if a.mode=='verify':verify(a.manifest)
    else:bind(a.manifest,a.report,a.macro,a.exe,a.pid,a.exit_code)
