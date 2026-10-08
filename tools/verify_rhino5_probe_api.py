"""Check this probe's Rhino API member names/arity against installed metadata.

This is a compatibility preflight, not a substitute for running in Rhino.
"""
import argparse, ast, json
from pathlib import Path

ALIASES={'active':'Rhino.RhinoDoc','model':'Rhino.FileIO.File3dm',
         'obj':'Rhino.FileIO.File3dmObject','brep':'Rhino.Geometry.Brep',
         'mass':'Rhino.Geometry.VolumeMassProperties'}

def chain(node):
    if isinstance(node,ast.Name):return [node.id]
    if isinstance(node,ast.Attribute):
        parent=chain(node.value)
        return parent+[node.attr] if parent else []
    return []

def verify(source,types):
    tree=ast.parse(source);checked=set();errors=[]
    for node in ast.walk(tree):
        if not isinstance(node,ast.Attribute):continue
        names=chain(node)
        if not names or names[0] not in ALIASES:continue
        kind=ALIASES[names[0]]
        for i,name in enumerate(names[1:]):
            info=types.get(kind,{})
            if name in info.get('properties',{}):
                checked.add(kind+'.'+name);kind=info['properties'][name]
            elif name in info.get('methods',{}) and i==len(names)-2:
                checked.add(kind+'.'+name)
            else:
                errors.append('line '+str(node.lineno)+': unavailable '+kind+'.'+name);break
    for node in ast.walk(tree):
        if not isinstance(node,ast.Call):continue
        names=chain(node.func)
        if not names:continue
        if names[0] in ALIASES and len(names)==2:kind=ALIASES[names[0]];method=names[1]
        elif names[0]=='Rhino' and len(names)>2:kind='.'.join(names[:-1]);method=names[-1]
        else:continue
        counts=types.get(kind,{}).get('methods',{}).get(method,[])
        if len(node.args) not in counts:
            errors.append('line '+str(node.lineno)+': no '+kind+'.'+method+' overload with '+str(len(node.args))+' arguments')
        else:checked.add(kind+'.'+method)
    return dict(ok=not errors,checked=sorted(checked),errors=sorted(set(errors)))

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('script',type=Path);parser.add_argument('metadata',type=Path)
    args=parser.parse_args();metadata=json.loads(args.metadata.read_text(encoding='utf-8-sig'))
    result=verify(args.script.read_text(encoding='utf-8-sig'),metadata['types'])
    result['assembly_sha256']=metadata['assembly_sha256']
    print(json.dumps(result,indent=2));raise SystemExit(0 if result['ok'] else 1)
