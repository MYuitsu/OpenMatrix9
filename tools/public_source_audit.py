"""Fail closed on private references, binaries, unsafe SVGs or missing bindings."""
import argparse
import configparser
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

FORBIDDEN_ROOTS={'ref','analysis'}
FORBIDDEN_SUFFIXES={'.dll','.exe','.pyd','.pdb','.lib','.obj','.pdf','.chm','.frx',
                   '.frm','.vbp','.vbw','.ocx','.tlb','.bin','.3dm','.ghidra'}
EXCLUDED_DIRS={'.git','build','target','__pycache__','.vs','.idea'}


def audit(root):
    root=Path(root).resolve();errors=[];files=[]
    for folder in FORBIDDEN_ROOTS:
        if (root/folder).exists():errors.append('Private directory: '+folder)
    stack=[root]
    while stack:
        directory=stack.pop()
        for file in directory.iterdir():
            relative=file.relative_to(root).as_posix()
            if file.is_symlink():errors.append('Symlink: '+relative);continue
            if file.is_dir():
                if file.name not in EXCLUDED_DIRS:stack.append(file)
                continue
            files.append(file)
            if file.suffix.lower() in FORBIDDEN_SUFFIXES:errors.append('Private/binary artifact: '+relative)
            if file.name=='.env' or file.name.startswith('.env.') and file.name!='.env.example':errors.append('Credential file: '+relative)
            if file.suffix.lower() in {'.pem','.key','.p12','.pfx'}:errors.append('Key material: '+relative)
            if relative.startswith('Resources/icons/') and file.suffix!='.svg':errors.append('Non-vector icon: '+relative)
    resources=root/'Resources'
    ini=configparser.ConfigParser(interpolation=None);ini.read(resources/'menu/icons.ini',encoding='utf-8')
    report=json.loads((resources/'menu/source-manifest.json').read_text(encoding='utf-8'))
    assets=set()
    for key in ini.sections():
        relative=ini[key].get('image','');file=(resources/relative).resolve()
        if not file.is_relative_to(resources.resolve()):errors.append('Escaping binding: '+key);continue
        if file.suffix!='.svg' or ini[key].get('source')!='OpenMatrix9-authored-svg':errors.append('Non-authored binding: '+key);continue
        if not file.is_file():errors.append('Missing asset: '+key);continue
        assets.add(file)
        try:svg=ET.parse(file).getroot()
        except ET.ParseError:errors.append('Invalid XML: '+key);continue
        if svg.attrib.get('viewBox')!='0 0 32 32':errors.append('Invalid viewBox: '+key)
        for node in svg.iter():
            tag=node.tag.split('}')[-1]
            if tag in {'image','script','foreignObject'}:errors.append('Non-self-contained SVG: '+key)
            if any(k.startswith('on') or k.split('}')[-1] in {'href'} for k in node.attrib):errors.append('External/executable SVG attribute: '+key)
        recorded=report.get('authored_symbols',{}).get(key,{})
        if recorded.get('sha256')!=hashlib.sha256(file.read_bytes()).hexdigest():errors.append('Hash mismatch: '+key)
        if 'reference_image' in recorded:errors.append('Private reference metadata: '+key)
    for file in (resources/'icons').rglob('*.svg'):
        if file.resolve() not in assets:errors.append('Unbound icon artifact: '+file.relative_to(root).as_posix())
    return {'source_files':len(files),'icon_bindings':len(ini.sections()),'svg_files':len(assets),'errors':errors}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args();result=audit(args.project_root)
    print(json.dumps(result,ensure_ascii=False));raise SystemExit(bool(result['errors']))
