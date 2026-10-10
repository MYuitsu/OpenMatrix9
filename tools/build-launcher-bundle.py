"""Package the Windows bootstrap around the already verified stock OM9 module."""
import argparse,hashlib,json,shutil,zipfile
from pathlib import Path

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    p=argparse.ArgumentParser();p.add_argument('--plugin',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    source=Path(__file__).resolve().parents[1];plugin=args.plugin.resolve();out=args.output.resolve()
    if out.exists():raise RuntimeError('Choose a new output folder; existing files are preserved')
    validation=json.loads((plugin/'release-validation.json').read_text(encoding='utf-8-sig'))
    assert validation['ok'] and validation['host']=='Official FreeCAD 1.1.4 Windows x64'
    assert sha(plugin/'bin/OpenMatrix9Gui.pyd')==validation['plugin_sha256']
    assert sha(plugin/'bin/OM9ThreeDmImportWorker.exe')==validation['worker_sha256']
    out.mkdir(parents=True);dest=out/'OpenMatrix9';dest.mkdir()
    for f in sorted(plugin.rglob('*')):
        if '__pycache__' in f.parts:continue
        if f.is_symlink():raise RuntimeError('Plugin payload contains a link')
        if f.is_file():
            target=dest/f.relative_to(plugin);target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(f,target)
    for name in ['Start-OM9.ps1','Start-OM9.cmd','Start-OM9.FCMacro','OpenMatrix9.ico','README.md']:
        shutil.copy2(source/'launcher'/name,out/name)
    rows=[dict(path=f.relative_to(dest).as_posix(),sha256=sha(f)) for f in sorted(dest.rglob('*')) if f.is_file()]
    bootstrap=[dict(path=name,sha256=sha(out/name)) for name in ['Start-OM9.ps1','Start-OM9.cmd','Start-OM9.FCMacro','OpenMatrix9.ico','README.md']]
    (out/'launcher-manifest.json').write_text(json.dumps(dict(version='0.0.2',freecad='1.1.4',launcher_revision=2,files=rows,launcher_files=bootstrap),indent=2)+'\n')
    archive=out.parent/'OpenMatrix9-0.0.2-SetupLauncher-r2-Windows-x64.zip'
    with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED,compresslevel=9) as z:
        for f in sorted(out.rglob('*')):
            if f.is_file():z.write(f,f.relative_to(out).as_posix())
    with zipfile.ZipFile(archive) as z:assert z.testzip() is None
    (out.parent/'Launcher-r2-SHA256SUMS.txt').write_text(sha(archive)+'  '+archive.name+'\n',encoding='ascii')
    print(json.dumps(dict(ok=True,files=len(rows),archive=str(archive),sha256=sha(archive)),indent=2))

if __name__=='__main__':main()
