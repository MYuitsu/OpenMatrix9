"""Produce exactly two verified ZIP downloads and one shared checksum file."""
import argparse,hashlib,json,shutil,subprocess,sys,zipfile
from pathlib import Path

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--root',type=Path,required=True);args=parser.parse_args()
    source=Path(__file__).resolve().parents[2];root=args.root.resolve()
    paths=json.loads((root/'build-paths.json').read_text(encoding='utf-8-sig'));plugin=Path(paths['plugin'])
    release=json.loads((source/'release.json').read_text());version=release['version']
    validation=json.loads((plugin/'release-validation.json').read_text(encoding='utf-8-sig'))
    if not validation['ok'] or sha(plugin/'bin/OpenMatrix9Gui.pyd')!=validation['plugin_sha256'] or sha(plugin/'bin/OM9ThreeDmImportWorker.exe')!=validation['worker_sha256']:
        raise RuntimeError('Only the current verified CI build can be packaged')
    if validation.get('version')!=version or validation.get('release_metadata_sha256')!=sha(source/'release.json'):
        raise RuntimeError('CI version or release metadata changed after validation')
    gumball=validation.get('gumball',{})
    if not gumball.get('ok') or len(gumball.get('runs',[]))!=30 or gumball.get('plugin_sha256')!=validation['plugin_sha256']:
        raise RuntimeError('All native Gumball cases must pass with the packaged plugin')
    for name in ('README.md','INSTALL.vi.md','INSTALL.en.md','LICENSE','THIRD_PARTY_NOTICES.md'):
        shutil.copy2(source/name,plugin/name)
    dest=root/'packages';dest.mkdir(exist_ok=False)
    # Builder creates a second tree for automatic installation. No compiled binary is fetched from releases.
    subprocess.run([sys.executable,str(source/'tools/build-launcher-bundle.py'),'--plugin',str(plugin),'--output',str(root/'launcher-bundle')],check=True)
    launcher=next(root.glob(f'OpenMatrix9-{version}-SetupLauncher*-Windows-x64.zip'))
    shutil.move(launcher,dest/launcher.name)
    manual=dest/f'OpenMatrix9-{version}-FreeCAD-1.1.4-Windows-x64.zip'
    with zipfile.ZipFile(manual,'w',zipfile.ZIP_DEFLATED,compresslevel=9) as archive:
        for file in sorted(plugin.rglob('*')):
            if file.is_symlink():raise RuntimeError('Linked package payload')
            if file.is_file() and '__pycache__' not in file.parts:archive.write(file,'OpenMatrix9/'+file.relative_to(plugin).as_posix())
    rows=[]
    for archive in sorted(dest.glob('*.zip')):
        with zipfile.ZipFile(archive) as z:
            if z.testzip() is not None:raise RuntimeError('Damaged ZIP')
        rows.append(sha(archive)+'  '+archive.name)
    if len(rows)!=2:raise RuntimeError('Expected exactly two release ZIPs')
    (dest/'SHA256SUMS.txt').write_text('\n'.join(rows)+'\n',encoding='ascii')
    print('\n'.join(rows))

if __name__=='__main__':main()
