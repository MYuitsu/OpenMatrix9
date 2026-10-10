"""Generate only binding headers/import libraries from a verified stock host."""
import argparse,json,re,struct,subprocess,sys,tarfile,urllib.request,hashlib
from pathlib import Path

SOURCE_URL='https://codeload.github.com/FreeCAD/FreeCAD/tar.gz/refs/tags/1.1.4'
SOURCE_SHA='b95270a74a26dbdde331c6ff8e764289da6c77713e2b2d4e0c0041dfe1950f3f'

def sections(file):
    b=file.read_bytes();pe=struct.unpack_from('<I',b,0x3c)[0]
    n,opt=struct.unpack_from('<H',b,pe+6)[0],struct.unpack_from('<H',b,pe+20)[0]
    rows=[]
    for i in range(n):
        p=pe+24+opt+40*i;size,rva=struct.unpack_from('<II',b,p+8)
        rows.append((rva,rva+size,bool(struct.unpack_from('<I',b,p+36)[0]&0x20000000)))
    return rows

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--root',type=Path,required=True);parser.add_argument('--host',type=Path,required=True)
    args=parser.parse_args();root=args.root.resolve();host=args.host.resolve()
    root.mkdir(parents=True,exist_ok=True);source=root/'sdk-source';sdk=root/'sdk'
    archive=root/'FreeCAD-1.1.4.tar.gz'
    if not archive.exists():urllib.request.urlretrieve(SOURCE_URL,archive)
    if hashlib.sha256(archive.read_bytes()).hexdigest()!=SOURCE_SHA:raise RuntimeError('FreeCAD source checksum mismatch')
    source.mkdir(exist_ok=True)
    with tarfile.open(archive) as tf:
        for member in tf:
            parts=Path(member.name).parts
            if len(parts)<2 or parts[1] not in ('src','cMake','CMakeLists.txt','LICENSE','COPYING'):continue
            target=(source/Path(*parts[1:])).resolve()
            if not target.is_relative_to(source):raise RuntimeError('Unsafe source archive path')
            if member.isdir():target.mkdir(parents=True,exist_ok=True)
            elif member.isfile():
                target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(tf.extractfile(member).read())
            else:raise RuntimeError('Unsupported source archive link')
    for name,file,rel in [('FreeCADGui',host/'bin/FreeCADGui.dll','src/Gui'),('FreeCADApp',host/'bin/FreeCADApp.dll','src/App'),
                          ('FreeCADBase',host/'bin/FreeCADBase.dll','src/Base'),('Part',host/'lib/Part.pyd','src/Mod/Part/App')]:
        dest=sdk/rel;dest.mkdir(parents=True,exist_ok=True)
        output=subprocess.check_output(['dumpbin.exe','/exports',str(file)],text=True)
        table=sections(file);exports=[]
        for line in output.splitlines():
            match=re.match(r'\s*\d+\s+[0-9A-F]+\s+([0-9A-F]{8})\s+(\S+)',line)
            if match:
                rva=int(match[1],16);code=any(lo<=rva<hi and execute for lo,hi,execute in table)
                exports.append(match[2]+('' if code else ' DATA'))
        if len(exports)<30:raise RuntimeError('Incomplete export table: '+name)
        definition=dest/(name+'.def');definition.write_text('LIBRARY "'+file.name+'"\nEXPORTS\n'+'\n'.join(exports)+'\n')
        subprocess.run(['lib.exe','/nologo','/machine:x64','/def:'+str(definition),'/out:'+str(dest/(name+'.lib'))],check=True,stdout=subprocess.DEVNULL)
        print(name,'exports',len(exports),flush=True)
    generator=source/'src/Tools/bindings/generate.py'
    for module in ('Base','App','Gui','Mod/Part/App'):
        directory=source/'src'/module
        for base in re.findall(r'generate_from_py\(([^)]+)\)',(directory/'CMakeLists.txt').read_text(encoding='utf8')):
            base=base.strip();input_file=directory/(base+'.pyi')
            if not input_file.exists():input_file=directory/(base+'.xml')
            if not input_file.exists():raise RuntimeError('Missing generator input '+str(input_file))
            output=sdk/'src'/module/Path(base).parent;output.mkdir(parents=True,exist_ok=True)
            subprocess.run([sys.executable,str(generator),'--outputPath='+str(output),str(input_file)],check=True,stdout=subprocess.DEVNULL)
        print('Generated SDK bindings',module,flush=True)
    (sdk/'stock-sdk.json').write_text(json.dumps({'version':'1.1.4','commit':'4fd3bf320d9566a27e60069fc8387448aaa3a094',
                                                'host':str(host),'kind':'source headers/import libraries only; no FreeCAD rebuild'},indent=2))

if __name__=='__main__':main()
